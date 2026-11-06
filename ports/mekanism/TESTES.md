# Plano de testes do port do Mekanism (APK v34 + mekanism.hmod 0.1.31)

Como usar: faça na ordem dos blocos (cada bloco prepara o seguinte). Anote **OK**, **errado** (e o que viu) ou
**não deu para testar**. A regra do `STATUS.md` continua: só conta como "testado no jogo" o que você viu no celular.

Antes de tudo: instale o APK v34, troque o `mekanism.hmod`, force o fechamento do app e abra o mundo (de preferência um
mundo **novo** em modo criativo, plano ou não, só para o teste; se usar um antigo, os cubos e máquinas colocados antes
podem não ter os espaços novos: quebre e ponha de novo).

Tudo que precisa vem da aba **Mekanism** do inventário criativo (os blocos estão na ordem alfabética: tanques, canos,
tubos, cabos, blocos, máquinas). Para água use um balde de água normal do jogo; para carvão, minérios etc., o inventário
criativo comum.

## Bancada sugerida (monte uma vez)

```
   [cubo criativo]--cabo--[máquinas...]      (a energia vem do Energy Cube criativo: ele se enche sozinho)
```
Um Energy Cube **criativo** encostado na máquina (pela face de trás da máquina, ou ligado por um Universal Cable) dá
toda a energia. Máquinas pegam energia por qualquer face que a janela de lados deixe em "Entrada" (padrão: todas).

---

## A. Correções que esperam a sua confirmação (rápido, 5 minutos)

| # | O que fazer | Esperado |
|---|---|---|
| A1 | Ponha um **Metallurgic Infuser** com blocos de terra/pedra em volta e no chão | Os blocos ao lado e embaixo **aparecem** (antes ficavam invisíveis). O bloco olha para você quando colocado |
| A2 | Pegue um **Fluid Tank** na aba Mekanism (está no começo da lista: "Advanced Fluid Tank"...) | O item tem forma (um tanque de vidro com moldura), não um slot vazio |
| A3 | Ponha o Fluid Tank no chão e dê um **balde de água** no slot de entrada da tela (o de cima, à direita) | A água entra no tanque e o balde vazio sai no slot de baixo. **Na tela a água é azul**, e **no bloco a água não pisca** nas bordas |
| A4 | Faça o mesmo com um **Chemical Tank** (qualquer nível) depois de ter químico (ver D) | O líquido dentro do bloco não pisca; na tela a cor é a do químico |
| A5 | Ponha um **Oxidizer** (carvão) ao lado de um **Osmium Compressor**, com o lado do oxidizer configurado para entregar | O carbono **não** entra no compressor (ele só aceita ósmio). O carbono fica no tanque do oxidizer |

## B. Armadura nas telas

| # | O que fazer | Esperado |
|---|---|---|
| B1 | Abra o Energy Cube, um Fluid Tank e um Chemical Tank | Há 5 slots de armadura (capacete, peitoral, calça, botas e mão secundária): no cubo à **direita**, nos tanques à **esquerda** |
| B2 | Em cada um, ponha uma peça de armadura **vanilla** no slot certo e uma no errado | A certa entra e aparece no personagem; a errada é recusada |
| B3 | Ponha peças do **MekaSuit / Hazmat / Jetpack / Free Runners / Scuba** | Entram no slot certo (a de cada parte do corpo). **O modelo vestido ainda não existe** (você não vê a peça no corpo): isso não é erro novo |
| B4 | Feche e reabra a tela | A armadura continua vestida; shift-clique numa peça de armadura no inventário não deve sumir com ela |

## C. Máquinas que dá para testar de ponta a ponta (itens que viram químico)

| # | O que fazer | Esperado |
|---|---|---|
| C1 | **Chemical Oxidizer**: carvão no slot de entrada, com energia | ~5 s por item: o tanque da direita ganha **10 mB de carbono** por carvão. Também funcionam carvão vegetal, cogumelos (fungi), pó de sal (brine) etc. |
| C2 | Oxidizer com **pó de redstone** | Redstone no tanque (10 mB por pó), com a cor vermelha do medidor |
| C3 | Oxidizer ao lado de um **Metallurgic Infuser** (o infusor recebe o que o oxidizer entrega) | A redstone passa para o tanque do infusor (só ela, ou o que ele usa) |
| C4 | **Metallurgic Infuser**: lingote de cobre + redstone no tanque | 200 ticks, vira Infused Alloy, gastando 10 mB |
| C5 | **Osmium Compressor**: osmium (pó/lingote de ósmio no slot de baixo) + pó de obsidiana refinada | Lingote de obsidiana refinada em ~10 s; gasta ~1 mB de ósmio por tick |
| C6 | Botão **Dump** do infusor (embaixo da barra do tanque) | Esvazia o tanque |
| C7 | Dica ao passar o dedo (segurar) nos medidores e barras | "Nome: n mB" / "Empty" / "Energy: a / b" / "Progress: n%" |

## D. Cadeia da água: o caminho principal da nova fatia

Monte: **Fluid Tank → (encostado ou cano) → Electrolytic Separator**, com energia no separador.

| # | O que fazer | Esperado |
|---|---|---|
| D1 | Balde de água no Fluid Tank, tanque **encostado** no separador | A água passa para o tanque de fluido do separador (o da esquerda); aparecem **hidrogênio** (esquerda) e **oxigênio** (direita) nos medidores pequenos, na proporção 2 : 1; a setinha do meio acende enquanto trabalha |
| D2 | O mesmo, mas com um **Mechanical Pipe** entre o tanque e o separador | Igual a D1: a água viaja pelo cano |
| D3 | Janela de lados do separador (aba da esquerda): **Fluidos, Químicos, Itens, Energia** | 4 abas na ordem do original; a janela abre em **Fluidos**; clicar nos botões muda a cor/tipo; "Can't Eject" nas abas que não ejetam |
| D4 | Botões de **modo de despejo** (os dois cantinhos embaixo do separador) | Cada clique muda: parado → despejando → despejando excesso; "despejando" esvazia o gás aos poucos |
| D5 | Fluid Tank: aba da direita (modo de edição), clique e clique-longo | Alterna "encher e esvaziar / só encher / só esvaziar"; no "só esvaziar" o tanque não aceita mais água do separador/balde |
| D6 | Balde **vazio** no slot de entrada de um tanque com água, e de lava | Fica cheio (balde de água ou de lava); água e lava nunca se misturam |
| D7 | **Chemical Tank criativo** encostado no separador (lado do oxigênio) ou ligado por **Pressurized Tube** | Recebe oxigênio e **nunca esvazia**: o medidor fica cheio e a tela mostra "Infinite" |
| D8 | Chemical Tank: botão de despejo (canto de baixo à direita) e janela de lados (aba da esquerda) | O modo muda e gasta aos poucos; a janela tem **Químicos e Itens** e abre em Químicos |
| D9 | **Purification Chamber** com minério (ferro, ouro, cobre...) e oxigênio vindo do tanque criativo/tubo | Cada minério vira **3 clumps** em ~10 s; o oxigênio é gasto aos poucos (às vezes 0, às vezes 2 por tick: é de propósito) |
| D10 | O tanque de oxigênio **vazio**: a Purification Chamber para sem perder o progresso e volta quando o gás chega | "Status" na tela mostra 4 enquanto espera |

## E. O que ainda **não** dá para testar (e por quê)

| Máquina | Falta | Quando destrava |
|---|---|---|
| Chemical Infuser | precisa de hidrogênio **e cloro**; o cloro só vem da água salgada (brine fluido) e não há fonte de brine | com um comando de teste ou a fonte de brine |
| Chemical Injection Chamber | precisa de ácido clorídrico/sulfúrico (do Chemical Infuser) | idem |
| Chemical Dissolution Chamber | precisa de ácido sulfúrico | idem |
| Chemical Washer | precisa de lama suja (da dissolução) | idem |
| Chemical Crystallizer | precisa de lama limpa (do washer) | idem |

O **plano para destravar**: um comando de teste (`/mekfill <químico> <quantidade>` para encher o tanque que você
está olhando) ou a lata de gás. Sem isso, essas cinco só passaram no teste simulado (a lógica de cada uma está
testada: os números batem), mas nenhuma tela dessas foi vista. **Mesmo assim vale abrir a tela de cada uma** e conferir
layout, textos e a janela de lados:

| # | Tela | Conferir |
|---|---|---|
| E1 | Purification / Injection / Osmium Compressor (mesma tela) | Barra de energia vertical à direita, setinha do meio, barrinha de químico entre a bateria e o slot de entrada, 3 slots + o de energia |
| E2 | Chemical Infuser | 3 medidores (esquerda, centro mais alto, direita), 2 setas, barra de energia horizontal embaixo, slots químicos embaixo |
| E3 | Chemical Washer | Suporte à esquerda com 2 slots e uma setinha para baixo, 3 medidores, seta grande |
| E4 | Dissolution | Medidor de entrada no alto à esquerda, saída à direita, 2 slots químicos (menos/mais) |
| E5 | Crystallizer | Medidor à esquerda, tela escura no alto (vazia por enquanto), seta grande embaixo |

## F. Transmissores

| # | O que fazer | Esperado |
|---|---|---|
| F1 | **Universal Cable** de cada nível (básico, avançado, elite, supremo) entre o cubo criativo e uma máquina | Todos levam energia; os modelos têm a cor do nível. (O básico já foi visto) |
| F2 | **Pressurized Tube** entre dois tanques | Passa químico de um para o outro (o que "dá" entrega ao que "recebe") |
| F3 | **Mechanical Pipe** entre dois Fluid Tanks | Passa água de um para o outro; a vazão cresce com o nível |
| F4 | Tubo/cano **não** se liga a cabo, e cada um só se liga ao que guarda o que ele leva | Cabo não encosta no tanque; tubo não encosta no Fluid Tank |
| F5 | Quebre um tubo/cano no meio | A rede se divide; nada some nem trava |

## G. Já valia antes e ninguém viu (backlog do `STATUS.md` §3)

Faça quando sobrar tempo; são os que mais podem esconder defeitos.

| # | O que fazer | Esperado |
|---|---|---|
| G1 | **Minérios no mundo**: um mundo novo, ande/escave nos níveis baixos | Ósmio, estanho, chumbo, urânio, fluorita aparecem em veios; soltam o item bruto certo |
| G2 | **Blocos de armazenamento** e Steel Casing | Texturas, nomes em inglês, dá para minerar e eles voltam |
| G3 | **Receitas na mesa** (crafting) | Os blocos e itens de Mekanism têm receita; conferir uns 5 |
| G4 | **Funil** alimentando e esvaziando as máquinas | Encaixa só nos lados que a janela de lados deixa (padrão: itens por todos os lados, saída à direita) |
| G5 | **Quebrar** uma máquina com itens, energia e químico dentro | Ela solta o bloco **com tudo guardado**; ao colocar de novo o conteúdo volta |
| G6 | **Salvar e reabrir** o mundo com máquinas, cabos, tubos, canos, tanques cheios | Tudo continua igual (tanques com o mesmo conteúdo, configuração de lados, modo do tanque) |
| G7 | **Precision Sawmill** (toras → tábuas + serragem) e **Combiner** (flint + pedregulho → cascalho) | A serragem sai por sorte (não toda vez); o combinador usa os dois slots |
| G8 | **Energy Cube** básico guardando energia, sendo carregado por cabo e entregando | Barra sobe/desce; o limite por tick aparece na dica da aba de energia |
| G9 | **LAN / vários jogadores** | Outro jogador abre as máquinas, as telas atualizam, a armadura dos slots é a dele |

## H. O que **não** é defeito (não precisa reportar)

- O modelo vestido das peças do MekaSuit, Hazmat, Jetpack, Free Runners e Scuba não existe; as peças só ocupam o slot.
- O jato do Jetpack, a absorção do Free Runners, módulos e energia do MekaSuit não existem.
- Free Runners, Jetpack e as versões "armored" aparecem como itens sem forma no inventário.
- O líquido nos tanques e medidores usa uma textura genérica tingida (não a do fluido original); o Crystallizer não
  mostra o resultado esperado na telinha escura; não há latas de gás nem conta-gotas.
- O texto "Status N" e a versão ("v0.1.31") nas telas das máquinas são de depuração e vão sair.
- Receitas que pedem uma etiqueta (tag) que o mod ainda não define foram descartadas, então algumas receitas existentes
  no original não aparecem.

## J. Rotary Condensentrator (mod 0.1.34)

| # | O que fazer | Esperado |
|---|---|---|
| J1 | Colocar o Rotary Condensentrator, energia numa face; abrir a tela | Slots de gás à esquerda, de fluido à direita, o de carga em cima à direita; medidores de gás (esquerda) e fluido (direita); botão quadrado no canto de cima à esquerda e o texto "Condensentrating" embaixo dele; título no alto, **sem** "Inventory" |
| J2 | Clicar no botão do canto | O botão vira ("flipped"), o texto passa a "Decondensentrating" e a seta do meio inverte de lado |
| J3 | No modo **Decondensentrating**, balde de água no slot de fluido da direita (155, 25) | O balde esvazia no tanque (balde vazio sai no slot de baixo) e a água vira **vapor d'água** no medidor da esquerda, com a seta acesa |
| J4 | Voltar ao modo **Condensentrating** | O vapor volta a virar água (gasta energia, 2 por tick) e o balde vazio no slot de fluido é enchido com a água do tanque |
| J5 | Janela de lados (aba da esquerda): Químicos, Fluidos, Itens, Energia | 4 abas, abre em Químicos; no modo descondensar só o gás sai, no outro só o fluido |
| J6 | Um Fluid Tank encostado: no modo condensar, a água do rotary passa para o tanque | O tanque recebe a água sem cano |

## K. Geradores (mod 0.1.35, **APK v36**)

Todos soltam energia só pela frente (o solar, pelo fundo): encoste um cabo ou uma máquina nessa face e veja a energia chegar.

| # | O que fazer | Esperado |
|---|---|---|
| K1 | Criativo: pegar os 4 geradores na aba do Mekanism e colocá-los | Os modelos aparecem com as texturas do Mekanism (o Bio com o tanque de vidro, o de Calor com a boca, o Solar achatado, o de Gás grande); viram para você |
| K2 | **Bio-Generator**: bio fuel (`bio_fuel`) no slot da esquerda | O medidor azul-claro-esverdeado à esquerda enche (64 por item) e o texto mostra "BioFuel", "Producing: 25 FE/t"; a barra de energia da direita sobe |
| K3 | **Heat Generator**: bloco de carvão (ou balde de lava) no slot da esquerda | O medidor largo enche de lava; a aba de calor (embaixo à esquerda) mostra a temperatura subindo até uns 488 K e "Producing" ~40 FE/t; bloco ativo troca de modelo (LED aceso) |
| K4 | Heat Generator com **lava ao lado** (sem combustível) | Esquenta sozinho devagar e produz um pouco |
| K5 | **Solar Generator** de dia, céu aberto | Ícone de sol amarelo, "Producing: 1 FE/t" na planície; coberto (um bloco em cima) ou de noite: ícone cinza e 0 |
| K6 | **Gas-Burning Generator** com hidrogênio (do separador por tubo, ou tanque criativo de químico) | Aceita hidrogênio/eteno (não oxigênio); queima até 10 mB/t, mais rápido cheio; "Burn Rate" na tela |
| K7 | Slots de armadura na tela dos geradores (suporte à esquerda) | Aceitam armadura como nos outros |
| K8 | Energia sai pela frente para um **cabo** ou **Energy Cube** encostado | Chega energia; pelos outros lados não |

## I. Como reportar

Para cada item: o número (A3, D7...), **OK** ou **errado**, e se errado: o que esperava, o que viu e, se der, uma
captura de tela. Se uma máquina "não faz nada", diga o texto de **Status N** que aparece nela:
0 parada (sem item), 1 trabalhando, 2 item sem receita, 3 sem energia, 4 faltando químico/espaço, 5 sem receitas carregadas.
