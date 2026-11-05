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
