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

## 1b. Confirmado no plano de testes (APK v34, mod 0.1.31; `TESTES.md` respondido pelo usuário)

**Funciona (visto):** Metallurgic Infuser sem esconder vizinhos; carbono não entra no compressor (filtro de tanque); oxidador (carvão → carbono) e
o oxidador entregando ao infusor; infusor (cobre + redstone), compressor de ósmio, botão de despejo, dicas dos medidores; armadura vanilla
nos slots (certa entra, errada é recusada), armadura continua ao reabrir; água do Fluid Tank ao separador encostado e por Mechanical Pipe;
janela de lados do separador (4 abas); modo de edição do tanque, baldes (enchem e esvaziam, água e lava não se misturam); tanque criativo
("Infinite"); despejo e janela do Chemical Tank; Purification Chamber (3 clumps por minério); telas das 5 máquinas sem fonte de químico
(layout); Universal Cable dos 4 níveis, Pressurized Tube, Mechanical Pipe, regra de ligação e quebra no meio; minérios no mundo; funil;
salvar e reabrir; serra de precisão (com serragem por sorte), combinador; Energy Cube básico.

**Defeitos vistos e corrigidos no mod 0.1.32 / APK v35 (aguardam confirmação):**
- armadura nos slots fora do painel (cubo e tanques) **não entrava com o cursor simulado do jogo** (só pelo toque) e o item "aparecia em todos os
  slots" sem poder ser retirado: o clique fora da janela de 176 px era tratado como "fora" mesmo sobre um slot (um arrasto de empilhar);
- a cor dos medidores era branca para quase todo químico (redstone, hidrogênio...): o arquivo do químico não era encontrado;
- o texto dos botões de despejo do separador saía da tela (estava para o lado errado);
- uma Purification Chamber com o tanque de oxigênio vazio **perdia o progresso**; agora espera com ele (receitas por tick);
- a água no bloco do tanque ainda piscava (profundidade do celular): o líquido ficou mais para dentro do vidro;
- o ícone dos canos, tubos e cabos no inventário era só o núcleo (quase invisível): agora é uma peça reta.

**Ainda não visto:** minérios (cada um), blocos de armazenamento e Steel Casing, receitas na mesa, quebrar com conteúdo e recolocar, LAN,
peças do MekaSuit nos slots (B3), item do Fluid Tank (A2), cor no Chemical Tank (A4).

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

7. **Configurador sem nenhuma mensagem (relatado com o mod 0.1.10/APK v13).** Causa (lida no código): o jogo
   deixa o bloco reagir primeiro ao clique (`use_hand`), e só depois chama `item-used` do mod; o modelo do cubo (baú de
   Ender, farol) consome o clique, então o mod nunca era chamado. A documentação da API já dizia que o mod vem antes do
   bloco (como o `onItemUseFirst` do NeoForge). APK v14: o jogo chama o mod antes do bloco.
8. **Estados do cubo (ver 5):** o mod 0.1.11 volta o cubo a **um estado só**, como o original, que desenha as portas
   pelos dados da entidade do bloco. Isso é o elemento novo `part` do renderizador (API do fork): desenha o modelo de
   um bloco auxiliar (36 blocos de um estado, sem item) quando um byte dos dados do mod bate; o servidor manda ao cliente
   os primeiros 64 bytes dos dados. **Não testado no jogo.**
9. **Carregador de mods (medido no computador; confirmado no celular com o APK v17: mais rápido e estável).** Medição: montar a raiz de
   conteúdo (feita quando o conjunto de mods muda; depois fica em cache) tinha **pico de 1,2 GB** com vanilla + Mekanism, e
   só carregar o resultado usa 140 MB. A causa não era o número de estados do Mekanism (cada 11.700 estados a mais
   custavam ~120 MB), e sim ler a tabela `gameplay-facts.json` (55 MB de texto) como árvore JSON (~1 GB) e fazer isso
   em 3 threads. Agora as linhas dos 35.727 estados do jogo ficam como texto e só as linhas dos blocos-modelo viram
   árvore (`J::Raw` em `mod_content.rs`), com 2 threads. Resultado: pico **322 MB** (era 1.198 MB), 3 s em vez de 7 s, e
   o mod com 11.700 estados do 0.1.8 monta com 500 MB (antes 1.318 MB, que derrubou o app). A raiz gerada é igual à
   de antes arquivo por arquivo (só espaços diferem em `facts.json`). Falta: as tabelas NBT (oclusão, colisão,
   movimento) ainda são árvores e o custo por estado continua ~10 KB.

11. **APK v16: as portas e LEDs do cubo sumiram (relatado), com o núcleo e o configurador funcionando; no v15 apareciam.**
    A raiz montada pelo carregador novo era igual à do antigo em todos os arquivos, menos o `facts.json`, que tinha
    `\u003d` no lugar de `=` (mesmo JSON). Não achei no código quem dependa disso, nem consegui reproduzir no computador
    (catálogo de modelos, itens e colisões carregam igual). APK v17: a saída do carregador novo é **idêntica byte a byte**
    à do antigo (as linhas originais são reescritas uma por vez), 327 MB de pico, e a raiz em cache é refeita na primeira
    abertura (o selo do cache passou a `2`; ele não mudava com o carregador, então um APK novo podia reaproveitar a
    raiz de outro). **Confirmado pelo usuário no celular: as portas voltaram no APK v17.** A causa exata não foi isolada (o `\u003d` do `facts.json` ou a raiz em cache de outro APK); as duas foram eliminadas.

## 2b. Fatia 2: Precision Sawmill e Combiner (mod 0.1.16, **não testados no jogo**)

Só passaram no teste simulado (`horizon-mod test`: serrote faz tábuas e serragem, combinador faz cascalho) e nos testes unitários.
- **Precision Sawmill:** 1 entrada, 200 ticks, 5 de energia por tick, saída principal e secundária **com chance** (a chance do
  Mekanism: toras 25 %, etc.); para rodar precisa de espaço nas duas saídas. A "sorte" é determinística por tick e posição.
  Receitas `mekanism:sawing` do Mekanism (38 arquivos, com portas, camas, barcos...).
- **Combiner:** 2 entradas (principal e extra) para uma saída, 200 ticks, 5 de energia por tick; receitas `mekanism:combining`
  (8). A ordem das entradas importa, como no original.
- Telas no layout lido do código do original (`GuiPrecisionSawmill`, `GuiCombiner`): quadro largo das duas saídas no serrote,
  slot de energia e slot extra (moldura laranja) no combinador. O slot de carga é só moldura.
- Tags de convenção criadas para as receitas (tintas, pedregulhos, trigo, abóbora, cercas, estantes).
- **Aproximações:** funil: serrote recebe por cima e pelos lados e esvazia por baixo; combinador recebe a entrada principal por
  cima, a extra pelos lados e esvazia por baixo (no original os lados são configuráveis). Sem fábricas, sem upgrades.

## 2c. Configuração de faces de todas as máquinas (mod 0.1.17, APK v19, **não testada no jogo**)

Passou no teste simulado (13 passos: botões da janela, eject, limpar, o Configurador, a energia seguindo as faces) e nos
testes unitários; **ninguém viu na tela do celular**. O que existe, lido do código do Mekanism:
- **Janela "Side Configuration"** (156x135, `GuiSideConfiguration`) aberta pela aba da esquerda da tela da máquina (a aba
  `button/configuration`): abas de **Items** e **Energy** (as dos outros tipos, como no original), os 6 botões em cruz
  (frente, esquerda, direita, trás, topo, base) coloridos pelo tipo de dado, o botão de **ejeção automática** (só itens), o
  botão de **limpar lados** e os textos "Items Config", "Eject: On/Off", "Can't Eject" (energia) e "Slots". Clique = próximo tipo,
  clique direito (toque longo) = o anterior. Cores e nomes são os do `DataType`/`EnumColor` (entrada vermelho escuro, saída azul,
  entrada/saída roxo, energia verde, extra amarelo, nenhum cinza claro).
- **Tipos suportados por máquina**, na ordem do original: itens = nenhum, entrada, saída, entrada/saída, energia (slot de carga), e
  **extra** no combinador; energia = nenhum e entrada. **Configuração padrão do original** (`LightConfigInfo.MACHINE`/`EXTRA_MACHINE`):
  itens entram por todos os lados, saem pela **direita**, o slot de carga fica **atrás** e o extra do combinador **embaixo**; energia entra
  por todos; ejeção desligada. Os lados são **relativos à frente** da máquina (`facing`).
- **Funcionamento:** o funil só alcança o que o tipo de cada face expõe (faces por entidade, `world.set-item-faces`, API 17);
  **ejeção automática** de itens a cada 10 ticks pelas faces de saída para o contêiner vizinho (`world.set-item-eject`, API 17); a
  energia só entra pelas faces com "entrada". A configuração é guardada na própria máquina (e vai junto no item que ela solta).
- **Configurador como o original:** clicar numa face **só diz** o comportamento ("Current Items behavior: Output (Blue)"); **agachado**
  passa para o próximo ("Items behavior bumped to: ..."); agachado no ar troca o que ele configura (Itens/Energia, "Configure State:
  Configurate (Energy)"). Vale para as máquinas e para o Energy Cube (só energia). **Mudança de comportamento do cubo:** antes o clique
  já mudava a face; agora é preciso trocar o Configurador para Energia (agachado no ar) e clicar agachado.
- **Serragem:** a "sorte" da serragem do Precision Sawmill agora é sorteada de verdade a cada receita (um gerador que guarda o estado,
  iniciado pelas posições dos jogadores e pelo tick), como o `level.random` do Java, e não mais determinística.
- **Motor de telas do jogo (fork):** telas ganharam **janelas** sobre o painel, grupos condicionais (`show`), amostras de cor por valor
  (`swatch`), botões com ação local (abrir/fechar janela, variável da tela) e um segundo id para o clique direito, e valores `nibble`.

**Modelo genérico (mod 0.1.21/0.1.22, **não testado no jogo**):** a configuração guarda 5 transmissões (itens, energia, fluidos, químicos,
calor) com 10 tipos de dado cada, 35 bytes (a de 13 bytes antiga ainda é lida); a janela gera uma aba por transmissão que a máquina
suporta (hoje só Itens e Energia) e o Configurador troca o modo na ordem do original (itens, fluidos, químicos, energia, calor).
As abas de fluido/gás/calor só aparecem quando existir uma máquina com esses tanques; a ligação aos tanques vem com a primeira.
**Janela (APK v26, **não testada**):** o **ícone do bloco vizinho** aparece em cada botão de lado (`neighbor`, na orientação da
máquina) e a janela **arrasta** pela borda de cima (18 px); os itens sob a janela já não aparecem nem recebem clique.

**Ainda não tem** (do original): aba de **Transporter Config**, as abas de redstone, segurança, upgrades e energia; o **fixar** da
janela; configuração de **fluidos e químicos** na prática (não há tanques no porte); os modos **Vazio, Girar e Chave inglesa** do
Configurador.

**Tela do Energy Cube (mod 0.1.23, APK v27, **não testada no jogo**):** `GuiEnergyCube` lida do código: medidor largo de energia em 55, 18
(64x48, energia ladrilhada), slot de **descarga** em 17, 35 (com o menos) e de **carga** em 143, 35 (com o mais), aba de configuração e aba
de energia ("Input Rate", "Max Output", "Unit: Forge Energy") à esquerda. A janela de lados do cubo tem a aba de **Energia**, relativa à
frente do cubo (guardada no byte 7 dos dados; cubos antigos usam a face que sai). O slot de descarga converte **redstone (10000) e bloco de
redstone (90000)** em energia, um por tick, como as receitas `energy_conversion`. Vale para o básico e o criativo.
**Slots de armadura (mod 0.1.24, APK v28, **não testados**):** o `armorSideBar(180, 41, 0)` do original: o suporte `GuiSideHolder` à direita
(176, 36) com capacete, peitoral, calça, botas e a mão secundária em 180, 41 e 18 abaixo. São slots do **próprio jogador** (não do
bloco): o menu ganhou o campo `armor` (`{x, y, offhand}`), o servidor os acrescenta depois dos slots do contêiner (`Slots::read/write`,
`View::worn`: só a peça certa em cada um, uma por slot, a mão secundária aceita qualquer coisa) e o cliente os liga aos ids de slot
500 a 504, com os ícones de slot vazio. **Armaduras do Mekanism (0.1.25):** as peças de MekaSuit, Hazmat, Jetpack, Free Runners e Scuba agora são equipáveis (`minecraft:equippable` com o slot de cada uma), então entram nesses slots e nos da pessoa; **o modelo vestido** delas (`mekanism:equipment/*`) e o que cada peça faz (energia, jato, queda) **não existem ainda**. **Falta nesse trecho:** o arrastar (distribuir) e o shift-click mandam para esses slots como
qualquer outro, sem a regra de prioridade do `MekanismContainer.quickMoveStack`.

**Falta:** o slot de **carga** não faz nada (o port não tem itens que guardem energia); a aba de **Itens** do cubo; as abas de redstone/segurança; funil nas faces do cubo; o medidor mostra números crus (sem kFE).

**Defeito visto no teste da 2c (mod 0.1.18 corrige, não testado):** ao clicar agachado numa máquina, o Configurador mudava a face e
também o modo dele. O cliente manda um uso no bloco e logo depois um uso no ar para o mesmo clique; o segundo, agachado, era lido como
a troca de modo. Agora o mod ignora o uso no ar até 2 ticks depois de um clique que ele aceitou num bloco (no original o clique numa
máquina não chega ao uso no ar). Clicar agachado num bloco que o Configurador não configura continua trocando o modo, como no original.

**Resultado do teste da 2c (usuário, APK v19): a janela abre, mas nenhum botão dela funciona (as cores não mudam, a ejeção não liga),
os itens do inventário aparecem por cima da janela e não há seletor entre Itens e Energia.** Defeitos achados (APK v20 e mod 0.1.19,
**não testados**): (1) as abas de tipo ficam fora do retângulo da janela e o clique só procurava botões da janela dentro dela, por isso o
seletor nunca respondia (corrigido, com teste); (2) os slots sob a janela continuavam desenhados e recebiam os cliques antes dos
botões: agora a janela cobre os slots que ela tapa (não são desenhados nem clicáveis, como no Mekanism). **Causa dos botões que mandam
ao servidor (lados, ejeção, limpar) não achada:** a lógica de clique com o JSON real da tela dá o id certo e o teste simulado do mod
passa, então o defeito está no caminho do jogo; o APK v20 e o mod 0.1.19 escrevem no chat "(menu button N sent to window W)" quando o
cliente manda e "(config button N reached the mod)" quando o mod recebe, para ver onde o clique para. **Falta de fidelidade assumida:** a
janela só tem Itens e Energia porque as máquinas atuais só têm isso no Mekanism; fluidos e químicos (gás) entram com as primeiras
máquinas que têm tanques (a janela e o modelo de configuração devem ser genéricos para qualquer tipo de transmissão).

**Causa achada dos botões da janela (APK v23 corrige, não testado):** com as mensagens de diagnóstico (APK v22) o clique chegou ao servidor,
foi aceito e enfileirado para o mod, e parava em "o jogador do botão não é um que os mods conhecem": o evento era guardado com o número
da conexão do convidado e, na hora do tick, o jogo traduzia esse número pela lista de convidados, que não devolvia o jogador. O uso de
item (`acting_server_player`) já achava o jogador pelo id dele. Agora os eventos de botão de menu e de bloco guardam o id do jogador que
age. (Os eventos de bloco quebrado/colocado tinham o mesmo defeito e nunca chegavam aos mods em partida de um jogador.)

**Janela de configuração, confirmado pelo usuário (APK v24):** os botões das faces **funcionam** (clique, cor, dados voltam à tela). A causa
era o jogador do botão: o servidor trata o clique do botão fora da troca que faz do convidado o jogador que age, então a identidade que
o jogo dava era a do dono da partida. Agora o evento guarda o convidado da conexão do pacote (e, com um só jogador, ele é o jogador).
Visto em seguida: o rótulo "Inventory" do jogo aparecia por cima da janela (o jogo desenha o título e esse rótulo depois da tela);
APK v25 e mod 0.1.20 (**não testados**): a tela diz `own_labels` e o jogo deixa os dele de fora (os rótulos são `text` da tela, que a
janela cobre). Mensagens de diagnóstico removidas.

## 2d. Metallurgic Infuser e o primeiro tanque de químico (mod 0.1.26, APK v30, **não testado no jogo**)

Passou no teste simulado (23 passos: o ferro/cobre + redstone, a conversão na ranhura de infusão e o botão de despejo) e nos testes
unitários; **ninguém viu na tela do celular**. O que existe, lido do código do Mekanism:
- **Bloco** `metallurgic_infuser` (modelo e texturas do original, `facing`, mesma carga/recipientes das outras máquinas, 5 de energia por tick).
- **Receitas** `mekanism:metallurgic_infusing` (10: ligas, musgo, terra, esporos...) e `mekanism:chemical_conversion`: um item na
  **ranhura de infusão** (17, 35) vira tipo de infusão no **tanque** (1000 mB no máximo, um item por tick, só do tipo que alguma receita usa
  e sem misturar), o item de entrada (51, 43) e a quantidade pedida fazem o resultado (109, 43) em 200 ticks. As receitas por tick
  (`per_tick_usage`) não existem em nenhum dado do mod e são ignoradas.
- **Tela** `GuiMetallurgicInfuser`: barra de energia, **barra do tanque** em 7, 15 (colorida pelo tipo: bio, fungi, ouro, obsidiana refinada,
  redstone, diamante, estanho e carbono, com a dica "Nome: n mB"), seta `RIGHT` em 72, 47 e o botão de **despejo** em 16, 59 (esvazia o tanque).
- **Configuração:** aba de **Químicos** (a primeira vez que a janela tem mais de duas): nenhum/entrada/saída/entrada e saída por lado,
  padrão entrada em todos os lados, sem ejeção; os lados do tanque seguem a configuração (`tank_faces`); itens como o combinador (extra embaixo).
- **Falta:** a barra de químico desenha o ícone parado (o original anima), o **Energy Tab** (-26, 137) das máquinas, o fixar/anexar de
  químicos por tubos (não há tubos), a barra de **guia de receitas** e os efeitos de som/partícula.

## 2e. Cadeia de químicos: 6 máquinas novas num motor só (mod 0.1.28, APK v31, **não testada no jogo**)

Passou no teste simulado (31 passos: oxidador, compressor de ósmio, cristalizador, além do infusor) e em 48 testes unitários do mod;
**ninguém viu na tela do celular**. Todas as máquinas que trabalham um item e um químico (infusão, gás, lama) passam agora por um
motor (`process.rs`) com quatro formas: item+químico→item (**Metallurgic Infuser, Purification Chamber, Chemical Injection Chamber,
Osmium Compressor**), item→químico (**Chemical Oxidizer**), item+químico→químico (**Chemical Dissolution Chamber**) e químico→item
(**Chemical Crystallizer**). Do código do Mekanism:
- **Tanques:** 1000 mB (infusor), 210 (câmaras e compressor), 10000 (oxidador, dissolução, cristalizador); um item no slot de enchimento
  vira químico (`mekanism:chemical_conversion`, um por tick) se alguma receita da máquina usa aquele químico.
- **Receitas por tick** (`per_tick_usage`: purificação, injeção, compressor, dissolução): gastam o químico aos poucos durante os 200 (100)
  ticks, esperando sem perder o progresso quando o tanque falta; as câmaras e a dissolução gastam **por sorte** (`StatUtils.inversePoisson`,
  média 1 por tick, a configuração `randomizedConsumption` ligada) e o compressor 1 por tick. As outras gastam tudo no fim.
- **Tempos e energia:** 200 ticks (infusor, câmaras, compressor, cristalizador), 100 (oxidador, dissolução); 5, 10, 15, 15, 15, 16, 15 por tick.
- **Telas:** os layouts do `Gui*` de cada uma (slots, barras de energia vertical/horizontal, setas de progresso `BAR`/`RIGHT`/`LARGE_RIGHT`,
  medidores `STANDARD` 16x58 e barras de químico), com o novo elemento **`gauge`** do motor de telas: o tanque é preenchido de baixo para
  cima com o ícone do químico ladrilhado e **tingido** com a cor dele (as 34 do `mekanism/chemical/*.json`) e a dica "Nome: n mB".
- **Configuração:** cada máquina tem a aba de **Químicos** com os tipos e o padrão do seu `AttachedSideConfig` (entrada em todos os lados;
  saída à direita no compressor, na dissolução e no oxidador; ejeção ligada nos dois últimos).
- **Blocos abertos:** o infusor, o oxidador, a dissolução e o cristalizador têm modelos com vãos; usam um bloco que não oculta os vizinhos.
- **Falta:** a tela do cristalizador não mostra o resultado esperado (`GuiCrystallizerScreen`); os slots de item químico (gás/lama) só
  convertem (não há tanques portáteis); a barra de recipe viewer; o Energy Tab; o **Chemical Infuser, o Washer e o Electrolytic Separator**
  (vários tanques e fluidos: próxima fatia); as receitas cujos itens de entrada usam tags que o mod ainda não define são descartadas.

## 2f. Tanques, tubos, canos e as máquinas de vários tanques (mod 0.1.29, APK v33, **não testado no jogo**)

Passou no teste simulado (40 passos: balde de água no tanque → separador → hidrogênio e oxigênio, infusor de químicos, lavadora,
tanque criativo) e em 57 testes unitários do mod; **ninguém viu na tela do celular**. O que existe, lido do código do Mekanism:
- **Filtro de tanque (API 18, `world.set-tank-filter`):** o tanque de entrada de uma máquina aceita de fora só os químicos que as receitas
  dela usam (antes um oxidador mandou carbono para o compressor de ósmio, que não o usa: o original não deixa).
- **Fluid Tank** (básico, avançado, elite, supremo e criativo: 32/64/128/256 baldes): slots de balde (146, 19) e (146, 51), medidor largo,
  a aba de **modo de edição** (encher e esvaziar / só encher / só esvaziar, que fecha as faces do tanque como o modo diz); balde de água ou
  de lava entra no tanque e sai vazio, balde vazio é enchido do tanque; o líquido aparece dentro do bloco (renderer `fluid`).
- **Chemical Tank** (64, 256, 1024, 8192 baldes e criativo): barra horizontal de 116 px, tela interna com o nome do químico e a quantidade,
  modo de despejo (parado / despejando / excesso acima de 85%), configuração de lados (químico e itens); **o criativo nunca esvazia**: o
  que entrar nele o enche de novo (é como se enche com um químico de teste, pois não há latas de gás).
- **Pressurized Tube** (químicos), **Mechanical Pipe** (fluidos) e **Universal Cable** nos quatro níveis: 750/2000/64000/256000,
  250/1000/8000/32000 mB e 8000/128000/1024000/8192000 por tick; cada um junta o que guarda aquilo que carrega (`*_connectable`).
- **Chemical Infuser** (dois químicos fazem um terceiro, nos dois lados), **Chemical Washer** (fluido + lama suja, com slots de balde no
  suporte da esquerda) e **Electrolytic Separator** (água em hidrogênio e oxigênio, botões de despejo de cada gás): operam todo tick,
  com os medidores, as setas ligadas quando trabalham e a janela de lados com **todas** as abas deles (Químicos, Fluidos, Itens, Energia,
  na ordem do original, e a janela abre na primeira).
- **Os slots de armadura** também estão nas telas dos tanques (suporte à esquerda).
- **Falta:** o medidor de líquido usa o ícone genérico tingido (o original usa a textura do fluido); as telas dos itens químicos (latas,
  conta-gotas) e a bomba/outras fontes de água; modo de ejeção dos itens dos tanques; o `GuiCrystallizerScreen`; gás no despejo
  do separador respeita só o modo básico (não o "excesso" para a máquina parar).

## 2g. Rotary Condensentrator (mod 0.1.34, usa o APK v35, **não testado no jogo**)

Passou no teste simulado (44 passos: vapor d'água vira água, o botão de modo inverte e a água volta a vapor) e em 58 testes unitários;
**ninguém viu na tela do celular**. Lido do código do Mekanism (`TileEntityRotaryCondensentrator`, `GuiRotaryCondensentrator`,
`AttachedSideConfig.ROTARY`):
- Dois tanques de 10 baldes (fluido e gás) e **modo** (condensa: gás vira fluido; descondensa: fluido vira gás), receitas `mekanism:rotary`
  lidas nos dois sentidos (as etiquetas `#c:hidrogênio` etc. viram os fluidos `mekanism:*`); 2 de energia por tick.
- Só o tanque que entra aceita de fora e só o que sai entrega, conforme o modo (`mode`/`!mode` do original); cada um empurra para os
  vizinhos só com a ejeção ligada. Configuração padrão do original: gás entra em todo lado e sai pela esquerda, fluido entra em todo lado e
  sai pela direita, ambos com ejeção; abas Químicos, Fluidos, Itens, Energia.
- Tela: slots de gás (5, 25 e 5, 56), de fluido (155, 25 e 155, 56) e de carga (155, 5); medidores em 25,13 (gás) e 133,13 (fluido); seta
  para a direita (condensa) ou para a esquerda (descondensa) em 64,39; **botão de alternar** (4, 4) com o nome do modo escrito embaixo; título
  em y=4 e sem "Inventory", como o original.
- Slots de fluido: balde cheio esvazia no tanque quando o modo é descondensar; balde vazio é enchido quando é condensar (só água e lava).
- **Falta:** os slots de gás (latas de gás não existem no porte, então ficam sem função); a aba de energia (-26,137); o único par que dá
  para alimentar no jogo é água ↔ vapor d'água (os outros fluidos de Mekanism não têm fonte ainda).

## 2h. Geradores: Bio, Heat, Solar e Gas-Burning (mod 0.1.35, APK v36, API 19, **não testados no jogo**)

Passou no teste simulado (46 passos: bio fuel vira bioethanol e queima 1 mB por tick, bloco de carvão vira lava e o gerador de calor
queima 100 mB por tick, o solar vê o sol ao meio-dia, o de gás queima 10 mB de hidrogênio por tick) e em 63 testes unitários;
**ninguém viu na tela do celular**. Lido do código 26.3 do `mekanismgenerators` (`TileEntityBioGenerator`, `HeatGenerator`,
`SolarGenerator`, `GasGenerator`, `GuiXxxGenerator`):
- **Visual:** o checkout 26.3 só tem o Java do módulo, então modelos, blockstates e texturas vêm do **jar 1.21.1 do MekanismGenerators**
  enviado pelo usuário (`tools/generators-assets/`, MIT); o modelo composto do Bio-Generator (`neoforge:composite`) foi achatado num só.
  Os ids são `mekanism:bio_generator` etc. (um mod só declara blocos no próprio namespace; no original são `mekanismgenerators:`).
- **Bio-Generator:** item de bio fuel (64 mB cada, 576 o bloco) vira bioethanol no tanque de 200 mB; gera 25 por tick gastando 1 mB;
  capacidade de energia 600. Medidor estreito à esquerda, tela interna com energia, bioethanol e produção.
- **Heat Generator:** tanque de 1000 mB de lava (balde de lava ou combustível comum: carvão, bloco de carvão, madeira, graveto, vara de
  blaze... na proporção do tempo de queima de um balde de lava, 20000); queima 100 mB por tick para +100 de calor; a lava ao lado (+7 por
  face) e o Nether (+10) esquentam sem combustível; capacitor de calor 10, perda 0,5 por kelvin, eficiência de Carnot, perda para o ar com o
  isolamento de cada face (o fundo sem isolamento); aba de calor com a temperatura. Estabiliza perto de 488 K fazendo ~40 por tick.
- **Solar Generator:** 2 por tick vezes o brilho do sol e o multiplicador do bioma (temperatura e chuva), cortado para inteiro como o
  original (na planície, 1 por tick); vê o céu varrendo os blocos acima; ícone de sol na tela.
- **Gas-Burning Generator:** tanque de 1000 mB que aceita só combustíveis (hidrogênio 2 por mB, eteno 8 por mB, até 10 mB por tick, mais
  rápido quanto mais cheio); recebe de tubos/tanques/separador; o texto "Burn Rate" da tela.
- **Faces:** a energia sai só pela frente (pelo fundo no solar); o tanque aceita por todas as faces menos a da frente; o slot de
  combustível por todas menos a direita, a do slot de energia.
- **API 19 (`world.get-environment`):** hora do dia, chuva, trovão, temperatura, umidade e precipitação do bioma e se há céu, para o solar;
  exige o **APK v36**.
- **Falta / diferenças:** o Advanced Solar e o Wind Generator (blocos grandes que precisam de "bounding blocks", que a API ainda não tem);
  os slots de energia dos geradores (não há itens de energia, como o tablet); o ícone de sol é desenhado à mão (os sprites
  `state/sees_sun` e `no_sun` não estão em nenhuma das fontes); receitas: só as com ingredientes que o porte tem (o bio e o de calor;
  o solar pede painel solar e tablet de energia, o de gás o núcleo eletrolítico); a tabela de queima do Heat Generator é uma lista dos
  combustíveis comuns (não há consulta de tempo de queima na API); perda de calor por face mostrada arredondada; ambiente do calor
  fixo em 300 K (o original varia com o bioma); luz do bloco ativo e partículas (fumaça/chama) não existem.

## 2i. Controle de redstone e aba de energia (mod 0.1.36, APK v37, API 20, **não testados no jogo**)

Passou no teste simulado (50 passos: um Crusher no modo "normal" não trabalha sem sinal e trabalha quando o sinal chega) e em 66 testes
unitários. Lido do código 26.3 (`TileEntityMekanism.isRedstoneActivated`, `GuiRedstoneControlTab`, `GuiEnergyTab`):
- **Aba de controle de redstone** (176, 137; 26 por 18; cor 0xC9071F) em **todas as máquinas e geradores**: clique esquerdo passa para o
  próximo modo, direito volta: ignorado, normal (trabalha com sinal), invertido (trabalha sem sinal), pulso (trabalha só no tick em que o
  sinal chega). O modo fica salvo na máquina. Parada pela redstone, a máquina descansa onde está (progresso, tanques e itens ficam).
  Nos geradores, a redstone só impede de gerar (o calor do Heat Generator continua esfriando).
- **Aba de energia** (-26, 137) das máquinas: "Using: N FE/t" (o uso dela enquanto trabalha), "Needed: faltando para encher" e "Unit: Forge Energy".
- **API 20 (`world.get-redstone`):** o sinal de redstone que chega a um bloco do mod, 0 a 15 (o maior dos seis vizinhos), lido no começo do
  tick; exige o **APK v37**.
- **Diferenças:** o original só pára a parte que processa e deixa o resto (trocar itens de lata por tanque) trabalhar; aqui a máquina
  inteira descansa. Tanques de fluido/químico e Energy Cubes não têm a aba (também não têm no original).

## 2j. Upgrades (mod 0.1.37, APK v38, API 21, **não testados no jogo**)

Passou no teste simulado (55 passos: dois upgrades de velocidade num Crusher instalam em um segundo e cortam a receita de 200 para 112
ticks; o upgrade de gás é recusado e fica no slot; o botão da janela devolve um upgrade) e em 71 testes unitários; **ninguém viu na tela
do celular**. Lido do código 26.3 (`TileComponentUpgrade`, `UpgradeUtils`, `GuiUpgradeWindow`, `GuiUpgradeScrollList`,
`GuiSupportedUpgrades`, `AttributeUpgradeSupport` e os tags `upgrade_support`):
- **Os 8 itens** (velocidade, energia, químico, silenciamento, contenção de XP, âncora, filtro e gerador de pedra), com a textura, o nome
  ("Speed Upgrade"...) e a receita do original. **Diferença:** no original é um item só (`mekanism:upgrade`) com o tipo num componente; aqui
  cada tipo é um item (`mekanism:upgrade_speed`...), porque a API do Horizon não tem componentes de dado de mod no modelo do item.
- **A aba de upgrades** (176, 6; 26 por 18) em todas as máquinas e geradores abre uma **janela** de 198 por 94 (arrastável como a de
  configuração) com: título, lista dos instalados (até 4 linhas, cada uma na cor do upgrade, a escolhida acesa), a tela de descrição ("No
  selection." ou o nome, "Amount: n/máx" e "Effect: ...x" em letra pequena a 60%), a seta que enche enquanto instala, o botão
  "Uninstall" (toque esquerdo tira um, **toque direito tira todos**: no original é com Shift), os dois slots (entrada e saída) dentro da
  janela e a fileira "Supported:" com todos os ícones, os não suportados esmaecidos.
- **Efeitos:** instala em 20 ticks, até o máximo (8 para velocidade, energia e químico; 1 para os outros). **Velocidade:** receita em
  `max(1, base × 10^(-n/8))` ticks e uso de energia `ceil(base × 10^(2·vel/8 − energia/8))`; nas máquinas de fluxo contínuo (infusor, lavadora,
  separador, rotary) é `2^n` operações por tick. **Energia:** uso menor e capacidade `base × 10^(energia/8)` (API 21: `set-energy-capacity`).
  **Químico:** a média do gasto aleatório sobe/desce (`10^(2·vel/8 − gás/8)`) nas três câmaras que o aceitam.
- **Quem aceita o quê:** velocidade, energia, silenciamento e XP nas máquinas de processo; mais químico na Purification Chamber, na
  Injection Chamber e na Dissolution Chamber; só silenciamento nos geradores. Um upgrade que a máquina não aceita nem entra no slot.
- **Fork (APK v38):** `MenuType.accepts` (slot que só aceita certos itens, checado no servidor), `Window.slots` (slots que só existem
  com uma janela aberta, no lugar dela e seguindo-a quando arrastada), `Widget::Text.scale` e a API 21 (`world.set-energy-capacity`).
- **Falta / diferenças:** silenciamento e contenção de XP são só guardados (o porte não tem som nem experiência); a lista não rola (a
  quinta linha, só nas câmaras com os cinco tipos instalados, fica escondida); o realce ao passar o dedo na lista não existe; instalar
  com Shift+clique direito do item no bloco não existe (só pelo slot da janela); a dica "Using" da aba de energia mostra o uso básico, não o
  com upgrades; o ritmo de despejo do Electrolytic Separator não acompanha a velocidade; os três upgrades que nenhuma máquina portada
  aceita (âncora, filtro, gerador de pedra) existem mas não têm uso.

## 2k. Pressurized Reaction Chamber e as máquinas de pigmento (mod 0.1.39, APK v40, **não testados no jogo**)

Passou no teste simulado (60 passos: carvão + água + oxigênio viram enxofre e hidrogênio em 100 ticks; o extrator faz pigmento de um corante,
a pintora pinta uma lã com ele, o misturador faz vermelho-escuro de cinza e vermelho) e em 75 testes unitários; **ninguém viu na tela**.
Lido do código 26.3 (`TileEntityPressurizedReactionChamber`, `GuiPRC`, `TileEntityPigmentExtractor`, `TileEntityPaintingMachine`,
`TileEntityPigmentMixer` e as telas):
- **Pressurized Reaction Chamber:** item + fluido + gás viram item + gás; cada receita traz a duração e a energia extra por tick
  (`PRCEnergyContainer`: base 2 mais a da receita); três tanques de 10 baldes (fluido, gás que entra, gás que sai); as 14 receitas
  `mekanism:reaction` (gaseificação do carvão e da madeira, substrato); configuração de lados (Itens, Químicos, Fluidos, Energia) como
  `AttachedSideConfig.REACTION`; upgrades de velocidade e energia valem; a tela é 5 pixels mais alta que as outras, como a original.
- **Pigment Extractor:** item (corante, lã, tapete, cama...) vira pigmento, 5 de energia, 100 ticks, tanque de 20 baldes; as 275 receitas.
- **Painting Machine:** item + pigmento viram o item pintado, 5 de energia, 200 ticks, tanque de 15 baldes; as 288 receitas (lã, tapete, cama, vidro,
  concreto, terracota, vela, estandarte...).
- **Pigment Mixer:** dois pigmentos viram um terceiro a cada tick, tanques de 1, 1 e 2 baldes; as 16 receitas.
- **Fork:** os ingredientes `neoforge:intersection` e `neoforge:difference` (as receitas de pintura e extração os usam) e as tags
  `c:dyed/<cor>` (geradas aqui com os itens de cada cor, no lugar das do NeoForge).
- **Diferenças / falta:** as setas de progresso do extrator, da pintora e do misturador mudam de cor com o pigmento no original (degradê) e aqui
  são sempre as mesmas; a energia do topo do misturador fica ligada (no original o topo é desativado: `addDisabledSides`); os slots de pigmento
  (lata) e a aba de energia (-26,137) só dão a dica básica; cada máquina nova usa o modelo e as texturas do original.

## 2l. Sal no mundo e baldes de fluidos e gases (mod 0.1.40, o mesmo APK v40, **não testados no jogo**)

Passou no teste simulado (62 passos: um balde de salmoura esvazia num tanque de fluido e o balde vazio sai) e em 77 testes unitários.
- **Sal (`SaltConfig`):** em chunks novos do overworld, 2 discos por chunk de raio 2 a 3 (altura 1 acima e abaixo) no fundo de lagos e oceanos
  (com água em cima), onde há terra ou argila vira bloco de sal. O centro fica a 3 blocos da borda do chunk para o disco não passar para o vizinho
  (no original os discos passam). Sal só aparece em mundo novo ou em região ainda não explorada.
- **Baldes (18):** salmoura, cloro, eteno, água pesada, ácido fluorídrico, hidrogênio, cloreto de hidrogênio, lítio, pasta nutritiva, oxigênio,
  sódio, vapor, dióxido e trióxido de enxofre, ácido sulfúrico, sódio superaquecido, hexafluoreto e óxido de urânio. Um balde vazio é um balde
  de vidro com a área do fluido pintada com a cor dele (a máscara vem da diferença entre o balde e o balde de água do jogo; a original do NeoForge
  não está nas fontes). Os tanques de fluido, a Rotary, a Chemical Washer e o Electrolytic Separator esvaziam e enchem esses baldes como os de
  água e lava; um balde cheio devolve um balde vazio.
- **Correção (0.1.41):** o sal não aparecia: a procura do fundo parava na primeira seção de chunk vazia (sem blocos para ler) e, nos oceanos,
  parava na grama marinha e nas algas (que não são o chão); agora essas seções são puladas e só um bloco com colisão conta como chão.
- **Diferença:** os baldes não colocam o fluido no mundo (o porte não tem fluidos de mod no mundo); servem para encher tanques e máquinas.

## 2m. Energy Cubes Advanced, Elite e Ultimate (mod 0.1.43, o mesmo APK v40, **não testados no jogo**)

Passou no teste simulado (67 passos) e em 77 testes unitários. Os três níveis têm o modelo do original (a moldura com os cantos na cor do
nível, o núcleo girando tingido na cor do nível, portas e LEDs por face), a tela, a configuração de lados, o configurador e os slots de armadura do
cubo básico; capacidade e taxa por tick de `EnergyCubeTier`: Advanced 16 milhões e 16 000, Elite 64 milhões e 64 000, Ultimate 256 milhões e
256 000 (o Basic, 4 milhões e 4 000). As receitas são as originais (o cubo do nível de baixo, tabletes de energia, ligas infusas e lingotes de
osmio), feitas como receitas comuns: o cubo de baixo **não passa a energia** que guarda para o novo (no original passa). Falta o instalador de nível.

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
| **Configurador e portas por face do Energy Cube** (mod 0.1.12, APK v15) | **confirmado pelo usuário**, inclusive a energia mudando de verdade numa máquina ligada à face. (o mod 0.1.8 derrubou o app; ver defeitos 5 a 10). Antes: | item `mekanism:configurator`: clicar numa face do Energy Cube passa o modo daquela face (desligada → entrada → saída → entrada/saída), agachado volta; a face é a que o olhar do jogador acerta (nova função `player-control.get-look`, API 16); o bloco mostra o modo (propriedades `side_*`: portas e LEDs mudam) e a energia só passa pelas faces conforme o modo. Só o modo de configuração de energia, só nos cubos; sem os outros modos do original (vazio, girar, chave inglesa) nem a aba de configuração na tela das máquinas |
| Carregadores de modelo próprios (cubo de energia, cabos, tanques, fábricas, modelos `.obj.mek`) | **aproximados à mão**: moldura do cubo sim, portas e LEDs por face não; cabos com núcleo e braços simples, sem os modelos grandes/pequenos do original |
| Núcleo do Energy Cube animado como o original (**confirmado**) | elemento novo `animated` do renderizador: escala 0,4, giro em Y a 4°/tick, giro em torno de (0,1,1) a 36°+4°/tick, balanço `sin(3°·t)/7`, brilho total, só aparece com energia |
| Telas das máquinas com o layout do original (mod 0.1.15, **não testado**) | painel de 176x166 montado na conversão com as texturas de interface do próprio Mekanism e as posições lidas do código dele (`GuiElectricMachine`, `TileEntityElectricMachine`, `GuiBar`): entrada em 64,17, saída em 116,35, slot de carga em 64,53 (só o quadro e a seta para cima: o slot não existe no porte, pois não há itens de energia), barra de progresso `progress/bar` 25x9 em 86,38, barra de energia com moldura 6x54 em 164,16 e conteúdo 4x52, título centralizado em y=6 e "Inventory" em 8,72 |
| Núcleo do cubo mais fraco com menos energia (APK v18, **não testado**) | o original muda o alfa do núcleo; aqui a cor escurece (mínimo 3/16), porque o desenho dos blocos não mistura transparência |
| Brilho (`light_emission` por elemento) | **funciona no mundo** (cantos e LEDs do cubo): o desenhista de blocos usa o valor do modelo |
| Partes desenhadas por código (TESR): núcleo do cubo, nível de fluido nos tanques, rotores, o Digital Miner | **mecanismo existe** (bloco 11: `cube`, `fluid` que sobe, `spin`), **nunca testado no jogo**; o núcleo girando do cubo é o primeiro uso (mod 0.1.5) |
| Cor por estado (`tintindex`: cor do nível, pigmentos, blocos coloridos) | **não há**: a cor do núcleo do cubo está gravada na textura, uma por nível |
| Brilho (`light_emission` por elemento, partes que brilham no escuro) | **não há** |
| Partículas, lasers, feixes, shaders | **não há** |
| Renderizadores de item (armadura, jetpack, ferramentas modelo 3D) | **não há** |
| Renderizadores de entidade (Robit) | **não há** (só displays de item) |
| Telas com o aspecto do original (fundo, botões, abas) | simples por enquanto; o mecanismo de telas declarativas aceita textura de fundo |

Ordem proposta para a fidelidade: (1) ~~núcleo `animated`~~ feito e confirmado; (2) ~~portas e LEDs do cubo e configurador~~ feito e confirmado;
(3) cor e brilho por estado; (4) tanques de fluido; (5) cabos com os modelos do original; (6) telas com o fundo do
original; (7) rotores, partículas e o resto conforme as máquinas chegarem.
