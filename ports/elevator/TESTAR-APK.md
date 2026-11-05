# Testar o elevador no Horizon Android (APK de teste)

**Este APK não foi testado num celular.** Ele compilou, foi assinado e passou na auditoria do próprio
projeto; o mod passou nos testes de roteiro num jogo simulado. Nada disso garante que funcione no seu
aparelho. Se algo falhar, me mande a mensagem que aparecer.

## Atenção: quatro arquivos, só dois servem para o APK

Dentro do app, os mods aparecem pelo nome. O que você precisa são exatamente estes dois:

| Nome no app | Arquivo | Serve no APK? |
|---|---|---|
| **Elevator Blocks** | `elevator-blocks-android.hmod` | **sim** |
| **Elevator** | `elevator-code-android.hmod` | **sim** |
| Elevator Blocks (PC only) | `elevatorid.hmod` | **não**: dá o erro `unknown field occludes` |

Se você já importou o `elevatorid.hmod` antigo (nome "Elevator" ou "Elevator Blocks (PC only)"), **remova-o**
em Installed antes de importar o novo: os dois usam o mesmo identificador (`elevatorid`) e um bloqueia o outro.

## O APK

| | |
|---|---|
| Arquivo | `horizon-android-public.apk` (enviado no chat; **não** está no repositório, 37 MB) |
| SHA-256 | `d891247803a3f8ad0ac710140b5088e8791fe4b98790825fbfae2caa069b58eb` (versão 3: a tecla de agachar é lida direto, não pela pose) |
| Pacote | `dev.horizon.android.dev`, versão `0.7.7-dev.elevator`, código 3 |
| Aparelho | 64 bits (`arm64-v8a`), Android 9 ou mais novo |
| Base | `Vasysik/horizon-android` no commit `150a414` + os 5 patches de `android-fork/` |

**Ele se instala ao lado do seu Horizon Android atual.** O pacote é outro (`.dev`), então você **não precisa
desinstalar nada** e seus mundos continuam onde estão. Os mundos do APK de teste ficam separados dos
do original.

Foi compilado com o perfil `fast` (sem LTO): é um pouco mais lento que o APK oficial.

## Passo a passo

1. Instale o APK (permita "instalar apps de fontes desconhecidas" para o app que você usar para abrir o arquivo).
2. Abra o app **Horizon Android** de nome igual (o de teste). Faça **Download Minecraft 26.3** (o jogo é baixado
   da Mojang no próprio celular, como no APK original).
3. Em **Mods**, importe os dois arquivos (pelo seletor de arquivos do sistema, ou por URL):
   * `elevator-blocks-android.hmod`: os blocos
   * `elevator-code-android.hmod`: o código

   URLs diretas (o repositório é público):
   * `https://raw.githubusercontent.com/Erickfjj2024/Teste/ccr-8f7d1cb2-4mjir6/ports/elevator/dist/elevator-blocks-android.hmod`
   * `https://raw.githubusercontent.com/Erickfjj2024/Teste/ccr-8f7d1cb2-4mjir6/ports/elevator/dist/elevator-code-android.hmod`
4. Em **Installed**, ligue os dois. O mod de código mostra o que pode fazer (ver quem está no servidor,
   ler e mudar blocos, ver se jogadores agacham e pulam, mover jogadores, tirar XP, tocar sons).
5. **Reinicie o app** (o conteúdo só é lido ao abrir). **Ligue os mods em Installed ANTES de abrir o mundo**: se você ligar com o jogo já aberto, os blocos não aparecem.
6. Crie um mundo em criativo (Superflat). Os elevadores ficam na aba **Elevators** do inventário criativo, ou pelo comando `/give @s elevatorid:elevator_white 2` (precisa de trapaças ligadas no mundo).

## Testes

| # | O que fazer | Esperado |
|---|---|---|
| 1 | Ponha um elevador no chão e outro 5 blocos acima (ar em cima dos dois). Fique no de baixo e **pule** | você vai para cima do de cima, com som |
| 2 | Fique no de cima e **agache** | volta para o de baixo |
| 3 | Ponha uma pedra em cima do elevador de cima e pule no de baixo | não sobe |
| 4 | Troque a pedra por um tapete | sobe |
| 5 | `/elevator show` | mostra as configurações |
| 6 | `/elevator set useXP true` e `/elevator set XPPointsAmount 3`, depois em sobrevivência sem XP, pule | mensagem "You do not have enough XP to use elevators" e não sobe |
| 7 | `/xp add @s 3 points` e pule | sobe e `/xp query @s points` mostra 0 |
| 8 | `/xp set @s 1 levels` (0 pontos) e pule | sobe e fica no nível 0 com cerca de 4 pontos |
| 9 | Em criativo, com `useXP` ligado | sobe sem gastar nada |

**Diferenças em relação ao Java** (não são defeito): o pulo só conta quando você pula **do chão**; monstros ainda
nascem nos elevadores; não há tela de opções nem camuflagem; a configuração é por comando. Mais detalhes em `ANDROID.md`.

## Se pular não sobe (diagnóstico)

Com `/elevator watch on` o mod de código **mostra no chat o que ele vê do seu movimento** (vem desligado), para descobrir por que o pulo
não é percebido. Fique parado em cima de um elevador (com outro acima, e ar livre sobre os dois) e **pule uma vez**.
Aparecem linhas como:

```
tick 812 (+1): y=64.420 rise=+0.420 onGround=false jumps=0 sneaking=false onElevator=true
jump seen by the movement: went up
```

Copie ou tire foto dessas linhas e me mande, **mesmo que o elevador não suba**. Para parar o aviso: `/elevator watch off`.


O mod de código 0.1.1 tem um comando que mostra o que ele enxerga de você. Fique em cima de um elevador
com outro acima e digite, **parado**:

```
/elevator debug
```

Ele responde algo como `sneaking=false onGround=true jumps=0 y=64.000 origin=0 63 0 up=0 68 0 down=none cooldown=0`:

* `origin` é o elevador onde você está (se for `none`, ele não te viu em cima de um);
* `up` é o elevador de destino para cima (se for `none`, falta espaço livre acima dele ou ele está longe demais);
* `jumps` e `onGround` mostram se o jogo informa os seus pulos e o chão.

Depois **pule uma vez** e digite o comando de novo: me diga os dois textos.

## Se não funcionar

| Sintoma | Provável causa |
|---|---|
| O mod de código não liga e diz "update Horizon" | você está usando o APK original, e não o de teste |
| O mod de código não aparece ou diz que não há suporte | o aparelho está rodando Android de 32 bits (o executor de mods não existe nele) |
| Os blocos existem, mas pular não faz nada | o mod de código não está ligado, ou só um dos dois `.hmod` foi importado |
| Sobe, mas sem som | o som `elevatorid:teleport` não foi reconhecido (detalhe) |
| Só falha no solo | pode ser o servidor integrado: me diga e eu olho o log |
