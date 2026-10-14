//! Storage compatibility ID for this vendored Madula RMK 0.9 fork.
//!
//! Seeded from the normal firmware flashed on 2026-10-14 so its existing
//! records stay readable on first upgrade. This is not a wildcard migration:
//! unknown IDs still fail the existing storage compatibility check.
//!
//! Keep stable for LED, sensor, logging, Dial and ordinary firmware changes.
//! Before changing serialized StorageData/ProfileInfo, enum discriminants,
//! capacity-dependent formats or layout dimensions (5x14, 5 layers, 2
//! encoders), implement migration or explicitly bump this ID and document
//! the resulting reset. Never derive this value from time or Git HEAD.
pub const STORAGE_SCHEMA_ID: u32 = 0x225059d6;
