# Cornix + Madula — RMK experimental firmware

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
