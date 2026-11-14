# Especificação Técnica: Grupo Calor e Transporte

**ID da Tarefa**: T20261109-194317-824  
**Referência de Verdade**: Mekanism 1.21.1-10.7.19.85 (`java/`)  
**Objetivo**: Fornecer a especificação completa e detalhada para implementação direta em Rust/Horizon, sem necessidade de consultar o código Java.

---

## 1. Thermodynamic Conductor (Rede de Calor Completa)

### 1.1 Identificação e Registros
- **Nomes de Registro**:
  - `mekanism:basic_thermodynamic_conductor` (Tier Básico)
  - `mekanism:advanced_thermodynamic_conductor` (Tier Avançado)
  - `mekanism:elite_thermodynamic_conductor` (Tier Elite)
  - `mekanism:ultimate_thermodynamic_conductor` (Tier Supremo)
- **Classes Java**:
  - `mekanism.common.content.network.transmitter.ThermodynamicConductor` (`ThermodynamicConductor.java`)
  - `mekanism.common.tile.transmitter.TileEntityThermodynamicConductor` (`TileEntityThermodynamicConductor.java`)
  - `mekanism.common.content.network.HeatNetwork` (`HeatNetwork.java`)
  - `mekanism.common.tier.ConductorTier` (`ConductorTier.java`)
  - `mekanism.api.heat.HeatAPI` (`HeatAPI.java`)
  - `mekanism.common.capabilities.heat.ITileHeatHandler` (`ITileHeatHandler.java`)

### 1.2 Estados de Bloco e Blockstates
- **Propriedades**:
  - 6 conexões espaciais: `down`, `up`, `north`, `south`, `west`, `east`.
  - Valores possíveis de cada face: `NONE` (sem conexão), `NORMAL` (conectado a outro condutor ou máquina), `PUSH` (emissão forçada), `PULL` (extração).
  - Estado de sensibilidade a redstone: `redstone_reactive` (boolean, padrão false).

### 1.3 Campos do Tile Entity e Persistência
- **Campos**:
  - `buffer`: capacitor de calor `VariableHeatCapacitor`.
  - `heat`: energia térmica total armazenada em Joules (`heat = temperature * heatCapacity`).
  - `temperature`: temperatura instantânea em Kelvin (`T = heat / heatCapacity`).
  - `connections`: vetor de 6 estados de conexão.
  - `redstone_reactive`: boolean.
- **Serialização (NBT)**:
  - Salvo via `ContainerType.HEAT` sob a tag `"mekanism:heat_capacitors"` (lista de compostos com campo `"Heat"` tipo double).
  - Conexões salvas em array de bytes (`"connections"`).
- **Pacotes de Rede (Cliente-Servidor)**:
  - Campo `"temperature"` (double) sincronizado para o cliente quando `abs(temp - clientTemp) > (temp / 20)`.

### 1.4 Todos os Números e Configurações
- **Parâmetros por Tier**:
  | Tier | Condução Inversa (`invConduction`) | Capacidade Térmica (`heatCapacity`) | Isolamento Inverso (`invInsulation`) | Cor Base RGBA |
  |---|---|---|---|---|
  | **Basic** | 5,0 | 1,0 J/K | 10,0 | `(0.2, 0.2, 0.2, 1.0)` |
  | **Advanced** | 5,0 | 1,0 J/K | 400,0 | `(0.2, 0.2, 0.2, 1.0)` |
  | **Elite** | 5,0 | 1,0 J/K | 8.000,0 | `(0.2, 0.2, 0.2, 1.0)` |
  | **Ultimate** | 5,0 | 1,0 J/K | 100.000,0 | `(0.2, 0.2, 0.2, 1.0)` |
- **Constantes Globais de Calor (`HeatAPI.java`)**:
  - `AMBIENT_TEMP`: 300,0 K (26,85 °C).
  - `AIR_INVERSE_COEFFICIENT`: 10.000,0.
  - Temperatura do bioma: `T_amb = clamp(300.0 + 25.0 * (biomeTemp - 0.8), 0.0, 500.0)`.

### 1.5 Algoritmo da Rede Térmica (`HeatNetwork.java`)
A cada tick do servidor:
1. **Dissipação Ambiente (`simulateEnvironment`)**:
   Para cada condutor na rede e para cada uma de suas 6 faces:
   $$\text{invTotal} = \text{AIR\_INVERSE\_COEFFICIENT} + \text{invInsulation} + \text{invConduction}$$
   $$\Delta T = \frac{T_{\text{condutor}} - T_{\text{ambiente}}}{\text{invTotal}}$$
   $$Q_{\text{perda}} = \Delta T \times \text{heatCapacity}$$
   $$\text{heat} \leftarrow \text{heat} - Q_{\text{perda}}$$
2. **Transferência Adjacente (`simulateAdjacent`)**:
   Para cada face conectada a um bloco adjacente com capacidade térmica:
   $$\text{invConducaoTotal} = \text{invConduction}_{\text{vizinho}} + \text{invConduction}_{\text{condutor}}$$
   $$\Delta T = \frac{T_{\text{condutor}} - T_{\text{vizinho}}}{\text{invConducaoTotal}}$$
   $$Q_{\text{transf}} = \Delta T \times \text{heatCapacity}$$
   $$\text{heat}_{\text{condutor}} \leftarrow \text{heat}_{\text{condutor}} - Q_{\text{transf}}$$
   $$\text{heat}_{\text{vizinho}} \leftarrow \text{heat}_{\text{vizinho}} + Q_{\text{transf}}$$
3. **Média e Métricas da Rede**:
   - $\text{meanTemp} = \frac{\sum T_{\text{condutores}}}{N}$
   - $\text{eficiência} = \frac{Q_{\text{transf}}}{Q_{\text{transf}} + Q_{\text{perda}}} \times 100\%$

### 1.6 Inventário, GUI e Telas
- Sem GUI própria. Informações exibidas no chat via clique com o Configurador (exibe temperatura média, perda e eficiência).

### 1.7 Receitas (JSON Datagen)
- **Basic**: `pattern: ["S#S"]`, `S: c:ingots/steel`, `#: c:ingots/copper` $\rightarrow$ 8 condutores básicos.
- **Advanced**: `pattern: ["PPP", "PAP", "PPP"]`, `P: basic_thermodynamic_conductor`, `A: mekanism:alloys/infused` $\rightarrow$ 8 condutores avançados.
- **Elite**: `pattern: ["PPP", "PAP", "PPP"]`, `P: advanced_thermodynamic_conductor`, `A: mekanism:alloys/reinforced` $\rightarrow$ 8 condutores elite.
- **Ultimate**: `pattern: ["PPP", "PAP", "PPP"]`, `P: elite_thermodynamic_conductor`, `A: mekanism:alloys/atomic` $\rightarrow$ 8 condutores supremos.

### 1.8 Redstone, Sons e Interações
- **Configurador**: clique na conexão cicla conexão (aberto / fechado / bomba).
- **Sneak-click com Configurador**: alterna sensibilidade a redstone. Se energizado por redstone, desconecta o fluxo.
- **Upgrade in-world**: clicar com uma liga (Infused, Reinforced, Atomic) em um condutor no mundo promove o condutor e os adjacentes para o próximo tier sem quebrar o bloco.

### 1.9 Requisitos do Horizon
- **FORK-4**: meio "calor" de primeira classe no motor do Horizon (`Medium::Heat`), suporte a condutores térmicos e trocas com ar e blocos vizinhos.

---

## 2. Resistive Heater (Aquecedor Resistivo)

### 2.1 Identificação e Registros
- **Nome de Registro**: `mekanism:resistive_heater`
- **Classes Java**:
  - `mekanism.common.tile.machine.TileEntityResistiveHeater` (`TileEntityResistiveHeater.java`)
  - `mekanism.client.gui.machine.GuiResistiveHeater` (`GuiResistiveHeater.java`)
  - `mekanism.common.capabilities.energy.ResistiveHeaterEnergyContainer` (`ResistiveHeaterEnergyContainer.java`)
  - `mekanism.common.network.to_server.PacketGuiSetEnergy` (`PacketGuiSetEnergy.java`)

### 2.2 Estados de Bloco e Blockstates
- **Propriedades**:
  - `facing` (horizontal: `north`, `south`, `east`, `west`).
  - `active` (boolean: true quando está consumindo energia e aquecendo).
- **Efeitos no Mundo**:
  - Nível de luz 8 quando ativo.
  - Derrete neve e gelo em contato direto.

### 2.3 Campos do Tile Entity e Persistência
- **Campos**:
  - `energyUsage`: taxa de consumo configurada pelo jogador em J/t (padrão 100 J/t = 40 FE/t).
  - `energy`: energia atualmente armazenada no buffer interno.
  - `maxEnergy`: capacidade do buffer elétrico, dinamicamente ajustada para `energyUsage * 400` J.
  - `heatCapacitor`: capacitor térmico (`heatCapacity = 100.0`, `invConduction = 5.0`, `invInsulation = 10.0`).
  - `clientEnergyUsed`: energia consumida no último tick.
  - `soundScale`: intensidade acústica baseada no consumo (`toUse / 100_000.0`).
- **NBT e Data Components**:
  - Componente `MekanismDataComponents.ENERGY_USAGE` (salvo no ItemStack ao minerar).
  - NBT: `"energyUsage"` (long), `"energyStored"` (long), `"heat"` (double), `"Items"` (1 slot).

### 2.4 Todos os Números e Configurações
- `BASE_USAGE`: 100 Joules/tick = 40 FE/tick.
- `USAGE_MULTIPLIER`: 400 ticks (20 segundos de buffer).
- `resistiveHeaterEfficiency`: 0,6 (60% de eficiência na conversão de energia elétrica em calor).
- Conversão para FE: $1\text{ FE} = 2,5\text{ J}$ (fator `fe() = J * 2 / 5`).
- Produção de calor por tick:
  $$Q = \text{energiaConsumida (J)} \times 0,6$$
  Exemplo: consumo padrão de 100 J/t (40 FE/t) gera 60 Joules de calor por tick. Consumo de 10.000 J/t (4.000 FE/t) gera 6.000 Joules de calor por tick.

### 2.5 Inventário e GUI (`GuiResistiveHeater.java`)
- **Dimensões da Tela**: 176x166 pixels.
- **Elementos**:
  - Inner Screen em `(48, 23, 80, 42)`: exibe temperatura atual em Kelvin e consumo configurado.
  - Barra de Energia Vertical em `(164, 15)`.
  - Campo de Texto Digital (`GuiTextField`) em `(50, 51, 76, 12)`: aceita até 7 dígitos para entrada numérica do consumo desejado.
  - Slot de Energia em `(15, 35)`: aceita itens de carga (Energy Tablet, baterias) para transferir energia para o buffer.
  - Abas laterais: Aba de Energia (esquerda) e Aba de Calor (exibe temperatura, taxa transferida aos vizinhos e perda por convecção no ar).

### 2.6 Receita (JSON Datagen)
- Arquivo: `data/mekanism/recipe/resistive_heater.json`
- Tipo: `mekanism:mek_data` (preserva energia e configurações ao craftar).
- Padrão:
  ```json
  {
    "type": "mekanism:mek_data",
    "pattern": [
      "IRI",
      "RXR",
      "IEI"
    ],
    "key": {
      "I": { "tag": "c:ingots/tin" },
      "R": { "tag": "c:dusts/redstone" },
      "X": { "item": "mekanism:steel_casing" },
      "E": { "item": "mekanism:energy_tablet" }
    },
    "result": { "id": "mekanism:resistive_heater" }
  }
  ```

### 2.7 Redstone, Sons e Interações
- **Redstone**: modos IGNORED (sempre ativo se houver energia), NORMAL (opera apenas com sinal de redstone), INVERTED (opera apenas sem redstone).
- **Som**: `mekanism:tile.machine.resistive_heater`, loop contínuo enquanto ativo, volume modulado por `sqrt(soundScale)`.
- **Card de Configuração**: copia e cola a taxa de consumo entre aquecedores.

### 2.8 Requisitos do Horizon
- **FORK-4**: emissão de calor nos condutores e válvulas vizinhas.
- **FORK-20**: suporte a caixa de texto digital interativa (`GuiTextField`) e evento de envio de valor numérico ao servidor.

---

## 3. Fuelwood Heater (Aquecedor a Lenha)

### 3.1 Identificação e Registros
- **Nome de Registro**: `mekanism:fuelwood_heater`
- **Classes Java**:
  - `mekanism.common.tile.machine.TileEntityFuelwoodHeater` (`TileEntityFuelwoodHeater.java`)
  - `mekanism.client.gui.machine.GuiFuelwoodHeater` (`GuiFuelwoodHeater.java`)
  - `mekanism.common.inventory.slot.FuelInventorySlot` (`FuelInventorySlot.java`)

### 3.2 Estados de Bloco e Blockstates
- **Propriedades**:
  - `facing` (horizontal: `north`, `south`, `east`, `west`).
  - `active` (boolean: true enquanto houver combustível queimando).
- **Efeitos no Mundo**:
  - Emite luz nível 8 quando ativo.
  - Emite partículas de fumaça (`smoke`) e faíscas vermelhas (`dust`) na frente quando em combustão.

### 3.3 Campos do Tile Entity e Persistência
- **Campos**:
  - `burnTime` (int): ticks restantes de queima do combustível atual.
  - `maxBurnTime` (int): tempo de queima total do item consumido.
  - `heatCapacitor`: capacitor térmico (`heatCapacity = 100.0`, `invConduction = 5.0`, `invInsulation = 10.0`).
  - `lastTransferLoss` (double): calor transferido a vizinhos no tick anterior.
  - `lastEnvironmentLoss` (double): calor perdido para o ar ambiente no tick anterior.
- **NBT**:
  - `"burnTime"` (int).
  - `"maxBurnTime"` (int).
  - `"Items"` (1 slot).
  - `"heat"` sob o container de calor.

### 3.4 Todos os Números e Configurações
- `heatPerFuelTick`: 400,0 Joules de calor por tick (`GeneralConfig.java:145`).
- `fuelwoodTickMultiplier`: 1 tick de combustível consumido por tick de jogo (`GeneralConfig.java:147`).
- Rendimento dos Combustíveis:
  - Carvão vegetal/mineral (1.600 ticks) $\rightarrow$ produz $1.600 \times 400 = 640.000$ J de calor.
  - Madeira / Tábuas (300 ticks) $\rightarrow$ produz $300 \times 400 = 120.000$ J de calor.
  - Varas de Blaze (2.400 ticks) $\rightarrow$ produz $2.400 \times 400 = 960.000$ J de calor.

### 3.5 Inventário e GUI (`GuiFuelwoodHeater.java`)
- **Elementos**:
  - Inner Screen em `(48, 23, 80, 28)`: mostra temperatura em Kelvin e ticks de combustível restantes (`burnTime`).
  - Ícone de Chama Animada (`GuiFlame`) em `(144, 31)`: progresso proporcional a `burnTime / maxBurnTime`.
  - Slot de Combustível em `(15, 29)`: aceita qualquer item com tag de combustível de fornalha.
  - Aba de Calor na lateral.

### 3.6 Receita (JSON Datagen)
- Arquivo: `data/mekanism/recipe/fuelwood_heater.json`
- Padrão:
  ```json
  {
    "type": "minecraft:crafting_shaped",
    "pattern": [
      "SCS",
      "#X#",
      "SSS"
    ],
    "key": {
      "S": { "tag": "c:ingots/steel" },
      "C": { "tag": "c:circuits/basic" },
      "#": { "item": "minecraft:furnace" },
      "X": { "item": "mekanism:steel_casing" }
    },
    "result": { "id": "mekanism:fuelwood_heater" }
  }
  ```

### 3.7 Redstone, Sons e Interações
- Não requer energia elétrica nem configurações de redstone: queima automaticamente ao receber combustível no slot e atinge equilíbrio térmico com a rede ou ambiente.
- Som: crepitação suave de fornalha vanilla.

### 3.8 Requisitos do Horizon
- **FORK-4**: capacitor térmico e transferência de calor.

---

## 4. Laser Tractor Beam (Feixe Trator a Laser)

### 4.1 Identificação e Registros
- **Nome de Registro**: `mekanism:laser_tractor_beam`
- **Classes Java**:
  - `mekanism.common.tile.laser.TileEntityLaserTractorBeam` (`TileEntityLaserTractorBeam.java`)
  - `mekanism.common.tile.laser.TileEntityLaserReceptor` (`TileEntityLaserReceptor.java`)
  - `mekanism.client.gui.GuiLaserTractorBeam` (`GuiLaserTractorBeam.java`)
  - `mekanism.common.capabilities.energy.LaserEnergyContainer` (`LaserEnergyContainer.java`)

### 4.2 Estados de Bloco e Blockstates
- **Propriedades**:
  - `facing` (6 direções: `down`, `up`, `north`, `south`, `west`, `east`).
  - Baseado na orientação selecionada pelo jogador ao posicionar (`SELECTED_FACE`).

### 4.3 Campos do Tile Entity e Persistência
- **Campos**:
  - `energyContainer`: buffer de energia com capacidade de **5.000.000.000 J** (2.000.000.000 FE).
  - Inventário interno: 27 slots de saída (`OutputInventorySlot`) organizados em matriz 9x3.
- **NBT**:
  - `"energyStored"` (long).
  - `"Items"` (lista com até 27 itens serializados).

### 4.4 Todos os Números e Configurações
- Capacidade do Buffer: 5.000.000.000 J = 2.000.000.000 FE (`StorageConfig.laserTractorBeam`).
- Custo de Mineração a Laser: `laserEnergyPerHardness = 100_000` J = 40.000 FE por 1 ponto de dureza do bloco minerado (`GeneralConfig.laserEnergyPerHardness`).
  - Pedra vanilla (dureza 1,5): $1,5 \times 100.000 = 150.000$ J (60.000 FE).
  - Obsidiana (dureza 50,0): $50 \times 100.000 = 5.000.000$ J (2.000.000 FE).

### 4.5 Funcionamento e Mineração Automática
1. **Quebra de Bloco**: Quando o feixe disparado pelo Laser atinge um bloco sólido à frente, o Laser Receptor/Tractor consome a energia equivalente à dureza.
2. **Coleta de Drops**: Em vez de soltar o item no mundo como o amplificador normal, o Tractor Beam intercepta a rotina de drops (`WorldUtils.getDrops`), insere os itens diretamente nos seus 27 slots internos.
3. **Coleta de Entidades Flutuantes**: Se uma entidade de item (`ItemEntity`) cruzar a trajetória do laser, ela é absorvida para os slots internos (`handleHitItem`).
4. **Transbordamento**: Caso os 27 slots encham, os itens excedentes são expelidos pelo centro frontal do bloco com vetor de velocidade direcionado para trás (`Block.popResourceFromFace`).

### 4.6 Inventário e GUI (`GuiLaserTractorBeam.java`)
- Tela de contêiner de 27 slots dispostos exatamente como um baú convencional:
  - 3 linhas de 9 slots em `(8 + x * 18, 16 + y * 18)`.
  - Inventário do jogador nos 36 slots inferiores.

### 4.7 Receita (JSON Datagen)
- Arquivo: `data/mekanism/recipe/laser_tractor_beam.json`
- Padrão:
  ```json
  {
    "type": "mekanism:mek_data",
    "pattern": [
      "P",
      "#"
    ],
    "key": {
      "P": { "tag": "mekanism:personal_storage" },
      "#": { "item": "mekanism:laser_amplifier" }
    },
    "result": { "id": "mekanism:laser_tractor_beam" }
  }
  ```

### 4.8 Redstone e Interações
- Emite sinal analógico de comparador proporcional ao nível de ocupação dos 27 slots de inventário (de 0 a 15).
- Segurança de acesso (Lock/Owner) suportada.

### 4.9 Requisitos do Horizon
- **FORK-7 / FORK-8**: API de física do raio laser, interceptação de quebra de bloco por dureza e captura de `ItemEntity` em trânsito.

---

## 5. Restrictive Transporter (Transportador Restritivo)

### 5.1 Identificação e Registros
- **Nome de Registro**: `mekanism:restrictive_transporter`
- **Classes Java**:
  - `mekanism.common.content.network.transmitter.RestrictiveTransporter` (`RestrictiveTransporter.java`)
  - `mekanism.common.tile.transmitter.TileEntityRestrictiveTransporter` (`TileEntityRestrictiveTransporter.java`)

### 5.2 Estados de Bloco e Blockstates
- 6 conexões espaciais independentes com estados: `NONE`, `NORMAL`, `PUSH`, `PULL`.

### 5.3 Números e Algoritmo de Roteamento
- **Custo no Caminho (A\*)**: **1.000** por bloco de duto (em comparação com o custo **1** do transportador logístico padrão).
- **Velocidade de Transporte**: baseada no tier Basic = **50 ticks** por bloco.
- **Finalidade**: atua como duto de prioridade mínima / transbordamento (overflow). Itens só passam por ele se todos os destinos em rotas normais estiverem completamente cheios ou inacessíveis.

### 5.4 Receita (JSON Datagen)
- Arquivo: `data/mekanism/recipe/transmitter/restrictive_transporter.json`
- Padrão:
  ```json
  {
    "type": "minecraft:crafting_shaped",
    "pattern": [
      "S#S"
    ],
    "key": {
      "S": { "tag": "c:ingots/steel" },
      "#": { "item": "minecraft:iron_bars" }
    },
    "result": { "count": 2, "id": "mekanism:restrictive_transporter" }
  }
  ```

### 5.5 Requisitos do Horizon
- **FORK-3**: suporte a peso de aresta/custo customizado (1.000) no algoritmo de busca de caminho logístico (`TransporterPathfinder`).

---

## 6. Diversion Transporter (Transportador de Desvio)

### 6.1 Identificação e Registros
- **Nome de Registro**: `mekanism:diversion_transporter`
- **Classes Java**:
  - `mekanism.common.content.network.transmitter.DiversionTransporter` (`DiversionTransporter.java`)
  - `mekanism.common.tile.transmitter.TileEntityDiversionTransporter` (`TileEntityDiversionTransporter.java`)

### 6.2 Estados de Bloco e Blockstates
- 6 conexões espaciais independentes.
- Cada uma das 6 faces possui um modo `DiversionControl`:
  - `DISABLED` (Desativado / Sempre Conectado)
  - `HIGH` (Ativo apenas com sinal alto de redstone)
  - `LOW` (Ativo apenas com sinal baixo / sem redstone)

### 6.3 Campos do Tile Entity e Persistência
- Array de 6 modos: `modes: [DiversionControl; 6]`.
- NBT: chave `"mode"` gravada como array de 6 inteiros (`int[]`).
- Sincronização: pacote de dados atualiza o modelo e conexões no cliente ao alternar.

### 6.4 Interação e Operação
- **Clique com a mão direita na face**: cicla o modo da face clicada:
  $$\text{DISABLED} \rightarrow \text{HIGH} \rightarrow \text{LOW} \rightarrow \text{DISABLED}$$
- Envia mensagem no chat da barra de ação (`displayClientMessage`):
  `"Diverter mode: Disabled / High / Low"` com texto colorido.
- Quando a condição de redstone da face não é satisfeita, a face é tratada como desconectada e a capacidade de inserção (`exposesInsertCap`) é desabilitada instantaneamente, recalculando as rotas da rede logística.

### 6.5 Receita (JSON Datagen)
- Arquivo: `data/mekanism/recipe/transmitter/diversion_transporter.json`
- Padrão:
  ```json
  {
    "type": "minecraft:crafting_shaped",
    "pattern": [
      "RRR",
      "S#S",
      "RRR"
    ],
    "key": {
      "R": { "tag": "c:dusts/redstone" },
      "S": { "tag": "c:ingots/steel" },
      "#": { "item": "minecraft:iron_bars" }
    },
    "result": { "count": 2, "id": "mekanism:diversion_transporter" }
  }
  ```

### 6.6 Requisitos do Horizon
- **FORK-3**: callbacks de roteamento dinâmico dependentes de sinal de redstone por face.
