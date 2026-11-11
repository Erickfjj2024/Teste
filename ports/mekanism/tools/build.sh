#!/bin/bash
# Builds mekanism.hmod: converts Mekanism's data and assets, then packs them with the code.
# Needs Python 3 with Pillow (it tints the energy cube core).
# Usage: build.sh <Mekanism checkout> <horizon-android fork> <cargo-horizon-mod>
set -euo pipefail
MEK="${1:?path to the Mekanism checkout (branch 26.3)}"
FORK="${2:?path to the horizon-android fork}"
TOOL="${3:?path to cargo-horizon-mod built from the fork}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"
python3 -I "$HERE/tools/convert.py" "$MEK" "$HERE/mod"
WORK="$(mktemp -d)"
# The copy of the mod goes when the script ends; the build output stays in CARGO_TARGET_DIR (by default ports/mekanism/.target,
# kept between runs so the next build is quicker).
trap 'rm -rf "$WORK"' EXIT
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HERE/.target}"
cp -r "$HERE/mod/." "$WORK/"
sed -i "s#@SDK@#$FORK/crates/horizon-mod-sdk#" "$WORK/Cargo.toml"
cp "$FORK/rust-toolchain.toml" "$WORK/"
(cd "$WORK" && CARGO_TARGET_DIR="$CARGO_TARGET_DIR" "$TOOL" horizon-mod build && \
 CARGO_TARGET_DIR="$CARGO_TARGET_DIR" "$TOOL" horizon-mod test)
echo "built: $CARGO_TARGET_DIR/horizon-mod/mekanism.hmod"
