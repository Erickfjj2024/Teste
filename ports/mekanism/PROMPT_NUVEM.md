Você é a sessão de build do projeto principal: port FIEL (100% igual) do Mekanism 1.21.1-10.7.19.85 para o Horizon (mods WASM `.hmod`, Rust). Responda em PT-BR. Em TODA resposta informe a porcentagem de conclusão (`python3 -I tools/progresso.py`), onde 100% = mod jogável igual ao original em tudo, incluindo a API do Horizon que isso exigir.

## Onde está tudo
- Repositório: github.com/Erickfjj2024/Teste, branch `ccr-8f7d1cb2-4mjir6`, pasta `ports/mekanism` (último commit d260fda). Leia primeiro: HANDOFF.md, BASE.md, STATUS.md, BUILD_NUVEM.md, ENTREGA.md, PROGRESSO.md, API-REQUISITOS.md (FORK-1…35), FORK-INDICE.md, SINTOMAS_MULTIBLOCOS.md, revisao_numeros/, revisao_logica/, rodada3/INTEGRACAO_*.md.
- Patches do fork: `ports/api-fork/0001…0017-*.patch`; funções novas pedidas: `ports/api-fork/PROPOSTA-API.wit` (105).
- Fork do Horizon: os patches NÃO aplicam em Vasysik/horizon-android (150a414 e HEAD) nem em Erickfjj2024/horizon (android-port, 0.4.5). A versão do fork com os patches está na SUA máquina (`/home/user/horizon-rust/horizon-android`). PRIMEIRA TAREFA: faça `git push` dela para um repositório privado do dono (ex.: Erickfjj2024/horizon-fork, branch `mekanism-api`) e me diga o nome. Sem isso eu não consigo ajudar de fora.
- Mekanism original: `git clone --depth 1 --branch v1.21.1-10.7.19.85 https://github.com/mekanism/Mekanism.git` (fonte da verdade); branch principal só para tints de químicos.

## Regras do dono (inegociáveis)
1. Fidelidade total ao original, em cada detalhe. Confira mecanicamente contra o Java; não resuma nem pule partes.
2. NÃO existem "limites do Horizon". O que faltar na API vira FORK/API nova no fork do dono (e depois API pública para outros devs). Implemente-a.
3. Docs em PT-BR e inglês. Cada diferença vai em DIFERENCAS.md no mesmo commit.
4. Sem push de coisas não pedidas; commits na branch acima; sem PR sem pedido.

## Estado (18,9%)
- 66 módulos Rust em `mod/src`; 28 conferidos contra o Java; 437 testes passam num SDK de mentira (`tools/verificador/check.sh mod/src all`). Nada foi compilado com o SDK real nem testado em jogo desde a v0.1.84.
- Os 25 módulos da rodada 3 (qio*, meka_tool, mekasuit, robit, heat*, chargepad…) NÃO estão declarados no `lib.rs` real: ligar conforme rodada3/INTEGRACAO_*.md (registro, `tools/convert.py`, `horizon-mod.toml`).
- Chamadas ao host sob `#[cfg(feature="fork")]` em `fork_utilidades.rs`, `fork_equipamento.rs`, `fork_qio_calor.rs`.
- Defeitos vistos em jogo (SINTOMAS_MULTIBLOCOS.md): S1 colocar bloco sobre multibloco não formado (precisa FORK-29), S2 faíscas ao formar (FORK-30), S3 válvula solta funcionando (corrigido no mod), S4 fluido visível dentro do tanque (FORK-31), S5 evaporação sem receita (corrigido no conversor).
- Pendente de conferir: Electric Bow, HDPE Elytra, módulos de capacete, Assemblicator, Dimensional Stabilizer, Cardboard Box, Personal Barrel/Chest, Seismic Vibrator, Oredictionificator, Modification Station, Security Desk, restante do qio.rs e do Meka-Tool; ~522 receitas do Additions e 14 do Generators não convertidas.

## O que fazer, em ordem
1. Subir o fork (acima) e ligar o ambiente (BUILD_NUVEM.md).
2. Implementar no fork as funções de `PROPOSTA-API.wit` e os FORK-n, por prioridade: 29, 34, 30, 31, 32, 33, 35, depois os demais; um patch por bloco, testado.
3. Ligar os módulos da rodada 3 ao `lib.rs`; `cargo check` e `horizon-mod test` verdes com o SDK real; corrigir só erros de compilação sem mudar valores do Java.
4. Gerar `mekanism.hmod`.
5. APK COMO ATUALIZAÇÃO do instalado (`dev.horizon.android.dev`): mesma keystore, mesmo applicationId, mesmo sharedUserId, `versionCode` maior que o instalado (v52 ou superior; o dono informa), build debuggable para eu rodar `run-as` e o socket `horizon-control.sock`. Se a keystore não estiver no ambiente, pare e peça ao dono.
6. Entregar: `.apk` + `.hmod` + sha256 + resultado dos testes, e rodar `TESTES_v0.1.84.md` e os TESTES_* novos. Atualize `progresso.json` (fork_implementados, build_real, jogo_testado) SÓ com prova.
7. Commit e push; mande porcentagem e lista do que falta.
