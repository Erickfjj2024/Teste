#!/bin/bash
# Builds the content mod for the Horizon Android fork, whose block format is older: it does not
# know the `occludes` property. Usage: make-android-hmod.sh /path/to/cargo-horizon-mod
set -euo pipefail
TOOL="${1:?path to the Android fork cargo-horizon-mod}"
HERE="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(mktemp -d)"
cp -r "$HERE/hmod" "$WORK/hmod" && rm -rf "$WORK/hmod/target"
python3 -I - "$WORK/hmod" <<'PY'
import json, glob, sys
root = sys.argv[1]
for f in glob.glob(root + '/data/elevatorid/horizon/block/*.json'):
    d = json.load(open(f)); d['properties'].pop('occludes', None)
    json.dump(d, open(f, 'w'), indent=2)
p = root + '/horizon-mod.toml'
t = open(p).read().replace('name = "Elevator Blocks (PC only)"', 'name = "Elevator Blocks"')
open(p, 'w').write(t)
PY
(cd "$WORK/hmod" && "$TOOL" horizon-mod build && "$TOOL" horizon-mod check target/horizon-mod/elevatorid.hmod)
cp "$WORK/hmod/target/horizon-mod/elevatorid.hmod" "$HERE/dist/elevator-blocks-android.hmod"
echo "wrote $HERE/dist/elevator-blocks-android.hmod"
