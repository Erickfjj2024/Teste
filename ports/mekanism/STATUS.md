# Mekanism no Horizon: o que foi testado, o que não foi, e o que está errado

Atualizado depois do teste do usuário com o APK v9 e o `mekanism.hmod` 0.1.5 (o cubo funciona; o núcleo girava só em um eixo, corrigido e confirmado no APK v11). A regra deste
arquivo: **só conta como "testado no jogo" o que o usuário viu funcionando no celular.** O que passou só
em teste simulado (os scripts `horizon-mod test` e os testes unitários) está marcado à parte.

Versão do mod citada: 0.1.2 (a 0.1.3 acrescenta pó para lingote no Smelter e as etiquetas do próprio Mekanism,
e **ainda não foi testada**). APK: v8.

## 1. Testado no jogo (o usuário viu)

| O quê | Resultado |
|---|---|
| Elevador em um arquivo só (blocos + código), APK v4 | funciona, nada quebrou |
| Abrir a tela de uma máquina (Crusher) com o APK v7 | abre. A tela é mais simples que a original (painel liso com barras, sem a textura de fundo) |
| Crusher | só processa **lingote de ferro** (vira pó). Pó, minério e ferro bruto não fazem nada, **o que é o certo no Mekanism** |
| Enrichment Chamber | processa **ferro bruto** e **minério de ferro**, como no original |
| Energized Smelter | processa **ferro bruto** e **minério de ferro** (receitas de fornalha do jogo) |
| Cabo de energia (Basic Universal Cable) | passa energia para as máquinas (APK v8) |
| Texto "Status N" na tela das máquinas | apareceu e mudou (viu o valor 2 = sem receita para o item) |
| Máquinas acendendo e animando ao trabalhar (APK v9) | funciona: trocam para a textura ativa animada e emitem luz |
| Energy Cube: formato (não tapa vizinhos), textura (APK v9, mod 0.1.5) | funciona |
| Núcleo do Energy Cube animado como o original: giro em dois eixos, balanço, brilho (APK v11, mod 0.1.6) | funciona, "perfeito" segundo o usuário |

## 2. Defeitos vistos pelo usuário (abertos)

1. **Energy Cube com textura errada.** O modelo atual monta a moldura de 28 peças do Mekanism com o núcleo,
   mas as texturas não batem com o original (o original usa um carregador de modelos próprio, com portas e LEDs).
2. **(mod 0.1.4/0.1.5: **confirmado pelo usuário**, só o giro do núcleo estava diferente)** O cubo agora usa o formato do baú do Ender (não tapa as faces dos vizinhos), a luz foi zerada e o núcleo ganhou a cor do nível, como no original. Falta o usuário confirmar. Antes: **A parte vazada do Energy Cube deixa invisíveis os blocos ao lado, acima e abaixo dele** (inclusive o chão
   debaixo). Causa provável: o bloco usa o formato do forno (cubo inteiro, que tapa as faces dos vizinhos), mas o
   modelo é oco; o jogo então não desenha as faces dos blocos encostados. Correção planejada: trocar o formato do
   bloco por um que não tape os vizinhos (por exemplo o do baú) ou dar um modelo sólido ao cubo. **Não corrigido.**
3. ~~**As máquinas não mudam de textura ao trabalhar, não acendem e não têm a textura animada**~~ **RESOLVIDO e confirmado pelo usuário no APK v9.** (o original troca para
   o modelo "ativo", animado, e emite luz). Funcionalmente elas processam, mas o bloco nunca vira `lit=true`. Hipótese:
   o jogo recusava a troca de propriedade (`native_set_block_property`) porque a sua checagem "este jogo é um servidor"
   não vale para o jogo do servidor interno do celular (a mesma checagem vale para `summon` e outras funções de
   entidade). Correção feita no jogo (APK v9, **ainda não testada no celular**).
4. A tela das máquinas é "mais simples que a original": só um painel liso, as barras de energia e de progresso, o
   texto de status e a versão. Falta o fundo e o aspecto do Mekanism. **Não é defeito de função, é visual.**

5. **Mod 0.1.8 travava na seleção de mundo e reiniciava o app (relatado pelo usuário, APK v12).** Causa provável (hipótese,
   sem log do celular): cada Energy Cube passou de 16 para 5832 estados (modelo do baú de Ender com `facing` e
   `waterlogged`, vezes 3 valores em 6 faces), 11.664 estados novos para os 35.727 do jogo; o jogo guarda uma linha por
   estado em tabelas grandes (`gameplay-facts.json` tem 55 MB), então a memória acabou. Mod 0.1.9: o bloco usa o modelo
   da mesa de encantamento (1 estado, formato que não tapa vizinhos), sem `facing`, e as faces são as do mundo
   (`side_down`... `side_east`): 729 estados por cubo, 1.458 no total. A frente do cubo novo é a que olha para o
   jogador mais próximo, escolhida no primeiro tick. Efeito colateral: a caixa de seleção do cubo tem 12/16 de altura
   (a da mesa de encantamento). **O app voltou a abrir com o 0.1.9 (confirmado pelo usuário), o que confirma a causa.**
6. **Mod 0.1.9 (relatado pelo usuário):** o cubo deixa invisível o bloco de baixo (o fundo da mesa de encantamento é um
   quadrado cheio e cobre a face do vizinho), o núcleo girando sumiu e o configurador não mudou nada na tela.
   Mod 0.1.10 e APK v13 (**não testados**): o bloco usa o farol como modelo (1 estado, caixa cheia, não oclui); o
   núcleo sumia porque mudar o estado do bloco faz o cliente refazer a entidade vazia e o servidor só reenviava quando
   a energia mudava (agora reenvia também quando o estado muda); a face do configurador cai para "a face virada para o
   jogador" se o olhar do jogador não for conhecido (hipótese: o jogo do servidor interno não tem a direção do olhar).

## 3. NÃO testado no jogo (o usuário não viu)

Tudo isto existe no mod 0.1.2/0.1.3, mas **ninguém viu funcionar**:

- **Minérios no mundo** (ósmio, estanho, chumbo, urânio, fluorita): a geração em chunks novos só passou em testes
  unitários (posição, altura, tamanho dos veios). Não se sabe se aparecem, onde, nem se a pedra vira minério.
- Blocos de armazenamento, Steel Casing, blocos brutos: aparência, textura, nome e se dá para minerar.
- Os 158 itens: texturas, nomes em inglês e a aba "Mekanism" do criativo (não foi relatado).
- Receitas de fabricação (os blocos e itens que se fazem na mesa): nada testado.
- Tabelas de saque: se os minérios soltam o minério bruto, se as máquinas soltam a si mesmas com a energia e os itens
  dentro (`keep-data`).
- Funil alimentando e esvaziando as máquinas, e o lado de cada face.
- O bloco acender (propriedade `lit`) quando a máquina trabalha, e a textura "ativa" animada: **visto que não acontece** (defeito 3); a correção do APK v9 não foi testada.
- Cubo básico de energia guardando e entregando energia (o Creative foi usado como fonte, mas o usuário relatou
  só o cabo e as três máquinas); o limite de 1024 por tick do cabo.
- As barras de energia e de progresso e as dicas ao passar o dedo: o usuário viu o texto de status, mas não relatou
  as barras.
- Pó de ferro no Smelter e o Crusher com "clump" (mod 0.1.3, não testado).
- Salvar e reabrir o mundo com máquinas, cabos e itens dentro.
- Jogar em LAN ou servidor dedicado, vários jogadores.

## 4. Os 14 blocos da API: o que o jogo real já exercitou

Só o que o Mekanism usou até agora, e só o que o usuário viu funcionar (cabos e as três máquinas):

| Bloco da API | Exercitado no jogo? |
|---|---|
| 1 Entidades de bloco | sim (as máquinas) |
| 2 Inventários (slots) | sim (entrada e saída das máquinas) |
| 3 Energia | sim (máquinas e cabo) |
| 4 Fluidos e químicos | **não** |
| 5 Componentes / `keep-data` | **não** |
| 6 Tipos de receita | sim (esmagar e enriquecer; fornalha para o Smelter) |
| 7 Menus e telas | sim, a tela abre e o status aparece (visual simples) |
| 8 Mensagens de rede | **não** |
| 9 Cabos e tubos | sim (o cabo de energia; tubos de fluido não) |
| 10 Multiblocos | **não** |
| 11 Renderizadores declarativos | **não** |
| 12 Entidades | **não** |
| 13 Itens com lógica | **não** |
| 14 Chunks e geração de mundo | **não** (a geração de minérios não foi vista) |
| `world.set-block-property` (o `lit`) | sim, depois da correção do APK v9 (as máquinas acendem e animam) |

Nenhum dos blocos "não" tem teste no jogo: só testes simulados e a compilação.

## 5. Aproximações e suposições feitas (a corrigir ou conferir)

- **Capacidade de energia das máquinas:** suposição (uso por tick × 2500). O Mekanism da 26.3 ainda está com esses
  valores por escrever (há um `TODO` no código dele). Valores reais usados: consumo por tick (Crusher 8, Enrichment
  5, Smelter 5), cubo básico (4 000 000, 4 000 por tick). Cabo básico: 1024 por tick, suposição.
- **Dureza e resistência dos blocos:** valores aproximados (minérios 3, deepslate 4,5, armazenamento 5, casing 3,5).
  Os do Mekanism não foram levantados.
- **Modelos do cubo de energia e do cabo:** feitos à mão com as texturas do Mekanism, porque o original usa um
  carregador de modelos que o Horizon não tem.
- **Itens "normais":** todos os 158 itens são itens comuns (um graveto com outro nome e textura). Os que têm
  comportamento no Mekanism (ferramentas, jetpack, tablet, configurador) não estão no mod.
- **Minérios:** quantidades por chunk, tamanho e alturas vêm do Mekanism (`OreType`), mas o minério entra **depois**
  que o chunk é gerado (pode ficar cortado na borda de chunks vizinhos).
- **Receitas:** vêm do Mekanism. Só as de esmagar, enriquecer, fornalha e as de fabricação de vanilla; as dos
  outros tipos (infusão, purificação, serrar, combinar, químicas) ficam para as máquinas que as usam.

## 6. O que falta do Mekanism

Quase tudo ainda: o resto das máquinas (Precision Sawmill, Combiner, Metallurgic Infuser, Purification Chamber,
Osmium Compressor, as fábricas de 3 a 9 linhas...), a parte química (tanques, infusor, oxidante, lavador,
cristalizador), geradores (solar, bio, calor, vento, gás), ferramentas e armaduras, upgrades, configuração de
lados, tanques de fluido, indução, turbina, reator, Robit, teleportador e mais. O módulo `generators`, `additions` e
`tools` do repositório ainda não foi tocado.

## 7. Como atualizar este arquivo

Depois de cada teste do usuário: mover o que ele viu funcionar para a seção 1, os defeitos que ele viu para a 2, e
deixar na 3 só o que ainda ninguém viu. Teste simulado nunca vira "testado no jogo".

## 8. Fidelidade visual: o "motor de texturas" do Mekanism

O usuário apontou (e tem razão) que o visual do Mekanism depende de um motor próprio de renderização. O que o
original usa, e o que o porte tem hoje:

| Recurso do Mekanism | No porte |
|---|---|
| Texturas animadas (`.mcmeta`) | **funciona** (as máquinas ativas animam, confirmado no jogo) |
| Estados do bloco trocando o modelo (`facing`, `active`) | **funciona** (confirmado) |
| Modelos JSON comuns (máquinas, minérios, blocos) | feito, copiados do original |
| Portas e LEDs do Energy Cube (mod 0.1.7) | **confirmado pelo usuário** ("ficou ótimo"): configuração padrão do original, frente = saída com LEDs acesos, outras 5 faces = entrada |
| **Configurador** (mod 0.1.9, APK v12, **não testado no jogo**; o mod 0.1.8 derrubou o app, ver defeito abaixo) | item `mekanism:configurator`: clicar numa face do Energy Cube passa o modo daquela face (desligada → entrada → saída → entrada/saída), agachado volta; a face é a que o olhar do jogador acerta (nova função `player-control.get-look`, API 16); o bloco mostra o modo (propriedades `side_*`: portas e LEDs mudam) e a energia só passa pelas faces conforme o modo. Só o modo de configuração de energia, só nos cubos; sem os outros modos do original (vazio, girar, chave inglesa) nem a aba de configuração na tela das máquinas |
| Carregadores de modelo próprios (cubo de energia, cabos, tanques, fábricas, modelos `.obj.mek`) | **aproximados à mão**: moldura do cubo sim, portas e LEDs por face não; cabos com núcleo e braços simples, sem os modelos grandes/pequenos do original |
| Núcleo do Energy Cube animado como o original (**confirmado**) | elemento novo `animated` do renderizador: escala 0,4, giro em Y a 4°/tick, giro em torno de (0,1,1) a 36°+4°/tick, balanço `sin(3°·t)/7`, brilho total, só aparece com energia |
| Partes desenhadas por código (TESR): núcleo do cubo, nível de fluido nos tanques, rotores, o Digital Miner | **mecanismo existe** (bloco 11: `cube`, `fluid` que sobe, `spin`), **nunca testado no jogo**; o núcleo girando do cubo é o primeiro uso (mod 0.1.5) |
| Cor por estado (`tintindex`: cor do nível, pigmentos, blocos coloridos) | **não há**: a cor do núcleo do cubo está gravada na textura, uma por nível |
| Brilho (`light_emission` por elemento, partes que brilham no escuro) | **não há** |
| Partículas, lasers, feixes, shaders | **não há** |
| Renderizadores de item (armadura, jetpack, ferramentas modelo 3D) | **não há** |
| Renderizadores de entidade (Robit) | **não há** (só displays de item) |
| Telas com o aspecto do original (fundo, botões, abas) | simples por enquanto; o mecanismo de telas declarativas aceita textura de fundo |

Ordem proposta para a fidelidade: (1) ~~núcleo `animated`~~ feito e confirmado; (2) ~~portas e LEDs do cubo~~ feito e confirmado, e o configurador (mod 0.1.8, falta confirmar);
(3) cor e brilho por estado; (4) tanques de fluido; (5) cabos com os modelos do original; (6) telas com o fundo do
original; (7) rotores, partículas e o resto conforme as máquinas chegarem.
