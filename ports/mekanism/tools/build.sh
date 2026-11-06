#!/bin/bash
# Builds mekanism.hmod: converts Mekanism's data and assets, then packs them with the code.
# Usage: build.sh <Mekanism checkout> <horizon-android fork> <cargo-horizon-mod>
set -euo pipefail
MEK="${1:?path to the Mekanism checkout (branch 26.3)}"
FORK="${2:?path to the horizon-android fork}"
TOOL="${3:?path to cargo-horizon-mod built from the fork}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"
python3 -I "$HERE/tools/convert.py" "$MEK" "$HERE/mod"
WORK="$(mktemp -d)"
cp -r "$HERE/mod/." "$WORK/"
sed -i "s#@SDK@#$FORK/crates/horizon-mod-sdk#" "$WORK/Cargo.toml"
cp "$FORK/rust-toolchain.toml" "$WORK/"
(cd "$WORK" && CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/target}" "$TOOL" horizon-mod build && \
 CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/target}" "$TOOL" horizon-mod test)
echo "built: ${CARGO_TARGET_DIR:-$WORK/target}/horizon-mod/mekanism.hmod"
