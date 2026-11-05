# Elevator (port do ElevatorMod para o Horizon)

Pular sobe para o elevador de cima, agachar desce para o de baixo. Port em Rust do
[ElevatorMod](https://github.com/VsnGamer/ElevatorMod) (VsnGamer, MIT). Veja `NOTICE.md`.

## Partes

| Pasta | O que é |
|---|---|
| `src/lib.rs` | A lógica pura (busca, validação, XP, destino, configuração). 23 testes, sem depender do jogo. |
| `api/` | O mod nativo ("trait mod"): liga a lógica aos ganchos do Horizon. |
| `horizon-mixin.toml` | O manifesto do mod nativo (pinado no Horizon `a1e0c0db`). |
| `hmod/` | O conteúdo: 16 blocos, itens, receitas, texturas, idiomas e o som. Vira `elevatorid.hmod`. |

## Como funciona (igual ao original)

* **Cliente:** quando o jogador aperta pular ou agachar, procura o elevador de destino na mesma
  coluna e manda um pedido ao servidor (`ElevatorHandler.java`).
* **Servidor:** valida o pedido, cobra o XP se `useXP` estiver ligado e teleporta
  (`TeleportPacket.java`).
* O mod precisa estar no cliente e no servidor, como o original.
* Configuração em `config/elevatorid-synced.toml`, com os mesmos nomes, padrões e limites.

## XP

Com `useXP = true` o custo é cobrado como no Java:

* o XP do jogador é lido do servidor (`XpLevel` e `XpP`);
* "tem XP" quando `round(progresso × XP do nível) − custo ≥ 0` **ou** o nível é maior que 0,
  com o mesmo arredondamento do `Math.round` do Java (`xp_progress_points`, testado);
* o desconto usa `/xp add <jogador> -<custo> points`, que no Horizon é uma cópia do
  `Player.giveExperiencePoints` (inclui a descida de nível com valores negativos);
* criativo não paga, e sem XP aparece a mensagem vermelha `elevatorid.message.missing_xp`.

## O que difere do original ou não foi verificado

Nada disto rodou dentro do jogo ainda. O que foi checado: os testes da lógica, a compilação do mod
contra a API real do Horizon e as checagens do pacote (`cargo horizon-mod build` e `check`).

Conferido no código do Horizon, que copia a lógica do Java (e por isso **não é** uma diferença):

* **Velocidade e queda:** o `/tp` do Horizon, num jogador, zera a velocidade e a distância de queda,
  como `ServerPlayer.teleportTo` do Java. O resultado é o mesmo do original.
* **Mobs nascendo nos elevadores:** com `mobSpawn = false` (padrão), o gancho `potential_spawns` deixa
  vazia a lista de mobs naturais de uma posição quando o bloco de baixo é um elevador. Cobre o
  nascimento natural, que é onde o `isValidSpawn` do original atua.
* **Som:** o `play_sound` do servidor aceita sons de mods (API 61), então o `elevatorid:teleport`
  do `sounds.json` original deve tocar.
* **Jogo solo:** a documentação diz que o jogo solo roda num servidor integrado (thread própria) e o
  cliente entra nele como em qualquer servidor ("joining a server, single player's own included"). Por
  isso o pedido do cliente deve chegar ao servidor. Isso ainda não foi visto funcionando.

Ainda difere ou falta:

* **Direcional, seta e camuflagem, tela de opções:** dependem de uma entidade de bloco, ainda não portada.
  O bloco já declara as propriedades `facing`, `directional` e `show_arrow` do original.
* **Configuração:** lê só `config/elevatorid-synced.toml`, sem os ajustes por mundo e sem sincronizar
  do servidor para o cliente.
* **Altura do mundo:** o cliente supõe as alturas padrão de cada dimensão.
* **Outras formas de nascer:** spawners e geração de mundo não passam pelo `potential_spawns`.

## Como construir

```sh
cargo test                                   # a lógica
cd hmod && cargo horizon-mod build           # elevatorid.hmod
cargo horizon-mod check target/horizon-mod/elevatorid.hmod
# o mod nativo: horizon-launcher native-build <esta pasta> --instance <instância> --trust-native-code
```
