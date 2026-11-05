# Elevator no Horizon Android (um APK, qualquer quantidade de mods)

O Horizon Android não roda mods nativos (`.hnmod`), mas roda mods **WebAssembly** (`.hmod` com código)
dentro de uma sandbox. Esta pasta leva o elevador para esse formato. O APK só precisa ser atualizado
**uma vez**, com capacidades novas na API de mods; depois disso cada mod é um arquivo.

## Duas peças, dois arquivos

| Arquivo | O que é | Em `dist/` |
|---|---|---|
| `elevatorid-android.hmod` | os 16 blocos, receitas, texturas, idiomas e o som | sim |
| `elevator-code-android.hmod` | o código do elevador (WASM, mod de servidor, API 0.3) | sim |

Os dois precisam estar instalados e ligados. Um mod do fork não mistura conteúdo (API 0.2) e código de
servidor (API 0.3) no mesmo arquivo, por isso são dois.

## O que foi acrescentado ao fork (`android-fork/`)

Três patches sobre o commit `150a414` do `Vasysik/horizon-android` (`git am` aplica sem conflito):

1. **Nova capacidade `player-control`** (experimental, só mod de servidor). Funções, todas no nível de API 8:
   * `get-motion`: se o jogador está agachado, no chão, quantos pulos já deu, nível e progresso de XP;
   * `teleport`: move o jogador como o `/tp` (zera velocidade e distância de queda);
   * `take-xp`: tira pontos com a mesma conta do `Player.giveExperiencePoints` do Java;
   * `play-sound`: toca um som do jogo ou do mod.
2. **`world.get-collision`**: as caixas de colisão de um bloco (para saber se o bloco de cima sufoca e
   a altura do elevador).
3. O exemplo `crates/horizon-mod-sdk/examples/elevator` (também em `wasm-mod/`), a documentação
   gerada da API e o suporte do `cargo horizon-mod test` a esses recursos.

Funciona com o servidor integrado em thread própria (o padrão, em que o dono do mundo é um jogador
"convidado") e com o jogo todo no mesmo quadro.

## Como obter o APK com isso

Eu não consigo gerar o APK aqui (falta o SDK/NDK do Android e os seus arquivos do Minecraft). No seu PC:

```sh
git clone https://github.com/Vasysik/horizon-android
cd horizon-android
git checkout 150a414
git am /caminho/para/Teste/ports/elevator/android-fork/*.patch
# depois, as instruções do próprio fork (docs/android.md, "Build"):
python tools/android_build.py --abi arm64-v8a
```

Se o fork tiver avançado, os patches podem pedir ajuste. Eles mexem em poucos arquivos:
`crates/horizon-mods/src/{manifest,runtime,api_level,testing}.rs`, `horizon-mod-sdk/wit/horizon.wit`,
`horizon-client/src/game/{mods,server_mods,commands}.rs` e `cargo-horizon-mod/src/check.rs`.

## Como usar no celular

1. Instale o APK feito com os patches.
2. **Mods > Importar** (URL HTTPS direta ou seletor de arquivos) os dois `.hmod`.
3. Em **Installed**, ligue **Elevator** (código) e **Elevator blocks** (blocos). O mod de código mostra, antes de
   ligar, o que ele pode fazer: ver quem está no servidor, ler e mudar blocos, ver se jogadores agacham/pulam,
   mover jogadores, tirar XP e tocar sons.
4. Reinicie o jogo (o conteúdo só é lido ao abrir).
5. `/give @s elevatorid:elevator_white 2`, ponha um acima do outro, e **pule** (sobe) ou **agache** (desce).

Configuração, pelo jogo (sem arquivo): `/elevator show` e `/elevator set <chave> <valor>`, com as mesmas
chaves do ElevatorMod: `sameColor`, `range`, `activationRange`, `precisionTarget`, `resetPitchNormal`,
`resetPitchDirectional`, `useXP`, `XPPointsAmount`. Só funciona quando você está sozinho no servidor.

## Testes: o que foi e o que não foi verificado

Verificado (nesta máquina):

* 8 roteiros do `cargo horizon-mod test`, com o mod WASM **de verdade** na sandbox do jogo, num jogo
  simulado: subir, descer, bloco em cima (sólido e tapete), mesma cor, elevador direcional, XP (sem XP, um
  ponto a menos, o suficiente com o arredondamento do Java, nível acima de 0), criativo e configuração;
* 32 testes de `horizon-mods` e os da ferramenta, todos passando; o fork compila (`mods` ligado);
* os patches se aplicam limpos em `150a414`.

**Não** verificado: nada disto rodou num jogo de verdade nem num celular. O código do lado do jogo
(teleporte, XP, som e colisão de blocos) compila e foi lido contra as funções do próprio jogo, mas nunca foi
executado.

## Diferenças em relação ao ElevatorMod original

* O pulo é lido pelo contador de pulos do jogo: só conta quando o jogador **pula do chão**. O original reage
  à tecla, até no ar.
* Sem tela de opções nem camuflagem (precisam de entidade de bloco). O elevador **direcional** funciona se
  o bloco tiver `directional=true` e `facing=...` (por exemplo com `/setblock`), mas não há a seta nem o
  botão para mudar.
* Sem `mobSpawn`: a API de mods não tem como impedir nascimentos. Monstros nascem nele como na lã.
* A mensagem de "sem XP" vai no chat, e não na barra de ação em vermelho.
* Configuração por comando, e não por `elevatorid-synced.toml`.
* `take-xp`, `teleport` e `get-motion` só alcançam o jogador local e os convidados da própria dimensão; trocar
  de dimensão não é suportado.
* O nível de API 8 e o nome `player-control` são meus: se o projeto do fork ou o Horizon principal
  adotarem outros, é preciso renumerar.
