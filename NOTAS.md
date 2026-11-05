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
