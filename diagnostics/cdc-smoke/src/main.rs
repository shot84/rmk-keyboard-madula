#![no_std]
#![no_main]

use embassy_nrf::{
    bind_interrupts,
    peripherals::USBD,
    usb::{
        Driver, InterruptHandler,
        vbus_detect::{HardwareVbusDetect, InterruptHandler as VbusInterruptHandler},
    },
};
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs {
    USBD => InterruptHandler<USBD>;
    CLOCK_POWER => VbusInterruptHandler;
});

#[embassy_executor::task]
async fn logger(driver: Driver<'static, HardwareVbusDetect>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    let p = embassy_nrf::init(Default::default());
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    spawner.spawn(logger(driver).unwrap());
    loop {
        log::info!("XIAO CDC smoke: alive; no matrix, BLE or settings writes");
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
