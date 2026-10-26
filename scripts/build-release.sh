#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
variant="${1:?Specify cornix-left, cornix-right, madula-lpps, madula-tb, madula-iqs or cornix-tp (append -rynk to a Madula variant for Rynk)}"
features=()
case "$variant" in
  cornix-left|cornix-right) binary="$variant" ;;
  cornix-tp) binary=cornix-tp; features=(--features cornix-tp) ;;
  madula-lpps-rynk) binary=madula-central; features=(--features lpps,battery-audit,madula-rynk) ;;
  madula-tb-rynk) binary=madula-central; features=(--features pmw3610,battery-audit,madula-rynk) ;;
  madula-iqs-rynk) binary=madula-central; features=(--features iqs9151,battery-audit,madula-rynk) ;;
  madula-lpps) binary=madula-central; features=(--features lpps,battery-audit,madula-vial) ;;
  madula-tb) binary=madula-central; features=(--features pmw3610,battery-audit,madula-vial) ;;
  madula-iqs) binary=madula-central; features=(--features iqs9151,battery-audit,madula-vial) ;;
  *) echo 'Unknown or unvalidated variant' >&2; exit 2 ;;
esac
cargo "+${RELEASE_TOOLCHAIN:-1.94.1}" build --release --locked --bin "$binary" "${features[@]}"
mkdir -p dist
elf="target/thumbv7em-none-eabihf/release/$binary"
"${OBJCOPY:-arm-none-eabi-objcopy}" -O binary "$elf" "target/$variant-release.bin"
python3 scripts/make-uf2.py "target/$variant-release.bin" "dist/$variant.uf2"
cp "$elf" "dist/$variant.elf"
python3 scripts/release-manifest.py "$variant" "${features[*]}"
