# Como testar o Elevator no Horizon (jogo solo)

**Aviso:** este mod ainda não rodou dentro do jogo. Os comandos abaixo vêm da documentação do
Horizon (`docs/mods.md` e `docs/native-mods.md`), mas eu não os executei de ponta a ponta. Se algo
não bater com o que você vê, me mande a mensagem de erro.

O elevador tem **duas partes**, e as duas precisam estar ligadas:

| Parte | O que é | Para que serve |
|---|---|---|
| `elevatorid.hmod` | conteúdo (os 16 blocos, receitas, texturas, som) | você poder ter e colocar o bloco |
| mod nativo `elevator` | código Rust compilado dentro do jogo | pular sobe, agachar desce, XP, mobs |

Sem a segunda parte os blocos existem, mas não fazem nada.

---

## 1. O que você precisa

- O **launcher do Horizon**, de [playhorizon.dev](https://playhorizon.dev).
- Uma **conta Microsoft que seja dona do Minecraft Java Edition** (o Horizon não traz o jogo).
- **Git**.
- **Rust** (só para montar o `.hmod`): instale em [rustup.rs](https://rustup.rs).
- Espaço em disco: a primeira compilação do jogo com o mod é grande (o cache pode chegar a 30 GB)
  e **demora de 20 a 30 minutos**.

## 2. Instale o Horizon e confirme que ele roda sem mods

1. Instale o launcher, crie uma instância e clique em **Play**.
2. Entre num mundo qualquer e saia. Se isso funcionar, o resto é só o mod.

## 3. Baixe o mod

```sh
git clone https://github.com/Erickfjj2024/Teste
cd Teste
git checkout ccr-8f7d1cb2-4mjir6
cd ports/elevator
```

## 4. Monte o `.hmod` (o conteúdo)

```sh
cargo install --git https://github.com/horizon-rust/horizon cargo-horizon-mod
cd hmod
cargo horizon-mod build
cargo horizon-mod check target/horizon-mod/elevatorid.hmod
```

O resultado é `hmod/target/horizon-mod/elevatorid.hmod`. O `check` deve mostrar
`elevatorid 0.1.0 (mod, API 0.2, both)` e "no code: content only".

(Se preferir não compilar a ferramenta, a documentação cita `cargo binstall --git
https://github.com/horizon-rust/horizon cargo-horizon-mod`, que baixa um binário pronto.)

Copie o `elevatorid.hmod` para a pasta **`horizon-mods/`** dentro da pasta do jogo da instância
(a que tem o `options.txt`; se a pasta `horizon-mods` não existir, crie). Depois, dentro do jogo:
**Options > Mods**, ligue **Elevator** (os mods começam desligados) e **reinicie o jogo**: o conteúdo só é
lido quando o jogo abre.

## 5. Compile o mod nativo (o código)

Na pasta `Teste/ports/elevator`, rode (troque `NOME` pelo nome da sua instância):

```sh
horizon-launcher native-build . --instance NOME --trust-native-code
```

- O `horizon-launcher` é o programa de linha de comando do launcher. Se o comando não for encontrado,
  procure o executável na pasta onde o launcher foi instalado.
- Ele mostra um aviso: mods nativos **não ficam em sandbox** e rodam com os seus direitos de usuário.
  Leia o código em `api/src/lib.rs` se quiser (cerca de 280 linhas).
- A **primeira vez** ele baixa o Rust do próprio launcher e compila o jogo inteiro: 20 a 30 minutos.
- O mod está fixado no Nightly **`2026-11-05-f319701`** do Horizon. O launcher compila o jogo nesse
  commit. Se ele avisar que há mods fixados em versões diferentes, escolha **Build anyway**.
- Quando terminar, a instância passa a aparecer como "Native build" e o aviso "Needs rebuild" some.

## 6. Teste no jogo (mundo criativo, tipo "Superflat")

Abra um mundo novo em criativo **com a instância compilada**.

| # | O que fazer | O que deve acontecer |
|---|---|---|
| 1 | `/give @s elevatorid:elevator_white 2` | você recebe 2 elevadores brancos |
| 2 | Coloque um no chão e outro 8 blocos acima, no ar, sobre um pilar ou ponte (deixe ar sobre os dois) | os dois ficam parecidos com lã branca |
| 3 | Fique em cima do de baixo e **pule** | você vai para cima do de cima, com som |
| 4 | Fique em cima do de cima e **agache** | você volta para o de baixo |
| 5 | Coloque um bloco qualquer em cima do elevador de cima e pule no de baixo | **não** teleporta (precisa de espaço livre acima) |
| 6 | Use um elevador de **outra cor** no de cima | teleporta (por padrão a cor não importa) |
| 7 | Faça um pilar de pedra de 4 blocos sobre o elevador de baixo, suba nele e pule | ainda sobe (o jogador está a 5 blocos do elevador, dentro do alcance de ativação 6) |
| 8 | Fique em pé no chão, longe de qualquer elevador, e pule | nada acontece |
| 9 | Receita (em sobrevivência): 8 lãs brancas em volta de 1 pérola do End | dá 1 elevador branco |
| 10 | 1 elevador + 1 corante, sem forma | troca a cor do elevador |

### Teste do XP (o ponto que você pediu)

1. Saia do jogo e crie o arquivo **`config/elevatorid-synced.toml`** na pasta do jogo da instância:

   ```toml
   [General]
   useXP = true
   XPPointsAmount = 3
   ```

2. Reinicie o jogo e entre num mundo em **sobrevivência** (ou mude o modo com `/gamemode survival`).
   Monte dois elevadores como antes.
3. Confira cada caso (os comandos são do próprio jogo):

| Preparação | Ao pular no elevador | Esperado |
|---|---|---|
| `/xp set @s 0 levels` e `/xp set @s 0 points` | tenta subir | **não sobe**, mensagem vermelha "You do not have enough XP to use elevators" |
| `/xp add @s 2 points` | tenta subir | continua sem subir (2 − 3 < 0) |
| `/xp add @s 1 points` (agora 3 pontos) | sobe | sobe e `/xp query @s points` mostra **0** |
| `/xp set @s 1 levels` (e 0 pontos) | sobe | sobe mesmo sem pontos (nível maior que 0 basta): termina no nível **0** com cerca de **4** pontos |
| `/gamemode creative` | sobe | sobe sem gastar nada |

Esses resultados vêm das contas do Java (`Math.round` e o ajuste de nível ao tirar pontos). Se algum
número for diferente, é justamente o que eu preciso ver.

## 7. Monstros

O padrão do original é **monstros não nascerem em cima de elevadores**. Teste difícil de ver: faça um
quarto escuro fechado com piso de elevadores (em sobrevivência, dificuldade normal) e outro com piso de
lã comum. Só o de lã deve gerar monstros. Para desligar, ponha `mobSpawn = true` no arquivo de configuração.

---

## Se algo der errado

| Sintoma | O que provavelmente é | O que fazer |
|---|---|---|
| Não acho o bloco / o comando `/give` não conhece `elevatorid:...` | o `.hmod` não está ligado | Options > Mods, ligar **Elevator**, reiniciar |
| Os blocos existem, mas pular não faz nada | o mod nativo não foi compilado ou a instância não é a "Native build" | refazer o passo 5; confira o selo "Up to date" |
| O launcher diz "Needs rebuild" | o conjunto de mods mudou | clique em **Build** |
| O launcher avisa que mods são de versões diferentes | normal: o mod pede o Nightly `2026-11-05-f319701` | **Build anyway** |
| Pular nada faz **só no solo** | pode ser o envio do pedido do cliente ao servidor (não testei) | mande o log do jogo |
| Sobe, mas **sem som** | o som `elevatorid:teleport` não foi reconhecido | me avise; é só um detalhe |

## O que ainda não existe (não é defeito seu)

- Elevador **direcional** (a seta e a direção do olhar), **camuflagem** e a tela de opções.
- Configuração por mundo e sincronizada do servidor para o cliente. Vale só o `config/elevatorid-synced.toml`.
- Monstros de spawners ou da geração do mundo ainda podem aparecer em elevadores.

## O que me mandar depois do teste

Para cada linha da tabela, diga **funcionou** ou **não funcionou**. Se falhar, mande:

1. o texto de qualquer mensagem de erro (na tela ou no log do jogo);
2. o conteúdo do arquivo `config/elevatorid-synced.toml`, se você criou um;
3. o que o launcher mostra na linha "Native build" da instância.
