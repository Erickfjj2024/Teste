# Especificação Técnica Completa: Utilitários do Mekanism (1.21.1-10.7.19.85)

Documento de especificação para implementação em Rust (mod WASM / Horizon) das 10 peças do grupo **Utilidades**:
1. Chargepad
2. Dimensional Stabilizer
3. Cardboard Box
4. Personal Barrel
5. Personal Chest
6. Seismic Vibrator
7. Oredictionificator
8. Formulaic Assemblicator
9. Crafting Formula (item)
10. Configuration Card (item)

**Conversão de energia**: $FE = J \times \frac{2}{5}$ via `crate::fe()`.

---

## 1. Chargepad

### 1.1 Identificação e Classes Java
- **Registro**: `mekanism:chargepad`
- **Classes**:
  - `mekanism.common.block.basic.BlockChargepad`
  - `mekanism.common.tile.TileEntityChargepad`
  - `mekanism.common.registries.MekanismBlocks.CHARGEPAD`
  - `mekanism.common.registries.MekanismBlockTypes.CHARGEPAD`

### 1.2 Blockstate e Modelo
- **Propriedades**:
  - `facing`: `Direction.Plane.HORIZONTAL` (north, south, east, west)
  - `active`: `boolean` (emite luz quando ativo)
- **Shape de Colisão**:
  - Base: `VoxelShape BASE = box(0, 0, 0, 16, 1, 16)`
  - Projéteis colidem com o volume total; entidades usam a base fina de placa.

### 1.3 Energia e Capacidades
- **Capacidade do Buffer**: `StorageConfig.chargepad` = 2.048.000 J = **819.200 FE** (`crate::fe(2_048_000)`).
- **Taxa de Carga / Consumo**: `UsageConfig.chargePad` = 1.024.000 J/t = **409.600 FE/t** (`crate::fe(1_024_000)`).
- **Conexões**: Aceita entrada de energia exclusivamente pelas faces `RelativeSide.BACK` e `RelativeSide.BOTTOM`.

### 1.4 Inventário e GUI
- Sem GUI própria e sem slots de inventário no bloco.

### 1.5 Lógica Operacional por Tick
1. A cada tick do servidor, se `energyContainer > 0`:
2. Busca entidades vivas na bounding box:
   $$\text{AABB}(x, y, z, x + 1.0, y + 0.4, z + 1.0)$$
   Filtro: `!entity.isSpectator() && (entity instanceof Player || entity instanceof EntityRobit)`.
3. Para `Player`:
   - Varre o inventário (`IItemHandler`) procurando o primeiro item recarregável (`IStrictEnergyHandler`).
   - Insere até `409.600 FE/t` no item (limite de 1 item recarregado por tick por jogador).
   - Se o inventário principal não tiver item recarregável e o Curios estiver ativo, tenta recarregar slots de Curios.
4. Para `EntityRobit`:
   - Insere até `409.600 FE/t` diretamente no buffer do Robit.
5. Se houve transferência de energia, define `active = true`; caso contrário, `active = false`.

### 1.6 Áudio, Partículas e Redstone
- **Som ao ativar (`active: false -> true`)**: `SoundEvents.STONE_PRESSURE_PLATE_CLICK_ON` (volume 0.3F, pitch 0.8F).
- **Som ao desativar (`active: true -> false`)**: `SoundEvents.STONE_PRESSURE_PLATE_CLICK_OFF` (volume 0.3F, pitch 0.7F).
- **Partículas**: Poeira de redstone (`DustParticleOptions.REDSTONE`) na base (`Pos3D(rand.nextDouble() - 0.5, -0.35, rand.nextDouble() - 0.5)`).

### 1.7 Receita
- **Tipo**: `mekanism:mek_data` (Shaped Crafting)
- **Grid**:
  ```
  ###
  SES
  ```
  - `#`: `minecraft:polished_blackstone_pressure_plate`
  - `S`: `#c:ingots/steel`
  - `E`: `mekanism:energy_tablet`
- **Resultado**: 1 `mekanism:chargepad`.

### 1.8 Requisitos do Horizon
- Suporte a detecção de entidades na área do bloco e API para recarga de itens do inventário do jogador em tempo de execução.

---

## 2. Dimensional Stabilizer

### 2.1 Identificação e Classes Java
- **Registro**: `mekanism:dimensional_stabilizer`
- **Classes**:
  - `mekanism.common.tile.machine.TileEntityDimensionalStabilizer`
  - `mekanism.client.gui.GuiDimensionalStabilizer`
  - `mekanism.client.render.tileentity.RenderDimensionalStabilizer`
  - `mekanism.common.registries.MekanismBlocks.DIMENSIONAL_STABILIZER`
  - `mekanism.common.registries.MekanismBlockTypes.DIMENSIONAL_STABILIZER`

### 2.2 Blockstate e Dimensões
- **Propriedades**: Sem facing (`without(AttributeStateFacing.class)`). Possui `active: boolean`.
- **Raio de Ação**:
  - `MAX_LOAD_RADIUS = 2` (diâmetro máximo de 5×5 chunks).
  - O chunk central onde o estabilizador está posicionado é **sempre** mantido carregado (`loadingChunks[2][2] = true`).
  - Total de chunks gerenciados: 1 a 25 chunks.

### 2.3 Números de Configuração e Energia
- **Consumo por Chunk**: `UsageConfig.dimensionalStabilizer` = 5.000 J/t = **2.000 FE/t** por chunk ativo.
  - Consumo mínimo (1 chunk): 2.000 FE/t.
  - Consumo máximo (25 chunks): 50.000 FE/t.
- **Capacidade do Buffer**: `StorageConfig.dimensionalStabilizer` = 40.000 J = **16.000 FE**.
- **Upgrades Suportados**: Apenas `Upgrade.ENERGY` (até 8 upgrades, reduz consumo e aumenta buffer até 10×).
- **Config Global**: `MekanismConfig.general.allowChunkloading` (se `false`, chunks não são carregados e energia não é drenada).

### 2.4 Inventário e GUI
- **Slots**:
  - 1 slot de energia em (143, 35) para descarregar baterias/energy tablets.
- **GUI (`GuiDimensionalStabilizer`)**:
  - Grade 5×5 representando chunks em coordenadas relativas $[-2, +2]$.
  - Clique em cada célula alterna (`toggleChunkLoadingAt(x, z)`).
  - Botões rápidos: carregar raio 1 (3×3 = 9 chunks) ou raio 2 (5×5 = 25 chunks), habilitar todos / desabilitar todos.

### 2.5 Persistência (Data Components / NBT)
- `MekanismDataComponents.STABILIZER_CHUNKS`: classe `StabilizedChunks` serializa um bitset de 25 bits representando o estado de cada chunk da matriz 5×5.
- `MekanismDataComponents.ENERGY_CONTAINERS`.

### 2.6 Receita
- **Tipo**: `minecraft:crafting_shaped`
- **Grid**:
  ```
  ICI
  ADA
  ICI
  ```
  - `I`: `#c:ingots/refined_obsidian`
  - `C`: `#c:circuits/ultimate`
  - `A`: `#mekanism:alloys/atomic`
  - `D`: `#c:storage_blocks/diamond`
- **Resultado**: 1 `mekanism:dimensional_stabilizer`.

### 2.7 Requisitos do Horizon
- `FORK-14`: Carregamento de chunks via tickets no servidor (`IChunkLoader`).
- `FORK-20`: Interface de mapa de chunks interativa.

---

## 3. Cardboard Box

### 3.1 Identificação e Classes Java
- **Registro**: `mekanism:cardboard_box`
- **Classes**:
  - `mekanism.common.block.BlockCardboardBox`
  - `mekanism.common.tile.TileEntityCardboardBox`
  - `mekanism.common.item.block.ItemBlockCardboardBox`
  - `mekanism.common.attachments.BlockData`
  - `mekanism.common.advancements.triggers.UnboxCardboardBoxTrigger`

### 3.2 Blockstate e Empilhamento
- **Propriedades**: `storage: boolean` (`BlockStateHelper.storageProperty`).
  - `storage = false`: caixa vazia.
  - `storage = true`: caixa empacotada com bloco interno.
- **Dureza**: 0.5F, resistência 0.6F, cor do mapa madeira.
- **Empilhamento de Itens**:
  - Caixa vazia: empilha até 16 (`stacksTo(16)`).
  - Caixa com dados (`BLOCK_DATA`): empilha até 1.

### 3.3 Mecânica de Empacotamento e Desempacotamento
1. **Empacotar (Clique direito no bloco)**:
   - Jogador clica com a caixa de papelão vazia na mão em um bloco (sem agachar).
   - Validações: bloco não pode ser ar, inquebrável (dureza < 0), nem constar em `MekanismTags.Blocks.CARDBOARD_BLACKLIST` ou `MekanismConfig.general.cardboardModBlacklist`.
   - Verifica permissões de proteção e segurança (`IBlockSecurityUtils`).
   - Captura:
     - `BlockState` original com todas as propriedades.
     - `CompoundTag` completo do `BlockEntity` (inventários, energia, fluidos, configs).
   - Substitui o bloco no mundo por `BlockCardboardBox[storage=true]` com `UPDATE_SUPPRESS_DROPS`.
   - Salva o objeto `BlockData` no componente `MekanismDataComponents.BLOCK_DATA` do tile entity da caixa.
   - Consome 1 item de caixa do jogador (se não estiver no criativo).
2. **Desempacotar (Shift + Clique direito sem item na mão)**:
   - Jogador agachado clica sem item na mão na caixa empacotada.
   - Lê `BlockData` do tile.
   - Restaura o bloco e o tile entity original no mundo com fidelidade completa.
   - Dropa 1 item `mekanism:cardboard_box` (vazio).
   - Dispara o avanço `UNBOX_CARDBOARD_BOX`.

### 3.4 Tooltip
- Se tiver `BLOCK_DATA`:
  - `Block Data: Yes` (cor indigo).
  - Nome do bloco empacotado e propriedades.
- Se vazio: `Block Data: No`.

### 3.5 Receita
- **Tipo**: `minecraft:crafting_shaped` (2×2)
  ```
  ##
  ##
  ```
  - `#`: `#c:dusts/wood` (sawdust)
- **Resultado**: 1 `mekanism:cardboard_box`.

### 3.6 Requisitos do Horizon
- `FORK-23`: Captura e reinstanciação genérica de `BlockState` e dados de `BlockEntity` serializados em componentes de item.

---

## 4. Personal Barrel & Personal Chest

### 4.1 Identificação e Classes Java
- **Registros**: `mekanism:personal_barrel`, `mekanism:personal_chest`
- **Classes**:
  - `mekanism.common.block.BlockPersonalBarrel`
  - `mekanism.common.block.BlockPersonalChest`
  - `mekanism.common.tile.TileEntityPersonalBarrel`
  - `mekanism.common.tile.TileEntityPersonalChest`
  - `mekanism.common.tile.TileEntityPersonalStorage`
  - `mekanism.common.item.block.ItemBlockPersonalStorage`

### 4.2 Blockstates
- **Personal Barrel**:
  - `facing`: 6 direções (`BlockStateProperties.FACING`).
  - `open`: `boolean` (`BarrelBlock.OPEN`).
  - `security`: `SecurityMode` (PUBLIC, PRIVATE, TRUSTED).
- **Personal Chest**:
  - `facing`: horizontal (north, south, east, west).
  - `security`: `SecurityMode`.
  - Controlador de animação de tampa: `ChestLidController` (ângulo 0° a 90°).

### 4.3 Inventário e Armazenamento
- **Capacidade**: 54 slots (6 linhas × 9 colunas), posições na GUI em `(8 + x*18, 18 + y*18)`.
- **Persistência ao Quebrar (`persistsToItem`)**:
  - Ao ser minerado/quebrado, os 54 slots **não** são dropados no mundo.
  - O item retém todos os itens internos em seus data components (`ItemBlockPersonalStorage`).
- **Uso Portátil**:
  - O jogador pode segurar o item `Personal Chest` ou `Personal Barrel` na mão e clicar com o botão direito para abrir o inventário de 54 slots diretamente, sem necessidade de posicioná-lo no chão.

### 4.4 Regras de Segurança
- Modos:
  - `PUBLIC`: qualquer jogador pode abrir; automação (hoppers, tubos) pode inserir e extrair.
  - `PRIVATE`: somente o proprietário (`ownerUUID`) pode abrir; automação bloqueada.
  - `TRUSTED`: proprietário e jogadores na lista de amigos/confiança podem abrir; automação bloqueada.

### 4.5 Sons e Efeitos
- **Personal Barrel**: `SoundEvents.BARREL_OPEN` e `BARREL_CLOSE` (volume 0.5F, pitch 0.9-1.0F).
- **Personal Chest**: `SoundEvents.CHEST_OPEN` e `CHEST_CLOSE` (volume 0.5F, pitch 0.9-1.0F).
- Irrita Piglins próximos ao ser aberto (`PiglinAi.angerNearbyPiglins`).

### 4.6 Receitas
- **Personal Chest**:
  ```
  SGS
  PCP
  SSS
  ```
  - `S`: `#c:ingots/steel`, `G`: `#c:glass_blocks/cheap`, `P`: `#c:chests/wooden`, `C`: `#c:circuits/basic`
- **Personal Barrel**:
  ```
  SGS
  PCP
  SSS
  ```
  - `S`: `#c:ingots/steel`, `G`: `#c:glass_blocks/cheap`, `P`: `#c:barrels/wooden`, `C`: `#c:circuits/basic`

### 4.7 Requisitos do Horizon
- Suporte a contêineres de 54 slots (acima do teto nativo de 32 slots do Horizon) e persistência de itens no item (`persistsToItem`).

---

## 5. Seismic Vibrator

### 5.1 Identificação e Classes Java
- **Registro**: `mekanism:seismic_vibrator`
- **Classes**:
  - `mekanism.common.tile.machine.TileEntitySeismicVibrator`
  - `mekanism.client.gui.machine.GuiSeismicVibrator`
  - `mekanism.client.render.tileentity.RenderSeismicVibrator`
  - `mekanism.common.registries.MekanismBlocks.SEISMIC_VIBRATOR`
  - `mekanism.common.registries.MekanismBlockTypes.SEISMIC_VIBRATOR`

### 5.2 Estrutura e Bounding Blocks
- **Estrutura**: Bloco de 1×2×1 de altura. Ocupa o bloco base e gera 1 `bounding_block` imediatamente acima (`pos.above()`).
- **Blockstates**: `facing` (horizontal), `active` (boolean).

### 5.3 Números de Configuração e Energia
- **Consumo de Energia**: `UsageConfig.seismicVibrator` = 50 J/t = **20 FE/t** (`crate::fe(50)`).
- **Capacidade do Buffer**: `StorageConfig.seismicVibrator` = 20.000 J = **8.000 FE** (`crate::fe(20_000)`).
- **Upgrades**: Não suporta upgrades (`without(AttributeUpgradeSupport.class)`).
- **Conexões**: Entrada de energia exclusivamente pela face traseira (`RelativeSide.BACK`).

### 5.4 Inventário e GUI
- **Slots**: 1 slot de energia em (143, 35) para carregar o buffer via bateria.

### 5.5 Comportamento e Eventos
- Quando ativo e energizado:
  - O pistão mecânico oscila visualmente (`clientPiston++`).
  - A cada 40 ticks (2 segundos = `2 * TICKS_PER_SECOND`), dispara o evento `MekanismGameEvents.SEISMIC_VIBRATION`.
  - Registra a coordenada global em `Mekanism.activeVibrators`.
  - Permite leitura geológica do chunk através do item `mekanism:seismic_reader`.
  - Métodos computacionais: `getBlockAt(chunkRelX, y, chunkRelZ)` e `getColumnAt(chunkRelX, chunkRelZ)`.

### 5.6 Receita
- **Tipo**: `minecraft:crafting_shaped`
  ```
  I#I
  CXC
  III
  ```
  - `I`: `#c:ingots/tin`
  - `#`: `#c:gems/lapis`
  - `C`: `#c:circuits/basic`
  - `X`: `mekanism:steel_casing`
- **Resultado**: 1 `mekanism:seismic_vibrator`.

### 5.7 Requisitos do Horizon
- Suporte a bounding block vertical 1×2 e registro de posições ativas para integração com ferramentas do mod.

---

## 6. Oredictionificator

### 6.1 Identificação e Classes Java
- **Registro**: `mekanism:oredictionificator`
- **Classes**:
  - `mekanism.common.tile.machine.TileEntityOredictionificator`
  - `mekanism.client.gui.machine.GuiOredictionificator`
  - `mekanism.common.content.oredictionificator.OredictionificatorFilter`
  - `mekanism.common.content.oredictionificator.OredictionificatorItemFilter`
  - `mekanism.common.registries.MekanismBlocks.OREDICTIONIFICATOR`

### 6.2 Blockstates e Energia
- **Propriedades**: `facing` (horizontal), `active` (boolean).
- **Consumo de Energia**: **0** (operação lógica passiva sem consumo de energia).

### 6.3 Inventário e Configuração de Lados
- **Slots Físicos**:
  - Slot de entrada em (56, 115).
  - Slot de saída em (164, 115).
- **Lados**: Configuração padrão de item com saída por `RelativeSide.RIGHT` e suporte a auto-ejeção.

### 6.4 Filtros e Lógica Operacional por Tick
1. Mantém uma lista de filtros configuráveis `OredictionificatorItemFilter`. Cada filtro define:
   - Tag de entrada (ex: `#c:ingots/copper`).
   - Item de saída correspondente.
2. A cada tick, se `canFunction()` e houver item no slot de entrada:
   - Avalia as tags do item contra os filtros ativos.
   - Converte 1 item por tick para o item alvo designado.
   - Transfere o resultado para o slot de saída.
3. Tags permitidas são controladas por `MekanismConfig.general.validOredictionificatorFilters` (prefixos como `c:ores`, `c:ingots`, `c:dusts`, `c:gems`, `c:storage_blocks`).

### 6.5 Receita
- **Tipo**: `minecraft:crafting_shaped`
  ```
  SGS
  CPC
  S#S
  ```
  - `S`: `#c:ingots/steel`
  - `G`: `#c:glass_panes`
  - `C`: `#c:circuits/basic`
  - `P`: `mekanism:dictionary`
  - `#`: `#c:chests/wooden`
- **Resultado**: 1 `mekanism:oredictionificator`.

### 6.6 Requisitos do Horizon
- `FORK-16`: Sistema de filtros virtuais dinâmicos em GUI rolável.
- Consulta de tags de itens (`Item.getTags()`) em tempo de execução no WASM.

---

## 7. Formulaic Assemblicator

### 7.1 Identificação e Classes Java
- **Registro**: `mekanism:formulaic_assemblicator`
- **Classes**:
  - `mekanism.common.tile.machine.TileEntityFormulaicAssemblicator`
  - `mekanism.client.gui.machine.GuiFormulaicAssemblicator`
  - `mekanism.common.inventory.container.tile.FormulaicAssemblicatorContainer`
  - `mekanism.common.content.assemblicator.RecipeFormula`
  - `mekanism.common.registries.MekanismBlocks.FORMULAIC_ASSEMBLICATOR`

### 7.2 Blockstates
- **Propriedades**: `facing` (horizontal), `active` (boolean).

### 7.3 Números de Configuração e Energia
- **Tempo Base de Fabricação**: `BASE_TICKS_REQUIRED = 40 ticks` (2,0 segundos).
- **Consumo de Energia**: `UsageConfig.formulaicAssemblicator` = 100 J/t = **40 FE/t** (`crate::fe(100)`).
- **Capacidade do Buffer**: `StorageConfig.formulaicAssemblicator` = 40.000 J = **16.000 FE** (`crate::fe(40_000)`).
- **Upgrades Suportados**:
  - `Upgrade.SPEED`: reduz o tempo de operação exponencialmente até 1 tick.
  - `Upgrade.ENERGY`: reduz o consumo de energia e amplia o buffer interno.

### 7.4 Inventário Completo (35 Slots)
1. **Slot de Fórmula (1 slot)**: em (6, 26). Aceita apenas `mekanism:crafting_formula`.
2. **Grade de Criação (9 slots)**: arranjo 3×3 em `(26 + x*18, 17 + y*18)`.
3. **Buffer de Entrada (18 slots)**: 2 linhas de 9 em `(8 + x*18, 98 + y*18)`.
4. **Buffer de Saída (6 slots)**: 3 linhas de 2 em `(116 + x*18, 17 + y*18)`.
5. **Slot de Energia (1 slot)**: em (152, 76).

### 7.5 Modos Operacionais
- **Modo Manual (`autoMode = false`)**: Funciona como bancada de trabalho tradicional 3×3 conectada aos buffers.
- **Modo Automático (`autoMode = true`)**:
  - Exige `Crafting Formula` codificada e válida no slot de fórmula.
  - Consome energia do buffer.
  - Abastece a grade 3×3 automaticamente a partir dos 18 slots de entrada.
  - A cada ciclo concluído (`operatingTicks >= ticksRequired`), executa a receita e deposita os produtos nos 6 slots de saída.
  - Lida com itens remanescentes (ex.: baldes de leite retornam baldes vazios).
- **Controle de Estoque (`stockControl = true`)**:
  - Trava cada um dos 18 slots de entrada para aceitar apenas os itens definidos como ingredientes na fórmula codificada.

### 7.6 Receita
- **Tipo**: `minecraft:crafting_shaped`
  ```
  SPS
  CXC
  S#S
  ```
  - `S`: `#c:ingots/steel`
  - `P`: `minecraft:crafter`
  - `C`: `#c:circuits/basic`
  - `X`: `mekanism:steel_casing`
  - `#`: `#c:chests/wooden`
- **Resultado**: 1 `mekanism:formulaic_assemblicator`.

### 7.7 Requisitos do Horizon
- Suporte a contêiner com 35 slots (supera o teto de 32 slots do Horizon).
- Resolução e execução dinâmica de receitas de crafting do jogo (`CraftingRecipe`).

---

## 8. Crafting Formula (Item)

### 8.1 Identificação e Classes Java
- **Registro**: `mekanism:crafting_formula`
- **Classes**:
  - `mekanism.common.item.ItemCraftingFormula`
  - `mekanism.common.attachments.FormulaAttachment`
  - `mekanism.common.registries.MekanismDataComponents.FORMULA_HOLDER`

### 8.2 Componentes e Estrutura de Dados
- `MekanismDataComponents.FORMULA_HOLDER`:
  - Contém `FormulaAttachment`, que encapsula um vetor de 9 `ItemStack` representando a grade 3×3 codificada.
  - Se vazia: `FormulaAttachment.EMPTY`.

### 8.3 Interações e Apresentação
- **Limpeza**: Agachado (Shift) + Clique com botão direito no ar limpa a fórmula, redefinindo o componente para `EMPTY`.
- **Nome de Exibição Dinâmico**:
  - Vazia: `"Crafting Formula"`
  - Codificada válida: `"Crafting Formula (Encoded)"` (texto verde escuro `EnumColor.DARK_GREEN`).
  - Codificada inválida: `"Crafting Formula (Invalid)"` (texto vermelho escuro `EnumColor.DARK_RED`).
- **Tooltip**: Lista todos os ingredientes agrupados por tipo e contagem.

### 8.4 Receita
- **Tipo**: `minecraft:crafting_shapeless`
  - 1 `minecraft:paper` + 1 `#c:circuits/basic`
- **Resultado**: 1 `mekanism:crafting_formula`.

### 8.5 Requisitos do Horizon
- `FORK-23`: Suporte a data components em itens contendo lista serializada de stacks.

---

## 9. Configuration Card (Item)

### 8.1 Identificação e Classes Java
- **Registro**: `mekanism:configuration_card`
- **Classes**:
  - `mekanism.common.item.ItemConfigurationCard`
  - `mekanism.api.IConfigCardAccess`
  - `mekanism.common.registries.MekanismDataComponents.CONFIGURATION_DATA`
  - `mekanism.common.advancements.triggers.ConfigurationCardTrigger`

### 9.2 Propriedades do Item
- Empilhamento: 1 stack (`stacksTo(1)`).
- Raridade: `Rarity.UNCOMMON` (amarelo).

### 9.3 Dados Armazenados
- `MekanismDataComponents.CONFIGURATION_DATA`: `CompoundTag` contendo:
  - `dataName`: chave de tradução da máquina de origem (ex.: `"tile.mekanism.enrichment_chamber.name"`).
  - `dataType`: ResourceLocation do bloco (ex.: `"mekanism:enrichment_chamber"`).
  - Tags completas de configuração: faces de transmissão (energia, itens, fluidos, gases), auto-eject, modos de redstone.

### 9.4 Interações
1. **Copiar Configuração (Shift + Clique direito na máquina)**:
   - Verifica segurança do jogador (`IBlockSecurityUtils`).
   - Obtém a capacidade `Capabilities.CONFIG_CARD` (`IConfigCardAccess`).
   - Grava a configuração no cartão.
   - Envia mensagem no action bar do jogador informando os dados copiados.
   - Aciona o gatilho de avanço `CONFIGURATION_CARD` com `copied = true`.
2. **Colar Configuração (Clique direito sem agachar na máquina)**:
   - Verifica compatibilidade (`isConfigurationDataCompatible`).
   - Aplica a configuração na máquina de destino.
   - Envia mensagem de sucesso ou mensagem de erro em vermelho se o tipo for incompatível.
   - Aciona o avanço `CONFIGURATION_CARD` com `copied = false`.
3. **Limpar Cartão (Shift + Clique direito no ar)**:
   - Remove o componente `CONFIGURATION_DATA`.
   - Envia mensagem de confirmação de limpeza.

### 9.5 Receita
- **Tipo**: `minecraft:crafting_shaped` (cruz 3×3)
  ```
   A 
  ACA
   A 
  ```
  - `A`: `#mekanism:alloys/infused`
  - `C`: `#c:circuits/basic`
- **Resultado**: 1 `mekanism:configuration_card`.

### 9.6 Requisitos do Horizon
- `FORK-23`: Persistência de NBT arbitrário em componentes de item e interface de configuração unificada entre máquinas.
