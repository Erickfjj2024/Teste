# Reference version of the port

From 0.1.86 on the port's source of truth is **Mekanism 1.21.1-10.7.19.85** (GitHub tag `v1.21.1-10.7.19.85`, the jars
the user supplied), no longer the 26.3 branch. Goal: 100% fidelity to that release.

## Pipeline
1. `tools/base_1211.py <checkout of the tag> <base folder> [<26.3 checkout>]` normalizes the release's data to the shape
   `convert.py` reads: recipes (item/tag ingredients, sized inputs, separating/reaction outputs, neoforge compound
   ingredients), the energy cube model (`leds`/`ports`), chemical textures (`liquid/`, `infuse_type/`, `slurry/`,
   `pigment/` aliased to `mek_liquid/`, `mek_chemical/...`), and `items/*.json` made from the item models (1.21.1 has none).
   The chemicals' tint/icon JSON (written in Java in the release) are the same in both versions: read from the 26.3 checkout.
2. `tools/build.sh <base folder> <fork> <cargo-horizon-mod>` converts, builds and runs the harness.

## Consequences already applied
- Upgrades: seven types (no experience upgrade in 1.21.1); item textures `item/upgrade_<type>`.
- GUI sprites are vendored in `tools/gui-sprites` (26.3 layout of the same pixel art; 1.21.1 keeps it in atlases cut by code).
- Energy is Joules in 1.21.1: the numbers audit against the Java configs is the next step (see STATUS.md).
