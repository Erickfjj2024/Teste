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

* **Mobs nascendo nos elevadores:** o original impede por padrão (`mobSpawn = false`). O formato de
  bloco do Horizon não tem como dizer isso, então aqui os mobs nascem como na lã.
* **Direcional, seta e camuflagem, tela de opções:** dependem de uma entidade de bloco, ainda não portada.
  O bloco já declara as propriedades `facing`, `directional` e `show_arrow` do original.
* **Teleporte:** é feito com o comando `/tp`. O original zera a velocidade vertical do jogador
  depois de teleportar, e isso não é possível para jogadores pela API atual.
* **Solo:** o envio do pedido do cliente ao servidor (`net_send_to_server`) não foi testado em mundo
  de um jogador só.
* **Configuração:** lê só `config/elevatorid-synced.toml`, sem os ajustes por mundo e sem sincronizar
  do servidor para o cliente.
* **Som:** usa `elevatorid:teleport` do `sounds.json` original; falta confirmar que o Horizon o registra.
* **Altura do mundo:** o cliente supõe as alturas padrão de cada dimensão.

## Como construir

```sh
cargo test                                   # a lógica
cd hmod && cargo horizon-mod build           # elevatorid.hmod
cargo horizon-mod check target/horizon-mod/elevatorid.hmod
# o mod nativo: horizon-launcher native-build <esta pasta> --instance <instância> --trust-native-code
```
