"""Test frozen public GUI assets against a host wire peer; no device is accessed."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import urllib.request

root = Path(__file__).resolve().parent.parent
os.chdir(root)
info = json.loads(Path("rynk-compat.json").read_text())
cache = Path("target/rynk-gui-test")
cache.mkdir(parents=True, exist_ok=True)
for name, asset in info["tested_assets"].items():
    out = cache / name
    if not out.exists():
        with urllib.request.urlopen(asset["url"], timeout=60) as response:
            out.write_bytes(response.read())
    if hashlib.sha256(out.read_bytes()).hexdigest() != asset["sha256"]:
        raise SystemExit(f"GUI asset hash mismatch: {name}")
subprocess.run(["cargo", "+" + os.getenv("RELEASE_TOOLCHAIN", "1.94.1"), "build", "--locked",
    "--manifest-path", "tests/rynk-compat/Cargo.toml", "--target", "x86_64-unknown-linux-gnu",
    "--bin", "wire-peer"], check=True)
command = ["node", "tests/rynk-compat/gui-client.mjs", str(cache / "gui-wasm.mjs"),
    str(cache / "gui-client.wasm"), "tests/rynk-compat/target/x86_64-unknown-linux-gnu/debug/wire-peer"]
subprocess.run(command, env={**os.environ, "TEST_OLD_PROTOCOL": "1"}, check=True, timeout=30)
subprocess.run(command, env={k: v for k, v in os.environ.items() if k != "TEST_OLD_PROTOCOL"}, check=True, timeout=30)
