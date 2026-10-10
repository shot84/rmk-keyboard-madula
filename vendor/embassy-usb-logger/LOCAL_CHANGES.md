# Vendored USB logger

Source: embassy-usb-logger 0.6.0 from crates.io (MIT OR Apache-2.0).
Upstream: https://github.com/embassy-rs/embassy

Local changes add an opt-in `nrf52840-adafruit-reset` feature. At 1200 baud,
observed DTR high followed by DTR low writes GPREGRET=0x57 and requests a
system reset, matching RMK's Adafruit bootloader path. Other baud rates
disarm the trigger. No upstream files outside this repository are modified.

The logger watches control changes even while idle or sending. USB control
notifications can coalesce: the host must hold DTR high for 500 ms before
dropping it. This is not an authenticated interface; a local process with
serial-port access can reset the device. Bootloader compatibility and USB
re-enumeration are still unverified on hardware.

`src/touch_reset.rs` has standalone host-side state-machine tests.
