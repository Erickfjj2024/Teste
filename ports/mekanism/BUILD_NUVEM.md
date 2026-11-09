# BUILD_NUVEM — compilar e testar o port do Mekanism numa sessão na nuvem / Build and test the port in a cloud session

> **PT-BR primeiro, English abaixo / English follows each section.**
> Estado: nada do que a auditoria da rodada 1 mudou (commits `8db905d`, `5240c46`) foi **compilado**; o Termux e as VMs do cshub não têm Rust nem o fork do Horizon.
> State: nothing the round-1 audit changed (commits `8db905d`, `5240c46`) has been **compiled**; Termux and the cshub VMs have neither Rust nor the Horizon fork.

---

## 1. Objetivo / Goal

**PT-BR.** Fidelidade de 100% ao Mekanism **1.21.1-10.7.19.85** (tag `v1.21.1-10.7.19.85`). Esta página é o roteiro para uma sessão do Claude Code na nuvem (ou um PC) montar o ambiente, compilar, rodar o harness e devolver os erros.

**EN.** 100% fidelity to Mekanism **1.21.1-10.7.19.85** (tag `v1.21.1-10.7.19.85`). This page is the script for a cloud Claude Code session (or a PC) to set up the environment, build, run the harness and report the errors.

## 2. O que precisa existir / What must exist

| Peça / Piece | Onde / Where | Observação / Note |
|---|---|---|
| Este repositório / this repo | `github.com/Erickfjj2024/Teste`, branch `ccr-8f7d1cb2-4mjir6` | `ports/mekanism` |
| Fork do Horizon / Horizon fork | `/home/user/horizon-rust/horizon-android` (API 23) | com os patches de `ports/api-fork/` (0001…0017) aplicados em ordem / patches applied in order |
| Checkout do Mekanism 1.21.1 | `git clone --branch v1.21.1-10.7.19.85 https://github.com/mekanism/Mekanism.git mekanism-1211` | fonte de verdade / source of truth |
| Checkout do Mekanism 26.3 | `git clone https://github.com/mekanism/Mekanism.git mekanism` | só para os JSON de tint dos químicos / only for the chemicals' tint JSON |
| `cargo-horizon-mod` | construído a partir do fork / built from the fork | binário usado pelo `build.sh` |
| Python 3 + Pillow | — | tinge o núcleo do Energy Cube / tints the Energy Cube core |
| Rust toolchain | `rust-toolchain.toml` do fork / of the fork | alvo WASM / WASM target |

**Atenção / Warning.** PT: a versão do fork deve conter *todos* os patches pendentes de `DIFERENCAS.md` seção 5 (5.1, 5.3, 5.4) e os novos de `API-REQUISITOS.md`. EN: the fork must contain *all* the pending patches of `DIFERENCAS.md` section 5 (5.1, 5.3, 5.4) and the new ones in `API-REQUISITOS.md`.

## 3. Passo a passo / Steps

```bash
# 0) clonar / clone
git clone git@github.com:Erickfjj2024/Teste.git && cd Teste && git checkout ccr-8f7d1cb2-4mjir6
cd ports/mekanism

# 1) checkouts do Mekanism / Mekanism checkouts
git clone --depth 1 --branch v1.21.1-10.7.19.85 https://github.com/mekanism/Mekanism.git ../../../mekanism-1211
git clone --depth 1 https://github.com/mekanism/Mekanism.git ../../../mekanism          # 26.3, só tint / tint only

# 2) fork do Horizon: aplicar ports/api-fork/*.patch em ordem, construir cargo-horizon-mod
#    PREENCHER/FILL IN: o comando exato de build do fork ainda não está documentado neste repositório.
#    The fork's exact build command is not documented in this repository yet.

# 3) normalizar a base 1.21.1 / normalise the 1.21.1 base (BASE.md)
python3 -I tools/base_1211.py ../../../mekanism-1211 /tmp/base ../../../mekanism

# 4) converter + compilar + testar / convert + build + test
tools/build.sh /tmp/base /caminho/do/fork /caminho/do/cargo-horizon-mod
#    resultado / result: <target>/horizon-mod/mekanism.hmod
```

## 4. Critério de sucesso / Success criteria

1. `horizon-mod build` sem erros / without errors. PT: os arquivos tocados pela rodada 1 são `entangloporter.rs`, `fission.rs`, `laser.rs`, `miner.rs`, `pump.rs`, `turbine.rs`, `cube.rs`, `generators.rs`, `plus.rs`, `transmitter.rs`, `tools/base_1211.py`. EN: the files touched in round 1 are the ones listed.
2. `horizon-mod test` verde / green (testes novos / new tests: `max_nodes_matches_java_config`, `energy_usage_scales_with_silk_radius_and_height`, os do solar e do turbine / the solar and turbine ones).
3. `python3 -I tools/inventario.py …` (ver `INVENTARIO.md`) com a cobertura **subindo**, nunca descendo / with coverage going **up**, never down.

## 5. Se o build falhar / If the build fails

**PT.** Corrija só o erro de compilação, sem mudar o valor que veio do Java. Se o valor do Java não puder ser expresso, registre em `API-REQUISITOS.md` (é um pedido de fork, não um limite). Commit na mesma branch, mensagem com o arquivo e o motivo.
**EN.** Fix the compile error only, without changing the value that came from Java. If the Java value cannot be expressed, record it in `API-REQUISITOS.md` (a fork request, not a limit). Commit on the same branch with the file and the reason.

## 6. Depois do build: teste no celular / After the build: phone test

**PT.** Gerar o APK do fork (debuggable, `horizon-android-v52-debuggable.apk` já serve de modelo), instalar o `.hmod`, e rodar `TESTES_v0.1.84.md` (e os `TESTES_*` das versões novas). Só conta como testado o que foi visto funcionando. Com o APK debuggable, o `run-as` + socket `horizon-control.sock` permitem rodar os testes por script.
**EN.** Build the fork's APK (debuggable; `horizon-android-v52-debuggable.apk` is the model), install the `.hmod`, run `TESTES_v0.1.84.md` (and the newer `TESTES_*`). Only what was seen working counts as tested. With the debuggable APK, `run-as` + the `horizon-control.sock` socket allow scripted tests.

## 7. Regras / Rules

- PT: cada diferença entra em `DIFERENCAS.md` no mesmo commit; sem processos em segundo plano no fim; sem PR sem pedido. EN: every difference goes into `DIFERENCAS.md` in the same commit; no background processes at the end; no PR unless asked.
- PT: **não existem "limites do Horizon"**: tudo o que falta na API é pedido de fork. EN: **there are no "Horizon limits"**: whatever the API lacks is a fork request.
