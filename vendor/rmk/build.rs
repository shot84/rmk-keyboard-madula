#[path = "./build_common.rs"]
mod common;

use std::path::Path;
use std::{env, fs};

#[path = "storage_schema.rs"]
mod storage_schema;

fn main() {
    // Set the compilation target configuration
    let mut cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut cfgs);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=storage_schema.rs");

    // Compute build hash and write to constants.rs
    // Keep the serialized field name for compatibility, but identify the
    // storage schema, NOT this compilation or Git revision.
    let build_hash = storage_schema::STORAGE_SCHEMA_ID;
    let constants = format!(
        "#[allow(clippy::redundant_static_lifetimes)]\npub(crate) const BUILD_HASH: u32 = {build_hash:#010x};\n"
    );

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");
    fs::write(&dest_path, constants).expect("Failed to write constants.rs file");
}
