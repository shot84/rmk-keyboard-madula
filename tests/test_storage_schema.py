"""Exercise the real build script twice, not a duplicate of its ID logic."""
import os
import pathlib
import shutil
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


@unittest.skipUnless(shutil.which("rustc"), "requires host rustc")
class StorageSchema(unittest.TestCase):
    def test_rebuilds_preserve_current_normal_firmware_schema(self):
        with tempfile.TemporaryDirectory() as tmp:
            temp = pathlib.Path(tmp)
            shutil.copy(ROOT / "vendor/rmk/build.rs", temp / "build.rs")
            shutil.copy(ROOT / "vendor/rmk/storage_schema.rs", temp / "storage_schema.rs")
            # Target-cfg helper is unrelated to persistence; isolate its interface.
            (temp / "build_common.rs").write_text(
                "pub struct CfgSet; impl CfgSet { pub fn new()->Self { Self } }"
                "pub fn set_target_cfgs(_: &mut CfgSet) {}"
            )
            results = []
            for index in range(2):
                binary = temp / f"build-{index}"
                subprocess.run(["rustc", "--edition=2024", str(temp / "build.rs"),
                                "-o", str(binary)], check=True)
                out = temp / f"out-{index}"
                out.mkdir()
                subprocess.run([str(binary)], env={**os.environ, "OUT_DIR": str(out)},
                               check=True, capture_output=True)
                results.append((out / "constants.rs").read_text())
            self.assertEqual(results[0], results[1])
            self.assertIn("BUILD_HASH: u32 = 0x225059d6", results[0])

    def test_unknown_schema_and_explicit_reset_remain_guarded(self):
        source = (ROOT / "vendor/rmk/src/storage/mod.rs").read_text()
        self.assertIn("config.build_hash == BUILD_HASH", source)
        self.assertIn("!storage.check_enable().await || storage_config.clear_storage", source)
