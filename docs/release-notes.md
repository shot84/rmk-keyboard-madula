# v0.1.0-rc.6 — Fix RMK GUI protocol mismatch

Fixes “firmware speaks v0.1, this tool speaks v0.2” when connecting the Rynk variants to https://gui.rmk.rs.

- Flash `madula-lpps-rynk.uf2`, `madula-tb-rynk.uf2`, or `madula-iqs-rynk.uf2`, matching your installed input module.
- Backports the exact Rynk v0.2 wire types used by RMK GUI (RMK revision `933b0630a23393961e5b3bfa7a176a68d9bb8b8c`), including capabilities and whole-macro requests.
- Retains the existing storage schema, keymap, pairing format, LEDs, and Dial support. Existing RMK peripheral firmware can remain installed.
- Macro conversion supports ordinary HID Tap/Press/Release, ASCII characters, and Delay up to 65024 ms. Up to 32 operations per macro and 32 slots, within the existing shared buffer. Unsupported extended actions/PauseForRelease and overflow are rejected without overwriting macros. Existing extended or longer Vial macros must be edited with Vial.
- Files without the `-rynk` suffix are Vial builds.

Validation: deployed GUI WASM handshake, capabilities, identity, key and macro read/write, invalid-write preservation, BLE status decoding; protocol snapshots and adapter tests; firmware CI builds. The wire test uses a host test peer, not physical hardware. USB/BLE hardware operation and persistence after reboot remain unverified for this fix.

[Connection instructions](https://github.com/te9no/rmk-keyboard-madula/blob/v0.1.0-rc.6/docs/rynk.md)

RMK GUI supports browser USB through WebUSB; the previous BLE-only guidance applied to upstream's reference demo. Rynk is experimental: other commits may use different formats even when labeled v0.2.

UF2 images are RMK SDC applications at 0x1000 and overwrite legacy SoftDevice; they are not ZMK application updates. ELF files are for debugging. JSON manifests record source and hashes. SHA256SUMS covers release payloads. Device backups are never included.
