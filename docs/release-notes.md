# v0.1.0-rc.5 — Madula Vial / Rynk

Adds browser keymap editing builds for Madula LPPS, TB (PMW3610), and IQS9151.

- `madula-{lpps,tb,iqs}.uf2`: Vial variants.
- `madula-{lpps,tb,iqs}-rynk.uf2`: experimental Rynk variants; use BLE/WebHID for the reference browser client.
- Choose the image matching your installed module and desired protocol. Vial and Rynk cannot be enabled together.
- Cornix left/right peripheral images are included. CornixTP has its own central image.

Rynk firmware builds are verified. Device flashing, browser editing, persistence after reboot, and the reference Web client build remain unverified for Madula Rynk. The Web client must be built locally at the pinned revision; this release does not host a Web editor.

See [Rynk setup](https://github.com/te9no/rmk-keyboard-madula/blob/v0.1.0-rc.5/docs/rynk.md) and [Vial Web setup](https://github.com/te9no/rmk-keyboard-madula/blob/v0.1.0-rc.5/docs/vial-web.md). Setup documents and Vial definitions are included as release assets.

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

## CornixTP experimental target

`cornix-tp.uf2` targets the XIAO nRF52840 + TPS43 central; use it with the two Cornix peripheral images. Hardware validation is pending. See [CornixTP notes](cornix-tp.md). Do not use the Madula image on this wiring.

CornixTP now includes Vial for keys, encoders and gesture bindings. USB/BLE transport integration and embedded layout are build-verified; device recognition and persistent editing still require hardware validation.

Madula Rynk variants are available as madula-{lpps,tb,iqs}-rynk.uf2. These are experimental alternatives to Vial, with no hardware validation yet. See [Rynk setup](rynk.md).
