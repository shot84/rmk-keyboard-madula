#![no_main]
#![no_std]
use rmk::macros::rmk_central;
mod radial_controller;
mod tps43;
#[cfg(any(
    feature = "lpps",
    feature = "pmw3610",
    feature = "iqs9151",
    feature = "battery-audit"
))]
compile_error!("CornixTP cannot use Madula module or battery circuit features");

#[rmk_central]
mod keyboard_central {
    #[Override(entry)]
    fn run() {
        // The driver rotates both pointer and scroll into enclosure coordinates.
        let mut pointer = rmk::input_device::pointing::PointingProcessor::new(
            &keymap,
            rmk::input_device::pointing::PointingProcessorConfig {
                device_id: 0,
                invert_x: false,
                invert_y: false,
                swap_xy: false,
            },
        );
        let mut dial = crate::radial_controller::RadialController;
        let mut usb_transport = rmk::usb::UsbTransport::new(driver, rmk_config.device_config)
            .with_host_service(&host_service);
        let mut ble_transport = rmk::ble::BleTransport::new(
            ble_controller,
            ble_addr,
            rmk_config,
            [
                rmk::split::PeripheralMatrixConfig {
                    rows: 4,
                    cols: 7,
                    row_offset: 0,
                    col_offset: 0,
                },
                rmk::split::PeripheralMatrixConfig {
                    rows: 4,
                    cols: 7,
                    row_offset: 0,
                    col_offset: 7,
                },
            ],
        )
        .with_host_service(&host_service);
        let mut wpm = rmk::processor::builtin::wpm::WpmProcessor::new();
        rmk::run_all!(
            matrix,
            storage,
            keyboard,
            usb_transport,
            ble_transport,
            wpm,
            watchdog_runner,
            pointer,
            dial
        )
        .await;
    }
    #[Override(usb)]
    fn init_usb() {
        use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
        let mut config = embassy_nrf::twim::Config::default();
        config.frequency = embassy_nrf::twim::Frequency::K400;
        config.sda_pullup = true;
        config.scl_pullup = true;
        static TX: static_cell::StaticCell<[u8; 32]> = static_cell::StaticCell::new();
        // Embassy nRF 0.11 Twim::new takes SDA before SCL.
        let i2c = embassy_nrf::twim::Twim::new(
            p.TWISPI0,
            crate::tps43::Irqs,
            p.P0_28,
            p.P0_05,
            config,
            TX.init([0; 32]),
        );
        spawner.spawn(
            crate::tps43::run(
                i2c,
                Input::new(p.P0_29, Pull::None),
                Output::new(p.P0_04, Level::High, OutputDrive::Standard),
            )
            .unwrap(),
        );
        embassy_nrf::usb::Driver::new(
            p.USBD,
            Irqs,
            embassy_nrf::usb::vbus_detect::HardwareVbusDetect::new(Irqs),
        )
    }
}
