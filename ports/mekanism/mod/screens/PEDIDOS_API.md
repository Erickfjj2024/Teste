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
