# Madula IQS9151 — experimental pointer port

Branch: `codex/madula-iqs`. Not included in v0.1.0-rc.1.

Build: `bash scripts/build-docker.sh --bin madula-central --features iqs9151,battery-audit`

The module uses TWIM0 at 400 kHz, SDA P1.14, SCL P1.13 and active-low RDY
P1.12. Its I2C address is 0x56 and expected product number is 0x09bc.
LPPS, TB and IQS features are mutually exclusive.

Initialization data and sequence originate from ShiniNet/zmk-driver-iqs9151
commit 08a6fd19c5aa5ae7f11daf371b5a391cd8596783 (MIT, retained under licenses/).
Settings originally supplied by Azoteq. Madula's ZMK rotation (270 degrees
counterclockwise), resolution and dynamic filter overrides are applied on-chip.
The system-control ATI request is encoded as LE16 bits 5 and 6.

This first port implements single-finger relative motion only. Tap, multi-finger
scroll, pinch and inertia are not implemented. Madula's three physical buttons
retain their existing keymap. Streaming mode is used during bring-up.

CDC logs show product number, initialization errors, finger count and one-second
motion totals. Each transfer and RDY wait is bounded; failure discards the frame
and retries initialization without intentionally resetting the keyboard.

Hardware validation is pending. Before flashing, power off and fit the IQS
module. Verify cold start, direction, finger release, USB/BLE and both Cornix
peripherals. A successful compile is not evidence of hardware operation.
