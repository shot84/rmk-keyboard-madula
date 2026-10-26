# Local patch to rmk-macro 0.8.0

Upstream get_rmk_features reads only dependency.features, ignoring features
forwarded from the application's [features]. This prevents a Vial-enabled
CornixTP target from coexisting with the existing non-Vial Madula target.

src/codegen/feature.rs also traverses application feature names supplied via
RMK_APPLICATION_FEATURES by build.rs. Only rmk/... entries in Cargo.toml
are added; existing direct dependency features and defaults are unchanged.
The firmware Cargo feature still activates the actual rmk/vial and host_lock code.

Source: crates.io rmk-macro 0.8.0, https://github.com/rmk-rs/rmk
License: MIT OR Apache-2.0 (upstream license files retained).
