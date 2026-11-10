Continue o port do Mekanism (projeto principal). Repositório Erickfjj2024/Teste, branch `ccr-8f7d1cb2-4mjir6`, pasta `ports/mekanism` (último commit 6c76988). Fork: Erickfjj2024/horizon, branch `mekanism-api`. Responda em PT-BR e informe a porcentagem de conclusão (`python3 -I tools/progresso.py`) em toda resposta. Regras: fidelidade total ao Java 1.21.1-10.7.19.85; não existem "limites do Horizon", o que faltar vira API nova no fork; um patch por bloco, com teste; docs em PT-BR e inglês. Atualize `progresso.json` (fork_implementados, build_real, jogo_testado) só com prova.

Estado: o mod compila sem erros contra o SDK REAL do fork (`CHECK_SDK=<fork>/crates/horizon-mod-sdk tools/verificador/check.sh mod/src check`) e 445 testes passam no SDK de mentira (`... test`). Ligados ao jogo: QIO armazenamento, calor (heat_world.rs), utilidades, teletransporte do Meka-Tool (equipment_world.rs). Pull antes de começar; não mexa em `mod/src/lib.rs` além do necessário (eu também edito).

Tarefas, em ordem (use `ports/api-fork/PROPOSTA-API.wit` e `API-REQUISITOS.md` como contrato; ajuste as assinaturas se o fork pedir, e me diga):
1. FORK-34: mascarar capacidades (tanques/energia/itens) por estado enquanto a estrutura é inválida (S3).
2. `compare-and-set-slot` (troca segura de item) + `set-motion` + `set-allow-flight` + evento de dano ao jogador com armadura (`player-damaged` devolve o dano) + dano de ataque modificável (`item-hit-damage`): destravam o MekaSuit e o dano do Meka-Tool.
3. FORK-30 (partículas de formação, S2) e FORK-31 (fluido visível dentro do multibloco, S4).
4. FORK-32 (tempo de queima de qualquer item), FORK-33 (vários grupos de multibloco por bloco), FORK-35 (bounding blocks nativos).
5. Meio "calor" (FORK-4), consulta de entidades por caixa + slots de energia do jogador (Chargepad), tags de item (Oredictionificator), carregamento de chunks por mod (Dimensional Stabilizer), funções de empacotar bloco (Cardboard Box).
6. Restante de PROPOSTA-API.wit.
Depois de cada item: compile o mod contra o SDK novo, rode os testes e faça push.

Entrega final (quando eu pedir): `mekanism.hmod` (`tools/build.sh`, BUILD_NUVEM.md) e APK como ATUALIZAÇÃO do instalado `dev.horizon.android.dev`: keystore `.local/android/debug.keystore` (SHA-256 A6:85:F1:0C:00:32:56:6E:30:00:D5:7B:9B:54:01:28:65:08:83:38:A5:B5:19:E9:0E:6F:62:7F:D1:4A:C7:65; confira antes e não gere outra), mesmo applicationId, `versionCode` maior que o instalado (peça o valor a mim), build debuggable. Libere espaço em disco antes do build do APK.
