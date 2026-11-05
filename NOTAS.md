# Notas da ideia

## Contexto (dito pelo usuário)
- Existe um projeto relativamente novo chamado **Horizon**: um "Minecraft feito em Rust".

## Ideia central
- Arranjar uma forma de rodar mods do Minecraft Java no Horizon, ou seja, fazer **ports** desses mods para ele.

## Decisão
- Os mods Java serão **reescritos em Rust, um por um** (sem camada de compatibilidade com Java).

## Referências
- Registro de mods do Horizon: https://github.com/horizon-rust/horizon-mods/releases
  - Mods vistos nas releases: wisdom_horizon(_lite), waymap, viahorizon, spider_man, speedrun_timer, skins_3d, server_browser, radiant, montarc (há mais páginas; 13 no total).
  - Cada release aponta para um `mods/<nome>.toml` com hash SHA-256 e 3 assets. A página não explica como os mods são carregados nem em que linguagem são escritos (o `.toml` e o nome da organização sugerem Rust, sem confirmação).
  - O repositório principal do Horizon e a documentação não apareceram nessa página.

## O que descobri lendo os repositórios (clones locais em /home/user/horizon-rust/)
- **Horizon** (github.com/horizon-rust/horizon, MIT): Minecraft Java 26.3 reescrito em Rust. Site: playhorizon.dev. Jars de mods Java **não rodam** nele.
- **Dois tipos de mod:**
  - `.hmod` (conteúdo): só dados e assets (blocos, itens, receitas, loot, modelos, texturas). Sem código. Roda em sandbox. Criado com `cargo horizon-mod new/build/run/check`.
  - `.hnmod` (nativo): bundle do **horizon-mixins** (`horizon-mixin.toml` + helpers `.rs`) que altera o código-fonte Rust do jogo antes de compilar. Roda com as permissões do jogador; o launcher compila um jogo próprio para cada instância. Template: github.com/horizon-rust/horizon-native-mod-template. Exemplo de port citado na doc: Veinminer (3 hooks HEAD + 1 módulo).
  - Para reescrever mods Java em Rust, o caminho é o **mod nativo (.hnmod)**. Ver docs/native-mods.md e docs/mods.md no repo do Horizon.
- **Registro (horizon-mods):** mods enviados por formulário em playhorizon.dev/mods/submit, abre PR, passa em checks e revisão humana. Já há ~90 mods (ex.: appleskin, baritone, jei, jade, minimap, freecam, create, distant_horizons...).
- **Regras para ports (MOD-POLICY.md):** a licença do original precisa permitir o port, ou precisa de permissão dos autores; manter avisos de copyright e licença, creditar e linkar o original. Licença do original com todos os direitos reservados exige permissão escrita. Para listar no registro, o código-fonte precisa ser MIT, MIT-0, BSD-2/3, 0BSD, Apache-2.0, ISC, zlib, Unlicense ou CC0 (GPL/LGPL/MPL só com concessão por escrito). Nada de código ou assets do Minecraft.

## A definir / perguntas em aberto
- Quais mods Java portar primeiro?
- Ler a fundo docs/native-mods.md e o horizon-native-mod-template (hooks disponíveis).
- Checar a licença de cada mod Java escolhido antes de portar.

## Objetivo do usuário: jogar o modpack FTB StoneBlock 3
- O usuário quer jogar o StoneBlock 3, mas não sabe quais mods ele tem nem as licenças.
- Segundo as buscas (Moddex, ModpackIndex): Forge, Minecraft 1.18.2, cerca de 222 mods, Java 17. Lista completa: https://www.modpackindex.com/modpack/89156/ftb-stoneblock-3 e a página do CurseForge.
- O Horizon roda o Minecraft 26.3. O pack é de 1.18.2, então cada port também precisa de adaptação de versão.
- Bloqueio: curseforge.com e modrinth.com não resolvem nesta sessão (fora dos domínios de rede permitidos). A lista completa e as licenças ainda não foram levantadas.
- Próximo passo: obter a lista de mods (permitir os domínios, ou o usuário colar/exportar a lista), conferir a licença de cada um e separar os portáveis.

### Atualização: rede liberada
- Liberados no ambiente Default: www.curseforge.com, modrinth.com, api.modrinth.com.
- Modrinth responde, mas o FTB StoneBlock 3 não está lá (só mods parecidos, como "Stoneblock" e "Vanilla Stoneblock").
- CurseForge responde 403 (bloqueia acesso automatizado), então a lista de mods não sai por lá.
- Sem acesso: api.modpacks.ch (API da FTB) e modpackindex.com. Alternativa: o usuário exporta a lista do pack pelo app da FTB ou do CurseForge e cola.

### Resultado: lista e licenças do StoneBlock 3
- Lista obtida pela API da FTB (api.modpacks.ch, pacote 100, versão 1.11.1): 222 mods, Forge 1.18.2.
- Licenças cruzadas com o Modrinth por hash. Relatório completo em `STONEBLOCK3-MODS.md` (seções A permissivos, B exigem permissão, C sem correspondência).
- Hosts liberados no ambiente Default: curseforge.com, modrinth.com, api.modrinth.com, api.modpacks.ch, modpackindex.com.

### SkyFactory 4
- Lista e licenças em `SKYFACTORY4-MODS.md` (versão 4.2.4, Minecraft 1.12.2, mesma metodologia do StoneBlock 3).

## Primeiro port: ElevatorMod ("OpenBlocks Elevator")
- Original: github.com/VsnGamer/ElevatorMod (MIT, Vasco), revisão 01502b60. Usuário chamou de "open elevators" (CurseForge: openblocks-elevator).
- Port em `ports/elevator/` (crate `horizon_elevator`): lógica pura em Rust, 16 testes passando.
  - Cobre: busca do elevador de destino (pular sobe, agachar desce), elevador de origem (alcance de ativação), validação do servidor, custo de XP, posição final, yaw/pitch e a configuração com os mesmos padrões do original.
- Ainda falta: a camada de integração com o Horizon (hooks no jogo), o bloco em 16 cores com receitas e modelos, a tela de opções (direcional, camuflagem) e o som. Os assets de bloco/receita do original são dados em JSON e podem virar um `.hmod`.
- Avisos de licença e crédito em `ports/elevator/NOTICE.md` e `ports/elevator/LICENSE-ElevatorMod`.

### Integração do Elevator com o Horizon (feita)
- Mod nativo "trait mod" em `ports/elevator/` (`horizon-mixin.toml` + `api/`), pinado no Nightly 2026-11-05-f319701 do Horizon (commit f319701e, compilado com sucesso).
- Compila contra a API real do Horizon (`cargo check` com o `horizon-client` do commit pinado). Não rodou ainda dentro do jogo.
- Como funciona: ganchos `player_jump` (sobe) e `player_toggle_sneak` (desce) guardam o pedido; no fim do tick (`server_tick`) executa `/tp`, porque `run_command` não funciona dentro do gancho de um jogador.
- Limitações atuais: 
  - O XP não é cobrado (`use_xp` fica desligado; o gancho não vê o XP do jogador e a API só tem `give_experience`).
  - Elevador direcional e camuflagem dependem de entidade de bloco (não portado).
  - A velocidade vertical não é zerada (o cliente move o jogador).
  - Som provisório do jogo (`minecraft:block.piston.extend`) em vez de `elevatorid:teleport`.
  - Só roda no mundo em que o gancho dispara (`host.dimension()` igual ao do jogador).
  - Falta o bloco em si (16 cores, receitas, modelos), que vem do `.hmod`.
- Para verificar o mod sem áudio: o `horizon-client` completo exige a biblioteca `alsa` do sistema; a checagem usou `default-features = false`.

### Elevator: segunda rodada (XP exato, cliente/servidor, .hmod)
- Descoberta: os ganchos `player_jump` e `player_toggle_sneak` do servidor só disparam para convidados, não para o dono do mundo. Por isso o mod foi refeito como o original: o cliente lê pular/agachar (`movement_input` + `tick`) e manda um pedido (`net_send_to_server`); o servidor (`net_from_client`) valida e teleporta.
- XP exato: leitura de `XpLevel`/`XpP` pelo `entity_nbt` e cobrança por `/xp add <jogador> -N points` (cópia do `giveExperiencePoints` do Java no Horizon). Fórmulas e `Math.round` testadas.
- Config lida de `config/elevatorid-synced.toml` (mesmos nomes, padrões e limites).
- `.hmod` pronto em `ports/elevator/hmod` (16 blocos, receitas, texturas, idiomas, som). Passa em `cargo horizon-mod build` e `check`.
- Lista completa de diferenças e itens não verificados: `ports/elevator/README.md`. Nada rodou dentro do jogo ainda.

### Elevator: conferência contra o código do Horizon (a pedido do usuário)
- O usuário apontou que o Horizon usa a mesma lógica do Java, então algumas "limitações" podiam não ser reais. Conferido:
  - Velocidade/queda no teleporte: o `/tp` do Horizon já zera ambas (igual ao Java). Não é limitação.
  - Mobs nascendo: resolvido com o gancho `potential_spawns` quando `mobSpawn = false`.
  - Som `elevatorid:teleport`: o `play_sound` do servidor aceita sons de mods (API 61).
  - Jogo solo: segundo a doc, roda em servidor integrado e o cliente entra nele; deve funcionar, ainda não visto.
- Correção de um palpite meu: eu tinha dito que no solo o elevador não funcionaria. Era uma conclusão apressada: li `native_net_send_to_server` e `native_run_command` do ponto de vista do jogo do servidor, não do cliente.
- Ainda difere: elevador direcional/camuflagem/tela (precisa de entidade de bloco), config por mundo/sincronizada, spawners.

### Passo a passo de teste
- `ports/elevator/COMO-TESTAR.md`: instalar o `.hmod` e compilar o mod nativo com `horizon-launcher native-build`, tabela de testes (inclui XP) e solução de problemas.
- O pin do mod foi trocado para o Nightly `2026-11-05-f319701` (o launcher baixa o código-fonte por Nightly); o mod compila nesse commit.

### Plataforma real do usuário: Horizon Android (github.com/Vasysik/horizon-android, fork de 4 nov 2026)
- Importa `.hmod` por URL HTTPS direta ou seletor de arquivos do sistema (64 bits). Mods novos começam desligados (aba Installed).
- Mods nativos `.hnmod` NÃO são suportados no Android. Logo o mod nativo `ports/elevator/api` não roda lá; só o `.hmod` (blocos, receitas, texturas, som).
- O fork ainda tem mods WASM (API 0.1/0.3, `crates/horizon-mod-sdk/wit/horizon.wit`), mas o cliente não lê blocos nem teleporta (só `command.run-quiet`, `game.get-player`, teclas próprias). Servidor (0.3, experimental) lê blocos mas não teleporta.
- Caminho viável para o comportamento no Android: datapack de funções no mundo (`<world>/datapacks`), usando os critérios `minecraft.custom:minecraft.jump` e `sneak_time` (existem no Horizon) e `execute if block ... #elevatorid:elevators`. Não é exato (validade "não sufoca", arredondamento do XP, latência de 1 tick).
- URL direta do .hmod (repo público): https://raw.githubusercontent.com/Erickfjj2024/Teste/ccr-8f7d1cb2-4mjir6/ports/elevator/dist/elevatorid.hmod
- Bloqueado: compilar o fork Android (código externo não autorizado) para validar o .hmod contra ele.

### Correção: um `.hmod` pode ter código (WebAssembly)
- Eu tinha escrito que `.hmod` é só conteúdo. Isso vale para a documentação do Horizon principal; o fork Android ainda roda `.hmod` com `mod.wasm` (código em sandbox, Wasmtime, 64 bits).
- No registro `horizon-mods` (125 versões): 102 são `.hnmod` (nativo, NÃO roda no Android), 13 são `.hmod` (alguns com código WASM, como compass_bar, e partes de "assets" de mods de duas partes), 7 shaders e 1 resource pack.
- Mods de duas partes (ex.: Wayfarer Stones, Deepbore Quarry): o `.hmod` só traz modelos/texturas/receitas ("does nothing alone") e o código está no `.hnmod`. É exatamente o formato do Elevator.

### Por que `.hnmod` não roda no Android, e forks
- `.hnmod` = código-fonte (mixins em Rust) que o launcher do PC compila junto com o jogo. O APK Android vem pronto e não tem compilador; a doc do fork (android-content.md, android-0.7.7.md) diz que isso exige "recompilar o jogo" e que "não há compilador no celular". `.hmod` WASM é bytecode pronto, carregado em tempo de execução.
- Licença do Horizon e do fork: MIT (copiar, modificar, distribuir, com aviso de copyright mantido). O próprio fork Android é um fork assim ("unofficial", mantém autoria e histórico). Texturas da Mojang não são MIT e ficam fora do repositório.
- Caminho técnico: gerar no PC um APK próprio com o mod compilado dentro ("separately compiled client").
- Obstáculo medido: o fork Android está numa base antiga do Horizon (`native-hooks.json` com 108 ganchos, contra 432 no principal). Faltam `movement_input`, `potential_spawns` e `server_players`, que o mod do Elevator usa; também não achei `native_entity_nbt` no fork.

### Requisito do usuário: um único APK que carregue qualquer mod
- Um APK por mod é inviável. Mods nativos (mixins) alteram o código do jogo e não carregam em tempo de execução; só dados (`.hmod` de conteúdo) e bytecode (`.hmod` WASM) carregam num APK só.
- Estratégia: estender a API WASM do fork (`crates/horizon-mod-sdk/wit/horizon.wit`) com poucas capacidades genéricas, uma vez, e portar os mods como WASM. A lógica pura do Elevator (`ports/elevator/src/lib.rs`, sem dependências) já compila para WASM.
- Capacidades que o Elevator precisa e o cliente WASM não tem: ler blocos/colisão, teleportar o jogador, ler/cobrar XP, tocar som de mod, e (a verificar) pular/agachar como teclas (`input.register-key`, se o toque do Android gera esses eventos).
- Limite: mods grandes (máquinas, entidades de bloco, GUIs, fluidos) exigiriam uma API muito maior.
- Pendente: autorização para compilar o fork Android (foi bloqueado como código externo).

### Compilei o fork Android (autorizado) e achei um erro no .hmod
- `cargo build -p cargo-horizon-mod` no fork OK. O `elevatorid.hmod` original foi RECUSADO pelo fork: `unknown field occludes` (a base do fork é mais antiga e só aceita hardness, explosion_resistance, light, requires_tool, friction, map_color, sounds).
- Gerei `ports/elevator/dist/elevator-blocks-android.hmod` (sem `occludes`), que passa em `check` com a ferramenta do fork. SHA-256 f66994804b6a2fa47dfec14da92877248345383a61401247e50314dc17c6daec.
- URL para o Android: https://raw.githubusercontent.com/Erickfjj2024/Teste/ccr-8f7d1cb2-4mjir6/ports/elevator/dist/elevator-blocks-android.hmod
- Ideia do usuário: API robusta para mods grandes baseada na API Java. O catálogo `native-hooks.json` já mapeia cada gancho a um evento do NeoForge (campo `neoforge`).

### Protótipo da API robusta: o que medi antes de começar
- `rustup target add wasm32-wasip2` funcionou aqui; `ports/elevator/src/lib.rs` compila para `wasm32-wasip2` sem mudanças.
- O fork Android compila no host (`cargo check -p horizon-client --no-default-features`, 1m18s). Mas ele NÃO tem `mod_api`, `Hooks` nem `Host`: o sistema de "trait mods" foi criado no Horizon principal depois da base do fork. O fork tem só 108 ganchos de mixin (`native-hooks.json`) e o SDK WASM antigo (`crates/horizon-mod-sdk/wit/horizon.wit`).
- Fusão de teste (git merge-tree, base = último upstream mesclado pelo fork, 8c02c0b de 1 nov): o Horizon atual (f319701, 5 nov) mudou 2272 arquivos (+539 mil linhas) em 4 dias; o fork mudou 623. Resultado: 54 arquivos em conflito (a maioria em horizon-client-render, game.rs, mods.rs, native_hooks.rs, Cargo.*, launcher).
- O Horizon principal ainda tem o executor WASM (wasmtime, recurso `runtime` em horizon-mods), mas removeu o crate `horizon-mod-sdk`; a doc diz "code is native now".
- Conclusão: o caminho mais curto é adicionar à API WASM atual do fork (WIT + host em horizon-mods/game/mods.rs) as capacidades que o Elevator precisa; a ponte gerada completa fica para depois, sobre uma base atualizada.

### Elevator em WebAssembly para o Horizon Android (feito)
- Capacidade nova `player-control` (get-motion, teleport, take-xp, play-sound) + `world.get-collision`, API nível 8, no fork Vasysik/horizon-android (branch local `elevator-capabilities`). Patches em `ports/elevator/android-fork/` (aplicam limpos em 150a414).
- Descoberta: o servidor integrado roda em thread própria por padrão (`HORIZON_SERVER_THREAD`), e o dono do mundo é um "convidado" para mods de servidor; a primeira versão só tratava o jogador local e não funcionaria no padrão. Corrigido.
- `ports/elevator/wasm-mod/` (exemplo) e `dist/elevator-code-android.hmod`. 8 roteiros de teste passam na sandbox real; nada rodou num jogo/celular de verdade.
- Limites: sem mobSpawn, sem tela/camuflagem, pulo só do chão, config por comando. Detalhes em `ports/elevator/ANDROID.md`.

### APK de teste compilado (arm64-v8a)
- Autorizado pelo usuário (tem o Minecraft Java); liberados no ambiente: dl.google.com, maven.google.com, plugins.gradle.org, piston-meta/piston-data/launchermeta.mojang.com, libraries.minecraft.net, resources.download.minecraft.net.
- SDK 35, build-tools 35.0.0, NDK 28.2.13676358 em /home/user/android-sdk; Mojang JDK via `tools/android_prepare.py` (26.3 baixado e verificado por hash).
- Precisou de um 4º patch: aceitar o JSON de versão 26.3 republicado pela Mojang (hash 80608ff6…, já aceito pelo Horizon principal). Lista `research-lock.json` copiada do principal.
- Build: `android_build.py --abi arm64-v8a --public --profile fast --dev-build elevator --dev-version-code 1`. Pacote `dev.horizon.android.dev` (instala ao lado do original, sem apagar mundos). Rust 7m46s; o Gradle precisou de retentativas por 429 do Maven Central.
- APK 37 MB, SHA-256 e601f7e7…5ea7; assinado (v2), `passed: true` na auditoria do projeto. Não está no repositório (enviado pelo chat).
- Roteiro de instalação e testes: `ports/elevator/TESTAR-APK.md`. Nada rodou num celular ainda.

### Erro `unknown field occludes` no APK (corrigido)
- O usuário importou o `elevatorid.hmod` do PC (tem `occludes`; o fork Android não conhece). Os três arquivos se chamavam "Elevator 0.1.0" no app, então ele não distinguia.
- Agora: `elevator-blocks-android.hmod` (nome "Elevator Blocks"; script `ports/elevator/tools/make-android-hmod.sh`), `elevator-code-android.hmod` (nome "Elevator") e o do PC `elevatorid.hmod` renomeado "Elevator Blocks (PC only)". O antigo `elevatorid-android.hmod` foi removido.

### Elevador não aparecia no criativo (corrigido)
- Faltava uma aba de inventário criativo. Adicionado `hmod/data/elevatorid/horizon/creative_tab/elevators.json` ("Elevators", 16 itens). `elevator-blocks-android.hmod` regenerado (SHA-256 d65cdce7…eb5e). Comando exato: `/give @s elevatorid:elevator_white`.

### Subir não funcionava (corrigido) e aprendizados
- O usuário descobriu: os mods precisam ser LIGADOS antes de abrir o jogo/mundo; ligar dentro do jogo não faz os blocos aparecerem. Depois disso os blocos apareceram e DESCER funcionou (prova que ler estado, achar elevador, colisão e teleporte funcionam no APK).
- Subir não funcionava: no servidor integrado o dono é "convidado" e o jogo não incrementava a estatística de pulos dele (só o jogador local, em game.rs). Corrigido em `lan_actions.rs` (5º patch, `0005-...`). APK versão 2: SHA-256 d8912478…58eb, mesma chave de assinatura do anterior (certificado a685f10c…c765), pacote dev.horizon.android.dev.

### Elevator 0.1.1: subir ainda não funcionava com o APK v2
- A correção do contador de pulos (patch 0005) foi compilada (biblioteca diferente da anterior) mas subir continuou sem funcionar; causa exata não confirmada (sem log do aparelho). Hipóteses descartadas por leitura: troca de estatísticas (igual nos dois caminhos), servidor em thread própria é o padrão.
- Solução que não exige APK novo: o mod (0.1.1, só o .hmod) agora também enxerga o pulo pelo movimento (flag de chão saindo + subida de 0.3 a 1.2 blocos, ou subida a partir do topo de um elevador), com espera de 3 ticks só para esses dois. Comando `/elevator debug` para diagnóstico. 11 roteiros passam. Patch `0006-…`.
- `elevator-code-android.hmod` agora 0.1.1, SHA-256 660c7ec0…a92d.

### Elevator 0.1.2 (diagnóstico)
- Com o 0.1.1 subir continuou sem funcionar e o usuário não trouxe a saída do /elevator debug. Descer funciona: logo origem, destino, colisão e teleporte estão certos; só a PERCEPÇÃO do pulo falha.
- 0.1.2: `/elevator watch on|off` (ligado por padrão) conta no chat, a cada mudança de y/chão/pulos/agachar, o que o mod vê (com o número do tick e o intervalo desde a última chamada). Faixa de subida ampliada para 0.3–3.0 blocos. Patch `0007-…`. `elevator-code-android.hmod` SHA-256 c2b7ef2f…81ff. 11 roteiros passam.
- Próximo passo depende das linhas que o usuário enviar.

### Funcionou (0.1.2) e 0.1.3
- Com o 0.1.2 o usuário relatou que o teste funcionou. Não sabemos qual dos três detectores de pulo resolveu (contador do jogo, flag de chão ou subida pelo movimento) nem se foi a faixa ampliada para 3 blocos; ele não mandou as linhas do /elevator watch.
- `/elevator debug` respondeu "No motion is reported for you": o id do jogador que digita o comando não é o mesmo que `players::online` lista (dono do mundo no servidor integrado). O elevador em si usa os ids de `players::online`, por isso funciona. 0.1.3 acha o jogador pelo nome no debug, e deixa `watch` desligado por padrão. Patch `0008-…`. `elevator-code-android.hmod` SHA-256 cca08b59…b86c.
- Possível problema relacionado a investigar depois: outros pontos do jogo que associam o id do comando ao jogador (se algum mod de servidor usar `player.id` de comando em `player-control`, falhará para o dono).

### Tapete: "funcionou só para subir" (a esclarecer)
- Usuário: cores diferentes funcionam; tapete só para subir. Em simulação (harness com tapete = laje de 1/16) o mod sobe e desce com tapete em cima de qualquer um dos elevadores (`tests/carpet_both_ways.toml`, patch 0009). Na simulação o pouso é no topo do elevador (a altura do tapete não conta), como no Java.
- Faltam dados do aparelho: onde está o tapete, o que acontece ao agachar, e a linha de `/elevator debug` (corrigido na 0.1.3) no ponto em que descer falha.

### Tapete: causa encontrada (relato do usuário) e correções
- Relato: com tapete nos dois elevadores, no de cima não conseguia descer logo depois de subir; um pulo no de cima e depois agachar funcionava; subir do de baixo sempre funcionava; `/elevator debug` seguia dando "No motion is reported".
- Causa: o teletransporte pousa os pés DENTRO da laje de 1/16 do tapete. `update_player_pose` (pose.rs) não consegue encaixar nem a pose em pé nem a agachada e usa a de rastejar; o mod lia "agachado" pela pose, então apertar agachar não virava uma borda de "agachar". O pulo tira o jogador de dentro do tapete. Descer fica sem o problema porque a ação do pulo não depende da pose.
- Correção no jogo (APK v3, patch 0010): `sneaking` = tecla (hub `g.sneaking` para convidados; `shift_key_down` para o local), não a pose.
- Correção no mod (0.1.4, patch 0011): pousa em cima de blocos finos sobre o elevador (+1/16), e o `/elevator debug` lê o que o tick guardou (o comando digitado não recebe `motions` no snapshot; o ajuste por nome em 0.1.3 não resolvia).
- `elevator-code-android.hmod` 0.1.4 SHA-256 7700c4eb…ff8f. 12 roteiros passam.
- APK v3 compilado: versão 3, SHA-256 8e1c5ae3…a279, mesma assinatura (a685f10c…c765), auditoria passou. Instala por cima da v2.


## Mekanism: decisão e levantamento
- Usuário preferiu levantar todo o uso de API do Mekanism antes e implementar as primitivas de uma vez (um APK).
- Levantamento em `ports/mekanism/API-SURVEY.md` (2416 arquivos Java; 154 entidades de bloco, 208 telas, 478 usos de transferência, 79 de químicos, 42 de multibloco).
- Plano: etapas A–D de primitivas no APK; depois máquinas só como .hmod.

- Correcao do usuario: Mekanism inteiro = UM mod (conteudo+codigo). Fase 1: API completa no APK (14 blocos em ordem de dependencia, ver API-SURVEY.md); Fase 2: portar Mekanism inteiro.

- Bloco 1 (entidades de bloco WASM): WIT registration.block-entities + block-entities-tick; SDK Registration::block_entity; runtime Mods::block_entities_tick; harness expect.block-entities; exemplo block-counter (teste passa); jogo: collect_mod_entities/apply_mod_entity_updates em server_mods.rs e native_block_entity em native_hooks.rs (verificacao de compilacao Android em andamento). Mods antigos precisam ser recompilados (WIT mudou).

- Bloco 2 (inventarios): container-spec/item-stack no WIT, SDK ContainerSpec::open, harness set-items/expect.block-items, exemplo block-mover (teste passa); jogo compila para Android (patch ports/api-fork/0002). Itens vao como id+count+SNBT dos componentes. Ainda nao testado dentro do jogo.

- Bloco 3 (energia): energy-spec/energy-side no WIT; transferencia por tick entre vizinhos (horizon_mods::energy, 3 testes); unidade e do mod (J no Mekanism); exemplo energy-demo (teste passa); compila Android; patch 0003. NAO feito: itens que carregam energia (dependem dos data components, bloco 5) e redes de cabos eficientes (bloco 9).

- Bloco 4 (fluidos e quimicos): tank-spec/tank no WIT (kind + substancia por id, unidades inteiras), energy-side virou flow-side, transferencia entre vizinhos (horizon_mods::flow, 4 testes), exemplo tank-demo; compila Android; patch 0004. NAO feito: interacao com baldes/fluidos do mundo, cores/texturas das substancias (bloco 11).

- Bloco 5 (data components): componentes proprios ja viajam nas pilhas (teste de ida e volta em horizon-items; vao como SNBT em item-stack.components); SDK ganhou components::Components (le/escreve sem mexer no resto, 3 testes + doctest); keep-data no WIT: o bloco derruba item com os dados da entidade (minecraft:block_entity_data) e ao colocar restaura; nao despeja os slots. Exemplo item-charger (teste passa). Compila Android; patch 0005. NAO testado no jogo: quebrar/colocar com keep-data. NAO feito: ler/escrever o inventario do jogador e itens de fora de containers (precisa de API de jogador), componentes em tooltips.

- Bloco 6 (tipos de receita): capacidade recipe-types (servidor, experimental): recipes-of-type(kind) devolve o JSON de cada receita (o jogo ja guardava Recipes.custom) e ingredient-items(json) resolve item/#tag/lista/objeto para ids (RecipeData, 2 testes). SDK reexporta serde_json. Exemplo recipe-crusher (teste passa). Nivel de API 9. Compila Android; patch 0006. Limite: tabelas montadas uma vez por sessao (nao acompanham /reload). NAO feito: tipos de receita que o jogador usa na mesa de crafting/livro de receitas (so o mod le).

- Bloco 7 (menus e telas): o menu type de conteudo (data/<ns>/horizon/menu/<bloco>.json) ganhou `screen` declarativo (slots, bar, sprite, text, button; valores = constante, data slot ou par lo/hi de 16 bits) validado em content.rs (teste); a tela e desenhada pelo jogo sem codigo no cliente (menu_screens.rs; matematica pura em horizon_mods::screen, 5 testes); container-spec.menu abre o menu do mesmo id do bloco; menu-data (data slots) em block-entity/update; evento menu-button do servidor devolve update. Exemplo menu-demo (teste passa). Compila Android; patch 0007. NAO testado no jogo: desenho, clique e tooltip. OBS: os testes do binario horizon-client ja nao compilam no fork (mods.rs: modulo tests duplicado; shader_packs.rs: falta has_settings) — nao e deste trabalho. PENDENTE IMPORTANTE: conteudo (api 0.2 registries) e codigo (api 0.3 servidor) sao .hmod separados; o Mekanism "num so mod" exige um empacotamento unico (ver modpack.rs) — tratar antes da fase 2.
