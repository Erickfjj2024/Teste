# Especificação Técnica Completa: QIO, Sistema de Segurança e Modification Station

**Grupo de Peças**: QIO Dashboard, QIO Drive Array, QIO Exporter, QIO Importer, QIO Redstone Adapter, QIO Drives (Base, Hyper-Dense, Time-Dilating, Supermassive), Security Desk (e Sistema de Segurança/Ownership), Modification Station.  
**Referência Canônica**: Mekanism Java `1.21.1-10.7.19.85` (`java/`)  
**Data**: 2026-11-09  
**Finalidade**: Especificação exaustiva para implementação direta em Rust no Horizon (mod WASM) sem necessidade de consultar o código Java.

---

## 1. Visão Geral da Arquitetura

O subsistema abordado nesta especificação divide-se em três pilares fundamentais:
1. **Rede de Armazenamento Quântico (QIO - Quantum Item Orchestration)**: Armazenamento massivo de itens operando em frequências universais sem cabos físicos, indexado em memórias de estado sólido (QIO Drives) hospedadas em matrizes (Drive Arrays) e acessado/manipulado via Dashboards, Importers, Exporters e Adaptadores de Redstone.
2. **Sistema de Segurança e Posse (Security & Ownership)**: Gestão de permissões de blocos e frequências por jogador (UUID), com três níveis de acesso (`PUBLIC`, `PRIVATE`, `TRUSTED`), lista de amigos confiáveis e override global via Security Desk com proteção de dureza inquebrável.
3. **Estação de Modificação (Modification Station)**: Unidade de engenharia eletromecânica para instalação e remoção de módulos tecnológicos em trajes (MekaSuit), ferramentas modulares (Meka-Tool) e recipientes modulares compatíveis.

---

## 2. QIO Drives (Itens de Armazenamento)

### 2.1 Registro e Classes Java
- **Classe do Item**: `mekanism.common.item.ItemQIODrive` em `java/main/java/mekanism/common/item/ItemQIODrive.java`
- **Enum de Tiers**: `mekanism.common.tier.QIODriveTier` em `java/main/java/mekanism/common/tier/QIODriveTier.java`
- **Nomes de Registro**:
  - `mekanism:qio_drive_base` (Tier Base / Basic)
  - `mekanism:qio_drive_hyper_dense` (Tier Hiper-Densa / Advanced)
  - `mekanism:qio_drive_time_dilating` (Tier Dilatação Temporal / Elite)
  - `mekanism:qio_drive_supermassive` (Tier Supermassiva / Ultimate)

### 2.2 Números e Capacidades por Tier
| Tier | BaseTier | Capacidade de Itens (Max Count) | Capacidade de Tipos (Max Types) |
|---|---|---|---|
| **Base** | Basic | 16.000 itens | 128 tipos |
| **Hyper-Dense** | Advanced | 128.000 itens | 256 tipos |
| **Time-Dilating** | Elite | 1.048.000 itens | 1.024 tipos |
| **Supermassive** | Ultimate | 16.000.000.000 itens (16 bilhões) | 8.192 tipos |

### 2.3 Componentes de Dados e Anexos (NBT / Data Components)
- `MekanismDataComponents.DRIVE_METADATA`:
  - `count`: `long` (número total de itens armazenados no drive).
  - `types`: `int` (quantidade de tipos distintos de itens armazenados).
- `MekanismDataComponents.DRIVE_CONTENTS`:
  - Serialização em mapa de pares `(HashedItem, long)`. Ao ser retirado do Drive Array, os dados reais do inventário são preservados no componente de dados do item stack.

### 2.4 Receitas Datagen (JSON)
- **qio_drive_base**:
  ```json
  {
    "type": "minecraft:crafting_shaped",
    "pattern": [
      "ICI",
      "C#C",
      "ICI"
    ],
    "key": {
      "#": { "tag": "c:ender_pearls" },
      "C": { "item": "mekanism:ultimate_control_circuit" },
      "I": { "tag": "c:ingots/lead" }
    },
    "result": { "count": 1, "id": "mekanism:qio_drive_base" }
  }
  ```
- **qio_drive_hyper_dense**:
  ```json
  {
    "type": "mekanism:mek_data",
    "pattern": [
      "IPI",
      "P#P",
      "IPI"
    ],
    "key": {
      "#": { "item": "mekanism:teleportation_core" },
      "I": { "tag": "c:pellets/plutonium" },
      "P": { "item": "mekanism:qio_drive_base" }
    },
    "result": { "count": 1, "id": "mekanism:qio_drive_hyper_dense" }
  }
  ```
- **qio_drive_time_dilating**:
  ```json
  {
    "type": "mekanism:mek_data",
    "pattern": [
      "IPI",
      "P#P",
      "IPI"
    ],
    "key": {
      "#": { "tag": "c:pellets/polonium" },
      "I": { "tag": "c:pellets/plutonium" },
      "P": { "item": "mekanism:qio_drive_hyper_dense" }
    },
    "result": { "count": 1, "id": "mekanism:qio_drive_time_dilating" }
  }
  ```
- **qio_drive_supermassive**:
  ```json
  {
    "type": "mekanism:mek_data",
    "pattern": [
      "IPI",
      "P#P",
      "IPI"
    ],
    "key": {
      "#": { "tag": "c:pellets/antimatter" },
      "I": { "tag": "c:pellets/polonium" },
      "P": { "item": "mekanism:qio_drive_time_dilating" }
    },
    "result": { "count": 1, "id": "mekanism:qio_drive_supermassive" }
  }
  ```

---

## 3. QIO Drive Array (Matriz de Unidades QIO)

### 3.1 Registro e Classes Java
- **Bloco**: `mekanism:qio_drive_array` (`MekanismBlocks.QIO_DRIVE_ARRAY`)
- **Block Class**: `BlockQIOComponent<TileEntityQIODriveArray>`
- **Tile Entity Class**: `mekanism.common.tile.qio.TileEntityQIODriveArray`
- **Container Class**: `MekanismTileContainer<TileEntityQIODriveArray>` (`MekanismContainerTypes.QIO_DRIVE_ARRAY`)
- **GUI Class**: `mekanism.client.gui.qio.GuiQIODriveArray`

### 3.2 Estados de Bloco e Modelagem
- Propriedades de Bloco:
  - `BlockStateProperties.HORIZONTAL_FACING`: Direção frontal (NORTH, SOUTH, EAST, WEST).
  - `AttributeStateActive.active`: Booleano (true quando vinculado a frequência QIO válida).
- Modelo de Renderização:
  - `DriveArrayBakedModel.java` com a propriedade `DRIVE_STATUS_PROPERTY` (`byte[12]`):
    - `0`: NONE (slot vazio, sem LED).
    - `1`: OFFLINE (drive inserido porém frequência offline ou nula).
    - `2`: READY (drive operacional com espaço e tipos disponíveis, LED verde).
    - `3`: NEAR_FULL (capacidade de tipos atingida OU $\ge 75\%$ da capacidade de itens ocupada, LED amarelo).
    - `4`: FULL (100% da capacidade de itens preenchida, LED vermelho).

### 3.3 Inventário e Slots na GUI
- Dimensões da GUI: `imageWidth = 176`, `imageHeight = 166 + 40 = 206`.
- Total de Slots de Drive: 12 slots dispostos em 2 fileiras horizontais de 6 slots:
  - Fileira 0 (slots 0 a 5): $Y = 70$, $X = 34 + (x \times 18)$ ($X \in \{34, 52, 70, 88, 106, 124\}$).
  - Fileira 1 (slots 6 a 11): $Y = 88$, $X = 34 + (x \times 18)$ ($X \in \{34, 52, 70, 88, 106, 124\}$).
- Slot Filter: Aceita estritamente instâncias de `IQIODriveItem` (`ItemQIODrive`).
- Visualização de Dados da Frequência:
  - Tela `GuiQIOFrequencyDataScreen`: $X = 15, Y = 19$, largura 144, altura 46. Mostra nome da frequência, capacidade total de itens usada/máx, capacidade total de tipos usada/máx e contagem de drives ativos.

### 3.4 Persistência e Atualização
- Atualização a cada 10 ticks (`MekanismUtils.TICKS_PER_HALF_SECOND = 10`):
  - Varre os 12 slots, calcula o status dos LEDs e emite pacote de sincronização leve se houver alteração de hash (`prevDriveHash != newHash`).
- Salvamento em NBT:
  - Chama `freq.saveAll()` antes de serializar o TileEntity.
  - Salva o array de slots padrão e a frequência associada (`FrequencyAware`).

### 3.5 Receita Datagen (JSON)
```json
{
  "type": "mekanism:mek_data",
  "pattern": [
    "TGT",
    "C#C",
    "TIT"
  ],
  "key": {
    "#": { "tag": "mekanism:personal_storage" },
    "C": { "tag": "c:circuits/ultimate" },
    "G": { "tag": "c:glass_panes" },
    "I": { "tag": "c:ender_pearls" },
    "T": { "item": "mekanism:teleportation_core" }
  },
  "result": { "count": 1, "id": "mekanism:qio_drive_array" }
}
```

---

## 4. QIO Dashboard (Painel de Acesso QIO)

### 4.1 Registro e Classes Java
- **Bloco**: `mekanism:qio_dashboard` (`MekanismBlocks.QIO_DASHBOARD`)
- **Tile Entity Class**: `mekanism.common.tile.qio.TileEntityQIODashboard`
- **Container Class**: `mekanism.common.inventory.container.tile.QIODashboardContainer`
- **GUI Class**: `mekanism.client.gui.qio.GuiQIODashboard`

### 4.2 Estados de Bloco
- `BlockStateProperties.FACING`: Direção 6D completa (UP, DOWN, NORTH, SOUTH, EAST, WEST), permitindo fixação em pisos, tetos ou paredes (`FacePlacementType.SELECTED_FACE`).
- `AttributeStateActive.active`: Ativo quando conectado a frequência válida.

### 4.3 Inventário e Grades de Crafting
- Suporta **3 Janelas de Crafting Independentes** (`MAX_CRAFTING_WINDOWS = 3`, índices 0, 1 e 2).
- Cada janela contém:
  - 9 slots de grade de crafting 3x3 ($X, Y$ definidos por janela na GUI).
  - 1 slot de saída de crafting (`CraftingWindowOutputInventorySlot`).
- Total de slots de inventário no Tile Entity: 30 slots (não expostos a automação externa, apenas leitura interna).
- Flag `insertIntoFrequency`: Booleano (padrão `true`). Quando o jogador dá shift-click em itens de seu inventário dentro da GUI do Dashboard:
  - `true`: Envia direto para a frequência QIO.
  - `false`: Move para a hotbar / inventário principal do jogador.
- Componente de dados associado: `MekanismDataComponents.INSERT_INTO_FREQUENCY`.

### 4.4 Pacotes e Sincronização
- Ao abrir o contêiner: o servidor envia `BulkQIOData` contendo a lista completa de tipos de itens e contagens armazenadas na frequência para que o cliente renderize a lista virtual rolável com barra de pesquisa.
- Interações do jogador no catálogo do Dashboard:
  - `PacketQIOItemViewerSlotTake`: Retira 1, meio stack ou 1 stack inteiro de um item pesquisado.
  - `PacketQIOItemViewerSlotPlace`: Insere item da mão do cursor na frequência.
  - `PacketQIOItemViewerSlotShiftTake`: Retira stack diretamente para o inventário do jogador.
  - `PacketQIOClearCraftingWindow`: Devolve os 9 itens da grade de crafting atual para a frequência QIO.
  - `PacketQIOFillCraftingWindow`: Preenche a grade 3x3 com base em uma receita JEI/EMI ou histórico.

### 4.5 Receita Datagen (JSON)
```json
{
  "type": "minecraft:crafting_shaped",
  "pattern": [
    "IAI",
    "AGA",
    "ITI"
  ],
  "key": {
    "A": { "tag": "c:ender_pearls" },
    "G": { "tag": "c:glass_panes" },
    "I": { "tag": "c:ingots/lead" },
    "T": { "item": "mekanism:teleportation_core" }
  },
  "result": { "count": 1, "id": "mekanism:qio_dashboard" }
}
```

---

## 5. QIO Importer (Importador QIO)

### 5.1 Registro e Classes Java
- **Bloco**: `mekanism:qio_importer` (`MekanismBlocks.QIO_IMPORTER`)
- **Tile Entity Class**: `mekanism.common.tile.qio.TileEntityQIOImporter`
- **GUI Class**: `mekanism.client.gui.qio.GuiQIOImporter`

### 5.2 Estados de Bloco e Upgrades
- `BlockStateProperties.FACING`: Direção 6D do bocal (`SELECTED_FACE`). O bloco extrai itens do contêiner localizado na sua face traseira (`getOppositeDirection()`).
- Suporte a Upgrades: Estritamente **Speed Upgrade** (`AttributeUpgradeSupport.SPEED_ONLY`), até 8 upgrades.
- Cálculo de Trânsito:
  - Quantidade Máxima por Pulso: $\text{maxTransitCount} = 64 + 32 \times \text{speedUpgrades}$ (64 itens com 0 upgrades até 320 itens com 8 upgrades).
  - Tipos Distintos por Pulso: $\text{maxTransitTypes} = \text{round}(1.0 + \frac{\text{speedUpgrades}}{2.0})$ (1 tipo com 0 upgrades até 5 tipos com 8 upgrades).

### 5.3 Lógica de Funcionamento por Tick
- Taxa de disparo: Opera a cada 10 ticks (`MAX_DELAY = MekanismUtils.TICKS_PER_HALF_SECOND = 10`), ou seja, 2 vezes por segundo.
- Modo de Extração:
  - `importWithoutFilter`: Booleano (padrão `true`). Se não houver filtros configurados, puxa qualquer item do contêiner vizinho traseiro. Se `false`, só puxa itens que casem com filtros ativos.
- Filtros Suportados (`SortableFilterManager<QIOFilter<?>>`):
  - `QIOItemStackFilter`: Filtra por Item / ItemStack exato.
  - `QIOTagFilter`: Filtra por tag do Minecraft/Forge/NeoForge (ex.: `c:ingots/iron`).
  - `QIOModIDFilter`: Filtra por mod de origem (ex.: `mekanism`, `minecraft`).
- Persistência:
  - Salva em NBT sob a tag `SerializationConstants.AUTO` ("auto") e DataComponent `MekanismDataComponents.AUTO`.

### 5.4 Receita Datagen (JSON)
```json
{
  "type": "minecraft:crafting_shaped",
  "pattern": [
    "ITI",
    "ACA",
    " # "
  ],
  "key": {
    "#": { "item": "minecraft:sticky_piston" },
    "A": { "tag": "c:ender_pearls" },
    "C": { "tag": "c:circuits/ultimate" },
    "I": { "tag": "c:ingots/lead" },
    "T": { "item": "mekanism:teleportation_core" }
  },
  "result": { "count": 1, "id": "mekanism:qio_importer" }
}
```

---

## 6. QIO Exporter (Exportador QIO)

### 6.1 Registro e Classes Java
- **Bloco**: `mekanism:qio_exporter` (`MekanismBlocks.QIO_EXPORTER`)
- **Tile Entity Class**: `mekanism.common.tile.qio.TileEntityQIOExporter`
- **GUI Class**: `mekanism.client.gui.qio.GuiQIOExporter`

### 6.2 Estados de Bloco e Upgrades
- `BlockStateProperties.FACING`: Direção 6D (`SELECTED_FACE`). Ejeta itens para o contêiner ou transportador logístico situado na face traseira.
- Suporte a Upgrades: Estritamente **Speed Upgrade**, teto de 8 upgrades.
- Capacidades de trânsito idênticas ao Importer:
  - $\text{maxTransitCount} = 64 + 32 \times \text{speedUpgrades}$ (64 a 320 itens).
  - $\text{maxTransitTypes} = \text{round}(1.0 + \frac{\text{speedUpgrades}}{2.0})$ (1 a 5 tipos).

### 6.3 Lógica de Funcionamento e Roteamento
- Atraso de 10 ticks (`MAX_DELAY = 10`).
- Configurações do Exportador:
  - `exportWithoutFilter`: Booleano (padrão `false`). Se ativado, ejeta qualquer item contido na rede QIO.
  - `roundRobin`: Booleano (padrão `false`). Quando conectado a uma rede de Logistical Transporters, distribui os itens alternadamente entre múltiplos destinos válidos.
- Integração com Logistical Transporters:
  - Se a face traseira possuir um transportador logístico, cria uma requisição de trânsito (`TransitRequest`) e calcula rota válida com cor do transmissor respeitada.
- Persistência:
  - Salva `exportWithoutFilter` em `SerializationConstants.AUTO` e `roundRobin` em `SerializationConstants.ROUND_ROBIN`.

### 6.4 Receita Datagen (JSON)
```json
{
  "type": "minecraft:crafting_shaped",
  "pattern": [
    "ITI",
    "ACA",
    " # "
  ],
  "key": {
    "#": { "item": "minecraft:piston" },
    "A": { "tag": "c:ender_pearls" },
    "C": { "tag": "c:circuits/ultimate" },
    "I": { "tag": "c:ingots/lead" },
    "T": { "item": "mekanism:teleportation_core" }
  },
  "result": { "count": 1, "id": "mekanism:qio_exporter" }
}
```

---

## 7. QIO Redstone Adapter (Adaptador de Redstone QIO)

### 7.1 Registro e Classes Java
- **Bloco**: `mekanism:qio_redstone_adapter` (`MekanismBlocks.QIO_REDSTONE_ADAPTER`)
- **Tile Entity Class**: `mekanism.common.tile.qio.TileEntityQIORedstoneAdapter`
- **GUI Class**: `mekanism.client.gui.qio.GuiQIORedstoneAdapter`

### 7.2 Emissão de Redstone e Estados
- `BlockStateProperties.FACING`: Orientação 6D (`SELECTED_FACE`).
- Emissão de Sinal Redstone:
  - Quando a condição é atendida e o bloco está ativo, emite **sinal de força 15** em **todas as 5 faces adjacentes**, EXCETO na face traseira (`side != getOppositeDirection()`).
- Condição de Emissão:
  $$\text{hasStored} = (\text{storedCount} > 0 \land \text{storedCount} \ge \text{countTrigger})$$
  $$\text{isEmitting} = (\text{hasStored} \neq \text{inverted})$$
- Modos de Configuração:
  - `itemType`: Item alvo a ser monitorado (`HashedItem`). Pode ser definido clicando com item na interface ou digitando o ID.
  - `count`: Quantidade de gatilho (`long`, padrão 0).
  - `fuzzy`: Booleano (padrão `false`). Se ativo, compara apenas o `Item` base, ignorando durabilidade, encantamentos ou data components adicionais.
  - `inverted`: Booleano (padrão `false`). Se ativo, emite sinal de redstone quando a contagem for MENOR que o gatilho.

### 7.3 Persistência e NBT
- `SerializationConstants.SINGLE_ITEM`: NBT composto do item alvo.
- `SerializationConstants.AMOUNT`: `count` (`long`).
- `SerializationConstants.FUZZY`: `fuzzy` (`boolean`).
- `SerializationConstants.INVERSE`: `inverted` (`boolean`).
- Data Components: `MekanismDataComponents.ITEM_TARGET`, `MekanismDataComponents.LONG_AMOUNT`, `MekanismDataComponents.FUZZY`, `MekanismDataComponents.INVERSE`.

### 7.4 Receita Datagen (JSON)
```json
{
  "type": "minecraft:crafting_shaped",
  "pattern": [
    "IWI",
    "CRC",
    "ITI"
  ],
  "key": {
    "C": { "tag": "c:circuits/ultimate" },
    "I": { "tag": "c:ender_pearls" },
    "R": { "tag": "c:dusts/redstone" },
    "T": { "item": "mekanism:teleportation_core" },
    "W": { "item": "minecraft:redstone_torch" }
  },
  "result": { "count": 1, "id": "mekanism:qio_redstone_adapter" }
}
```

---

## 8. Security Desk e Sistema de Segurança/Ownership

### 8.1 Registro e Classes Java
- **Bloco**: `mekanism:security_desk` (`MekanismBlocks.SECURITY_DESK`)
- **Block Class**: `BlockTileModel<TileEntitySecurityDesk>`
- **Tile Entity Class**: `mekanism.common.tile.TileEntitySecurityDesk` em `java/main/java/mekanism/common/tile/TileEntitySecurityDesk.java`
- **Frequência de Segurança**: `mekanism.common.lib.security.SecurityFrequency`
- **Utilitários e Enums**: `mekanism.api.security.SecurityMode`, `mekanism.common.lib.security.SecurityUtils`, `mekanism.common.tile.component.TileComponentSecurity`
- **GUI Class**: `mekanism.client.gui.GuiSecurityDesk`

### 8.2 Geometria e Bounding Blocks (Multibloco 1x2x1)
- O Security Desk ocupa **2 blocos de altura**:
  - Bloco Base: Posição `pos` onde o bloco principal foi colocado.
  - Bloco Bounding: Posição `pos.above()` gerenciada via `IBoundingBlock` (`AttributeHasBounding.ABOVE_ONLY`).
  - Ambas as posições apontam para o mesmo Tile Entity base. Se qualquer um dos dois blocos for quebrado, o conjunto inteiro é destruído.
- Resistência Inquebrável: Possui `AttributeCustomResistance(-1)`. Enquanto estiver trancado por um dono, jogadores sem permissão não conseguem quebrá-lo no modo sobrevivência.

### 8.3 Modos de Segurança (`SecurityMode`)
1. **`PUBLIC`**: Qualquer jogador pode interagir com a máquina, abrir sua GUI, quebrar ou reconfigurar.
2. **`PRIVATE`**: Exclusivo do proprietário (comparação de UUID do jogador com `ownerUUID`).
3. **`TRUSTED`**: O proprietário E qualquer jogador presente na lista de confiança (`trustedPlayers`) do proprietário têm acesso livre.

### 8.4 Funcionamento da Frequência de Segurança
- Cada jogador possui uma única frequência global de segurança (`SecurityFrequency`) associada ao seu UUID.
- No Security Desk:
  - `trustedPlayers`: Lista ordenada de perfis autorizados (UUID + Nome do jogador). Adicionados pelo campo de texto na GUI via `PacketAddTrusted`.
  - `isOverridden`: Booleano (padrão `false`). Quando o botão **Override** está ativado no Security Desk, **todas as máquinas e tanques** do dono passam a adotar compulsoriamente o modo de segurança configurado na mesa, ignorando as configurações locais de cada máquina.
  - Se um jogador perde o acesso ou é removido da lista de confiança enquanto está com a GUI de uma máquina aberta, o servidor imediatamente força o fechamento do contêiner (`player.closeContainer()`).

### 8.5 Inventário e Slots do Security Desk
- Dimensões e Slots:
  - Slot 0 (Unlock Slot): $X = 146, Y = 18$. Remove o dono e a segurança de itens de segurança inseridos (ex.: Security Cards ou blocos desmontados).
  - Slot 1 (Lock Slot): $X = 146, Y = 97$. Atribui o dono atual e a frequência de segurança ao item inserido.

### 8.6 Receita Datagen (JSON)
```json
{
  "type": "minecraft:crafting_shaped",
  "pattern": [
    "SGS",
    "CXC",
    "S#S"
  ],
  "key": {
    "#": { "item": "mekanism:network_reader" },
    "C": { "tag": "c:circuits/elite" },
    "G": { "tag": "c:glass_blocks/cheap" },
    "S": { "tag": "c:ingots/steel" },
    "X": { "item": "mekanism:steel_casing" }
  },
  "result": { "count": 1, "id": "mekanism:security_desk" }
}
```

---

## 9. Modification Station (Estação de Modificação)

### 9.1 Registro e Classes Java
- **Bloco**: `mekanism:modification_station` (`MekanismBlocks.MODIFICATION_STATION`)
- **Tile Entity Class**: `mekanism.common.tile.TileEntityModificationStation` em `java/main/java/mekanism/common/tile/TileEntityModificationStation.java`
- **GUI Class**: `mekanism.client.gui.GuiModificationStation`

### 9.2 Geometria e Bounding Blocks (Multibloco 2x2x1)
- Ocupa um footprint total de **4 blocos (2 de largura por 2 de altura)** relativo ao `facing`:
  1. Base: `pos`
  2. Superior: `pos.relative(Direction.UP)`
  3. Lateral Direita: `pos.relative(MekanismUtils.getRight(facing))`
  4. Superior Direita: `pos.relative(MekanismUtils.getRight(facing)).relative(Direction.UP)`
- Destruição: Quebrar qualquer um dos 4 blocos desfaz a estação inteira e devolve os itens contidos nos slots.

### 9.3 Configurações de Energia e Operação
- **Armazenamento de Energia (`StorageConfig.java:136`)**:
  - $40.000\text{ Joules} = \mathbf{16.000\text{ FE}}$
- **Consumo de Energia por Tick (`UsageConfig.java:129`)**:
  - $400\text{ Joules/tick} = \mathbf{160\text{ FE/tick}}$
- **Duração da Operação**:
  - $\text{ticksRequired} = 10\text{ ticks}$ ($0,5\text{ segundo}$, `MekanismUtils.TICKS_PER_HALF_SECOND`).
- **Energia Total por Módulo Instalado**:
  - $10\text{ ticks} \times 160\text{ FE/t} = \mathbf{1.600\text{ FE}}$ ($4.000\text{ Joules}$).
- Não aceita Speed nem Energy Upgrades.

### 9.4 Inventário e Slots da GUI
- **Slot de Módulo** (`moduleSlot`): $X = 35, Y = 118$. Aceita itens `IModuleItem` (unidades de módulo MekaSuit / Meka-Tool).
- **Slot de Equipamento** (`containerSlot`): $X = 125, Y = 118$. Aceita recipientes de módulos (`IModuleHelper.isModuleContainer`, como MekaSuit Helmet, BodyArmor, Pants, Boots, Meka-Tool).
- **Slot de Energia** (`energySlot`): $X = 151, Y = 21$. Aceita itens para descarga na máquina.
- Lista Lateral de Módulos Instalados (`GuiModuleScrollList`): Exibe os módulos atualmente presentes no item do slot de equipamento, permitindo desinstalação via botão "Remove" (`PacketRemoveModule`).

### 9.5 Receita Datagen (JSON)
```json
{
  "type": "minecraft:crafting_shaped",
  "pattern": [
    "PWP",
    "CXC",
    "PAP"
  ],
  "key": {
    "A": { "tag": "c:pellets/polonium" },
    "C": { "tag": "c:circuits/ultimate" },
    "P": { "item": "mekanism:hdpe_sheet" },
    "W": { "tag": "c:chests/wooden" },
    "X": { "item": "mekanism:steel_casing" }
  },
  "result": { "count": 1, "id": "mekanism:modification_station" }
}
```

---

## 10. Requisitos de Plataforma Horizon (Mapeamento de FORKs)

Para viabilizar a implementação fiel destas peças no mod Horizon em Rust, são requeridas as seguintes capacidades de API formalizadas em `port/API-REQUISITOS.md`:

| Capacidade Necessária | FORK ID | Peças que Exigem | Descrição da Capacidade de API |
|---|---|---|---|
| **Frequências Universais por UUID e Nome** | `FORK-18` | QIO Drive Array, Dashboard, Importer, Exporter, Redstone Adapter, Security Desk | Suporte no host Horizon para gerenciamento global de frequências nomeadas (`QIOFrequency`, `SecurityFrequency`), com persistência de mundo e suporte a modo Público/Privado por jogador |
| **Componentes de Dados em Itens (Data Components)** | `FORK-23` | QIO Drives, Dashboard, Redstone Adapter | Capacidade de serializar estruturas ricas dentro do itemstack (`DriveContents`, `DriveMetadata`, filtros, alvos) sem perder estado ao dropar ou mover no inventário |
| **Pacotes de Rede e Sincronização em Lote** | `FORK-24` | QIO Dashboard, Security Desk, Modification Station | Transmissão de catálogos volumosos (`BulkQIOData`), requisições de crafting/clique de slots virtuais e pacotes de sincronização de segurança (`PacketSyncSecurity`) |
| **GUI com Widgets Customizados e Listas Virtuais** | `FORK-20` | QIO Dashboard, Drive Array, Security Desk, Modification Station | Elementos de interface gráfica: campo de busca de texto, barra de rolagem virtual contínua de milhares de itens, abas de frequência e telas integradas |
| **Bounding Blocks com Múltiplas Caixas** | `5.1` / `FORK novo` | Security Desk (1x2x1), Modification Station (2x2x1) | API do Horizon para declaração de blocos auxiliares invisíveis (bounding blocks) que redirecionam cliques, quebra e contêiner para a posição mestre |
| **Sistema de Módulos de Equipamentos** | `FORK-27` | Modification Station | Suporte a dados acoplados em armaduras e ferramentas para instalação e ativação em tempo de execução dos módulos tecnológicos |
| **Resistência Inquebrável Customizada** | `FORK novo` | Security Desk | Capacidade de definir resistência e dureza dinâmica de blocos (valor `-1.0f`) baseada no proprietário para prevenir quebra por terceiros |
