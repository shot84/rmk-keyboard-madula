use std::{env, fs, path::PathBuf};
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut memory = include_str!("memory.x").to_owned();
    if env::var_os("CARGO_FEATURE_BOOT_AUDIT").is_some() {
        // Reserve upper RAM for crash capture across reset, including the reader image.
        memory = memory.replace("LENGTH = 255K", "LENGTH = 127K");
    }
    fs::write(out.join("memory.x"), memory).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=keyboard.toml");
    println!("cargo:rerun-if-changed=vial.json");
    // Vial reads the layout from the keyboard as xz-compressed JSON
    let mut def = Vec::new();
    std::io::Read::read_to_end(&mut xz2::read::XzEncoder::new(fs::File::open("vial.json").unwrap(), 6), &mut def).unwrap();
    // Fixed ID so Vial treats every build as the same keyboard
    let id = [0x6d, 0x61, 0x64, 0x75, 0x6c, 0x61, 0x52, 0x4b];
    fs::write(
        out.join("config_generated.rs"),
        format!("pub const VIAL_KEYBOARD_DEF: [u8; {}] = {:?};\npub const VIAL_KEYBOARD_ID: [u8; 8] = {:?};\n", def.len(), def, id),
    )
    .unwrap();
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
}
