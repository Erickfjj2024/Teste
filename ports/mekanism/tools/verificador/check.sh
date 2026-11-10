#!/bin/bash
# Verificador de compilacao / compile checker for the Mekanism port.
# Compila o mod INTEIRO (todos os .rs de mod/src) contra um SDK de mentira ("stub") gerado do horizon.wit, e roda os testes unitarios.
# Compiles the WHOLE mod against a stub SDK made from horizon.wit and runs the unit tests.
# NAO substitui o build real (cargo-horizon-mod), mas pega erro de tipo, emprestimo e API inventada.
# Uso / usage: tools/verificador/check.sh <pasta mod/src> [check|test]   (padrao/default: check + test)
set -euo pipefail
SRC="$(cd "${1:?pasta mod/src / mod/src folder}" && pwd)"
MODE="${2:-all}"
HERE="$(cd "$(dirname "$0")" && pwd)"
WORK="${CHECK_WORK:-/tmp/mek-check}"
rm -rf "$WORK/mod/src"; mkdir -p "$WORK/mod"
cp -r "$SRC" "$WORK/mod/src"
# CHECK_SKIP="modulo1 modulo2": exclui modulos que ainda nao compilam (outro grupo) / skip modules that do not compile yet (another group)
for m in ${CHECK_SKIP:-}; do rm -f "$WORK/mod/src/$m.rs"; done
# CHECK_SDK=<pasta horizon-mod-sdk do fork>: usa o SDK REAL em vez do de mentira / use the REAL SDK instead of the stub
if [ -n "${CHECK_SDK:-}" ]; then SDK_PATH="$(cd "$CHECK_SDK" && pwd)"; else rm -rf "$WORK/sdk"; cp -r "$HERE/sdk" "$WORK/sdk"; SDK_PATH="../sdk"; fi
cat > "$WORK/mod/Cargo.toml" <<TOML
[package]
name = "mekanism"
version = "0.1.0"
edition = "2024"
[lib]
crate-type = ["rlib"]
[dependencies]
horizon-mod-sdk = { path = "$SDK_PATH" }
[features]
fork = []
[workspace]
TOML
# declara no lib.rs os modulos que existem em src/ e ainda nao estao declarados / declare modules present in src/ but not in lib.rs
for f in "$WORK"/mod/src/*.rs; do
  m="$(basename "$f" .rs)"; [ "$m" = lib ] && continue
  grep -q "^mod $m;" "$WORK/mod/src/lib.rs" || sed -i "0,/^mod combiner;/s//mod combiner;\nmod $m;/" "$WORK/mod/src/lib.rs"
done
cd "$WORK/mod"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WORK/target}"
if [ "$MODE" = check ] || [ "$MODE" = all ]; then cargo check --message-format short 2>&1 | grep -E "error|could not compile|Finished" || true; fi
if [ "$MODE" = test ] || [ "$MODE" = all ]; then cargo test --message-format short 2>&1 | grep -E "^test result|FAILED|panicked|error" || true; fi
