#!/usr/bin/env bash
# Build upstream's reference WebHID tool at the firmware protocol revision.
set -euo pipefail
cd "$(dirname "$0")/.."
revision=933b0630a23393961e5b3bfa7a176a68d9bb8b8c
checkout="$PWD/target/rynk-web-source"
command -v wasm-pack >/dev/null || { echo 'Install wasm-pack first: cargo install wasm-pack --locked' >&2; exit 2; }
rustup target add wasm32-unknown-unknown --toolchain 1.94.1
if [ ! -d "$checkout/.git" ]; then
  mkdir -p "$checkout"
  git -C "$checkout" init
  git -C "$checkout" remote add origin https://github.com/rmk-rs/rmk.git
fi
git -C "$checkout" fetch --depth 1 origin "$revision"
# Refuse to discard modifications in a previously prepared checkout.
if [ -n "$(git -C "$checkout" status --porcelain)" ]; then
  echo "Web source has local changes: $checkout" >&2; exit 2
fi
git -C "$checkout" checkout --detach "$revision"
cd "$checkout/rynk/rynk-wasm"
RUSTUP_TOOLCHAIN=1.94.1 wasm-pack build --target web
printf 'Serve locally: python3 -m http.server 8000 --bind 127.0.0.1 --directory %q\n' "$PWD"
