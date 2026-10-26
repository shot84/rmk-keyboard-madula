# RMK GUI wire compatibility backport

Base: crates.io rmk-types 0.4.0 (RMK a917b60).
Rynk wire definitions, macro types and snapshots: upstream RMK
933b0630a23393961e5b3bfa7a176a68d9bb8b8c, matching gui.rmk.rs.
Local build constants add MACRO_MAX_NUM=32 and MACRO_MAX_SIZE=32.
The generated protocol reference test uses the bundled rynk_protocol.md.
No persisted KeyAction, ProfileInfo, StorageData or StorageKey formats change.
The firmware maps whole-macro requests through host/rynk/legacy_macros.rs.
This is not the later upstream main v0.2 variant using macro_space_size or bonded.
Upstream MIT/Apache licenses are retained.
