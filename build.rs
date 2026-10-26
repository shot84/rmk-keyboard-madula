use std::{env, fs, path::PathBuf};
fn main() {
    // proc-macros cannot inspect Cargo's activated parent feature graph.
    // Forward names only; the patched macro resolves dependency features from Cargo.toml.
    let active: Vec<_> = ["cornix-tp", "madula-vial", "madula-rynk"]
        .into_iter()
        .filter(|name| {
            env::var_os(format!(
                "CARGO_FEATURE_{}",
                name.replace('-', "_").to_uppercase()
            ))
            .is_some()
        })
        .collect();
    println!(
        "cargo:rustc-env=RMK_APPLICATION_FEATURES={}",
        active.join(",")
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let cornix_tp = env::var_os("CARGO_FEATURE_CORNIX_TP").is_some();
    let madula_vial = env::var_os("CARGO_FEATURE_MADULA_VIAL").is_some();
    let madula_rynk = env::var_os("CARGO_FEATURE_MADULA_RYNK").is_some();
    assert!(usize::from(cornix_tp) + usize::from(madula_vial) + usize::from(madula_rynk) <= 1,
        "Select exactly one host variant: CornixTP, Madula Vial or Madula Rynk");
    let config = if cornix_tp {
        root.join("cornix-tp.toml")
    } else if madula_vial || madula_rynk {
        // Keep the keymap, matrix and storage source of truth in keyboard.toml.
        let source = fs::read_to_string(root.join("keyboard.toml")).unwrap();
        assert_eq!(source.matches("vial_enabled = false").count(), 1);
        let host = if madula_rynk {
            "vial_enabled = false\nrynk_enabled = true\nunlock_keys = [[0, 0], [0, 7]]\ninsecure = false"
        } else {
            "vial_enabled = true\nunlock_keys = [[0, 0], [0, 7]]\ninsecure = false"
        };
        let configured = source.replacen(
            "vial_enabled = false",
            host,
            1,
        );
        let path = out.join(if madula_rynk { "madula-rynk.toml" } else { "madula-vial.toml" });
        fs::write(&path, configured).unwrap();
        path
    } else {
        root.join("keyboard.toml")
    };
    println!("cargo:rustc-env=KEYBOARD_TOML_PATH={}", config.display());
    println!("cargo:rerun-if-changed=cornix-tp.toml");
    if cornix_tp {
        generate_vial_config(
            &out,
            "vial.json",
            [0x8d, 0xb3, 0x6e, 0x54, 0xfa, 0x27, 0xc1, 0x90],
        );
    } else if madula_vial {
        generate_vial_config(
            &out,
            "madula-vial.json",
            [0xa6, 0x2b, 0x71, 0x90, 0xcd, 0x4e, 0x53, 0x18],
        );
    }
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

// Embed the Vial definition using the upstream RMK template's XZ format.
fn generate_vial_config(out: &std::path::Path, definition: &str, uid: [u8; 8]) {
    use std::io::Read;
    println!("cargo:rerun-if-changed={definition}");
    let source = fs::read_to_string(definition).expect("read Vial definition");
    let compact = json::stringify(json::parse(&source).expect("valid Vial definition"));
    let mut compressed = Vec::new();
    xz2::read::XzEncoder::new(compact.as_bytes(), 6)
        .read_to_end(&mut compressed)
        .expect("compress Vial definition");
    fs::write(out.join("vial_definition.xz"), &compressed).unwrap();
    // Stable unique model UID: never regenerate on rebuild or keymap edits.
    fs::write(
        out.join("config_generated.rs"),
        format!(
            "pub const VIAL_KEYBOARD_ID: [u8; 8] = {:?};\n\
         pub const VIAL_KEYBOARD_DEF: [u8; {}] = {:?};\n",
            uid,
            compressed.len(),
            compressed,
        ),
    )
    .unwrap();
}
