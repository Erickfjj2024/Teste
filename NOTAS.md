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

## A definir / perguntas em aberto
- Quais mods Java portar primeiro?
- Como os mods do Horizon são estruturados e carregados (ler o README de horizon-mods e o repositório principal).
- Link do repositório principal do Horizon.
