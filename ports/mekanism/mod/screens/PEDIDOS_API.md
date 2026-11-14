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
