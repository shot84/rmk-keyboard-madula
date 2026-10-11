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
The system-control TP ATI request is encoded as LE16 bit 5. ALP calibration
is not awaited during active-mode initialization; its sensing occurs in
low-power modes. The default automatic re-ATI settings remain enabled.

Single-finger motion is scaled by 1/2 with signed fractional remainders.
One-, two-, and three-finger taps produce left, right, and middle clicks.
Two-finger centroid motion produces horizontal/vertical wheel steps (32 coordinate
units per step). A tap must finish within 250 ms and 40 units of accumulated
centroid travel. Additional fingers must arrive within 100 ms. Finger-count
transitions reset movement remainders and suppress one-finger cursor tails.
Pinch, tap-drag and inertia are not implemented.

Virtual matrix positions (4,3)..(4,9) carry the gestures through RMK's normal
mouse-key handling on every layer. The 53 physical keys and three Madula button
bindings are unchanged. Streaming mode is used during bring-up.

CDC logs show product number, initialization errors, finger count and one-second
motion totals. Each transfer and RDY wait is bounded; failure discards the frame
and retries initialization without intentionally resetting the keyboard.

2026-10-10: flashed through COM79's 1200-baud reset. Product 0x09bc was read.
The first version failed ATI initialization; the subsequent revision reached
configuration but retained Show Reset after bulk defaults. The current revision
acknowledges reset after configuration, verifies the flag, and produced 12 seconds
of continuous no-touch frames (zero deltas) with no initialization errors.
Bounded clock-stretched access recovers when RDY is inactive across MCU restarts,
and a 1 ms post-STOP delay separates communication windows.
Reference: Azoteq IQS9150/IQS9151 datasheet sections 12.8–12.9 and Appendix A.12.

2026-10-10: user confirmed cursor motion, but reported excessive sensitivity.
The subsequent half-speed and gesture implementation passed seven host unit
tests and twelve configuration tests and built successfully.

2026-10-10: gesture firmware from commit 3a7256e was flashed via COM79 after
backing up the previous firmware. CDC confirmed product 0x09bc, completed
initialization, and the half-speed / multi-finger gesture build running without
errors during the observation window.
2026-10-11: user confirmed the IQS firmware worked ("IQSもOKでした"). This records
overall hardware acceptance of the flashed build, not separate certification of
every gesture or transport. Dedicated cold-start, long-duration and USB/BLE
coverage remain unverified.
Before flashing, power off and fit the IQS
module. Verify cold start, direction, finger release, USB/BLE and both Cornix
peripherals. A successful compile is not evidence of hardware operation.
