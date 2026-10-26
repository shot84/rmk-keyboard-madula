"""Record release provenance; never reads device backups."""
import hashlib
import json
import os
import pathlib
import subprocess
import sys

variant, features = sys.argv[1:]
if variant not in {"cornix-left", "cornix-right", "madula-lpps", "madula-tb", "madula-iqs", "cornix-tp", "madula-lpps-rynk", "madula-tb-rynk", "madula-iqs-rynk"}:
    raise SystemExit("Unknown variant")
root = pathlib.Path("dist")
madula_vial = "madula-vial" in features
definition = "madula-vial.json" if madula_vial else "vial.json"
vial_enabled = variant == "cornix-tp" or madula_vial
data = {
    "variant": variant,
    "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "working_tree_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], text=True).strip()),
    "features": features,
    "vial_enabled": vial_enabled,
    "vial_definition_sha256": hashlib.sha256(pathlib.Path(definition).read_bytes()).hexdigest() if vial_enabled else None,
    "keyboard_config": "cornix-tp.toml" if variant == "cornix-tp" else "keyboard.toml",
    "host_override": "madula-rynk" if "madula-rynk" in features else "madula-vial" if madula_vial else None,
    "host_protocol": "rynk" if "madula-rynk" in features else "vial" if vial_enabled else None,
    "rmk_types_version": "0.4.0",
    "rust": subprocess.check_output(["rustc", "+" + os.getenv("RELEASE_TOOLCHAIN", "1.94.1"), "--version"], text=True).strip(),
    "application_base": "0x1000",
    "warning": "RMK SDC: overwrites legacy SoftDevice; not a ZMK application update",
    "sha256": {ext: hashlib.sha256((root / f"{variant}.{ext}").read_bytes()).hexdigest()
               for ext in ("uf2", "elf")},
}
(root / f"{variant}.json").write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
