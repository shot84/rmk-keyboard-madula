#![no_main]
#![no_std]
use rmk::macros::rmk_central;
#[cfg(feature = "lpps")]
mod lpps;
mod status_led;
mod radial_controller;
#[cfg(all(feature = "lpps", feature = "pmw3610"))]
compile_error!("Select either LPPS or PMW3610: they share the module connector pins");
#[cfg(feature = "pmw3610")]
mod pmw3610;
#[cfg(feature = "iqs9151")]
mod iqs9151;
#[cfg(feature = "iqs9151")]
mod iqs9151_config;
#[cfg(feature = "iqs9151")]
mod iqs9151_tune;
#[cfg(all(feature = "iqs9151", any(feature = "lpps", feature = "pmw3610")))]
compile_error!("Select only one Madula input module");
#[cfg(feature = "battery-audit")]
mod battery_audit;
#[rmk_central]
mod keyboard_central {
    // RMK 0.9 initializes registered processors before keymap. Construct this
    // keymap-dependent processor at entry instead; retain all default tasks.
    #[Override(entry)]
    fn run() {
        let mut pointer = rmk::input_device::pointing::PointingProcessor::new(
            &keymap,
            rmk::input_device::pointing::PointingProcessorConfig {
                device_id: 0,
                invert_x: !cfg!(feature = "iqs9151"),
                invert_y: !cfg!(feature = "iqs9151"),
                swap_xy: cfg!(feature = "pmw3610"),
            },
        );
        let mut dial = crate::radial_controller::RadialController;
        // The generated entry attaches Vial to both transports; this override has to do it too
        let mut usb_transport = rmk::usb::UsbTransport::new(driver, rmk_config.device_config)
            .with_host_service(&host_service);
        let mut ble_transport = rmk::ble::BleTransport::new(
            ble_controller, ble_addr, rmk_config,
            [
                rmk::split::PeripheralMatrixConfig { rows: 4, cols: 7, row_offset: 0, col_offset: 0 },
                rmk::split::PeripheralMatrixConfig { rows: 4, cols: 7, row_offset: 0, col_offset: 7 },
            ],
        )
        .with_host_service(&host_service);
        let mut wpm = rmk::processor::builtin::wpm::WpmProcessor::new();
        rmk::run_all!(matrix, storage, keyboard, usb_transport, ble_transport,
                      wpm, watchdog_runner, pointer, dial).await;
    }
    #[Override(usb)]
    fn init_usb() {
        spawner.spawn(crate::runtime_status().unwrap());
        #[cfg(feature = "iqs9151")]
        {
            let mut config = embassy_nrf::twim::Config::default();
            config.frequency = embassy_nrf::twim::Frequency::K400;
            config.sda_pullup = true;
            config.scl_pullup = true;
            static TX: static_cell::StaticCell<[u8; 32]> = static_cell::StaticCell::new();
            let i2c = embassy_nrf::twim::Twim::new(
                p.TWISPI0, crate::iqs9151::Irqs, p.P1_14, p.P1_13, config,
                TX.init([0; 32]),
            );
            spawner.spawn(crate::iqs9151::run(i2c,
                embassy_nrf::gpio::Input::new(p.P1_12, embassy_nrf::gpio::Pull::Up),
            ).unwrap());
        }
        #[cfg(feature = "pmw3610")]
        {
            use embassy_nrf::gpio::{Output, Input, Flex, Level, OutputDrive, Pull};
            spawner.spawn(crate::pmw3610::run(
                Output::new(p.P1_13, Level::High, OutputDrive::Standard),
                Flex::new(p.P1_15),
                Output::new(p.P1_12, Level::High, OutputDrive::Standard),
                Input::new(p.P1_14, Pull::Up),
            ).unwrap());
        }
        #[cfg(feature = "battery-audit")]
        {
            let mut config = embassy_nrf::saadc::Config::default();
            config.oversample = embassy_nrf::saadc::Oversample::Over16x;
            let mut channel = embassy_nrf::saadc::ChannelConfig::single_ended(p.P0_02);
            channel.time = embassy_nrf::saadc::Time::_40US;
            let adc = embassy_nrf::saadc::Saadc::new(
                p.SAADC, crate::battery_audit::Irqs, config, [channel],
            );
            spawner.spawn(crate::battery_audit::run(adc).unwrap());
        }
        {
            use embassy_nrf::gpio::{Output, Level, OutputDrive};
            use embassy_nrf::pwm::{Config, Prescaler, SequenceLoad, SequencePwm};
            let mut config = Config::default();
            config.prescaler = Prescaler::Div1;
            config.max_duty = 20;
            config.sequence_load = SequenceLoad::Common;
            let pwm = SequencePwm::new_1ch(p.PWM0, p.P0_28, config).unwrap();
            spawner.spawn(crate::status_led::run(
                pwm,
                Output::new(p.P0_03, Level::High, OutputDrive::Standard),
                Output::new(p.P0_26, Level::High, OutputDrive::Standard),
                Output::new(p.P0_30, Level::High, OutputDrive::Standard),
                Output::new(p.P0_06, Level::High, OutputDrive::Standard),
            ).unwrap());
        }
        #[cfg(feature = "lpps")]
        {
            let mut config = embassy_nrf::spim::Config::default();
            config.frequency = embassy_nrf::spim::Frequency::M1;
            config.mode = embassy_nrf::spim::MODE_1;
            let spi = embassy_nrf::spim::Spim::new(
                p.TWISPI0, crate::lpps::Irqs, p.P1_15, p.P1_13, p.P1_12, config,
            );
            spawner.spawn(crate::lpps::run(spi).unwrap());
        }
        #[cfg(feature = "boot-audit")]
        {
            spawner.spawn(crate::boot_recovery().unwrap());
            // Diagnostic comparison: allow the spawned MPSL task to run before storage.
            embassy_time::Timer::after_millis(100).await;
        }
        embassy_nrf::usb::Driver::new(
            p.USBD,
            Irqs,
            embassy_nrf::usb::vbus_detect::HardwareVbusDetect::new(Irqs),
        )
    }
}

#[embassy_executor::task]
async fn runtime_status() {
    loop {
        embassy_time::Timer::after_secs(10).await;
        log::info!(
            "Madula Central alive: uptime={}s; storage=enabled; build={}; boot_audit={}",
            embassy_time::Instant::now().as_secs(),
            env!("CARGO_PKG_VERSION"),
            cfg!(feature = "boot-audit")
        );
    }
}

#[cfg(feature = "boot-audit")]
#[embassy_executor::task]
async fn boot_recovery() {
    embassy_time::Timer::after_secs(20).await;
    rmk::boot::jump_to_bootloader();
}
