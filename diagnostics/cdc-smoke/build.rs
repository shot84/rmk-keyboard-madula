use std::{env, fs, path::PathBuf};
fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    // Legacy-layout rescue diagnostic: intentionally independent of RMK/SDC layout.
    fs::write(out.join("memory.x"), b"MEMORY { FLASH : ORIGIN = 0x27000, LENGTH = 692K\n RAM : ORIGIN = 0x20000008, LENGTH = 255K }").unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=../../memory.x");
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
}
