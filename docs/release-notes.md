# Cornix + Madula — RMK firmware, review build by shot84 (not official)

Based on te9no/rmk-keyboard-madula v0.1.0-rc.4 (`feb351c`), branch `review-daily`.
See the README of this branch for what changed and why (Japanese).

- BLE mouse reports are merged, paced to the connection interval (15 ms or slower)
  and interpolated 8 ms back: the BLE lag of the LPPS is largely gone in our tests
  (measured on macOS; Windows only briefly).
- The Dial CCCD is stored, so the Dial keeps working after a bonded reconnect.
- Vial keymap editing (vial.rocks / Vial app).
- My own 8-layer keymap with the left wheel reversed.
- Per-second diagnostics on the USB log only.

**Switching between builds with and without Vial clears pairings and the stored keymap**
(seen going from without to with Vial; the other way is expected to do the same).
Vial is always unlocked (no host lock): any connected computer can rewrite the keymap.
The trackball and touchpad builds are untested on hardware.
Cornix peripherals were left on rc.4 during testing.

---

# Cornix + Madula — RMK experimental firmware

## v0.1.0-rc.4

- Cornix dual LEDs: layer/Caps Lock colors, Central connection indication,
  300ms fade-in and 600ms fade-out, using PWM + EasyDMA.
- Left encoder scrolls; right encoder provides Windows Radial Controller
  rotation and press, over USB or BLE. Long-press opens the Dial menu.
- Stable storage compatibility ID prevents ordinary rebuilds from resetting
  pairing data. Re-pair once if an older build already lost its keys.
- USB/BLE Dial, LED colors and the storage fix have user hardware acceptance.
  This is still a prerelease, not exhaustive hardware certification.
- No BLE-forced diagnostic routing or LED startup self-test in release builds.

Download the UF2 matching your hardware:

| File | Target |
| --- | --- |
| `cornix-left.uf2` | Cornix left peripheral |
| `cornix-right.uf2` | Cornix right peripheral |
| `madula-lpps.uf2` | Madula central with LPPS |
| `madula-tb.uf2` | Madula central with PMW3610 trackball |
| `madula-iqs.uf2` | Madula central with IQS9151 touchpad |

Use all three boards with RMK. ZMK peripherals are not compatible.
Turn power off before exchanging Madula modules.
IQS provides half-speed pointer motion, one/two/three-finger taps for
left/right/middle click, and two-finger vertical/horizontal scrolling.
IQS basic operation has been user-confirmed; exhaustive gesture and long-duration
testing remain incomplete. Pinch, tap-drag and inertia are not implemented.
The Madula builds include the NiMH single-cell battery voltage estimate for
the verified 1MΩ/470kΩ Bat divider and status LEDs.

**Migration warning:** These images start at 0x1000 and overwrite the legacy
SoftDevice on XIAO. They are not ordinary ZMK firmware updates. Returning to
ZMK may require restoring the SoftDevice. Back up your device and check the
repository's bootloader/migration instructions before flashing. Do not use on
other boards. Existing settings and pairing may be affected.

Basic operation was tested on the author's hardware; long-duration testing,
all gestures and all battery conditions are not certified. Battery percentage
is a voltage-derived estimate. This is an experimental prerelease.

ELF files are for debugging. Per-variant JSON records the source commit and
hashes; `SHA256SUMS` verifies all payloads. Device backups are never included.
