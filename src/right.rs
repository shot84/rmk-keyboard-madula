#![no_main]
#![no_std]
use rmk::macros::rmk_peripheral;
mod peripheral_led;
#[rmk_peripheral(id = 1)]
mod keyboard_peripheral {
    #[Override(usb)]
    fn init_usb() {
        use embassy_nrf::gpio::{Level, Output, OutputDrive};
        use embassy_nrf::pwm::{Config, Prescaler, SequenceLoad, SequencePwm};
        let mut config = Config::default();
        config.prescaler = Prescaler::Div1;
        config.max_duty = 20;
        config.sequence_load = SequenceLoad::Common;
        let pwm = SequencePwm::new_1ch(p.PWM0, p.P0_13, config).unwrap();
        spawner.spawn(crate::peripheral_led::run(
            pwm, Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
        ).unwrap());
        embassy_nrf::usb::Driver::new(
            p.USBD, Irqs,
            embassy_nrf::usb::vbus_detect::HardwareVbusDetect::new(Irqs),
        )
    }
}
