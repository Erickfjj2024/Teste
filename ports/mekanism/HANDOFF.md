# Handoff: continuing the Mekanism port in another Claude Code session

Read first: `ports/mekanism/BASE.md`, `STATUS.md` (last section 2ap), `DIFERENCAS.md` (section 19), `TESTES_v0.1.84.md`.

## Goal and rules (from the user, who replies in Portuguese)
- Faithful ("100% fiel") port of Java Mekanism to Horizon (Minecraft Java in Rust, WASM server mods, `.hmod`).
- Reference version: Mekanism **1.21.1-10.7.19.85** (tag `v1.21.1-10.7.19.85`), no longer 26.3. Full autonomy, rework accepted.
- Build everything first, test at the end on the user's Android phone. Only what the user saw working counts as tested.
- Catalogue every difference vs the original in `DIFERENCAS.md` in the same commit. Never leave background tasks running.
- Branch: `ccr-8f7d1cb2-4mjir6` (repo erickfjj2024/teste). Do not open PRs unless asked.

## State (mod 0.1.86)
- Converter runs against the release via `tools/base_1211.py` (needs a checkout of the tag, and the 26.3 checkout only for chemical tint JSON).
- Energy is Joules in the release; the port keeps FE = J*2/5 (`fe()` in `mod/src/lib.rs`). Done: machines, cubes, cables, matrix, generators, gas generator, laser, teleporter, turbine, fusion, SPS.
- Next: audit tanks/amounts of chemical machines, Dynamic Tank, Boiler, Evaporator, Fission, Miner, Entangloporter, Chargepad, worldgen; then missing machines/items (QIO, heat, Formulaic Assemblicator, MekaSuit, Meka-Tool, Robit...).

## Environment needed (not in the repo; lived in the cloud container)
- Fork of Horizon at `/home/user/horizon-rust/horizon-android` (API level 23, patch `ports/api-fork/0017-*.patch`), Mekanism checkouts `mekanism-1211` and `mekanism`, built `cargo-horizon-mod`. Rebuild these before `tools/build.sh <base> <fork> <cargo-horizon-mod>`.
- Heavy Rust/Android builds likely do not fit Termux: prefer running the Claude Code session in the cloud, or on a PC.
