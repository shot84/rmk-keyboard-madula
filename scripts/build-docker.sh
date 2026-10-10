#!/usr/bin/env bash
set -euo pipefail
repo="$(cd "$(dirname "$0")/.." && pwd)"
rust_cache="$(realpath "${CARGO_HOME:-$HOME/.cargo}")"
rust_tools="$(realpath "${RUSTUP_HOME:-$HOME/.rustup}")"
docker build -f "$repo/Dockerfile.build" -t rmk-madula-build:local "$repo"
docker run --rm --user "$(id -u):$(id -g)" \
  --env CARGO_HOME="$rust_cache" --env RUSTUP_HOME="$rust_tools" \
  --mount "type=bind,source=$repo,target=$repo" \
  --mount "type=bind,source=$rust_cache,target=$rust_cache" \
  --mount "type=bind,source=$rust_tools,target=$rust_tools" \
  --workdir "$repo" rmk-madula-build:local \
  "$rust_cache/bin/cargo" +stable build --release --locked --bins
