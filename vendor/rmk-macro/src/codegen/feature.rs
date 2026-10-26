//! Utilities of check cargo feature
//!

/// Get enabled RMK features list
pub(crate) fn get_rmk_features() -> Option<Vec<String>> {
    // Use an absolute path. `cargo_toml::Manifest::from_path` resolves the
    // workspace root by walking ancestors of the given path; passing a
    // relative `"./Cargo.toml"` makes its fallback canonicalize an empty
    // path and fail with ENOENT inside workspace members.
    let manifest_path = std::path::Path::new(
        &std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"),
    )
    .join("Cargo.toml");
    match cargo_toml::Manifest::from_path(&manifest_path) {
        Ok(manifest) => manifest
            .dependencies
            .iter()
            .find(|(name, _dep)| *name == "rmk")
            .map(|(_name, dep)| {
                let default_features = if let Some(d) = dep.detail() {
                    d.default_features
                } else {
                    true
                };

                let mut feature_set = dep.req_features().to_vec();

                // Resolve application feature forwarding (e.g. cornix-tp = ["rmk/vial"]).
                // The parent build script supplies the active application feature names.
                if let Ok(active) = std::env::var("RMK_APPLICATION_FEATURES") {
                    let mut seen = std::collections::BTreeSet::new();
                    let mut pending: Vec<String> = active.split(',')
                        .filter(|name| !name.is_empty()).map(str::to_owned).collect();
                    while let Some(name) = pending.pop() {
                        if !seen.insert(name.clone()) { continue; }
                        if let Some(entries) = manifest.features.get(&name) {
                            for entry in entries {
                                if let Some(feature) = entry.strip_prefix("rmk/") {
                                    feature_set.push(feature.to_owned());
                                } else if manifest.features.contains_key(entry) {
                                    pending.push(entry.clone());
                                }
                            }
                        }
                    }
                }

                // Add default features to the feature list.
                // Must mirror `default = [...]` in rmk/Cargo.toml.
                if default_features {
                    feature_set.push("defmt".to_string());
                    feature_set.push("storage".to_string());
                    feature_set.push("vial".to_string());
                    feature_set.push("host_lock".to_string());
                    feature_set.push("watchdog".to_string());
                }
                feature_set
            }),
        Err(_e) => None,
    }
}

/// Check whether the given feature is enabled
pub(crate) fn is_feature_enabled(feature_list: &Option<Vec<String>>, feature: &str) -> bool {
    if let Some(rmk_features) = feature_list {
        for f in rmk_features {
            if f == feature {
                return true;
            }
        }
    }
    false
}
