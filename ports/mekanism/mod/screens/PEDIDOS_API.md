# Pedidos de API / SDK Requests

1. **Interação com Gauge Dropper na barra de químicos e medidores (`GuiTankBar.java:93-109`)**
   - **Telas:** `factory.rs`, `fluid_tank.rs`, `chemical_tank.rs`
   - **Motivo:** No Java, o jogador pode interagir com barras e medidores segurando um `Gauge Dropper` para esvaziar ou encher o tanque gota a gota enviando `PacketDropperUse`. Atualmente o SDK de telas (`horizon_mod_sdk::screen`) não expõe o item no cursor do mouse para cliques em gauges nem mensagens de rede para o Dropper.

2. **Integração de visualizador de receitas (JEI / REI / EMI) nas setas de progresso (`GuiProgress.java:104`)**
   - **Telas:** `factory.rs`
   - **Motivo:** No Java, clicar nas setas de progresso abre a categoria correspondente de receitas no JEI (`recipeViewerCategory(tile)`). A API de telas não possui comando ou evento para acionar o visualizador de receitas externo.

3. **Aba de visualização de chunks no mundo (`GuiVisualsTab.java`)**
   - **Telas:** `dimensional_stabilizer.rs`
   - **Motivo:** No Java, `GuiVisualsTab` alterna a renderização no mundo dos limites dos chunks mantidos carregados pelo estabilizador dimensional. A capacidade `screen` atua apenas no desenho de interface em 2D na janela, sem canal para habilitar renderizadores de bounding box no mundo 3D.
# Pedidos de Extensão da API de Telas (SDK / Servidor)

Durante a auditoria e correção cruzada das 13 telas de multiblocos e QIO contra o código Java original do Mekanism, foram identificados os seguintes recursos que não puderam ser integrados diretamente no cliente de telas Rust por limitações da API atual do SDK ou pela falta de suporte de sincronização no servidor:

---

## 1. Suporte a Abas de Alertas e Avisos (`GuiWarningTab`)
- **Classes Java**: `mekanism.client.gui.element.tab.GuiWarningTab`, `IWarningTracker`.
- **Telas afetadas**: `sps.rs`, `industrial_turbine.rs`, `dynamic_tank.rs`, `thermoelectric_boiler.rs`, `fission_reactor.rs`, `thermal_evaporation_controller.rs`, `induction_matrix.rs`, `fusion_reactor_controller.rs`, `chemical_tank.rs`, `quantum_entangloporter.rs`.
- **Limitação**: O SDK de telas e o protocolo atual de contêineres não expõem um mecanismo padronizado para sincronizar avisos de operação (`WarningType`), tais como falta de espaço na saída, entrada bloqueada, resfriamento insuficiente ou sobreaquecimento.
- **Pedido de API**: Expor na estrutura de sincronização de contêiner ou via evento de rede uma lista de avisos ativos (`Vec<WarningType>`) e fornecer suporte a sprite e clique na `GuiWarningTab` no lado direito da tela (x = `width`).

---

## 2. Redimensionamento Dinâmico de Grade e Janela (`GuiResizeControls`)
- **Classes Java**: `mekanism.client.gui.qio.GuiQIOItemViewer`, `mekanism.client.gui.element.GuiResizeControls`.
- **Telas afetadas**: `qio_item_viewer.rs`, `qio_dashboard.rs`, `portable_qio_dashboard.rs`.
- **Limitação**: O SDK de telas fixa a resolução (`Setup { width, height, ... }`) na abertura da tela (`open`). O Java permite ao usuário expandir ou contrair o número de colunas (8 a 16) e linhas (4 a 8) dinamicamente com recriação do contêiner.
- **Pedido de API**: Permitir alterar dinamicamente as dimensões do Canvas e as coordenadas/quantidade de slots do `menu_grid` em tempo de execução sem fechamento e reabertura da janela pelo jogador.

---

## 3. Configurações de Cliente Locais e Integração com JEI/Recipe Viewer
- **Classes Java**: `mekanism.client.gui.element.tab.GuiToggleClientConfigTab`, `GuiTargetDirectionTab`.
- **Telas afetadas**: `qio_item_viewer.rs`, `qio_dashboard.rs`.
- **Limitação**: O cliente WebAssembly não possui integração nativa com JEI/REI/EMI nem armazenamento local de preferências de cliente (auto-focus na busca, destino de rejeitos de receitas para o inventário ou para a frequência).
- **Pedido de API**: Suporte a armazenamento de preferências locais do cliente (`ClientStorage` / `Preferences`) e ganchos de eventos para visualizadores de receitas compatíveis com WebAssembly.

---

## 4. Sub-telas de Estatísticas de Multiblocos no Servidor
- **Classes Java**: `GuiTurbineStats`, `GuiBoilerStats`, `GuiMatrixStats`.
- **Telas afetadas**: `industrial_turbine.rs`, `thermoelectric_boiler.rs`, `induction_matrix.rs`.
- **Limitação**: No servidor Rust (`port/mod/src/`), os menus dessas máquinas só expõem os dados da página principal e não possuem a máquina de estados ou contêineres para as páginas detalhadas de componentes internos (ex.: número de pás/estatores na turbina, cavidades/dispersores na caldeira, células/provedores na matriz).
- **Pedido de API**: Extensão dos contêineres no servidor para transmitir os contadores estruturais do multibloco quando solicitados pelo cliente.
# Pedidos de Extensão da API do SDK / Servidor de Telas

Durante a revisão cruzada linha por linha das telas em Rust contra a implementação de referência Java (Mekanism 1.21.1), foram identificados os seguintes itens que dependem de suporte adicional do SDK ou do servidor de menus:

1. **Digital Miner (`digital_miner.rs`): Switch Auto Pull**
   - **Java:** `GuiDigitalMiner.java:87-89` (`GuiDigitalSwitch(this, 38, 56, INPUT, tile::getDoPull, ...)` com `GuiInteraction.AUTO_PULL_BUTTON`).
   - **Situação:** O layout da tela do port foi adaptado para renderizar o botão em `(38, 56, 16, 16)`, mas o servidor/menu_data não expõe nem processa o estado do Auto Pull.
   - **Pedido:** Expor slot de dado para estado do Auto Pull no menu e processar evento de interação correspondente.

2. **Modification Station (`modification_station.rs`): Lista Rolável de Módulos Instalados**
   - **Java:** `GuiModificationStation.java:53` (`GuiModuleScrollList(this, 28, 20, 74, ...)`).
   - **Situação:** O port atualmente expõe até 6 linhas fixas via menu strings. Não há componente nativo de lista rolável dinâmica para módulos arbitrários com seleção interativa de alta granularidade (requer suporte de widget de lista rolável conforme FORK-20).
   - **Pedido:** Suporte a widget de lista rolável (`list-widget`) no SDK e servidor.

3. **Oredictionificator (`oredictionificator.rs`): Janela Modal de Filtro (`GuiOredictionificatorFilter`) e Resolução de Tags**
   - **Java:** `GuiOredictionificator.java:55-58` (`GuiOredictionificatorFilter.create(gui, gui.tile)` e `.edit(...)`).
   - **Situação:** A criação e edição de filtros na versão original abre uma janela modal sobreposta com busca e inspeção de tags de itens. O port atual usa campo de texto inline (`c.field`) para digitação direta da tag e botões inline nas linhas de filtro.
   - **Pedido:** Suporte a janelas modais de configuração de filtros e consulta de membros de tags de itens (`item-tag-members`).

4. **Logistical Sorter (`logistical_sorter.rs`): Alternância de Round Robin**
   - **Java:** `GuiLogisticalSorter.java:54-56` (`GuiInteraction.ROUND_ROBIN_BUTTON`).
   - **Situação:** O botão visual e área de clique foram adicionados em `(12, 76, 14, 14)`, mas o servidor e menu_data do port não suportam o toggle de Round Robin.
   - **Pedido:** Expor flag de Round Robin no menu_data e tratar id de botão 7403 no servidor.

5. **Quantum Entangloporter (`quantum_entangloporter.rs`): Abas de Calor e Aviso**
   - **Java:** `GuiQuantumEntangloporter.java:44-48` (`GuiHeatTab` e `GuiWarningTab`).
   - **Situação:** A transferência de calor do entangloporter e o sistema de warning tracker não estão modelados no servidor do port.
   - **Pedido:** Modelagem de transferência de calor interdimensional e tracker de warnings quando suportados pelo núcleo.
# Pedidos de Extensão da API do SDK (Horizon Mod SDK - Telas)

Durante a auditoria cruzada linha por linha das telas Mekanism (`port/mod/screens/src/`) contra o Java original (`mekanism.client.gui.*`), foram identificadas as seguintes limitações da API atual do SDK que motivaram anotações `// NÃO FEITO: motivo`:

1. **Acesso aos blocos vizinhos no mundo (`Level`) para `SideDataButton`**:
   - **Classe Java:** `mekanism.client.gui.element.button.SideDataButton`
   - **Módulo Rust:** `port/mod/screens/src/side_config.rs`
   - **Necessidade:** O Mekanism renderiza o item/ícone do bloco vizinho (`otherBlockItem`) sobreposto à cor da face correspondente (Top, Bottom, Front, Back, Left, Right). Atualmente, `screens.wasm` é puramente desacoplada do cliente do mundo e não recebe informações sobre blocos adjacentes ao bloco configurado, limitando-se a desenhar o botão colorido e o texto da face.
   - **Sugestão de API:** Permitir passagem de IDs de itens dos vizinhos via payload do menu ou query do host.

2. **Seletor de cor interativo (Color Picker / Sliders RGBA)**:
   - **Classe Java:** `mekanism.client.gui.element.window.GuiColorScreen` / `GuiModuleTweaker`
   - **Módulo Rust:** `port/mod/screens/src/module_tweaker.rs`
   - **Necessidade:** Módulos de armadura MEK com ajuste de cor (ex.: cor do HUD ou armadura MekaSuit) abrem uma subjanela com seletor HSV/RGB. O SDK atual oferece campos de texto (`FieldText`), mas não possui componente de seleção de cor interativo.

3. **Arrasto contínuo da barra de rolagem (Scrollbar Dragging)**:
   - **Classe Java:** `mekanism.client.gui.element.scroll.GuiScrollBar`
   - **Módulos Rust:** `port/mod/screens/src/seismic_reader.rs`, `port/mod/screens/src/dictionary.rs`
   - **Necessidade:** A barra de rolagem no Java permite arrasto contínuo com o mouse (`mouseDragged`). No SDK, o controle de rolagem é feito por eventos discretos de roda do mouse (`MouseWheel`) e botões de incremento/decremento.

4. **Janelas flutuantes dinâmicas (`GuiWindow`) com sobreposição de foco**:
   - **Classe Java:** `mekanism.client.gui.element.window.GuiWindow` (`GuiRobitRename`, `GuiRobitSkinSelect`)
   - **Módulo Rust:** `port/mod/screens/src/robit.rs`
   - **Necessidade:** No Java, sub-janelas são instanciadas como camadas sobrepostas (`GuiWindow`) com botão `x` automático de fechar e bloqueio de input da tela de fundo. No Rust, estas foram mapeadas para sub-estados da tela (`RobitTab::Rename`, `RobitTab::Skin`) com botão Back no canto superior.
