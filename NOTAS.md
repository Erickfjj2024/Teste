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
