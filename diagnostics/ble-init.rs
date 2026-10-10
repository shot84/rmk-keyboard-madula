#![no_std]
#![no_main]
use core::sync::atomic::{AtomicU8, Ordering};
use defmt_rtt as _;
use embassy_nrf::{
    bind_interrupts, peripherals,
    usb::{Driver, vbus_detect::HardwareVbusDetect},
};
use embassy_time::Timer;
use nrf_sdc::mpsl;
use panic_probe as _;
use static_cell::StaticCell;

static STAGE: AtomicU8 = AtomicU8::new(0);

#[embassy_executor::task]
async fn heartbeat() {
    loop {
        #[cfg(feature = "boot-audit")]
        {
            let mut report = [0u8; 384];
            if let Some(len) = panic_probe::read(&mut report) {
                log::error!(
                    "Retained crash: {}",
                    core::str::from_utf8(&report[..len]).unwrap_or("invalid UTF-8")
                );
            }
        }
        log::info!(
            "Boot stage={} (1=USB, 2=SDC, 3=storage/keymap pending, 4=complete, 5=timeout)",
            STAGE.load(Ordering::Relaxed)
        );
        Timer::after_secs(2).await;
    }
}

bind_interrupts!(struct Irqs {
    USBD => embassy_nrf::usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => mpsl::ClockInterruptHandler, embassy_nrf::usb::vbus_detect::InterruptHandler;
    RNG => embassy_nrf::rng::InterruptHandler<peripherals::RNG>;
    EGU0_SWI0 => mpsl::LowPrioInterruptHandler;
    RADIO => mpsl::HighPrioInterruptHandler;
    TIMER0 => mpsl::HighPrioInterruptHandler;
    RTC0 => mpsl::HighPrioInterruptHandler;
});
#[embassy_executor::task]
async fn logger(driver: Driver<'static, HardwareVbusDetect>) {
    embassy_usb_logger::run!(4096, log::LevelFilter::Debug, driver);
}
#[embassy_executor::task]
async fn radio(service: &'static mpsl::MultiprotocolServiceLayer<'static>) -> ! {
    service.run().await
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mp = mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    let clock = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: 500,
        skip_wait_lfclk_started: false,
    };
    static MPSL: StaticCell<mpsl::MultiprotocolServiceLayer> = StaticCell::new();
    static SESSION: StaticCell<mpsl::SessionMem<1>> = StaticCell::new();
    let service = MPSL.init(
        mpsl::MultiprotocolServiceLayer::with_timeslots(
            mp,
            Irqs,
            clock,
            SESSION.init(mpsl::SessionMem::new()),
        )
        .unwrap(),
    );
    spawner.spawn(radio(service).unwrap());
    spawner.spawn(logger(Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs))).unwrap());
    STAGE.store(1, Ordering::Relaxed);
    spawner.spawn(heartbeat().unwrap());
    Timer::after_secs(5).await;
    log::info!("BLE diagnostic: MPSL initialized; next SDC with RMK two-peripheral configuration");
    Timer::after_secs(1).await;
    let sdc_p = nrf_sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
        p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );
    let mut rng = embassy_nrf::rng::Rng::new(p.RNG, Irqs);
    let mut memory = nrf_sdc::Mem::<8368>::new();
    let result = nrf_sdc::Builder::new()
        .unwrap()
        .support_scan()
        .support_central()
        .support_adv()
        .support_peripheral()
        .support_dle_peripheral()
        .support_dle_central()
        .support_phy_update_central()
        .support_phy_update_peripheral()
        .support_le_2m_phy()
        .central_count(2)
        .unwrap()
        .peripheral_count(1)
        .unwrap()
        .buffer_cfg(251, 251, 3, 3)
        .unwrap()
        .build(sdc_p, &mut rng, service, &mut memory);
    STAGE.store(2, Ordering::Relaxed);
    log::info!(
        "Storage diagnostic: entering Central-sized storage/keymap init (5x14, 5 layers; inert keys)"
    );
    let flash = nrf_mpsl::Flash::take(service, p.NVMC);
    let cfg = rmk::config::StorageConfig {
        start_addr: 0xE0000,
        num_sectors: 16,
        clear_storage: false,
        clear_layout: false,
    };
    let mut data = rmk::KeymapData::new([[[rmk::types::action::KeyAction::No; 14]; 5]; 5]);
    let mut behavior = rmk::config::BehaviorConfig::default();
    let positional = rmk::config::PositionalConfig::<5, 14>::default();
    STAGE.store(3, Ordering::Relaxed);
    let storage = embassy_time::with_timeout(
        embassy_time::Duration::from_secs(10),
        rmk::initialize_keymap_and_storage(&mut data, flash, &cfg, &mut behavior, &positional),
    )
    .await;
    log::info!("Storage diagnostic: init completed = {}", storage.is_ok());
    STAGE.store(if storage.is_ok() { 4 } else { 5 }, Ordering::Relaxed);
    loop {
        match &result {
            Ok(_) => log::info!(
                "SDC OK; storage completed = {}; matrix disabled",
                storage.is_ok()
            ),
            Err(error) => log::error!("BLE diagnostic: SDC init failed: {:?}", error),
        }
        Timer::after_secs(2).await;
    }
}
