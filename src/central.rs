#![no_main]
#![no_std]
use rmk::macros::rmk_central;
#[rmk_central]
mod keyboard_central {
    #[Override(usb)]
    fn init_usb() {
        spawner.spawn(crate::runtime_status().unwrap());
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
