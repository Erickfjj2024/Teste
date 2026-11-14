# Telas originais 100% idênticas — como o Mekanism monta as telas e o que falta no Horizon

Data: 2026-11-08. Base: auditoria de menus (`menus-maquinas.md`, ~85 divergências, 90 menus) e propostas de formato (`menus-format-proposals.md`, F1–F15, camadas A/B/C). Este arquivo não repete nenhum dos dois: ele responde à pergunta "dá para usar as telas originais?" mapeando **como o original constrói as telas** contra **como o Horizon constrói as telas**, elemento por elemento, e fechando com o plano para 100% idêntico.

## A premissa honesta

As telas originais são código Java — não rodam no Horizon, pelo mesmo motivo que o mod não roda (sem JVM, sem classes do Minecraft Java). Mas "100% idênticas" continua sendo um objetivo alcançável, porque uma tela não é código: é **dados + textura + comportamento**.

- **Textura:** já idêntica. Os assets foram copiados (o port usa as texturas originais; a auditoria de modelos confirma 188/227 idênticos).
- **Layout:** extraível. Cada tela original é uma lista de elementos com posição e tamanho — o `convert.py` já transcreveu 78 das 90 telas para JSON.
- **Valores vivos:** já chegam. `menu-data` é o `ContainerData` do vanilla (ver §1).
- **Comportamento:** é o que falta. Clique, digitação, slider, rolagem, abas — cada gesto precisa de um equivalente no formato ou no WIT.

Ou seja: o roubo não é o código da tela, é a **especificação** da tela. O que se rouba do NeoForge/Mekanism é o vocabulário de elementos e o contrato de dados — e o fork ganha um renderizador que desenha essa especificação.

## 1. Como o original monta as telas (NeoForge 1.21.1 + Mekanism 10.7.x)

Duas metades, como tudo no Minecraft Java:

**Lado servidor — o menu.** Cada máquina tem um `AbstractContainerMenu` (o "container"): declara os `Slot`s (inventário do bloco + do jogador) e um `ContainerData` — uma lista de `short`s (16 bits!) sincronizada com o cliente todo tick. É daí que vem o limite de 16 bits do WIT: `menu-data: list<s32>`, "16 bits each on the wire", é o `ContainerData` com outro nome. O servidor também recebe cliques: `menu.clickMenuButton(id)` vira um pacote vanilla e cai em `ContainerMenu.clickMenuButton` no servidor.

**Lado cliente — a tela.** Cada máquina tem uma classe `Gui* extends GuiMekanism` (que estende `AbstractContainerScreen`, do vanilla). A tela não desenha nada diretamente: ela **adiciona elementos** — `addElement(new GuiEnergyGauge(...))` — com posição x/y relativa ao canto da tela (`guiLeft`/`guiTop`), uma textura (`mekanism:textures/gui/...`) e uma fonte de dados (um slot do `ContainerData`, um tanque, um estado). O vanilla desenha o fundo, os slots e os widgets; o Mekanism desenha os elementos por cima, com `GuiGraphics`.

O framework de elementos do Mekanism (ramo `1.21.x`, `client/gui/element/`, lido no GitHub em 08/11/2026) é o vocabulário real das telas. As famílias:

| Família (diretório) | O que é | Exemplos |
|---|---|---|
| `bar/` | barras de preenchimento (energia, fluido, progresso) | `GuiBar`, `GuiDynamicHorizontalRateBar` |
| `button/` | botões | `GuiButton`, `GuiDumpButton`, `GuiDigitalSwitch` |
| `custom/` | elementos sob medida por tela | — |
| `gauge/` | medidores (energia, fluido, gás, calor, infusão) | `GuiEnergyGauge`, `GuiFluidGauge`, `GuiChemicalGauge` |
| `graph/` | gráficos de histórico | `GuiGraph`, `GuiDoubleGraph`, `GuiLongGraph` |
| `progress/` | setas e barras de progresso de receita | `GuiProgressBar` |
| `scroll/` | listas roláveis | `GuiScrollList` |
| `slot/` | slots com renderização própria | `GuiSlot`, `GuiColorPickerSlot` |
| `tab/` | abas laterais | `GuiEnergyTab`, `GuiRedstoneTab`, `GuiSideConfigurationTab`, `GuiUpgradeTab`, `GuiSecurityTab` |
| `text/` | texto e campos de texto | `GuiTextField`, labels |
| `window/` | janelas arrastáveis | `GuiWindow` (side config, upgrades) |
| avulsos | — | `GuiSlider`, `GuiDropdown`, `GuiArrowSelection`, `GuiScreenSwitch`, `GuiInnerScreen`, `GuiSecurityLight`, `GuiBigLight`, `GuiSideHolder`, `GuiElementHolder` |

Três detalhes que decidem o mapa:

1. **Todo elemento é posição + textura + fonte de dados.** Não há layout automático: cada `Gui*` escreve as coordenadas na mão. É por isso que a transcrição funciona — o layout é dado, não algoritmo.
2. **Texto digitado não usa `ContainerData`.** Campo de texto (`GuiTextField`, ex.: injeção da fusão, taxa de queima da fissão, thresholds do laser) e slider (`GuiSlider`) voltam ao servidor pelos **pacotes próprios do Mekanism** (`MekanismPacketHandler`), não pelo pacote vanilla de botão. É por isso que o `menu-button` não basta e a camada C existe.
3. **Abas e janelas são elementos como os outros.** Side config, upgrades, redstone e energia são abas (`GuiTab`) que abrem janelas (`GuiWindow`) — não são telas separadas. É por isso que o formato do Horizon já consegue expressá-las com `windows` (A4/A5).

## 2. Como o Horizon monta as telas hoje

O formato **não está no WIT**: vive nas structs Rust do fork (`MenuScreen`/`Widget`/`Value`, patches 0007 e 0017). O WIT só tem três pontos de contato: `container-spec.menu: bool` (o bloco tem menu, `data/<ns>/horizon/menu/<block>.json`), `menu-data` (os 16 bits) e `menu-button` (o único evento de menu, só com `id: s32`).

Vocabulário real do formato (patch 0017, conferido no fork):

| Widget do fork | O que faz | Valores |
|---|---|---|
| `Bar` | retângulo preenchido `value` de `max`, com cor e direção | `Value` |
| `Sprite` | parte de uma textura, ou os primeiros `value` de `max` (seta de progresso) | `Value`, `src`, `tints` |
| `Text` | linha de texto; `{0}`, `{1}`... são data slots | `Value`, `scale` (0017) |
| `Button` | botão; clique vira `menu-button(id)` | `id` |
| `Swatch` | quadrado de cor por valor (ex.: cor de lado) | `Value`, `colors` |
| `Gauge` | medidor com a tabela de 72 tipos de químico embutida | `Value`, `kind`, `types` |
| `Neighbor` | indicador de vizinho por lado | `side`, `facing` |
| `Item` | ícone de item fixo | id fixo |
| `Show` | mostra/esconde um grupo de widgets por condição | condição sobre data slots |
| `windows` | janelas arrastáveis (máx. 8 por tela) | — |

`Value` é `Const(i64)` | `Data(u16)` | `Pair([u16; 2])` — um slot, ou um par que reconstrói 32 bits. É o `ContainerData` inteiro: cada número da tela é um slot, um par, ou uma constante.

O que o `convert.py` já faz: gera os 90 menus (11,3 MB, 37.932 widgets) a partir das definições das máquinas no mod. A geometria fixa da tela vive na **textura PNG pré-composta**, não no JSON — mudar um layout exige regenerar a textura.

## 3. O mapa elemento por elemento — GuiElement → widget do fork

| Elemento original (Mekanism) | Widget do fork | Estado |
|---|---|---|
| `GuiTexturedElement` / `GuiTextureOnlyElement` (fundo, ícones) | `Sprite` | ✅ |
| `GuiButton`, `GuiDigitalSwitch`, toggles | `Button` (+ `Show` para estado) | ✅ |
| `GuiSlot` (slots normais) | `slots` do menu JSON (posição `[x, y]`) | ✅ |
| `GuiGauge` (energia, fluido, gás, calor) | `Gauge` | ✅ |
| `GuiProgressBar` (seta de progresso) | `Sprite` com `value`/`max` | ✅ |
| `GuiBar` (barras) | `Bar` — **existe no formato, o `convert.py` nunca emite** | ◐ (usar) |
| Texto/labels | `Text` | ◐ (sem formatação F9, sem cor condicional F10) |
| `GuiTab` + `GuiWindow` (side config, upgrades, redstone, energia) | `windows` | ◐ (mecanismo existe; faltam as janelas A4/A5) |
| `GuiScrollList` (lista de upgrades, modos lógicos) | — | ❌ F5 (rolagem falsa; 31 combos enumerados = os 11,3 MB) |
| `GuiTextField` (injeção, taxa, thresholds, busca, nomes) | — | ❌ F2 (precisa da camada C) |
| `GuiSlider` (MekaSuit, thresholds) | — | ❌ F2 (precisa da camada C) |
| `GuiGraph` / `GuiDoubleGraph` (temperatura, taxa) | — | ❌ F4 (deferir; só tendência) |
| `GuiDropdown` (seletores) | — | ❌ (cobre-se com `list` + `Button` depois de B1) |
| `GuiColorPickerSlot` (cores do Configurator/transmissor) | — | ❌ (precisa de widget novo ou de botões) |
| `GuiArrowSelection`, `GuiScreenSwitch`, `GuiInnerScreen` | `Show` + `windows` | ◐ (a maioria se cobre) |
| `GuiSecurityLight`, `GuiBigLight`, indicadores | `Sprite` / `Show` | ◐ |
| `WarningTracker` (ícones de aviso: sem energia, sem receita...) | — | ❌ D-12/F8 (sem elemento de aviso; `Text`/`Item` não têm tooltip) |
| Tooltips (valores formatados, conteúdo do tanque, receita) | `Bar`/`Sprite` têm `tooltip`; `Text`/`Item`/`Neighbor` **não têm** | ◐ |
| Receita fantasma no gauge / clique na seta abre o JEI | — | ❌ F6 (precisa de B4, item dinâmico) |
| Animação de fluido/gás (frames do `.mcmeta`) | — | ❌ F3 (gauge ladrilha só o 1º frame) |

O mapa diz: **o grosso já é expressável**. Dos ~12 tipos de elemento que as telas usam, 8 têm equivalente direto, 2 têm equivalente parcial e só 4 exigem formato novo — e desses 4, dois (`list`, `text-field`/`slider`) são os que o plano já prevê nas camadas B e C.

## 4. O plano para 100% idênticas, em ordem

Nada aqui é novo em relação às propostas de formato — é a mesma ordem (A → B → C), agora com a transcrição das telas e a verificação no fim.

### Passo 1 — camada A: os números certos (só no mod)

A1 (plasma/case/`last_power` da fusão em pares), A3 (escala do SPS), A2 (slots mortos), A4/A5 (janelas de multibloco e Dynamic Tank). Sem dependência de ninguém. É o que o jogador vê errado hoje — e é o único passo que corrige um número errado em tela.

### Passo 2 — camada B: os widgets que faltam (só no fork)

B1 (`list` — e desincha os 11,3 MB), B2 (animação dos gauges), B3 (formatação de números), B4 (item dinâmico). B1 é o maior ganho estrutural: sem ele, qualquer lista vira enumeração de combos no JSON. B4 destrava F6 (receita fantasma).

### Passo 3 — camada C: texto e slider (WIT + fork)

C1 (`menu-text-submit`, espelho de `menu-button` com payload) — a única mudança de WIT do plano inteiro. Destrava: taxa de queima da fissão, injeção da fusão, thresholds do laser, busca do QIO, frequências, nomes (Robit, teleporter). Sem C1, esses valores continuam sendo botões de passo — e "valor exato" continua impossível.

### Passo 4 — transcrever as telas que faltam

A auditoria já diz quais são (não re-auditar, transcrever):

- **9 telas de multibloco ausentes:** `GuiMatrixStats`, `GuiBoilerStats`, `GuiTurbineStats`, `GuiFissionReactorStats`, `GuiFissionReactorLogicAdapter`, `GuiFusionReactorHeat`, `GuiFusionReactorFuel`, `GuiFusionReactorStats`, `GuiFusionReactorLogicAdapter` (o laser não tem tela nem no original).
- **Telas de itens:** Robit (Main/Inventory/Crafting/Smelting/Repair + Rename + Skin Select + segurança), MekaSuit Helmet Options (3 seletores de cor + sliders — precisa de C1) e Module Tweaker, Security Desk.
- **Janelas laterais:** Transporter Config (todas as telas configuráveis), segurança, warning, sorting — o bloqueio aqui não é o formato e sim o estado no servidor (F8), então essas entram depois que os sistemas existirem.
- **Detalhes já listados:** slots químicos mortos (N4), ícones de aviso (N6/D-12), seta com clique para o recipe viewer (N7), aba de energia com troca de unidade (N8).

Cada tela transcrita é um menu JSON + uma textura PNG regenerada + os `menu-data` correspondentes. O trabalho por tela é mecânico depois que o formato expressa os elementos — o custo está nos passos 2 e 3, não aqui.

### Passo 5 — o compilador de telas (o roubo de verdade)

Hoje o `convert.py` gera os menus a partir de tabelas mantidas à mão no script, alimentadas pelas definições das máquinas no Rust. Funciona, mas cada layout é transcrito duas vezes: uma na tabela, outra na textura PNG. O roubo completo é **uma especificação de layout por tela, num só lugar, no repo** — um arquivo por tela com os elementos (tipo, posição, textura, fonte de dados), do qual o `convert.py` gera o JSON **e** a textura. A tela passa a ser "transcrita uma vez, verificada uma vez": a revisão é contra a classe `Gui*` original, e o JSON gerado é derivado, não fonte.

Não é parser do Java (frágil — a construção dos elementos é código com condicionais) nem reescrita à mão por tela (o que já existe). É o meio-termo: o layout vira dado versionado, e o conversor vira compilador. Quando o formato ganhar `list` ou `text-field`, nenhuma tela precisa ser re-transcrita — só o compilador aprende o widget novo.

### Passo 6 — verificação visual (o único jeito de dizer "100%")

Nenhuma tela do port foi vista aberta no jogo (ressalva da auditoria). "Idêntica" não se prova lendo JSON — se prova abrindo a tela no APK ao lado do original e comparando. O loop: abrir cada tela no celular, captura ao lado da original, marcar divergência visual. O `TESTES.md` já tem o padrão de checklist; cada tela transcrita no passo 4 entra nele com captura. Sem este passo, "100%" é afirmação sobre arquivo, não sobre tela.

## 5. O que nunca vai ser 100% (limites honestos)

- **Física de interação do cliente:** momentum de rolagem, timing de hover, anel de foco, arraste com inércia. O renderizador do fork decide; o objetivo é "indistinguível em uso", não "mesmo quadro por quadro".
- **Conteúdo dinâmico que precisa de código no cliente:** receita fantasma (F6), tooltips com valores vivos em `Text`/`Item`, gráficos (F4). Cada um precisa de um widget novo — não é impossível, é só trabalho de fork.
- **Latência de texto:** no original o campo de texto manda pacote ao digitar e o servidor responde; no Horizon o caminho é `menu-text-submit` → servidor → `menu-data` no próximo tick. O `ContainerData` do vanilla também sincroniza por tick, então a latência é comparável — mas o "ao digitar" (cada tecla) vira "ao confirmar", a menos que o evento dispare por tecla.
- **Fonte:** o fork renderiza o texto com a fonte dele; o Mekanism usa a fonte do vanilla. Mesmos glifos, mas escala e largura precisam de conferência visual (o `Text` ganhou `scale` no patch 0017 justamente para isso).
- **Geometria na textura:** enquanto o layout fixo viver no PNG pré-composto, qualquer mudança de layout regenera a textura. O passo 5 resolve isso gerando a textura a partir da mesma especificação — mas até lá, layout e textura são duas fontes que podem divergir.

## 6. O vocabulário a roubar (nomes)

O fork já usa os nomes certos sem perceber (`Gauge`, `Bar`, `Sprite` = `GuiGauge`, `GuiBar`, `GuiTexturedElement`). Para fechar, os widgets novos devem herdar os nomes do framework original, para um modder de Mekanism ler o formato e reconhecer:

| Nome no formato do fork | Origem | Estado |
|---|---|---|
| `Gauge`, `Bar`, `Sprite`, `Button`, `Text` | `GuiGauge`, `GuiBar`, `GuiTexturedElement`, `GuiButton` | existe |
| `List` | `GuiScrollList` | B1 |
| `TextField` | `GuiTextField` | C1 |
| `Slider` | `GuiSlider` | C1 |
| `Graph` | `GuiGraph` / `GuiDoubleGraph` | B5 (deferir) |
| `Dropdown` | `GuiDropdown` | depois de B1 |
| `menu-data` | `ContainerData` (shorts) | existe |
| `menu-button(id)` | `clickMenuButton(id)` | existe |
| `menu-text-submit(id, text)` | pacotes de texto do Mekanism | C1 |

E a regra de bolso para o guia de port: **o que no original é um `GuiElement`, no Horizon é um widget no menu JSON; o que no original é um pacote do `MekanismPacketHandler`, no Horizon é um evento no WIT.**

## Verificações em aberto

1. `Bar` existe no formato mas o `convert.py` nunca emite — confirmar se o renderizador do fork o desenha (se não, B0: ligar o `Bar` antes de qualquer coisa).
2. `GuiTextField` no original: confirmar que todo campo manda pacote por tecla ou só ao confirmar (decide se `menu-text-submit` dispara por tecla).
3. `GuiColorPickerSlot`: confirmar como o original implementa a escolha de cor (slot clicável com popup?) — decide se vira widget novo ou botões.
4. Tamanho real das 9 telas de multibloco ausentes: confirmar o `menu-data` necessário por tela (a fusão já estourou 16 bits em A1; as Stats podem precisar de pares novos).
5. Verificação visual (passo 6): nenhuma tela foi vista no jogo; o primeiro lote de capturas decide se o passo 5 vale o trabalho.

## Arquivos relacionados

- `menus-maquinas.md` — a auditoria (§4.3, F1–F15; §2.1, telas ausentes; §3, D-01–D-26).
- `menus-format-proposals.md` — as correções por camada (A1–A6, B1–B7, C1) — este arquivo não as repete.
- `neoforge-wit-map.md` — áreas 8 (menus) e 20 (cliente/render) do mapa NeoForge → WIT; este arquivo é o aprofundamento delas.
- `configurator-api-proposals.md` — P5/P7 (indicador de modo) e P1 (slot no `item-used`) tocam as telas por outro lado.
- Fork: `ports/api-fork/0007-*.patch` (formato do menu) e `0017-*.patch` (Gauge, Neighbor, Item, `Text.scale`, tints).
