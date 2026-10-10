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
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
}
