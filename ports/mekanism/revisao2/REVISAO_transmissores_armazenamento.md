# Relatório de Revisão Cruzada: Transmissores e Armazenamento (Mekanism 1.21.1-10.7.19.85 vs Port Horizon)

**Tarefa**: `T20261109-195239-ljc`  
**Escopo**: `port/mod/src/transmitter.rs`, `port/mod/src/induction.rs`, `port/mod/src/bin.rs`, `port/mod/src/cube.rs`, `port/mod/src/continuous.rs`, `port/mod/src/sides.rs`, `port/mod/src/bounding.rs`  
**Referência oficial**: Mekanism 1.21.1-10.7.19.85 (`java/`)  
**Conversão energética**: $1\text{ J} = 0{,}4\text{ FE}$ (`FE = J * 2 / 5` via `fe()`, `feConversionRate = 2.5`)

---

## 1. Avaliação das Correções da Rodada 1

| Correção da Rodada 1 | Avaliação na Rodada 2 | Detalhes no Java / Código | Estado |
|---|---|---|---|
| **Conversão de Redstone no Energy Cube (`cube.rs`)**: 10.000 J → 4.000 FE; 90.000 J → 36.000 FE | **CONFIRMADA** | No Java (`redstone.json:7` e `redstone_block.json:7`), as receitas de `energy_conversion` concedem 10.000 J e 90.000 J. Como o port opera em FE, 10.000 * 2 / 5 = 4.000 FE e 90.000 * 2 / 5 = 36.000 FE. Teste `redstone_gives_the_energy_the_recipes_say` em `cube.rs:123` confere. | CONFIRMADA |
| **Paleta de 18 Cores do Logistical Transporter (`transmitter.rs`)**: 19 estados (None + 18 cores de EnumColor) | **ERRADA-CORRIGIDA** | A rodada 1 expandiu a lista de 17 para 19 entradas, mas manteve nomes errados nos índices 2, 3 e 4: colocou `"Dark Blue"`, `"Dark Green"`, `"Dark Aqua"` quando no Java (`EnumColor.java:28-46` e `en_us.json:666-683`) os ordinais 1, 2 e 3 possuem nome oficial/tradução `"Blue"`, `"Green"` e `"Cyan"`. Corrigido em `transmitter.rs:80-83` e teste `the_connections_cycle_and_the_state_round_trips` em `transmitter.rs:178`. | ERRADA-CORRIGIDA |

---

## 2. Tabela Completa de Revisão Cruzada

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| **Cabos Universais (Universal Cable)** | | | |
| Basic Universal Cable (capacidade) | 8.000 J (`CableTier.java:12`) | `fe(8_000)` = 3.200 FE (`lib.rs:332`) | OK |
| Advanced Universal Cable (capacidade) | 128.000 J (`CableTier.java:13`) | `fe(128_000)` = 51.200 FE (`lib.rs:332`) | OK |
| Elite Universal Cable (capacidade) | 1.024.000 J (`CableTier.java:14`) | `fe(1_024_000)` = 409.600 FE (`lib.rs:332`) | OK |
| Ultimate Universal Cable (capacidade) | 8.192.000 J (`CableTier.java:15`) | `fe(8_192_000)` = 3.276.800 FE (`lib.rs:332`) | OK |
| Cable taxa de bombeamento / extração | Não possui (rede contínua) (`CableTier.java`) | Não aplicável (`lib.rs:1964`) | OK |
| Cable sensibilidade a redstone | Desconecta conexão com sinal alto (`Transmitter.java:421`) | Mantido em byte de estado, sem efeito na API do jogo (`transmitter.rs:90, 148`) | FORK |
| **Tubos Pressurizados (Pressurized Tube)** | | | |
| Basic Pressurized Tube (capacidade) | 4.000 mB (`TubeTier.java:10`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Basic Pressurized Tube (taxa extração/pull) | 750 mB/t (`TubeTier.java:10`) | 750 mB/t (`lib.rs:333, TUBE_RATES[0]`) | OK |
| Advanced Pressurized Tube (capacidade) | 16.000 mB (`TubeTier.java:11`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Advanced Pressurized Tube (taxa extração/pull) | 2.000 mB/t (`TubeTier.java:11`) | 2.000 mB/t (`lib.rs:333, TUBE_RATES[1]`) | OK |
| Elite Pressurized Tube (capacidade) | 256.000 mB (`TubeTier.java:12`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Elite Pressurized Tube (taxa extração/pull) | 64.000 mB/t (`TubeTier.java:12`) | 64.000 mB/t (`lib.rs:333, TUBE_RATES[2]`) | OK |
| Ultimate Pressurized Tube (capacidade) | 1.024.000 mB (`TubeTier.java:13`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Ultimate Pressurized Tube (taxa extração/pull) | 256.000 mB/t (`TubeTier.java:13`) | 256.000 mB/t (`lib.rs:333, TUBE_RATES[3]`) | OK |
| Tube sensibilidade a redstone | Desconecta com sinal alto (`Transmitter.java:421`) | Mantido em byte de estado, sem efeito na rede (`transmitter.rs:90, 148`) | FORK |
| **Canos Mecânicos (Mechanical Pipe)** | | | |
| Basic Mechanical Pipe (capacidade) | 2.000 mB (`PipeTier.java:10`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Basic Mechanical Pipe (taxa extração/pull) | 250 mB/t (`PipeTier.java:10`) | 250 mB/t (`lib.rs:334, PIPE_RATES[0]`) | OK |
| Advanced Mechanical Pipe (capacidade) | 8.000 mB (`PipeTier.java:11`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Advanced Mechanical Pipe (taxa extração/pull) | 1.000 mB/t (`PipeTier.java:11`) | 1.000 mB/t (`lib.rs:334, PIPE_RATES[1]`) | OK |
| Elite Mechanical Pipe (capacidade) | 32.000 mB (`PipeTier.java:12`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Elite Mechanical Pipe (taxa extração/pull) | 8.000 mB/t (`PipeTier.java:12`) | 8.000 mB/t (`lib.rs:334, PIPE_RATES[2]`) | OK |
| Ultimate Mechanical Pipe (capacidade) | 128.000 mB (`PipeTier.java:13`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | FORK |
| Ultimate Mechanical Pipe (taxa extração/pull) | 32.000 mB/t (`PipeTier.java:13`) | 32.000 mB/t (`lib.rs:334, PIPE_RATES[3]`) | OK |
| Pipe sensibilidade a redstone | Desconecta com sinal alto (`Transmitter.java:421`) | Mantido em byte de estado, sem efeito na rede (`transmitter.rs:90, 148`) | FORK |
| **Transportadores Logísticos (Logistical Transporter)** | | | |
| Basic Logistical Transporter (throughput/pull) | 1 item/pull (`TransporterTier.java:9`) | 1 item/pull (`lib.rs:336, TRANSPORTER_RATES[0]`) | OK |
| Basic Logistical Transporter (speed/delay) | 5 ticks (`TransporterTier.java:9`) | Não modelado na API do Horizon | FORK |
| Advanced Logistical Transporter (throughput/pull) | 16 itens/pull (`TransporterTier.java:10`) | 16 itens/pull (`lib.rs:336, TRANSPORTER_RATES[1]`) | OK |
| Advanced Logistical Transporter (speed/delay) | 10 ticks (`TransporterTier.java:10`) | Não modelado na API do Horizon | FORK |
| Elite Logistical Transporter (throughput/pull) | 32 itens/pull (`TransporterTier.java:11`) | 32 itens/pull (`lib.rs:336, TRANSPORTER_RATES[2]`) | OK |
| Elite Logistical Transporter (speed/delay) | 20 ticks (`TransporterTier.java:11`) | Não modelado na API do Horizon | FORK |
| Ultimate Logistical Transporter (throughput/pull) | 64 itens/pull (`TransporterTier.java:12`) | 64 itens/pull (`lib.rs:336, TRANSPORTER_RATES[3]`) | OK |
| Ultimate Logistical Transporter (speed/delay) | 50 ticks (`TransporterTier.java:12`) | Não modelado na API do Horizon | FORK |
| Restrictive Transporter | Custo 1.000, Tier Basic (`RestrictiveTransporter.java:14`) | Não registrado (`transmitter.rs:15-16`) | FORK |
| Diversion Transporter | Modos redstone por face (`DiversionTransporter.java:38`) | Não registrado (`transmitter.rs:15-16`) | FORK |
| Nomes das 18 cores de EnumColor | Black, Blue, Green, Cyan, Dark Red... (`EnumColor.java:28-46`, `en_us.json:666-683`) | Corrigido para `Blue`, `Green`, `Cyan` (`transmitter.rs:80-83`) | ERRADA-CORRIGIDA |
| Alternância de cores e conexões | Incremento cíclico via configurator (`LogisticalTransporter.java:50`) | Sneak-click em face desconectada avança cor (`transmitter.rs:136`) | OK |
| Strings e mensagens de chat | "Color bumped to: %1$s", "Connection type changed to: %1$s" (`MekanismLangProvider.java:1596, 1601`) | Idênticas (`transmitter.rs:137, 142`) | OK |
| **Condutores Termodinâmicos (Thermodynamic Conductor)** | | | |
| Inverse Conduction (todos os tiers) | 5,0 (`ConductorTier.java:11-14`) | Não implementado (sem sistema de calor) | FORK |
| Heat Capacity (todos os tiers) | 1,0 (`HeatAPI.java:28`, `ConductorTier.java:11-14`) | Não implementado (sem sistema de calor) | FORK |
| Basic Conduction Insulation | 10,0 (`ConductorTier.java:11`) | Não implementado (sem sistema de calor) | FORK |
| Advanced Conduction Insulation | 400,0 (`ConductorTier.java:12`) | Não implementado (sem sistema de calor) | FORK |
| Elite Conduction Insulation | 8.000,0 (`ConductorTier.java:13`) | Não implementado (sem sistema de calor) | FORK |
| Ultimate Conduction Insulation | 100.000,0 (`ConductorTier.java:14`) | Não implementado (sem sistema de calor) | FORK |
| **Bins (`bin.rs`)** | | | |
| Basic Bin (capacidade) | 4.096 itens (`BinTier.java:8`) | 4.096 itens (`bin.rs:12`) | OK |
| Advanced Bin (capacidade) | 8.192 itens (`BinTier.java:9`) | 8.192 itens (`bin.rs:13`) | OK |
| Elite Bin (capacidade) | 32.768 itens (`BinTier.java:10`) | 32.768 itens (`bin.rs:14`) | OK |
| Ultimate Bin (capacidade) | 262.144 itens (`BinTier.java:11`) | 262.144 itens (`bin.rs:15`) | OK |
| Creative Bin (capacidade) | `Integer.MAX_VALUE` = 2.147.483.647 (`BinTier.java:12`) | `u64::MAX / 2` (`bin.rs:16`) | FORK |
| Bin Auto-eject cadência | 10 ticks = 0,5s (`TileEntityBin.java:110`) | 10 ticks (`bin.rs:22, EJECT_EVERY`) | OK |
| Bin Auto-eject direção | Face inferior (DOWN) (`TileEntityBin.java:103`) | Face inferior (`bin.rs:178`) | OK |
| Bin Configuração de slots | Slot 0 saída (bottom stack), Slot 1 entrada (`TileEntityBin.java`, `BinInventorySlot.java`) | `OUTPUT: 0`, `INPUT: 1` (`bin.rs:18-19`) | OK |
| Bin Extração por ataque/soco | Soco = 1 item; Shift-soco = 1 stack (`BlockBin.java:38-71`) | Não implementado (sem evento de ataque na API) | FORK |
| Bin Bloqueio (Lock) | Shift-clique mão vazia na frente (`BlockBin.java:88`) | Alternado por botão na interface (`bin.rs:21, 129`) | OK |
| Bin Alternância de auto-eject | Sneak clique com configurator (`TileEntityBin.java:123`) | Sneak clique com configurator (`bin.rs:136-139`) | OK |
| **Cubos de Energia (Energy Cubes - `cube.rs`, `lib.rs`, `sides.rs`)** | | | |
| Basic Energy Cube (capacidade) | 4.000.000 J (`EnergyCubeTier.java:13`) | `fe(4_000_000)` = 1.600.000 FE (`lib.rs:312`) | OK |
| Basic Energy Cube (taxa de saída) | 4.000 J/t (`EnergyCubeTier.java:13`) | `fe(4_000)` = 1.600 FE/t (`lib.rs:312`) | OK |
| Advanced Energy Cube (capacidade) | 16.000.000 J (`EnergyCubeTier.java:14`) | `fe(16_000_000)` = 6.400.000 FE (`lib.rs:313`) | OK |
| Advanced Energy Cube (taxa de saída) | 16.000 J/t (`EnergyCubeTier.java:14`) | `fe(16_000)` = 6.400 FE/t (`lib.rs:313`) | OK |
| Elite Energy Cube (capacidade) | 64.000.000 J (`EnergyCubeTier.java:15`) | `fe(64_000_000)` = 25.600.000 FE (`lib.rs:314`) | OK |
| Elite Energy Cube (taxa de saída) | 64.000 J/t (`EnergyCubeTier.java:15`) | `fe(64_000)` = 25.600 FE/t (`lib.rs:314`) | OK |
| Ultimate Energy Cube (capacidade) | 256.000.000 J (`EnergyCubeTier.java:16`) | `fe(256_000_000)` = 102.400.000 FE (`lib.rs:315`) | OK |
| Ultimate Energy Cube (taxa de saída) | 256.000 J/t (`EnergyCubeTier.java:16`) | `fe(256_000)` = 102.400 FE/t (`lib.rs:315`) | OK |
| Creative Energy Cube (capacidade) | `Long.MAX_VALUE` J (`EnergyCubeTier.java:17`) | 1.000.000.000.000 FE (`lib.rs:309`) | FORK |
| Creative Energy Cube (taxa de saída) | `Long.MAX_VALUE` J/t (`EnergyCubeTier.java:17`) | 1.000.000.000 FE/t (`lib.rs:1958`) | FORK |
| Descarga de Redstone (slot discharge) | 10.000 J = 4.000 FE (`redstone.json:7`) | 4.000 FE (`cube.rs:15`) | CONFIRMADA |
| Descarga de Bloco de Redstone (slot discharge) | 90.000 J = 36.000 FE (`redstone_block.json:7`) | 36.000 FE (`cube.rs:16`) | CONFIRMADA |
| Slots de inventário | 0=discharge (convert), 1=charge (`TileEntityEnergyCube.java:85-86`) | 0=DISCHARGE, 1=CHARGE (`cube.rs:9-10`) | OK |
| Configuração inicial de lados (FRONT_OUT_EJECT) | Frente = Output, outras 5 = Input (`TileEntityEnergyCube.java:61-62`) | Frente = Output, outras 5 = Input (`sides.rs:63-67`) | OK |
| Orientação de colocação no mundo | Olha para o jogador (`TileEntityEnergyCube.java:60`) | Olha para o jogador (`sides.rs:87-97, lib.rs:1282`) | OK |
| **Matriz de Indução (Induction Matrix - `induction.rs`)** | | | |
| Basic Induction Cell (capacidade) | 8.000.000.000 J (`InductionCellTier.java:11`) | `fe(8_000_000_000)` = 3.200.000.000 FE (`induction.rs:16`) | OK |
| Advanced Induction Cell (capacidade) | 64.000.000.000 J (`InductionCellTier.java:12`) | `fe(64_000_000_000)` = 25.600.000.000 FE (`induction.rs:17`) | OK |
| Elite Induction Cell (capacidade) | 512.000.000.000 J (`InductionCellTier.java:13`) | `fe(512_000_000_000)` = 204.800.000.000 FE (`induction.rs:18`) | OK |
| Ultimate Induction Cell (capacidade) | 4.000.000.000.000 J (`InductionCellTier.java:14`) | `fe(4_000_000_000_000)` = 1.600.000.000.000 FE (`induction.rs:19`) | OK |
| Basic Induction Provider (taxa transferência) | 256.000 J/t (`InductionProviderTier.java:11`) | `fe(256_000)` = 102.400 FE/t (`induction.rs:22`) | OK |
| Advanced Induction Provider (taxa transferência) | 2.048.000 J/t (`InductionProviderTier.java:12`) | `fe(2_048_000)` = 819.200 FE/t (`induction.rs:23`) | OK |
| Elite Induction Provider (taxa transferência) | 16.384.000 J/t (`InductionProviderTier.java:13`) | `fe(16_384_000)` = 6.553.600 FE/t (`induction.rs:24`) | OK |
| Ultimate Induction Provider (taxa transferência) | 131.072.000 J/t (`InductionProviderTier.java:14`) | `fe(131_072_000)` = 52.428.800 FE/t (`induction.rs:25`) | OK |
| Dimensões mínimas e máximas | 3 a 18 blocos por lado (`CuboidStructureValidator.java:34`) | 3 a 18 blocos por lado (`induction.rs:66`) | OK |
| Casca válida e restrição de arestas | Arestas exigem Casing; Glass só nas paredes (`CuboidStructureValidator.java:115`) | Glass aceito em qualquer parte da casca (`induction.rs:52, 88`) | FORK |
| Posicionamento de portas | Apenas nas faces (`edges == 1`) (`CuboidStructureValidator.java:135`) | Apenas nas faces (`edges == 1`) (`induction.rs:89`) | OK |
| Alternância de modo da porta | Sneak right-click: Input / Output (`TileEntityInductionPort.java:60-65`) | Sneak right-click: Input / Output (`induction.rs:231-237`) | OK |
| Mensagem de alternância da porta | "Toggled Induction Port transfer mode to %1$s." (`MekanismLangProvider.java:1485`) | Idêntica em inglês (`induction.rs:236`) | OK |
| **Máquinas Contínuas (`continuous.rs`, `lib.rs`)** | | | |
| Electrolytic Separator tanques | Água 24.000 mB, Gases 2.400 mB (`TileEntityElectrolyticSeparator.java:94-95`) | `[24_000, 2_400, 2_400]` (`lib.rs:202`) | OK |
| Chemical Washer tanques | Água 10.000 mB, Lodo 10.000 mB (`TileEntityChemicalWasher.java:74-75`) | `[10_000, 10_000, 10_000]` (`lib.rs:199`) | OK |
| Chemical Infuser tanques | Gases 10.000 mB cada (`TileEntityChemicalInfuser.java:67`) | `[10_000, 10_000, 10_000]` (`lib.rs:197`) | OK |
| Rotary Condensentrator tanques | Fluido 10.000 mB, Gás 10.000 mB (`TileEntityRotaryCondensentrator.java:87`) | `[10_000, 10_000, 0]` (`lib.rs:205`) | OK |
| Pigment Mixer tanques | Pigmentos 1.000 mB / Saída 2.000 mB (`TileEntityPigmentMixer.java:70-71`) | `[1_000, 1_000, 2_000]` (`lib.rs:214`) | OK |
| Isotopic Centrifuge tanques | Gás 10.000 mB entrada e saída (`TileEntityIsotopicCentrifuge.java:67`) | `[10_000, 10_000, 0]` (`lib.rs:218`) | OK |
| Solar Neutron Activator tanques | Gás 10.000 mB entrada e saída (`TileEntitySolarNeutronActivator.java:64`) | `[10_000, 10_000, 0]` (`lib.rs:220`) | OK |
| **Bounding Blocks (`bounding.rs`)** | | | |
| Advanced Solar Generator offsets | `[0, 1, 0]` + plano 3x3 em `y=2` (10 blocos) (`GeneratorsBlockTypes.java:142-154`) | 10 blocos subordinados (`bounding.rs:25-30`) | OK |
| Wind Generator offsets | 4 blocos acima (`y=1..=4`) (`GeneratorsBlockTypes.java:108-113`) | 4 blocos acima (`bounding.rs:49`) | OK |
| Pigment Mixer, Centrifuge, Activator | `[0, 1, 0]` (`MekanismBlockTypes.java:393, 562, 605`) | `[0, 1, 0]` (`bounding.rs:19-23`) | OK |
| Seismic Vibrator, Security Desk offsets | `[0, 1, 0]` (`MekanismBlockTypes.java:464, 527`) | `[0, 1, 0]` adicionados em `bounding.rs:22-23` | CORRIGIDO |
| Digital Miner offsets | Cubóide 3x3x2 excluindo origem (17 blocos) (`MekanismBlockTypes.java:338-348`) | Cubóide 3x3x2 excluindo origem (`bounding.rs:35-46`) | OK |
| Interação do bounding block | Encaminha clique e desmontagem para a máquina pai (`BlockBounding.java`) | Bounding block não responde a cliques nem abre GUI | FORK |
| **Configuração de Lados (`sides.rs`)** | | | |
| Modos de energia suportados | None (0), Input (1), Output (2), Both (3) (`DataType.java`) | None, Input, Output, Both (`sides.rs:7-12`) | OK |
| Ciclo de modos de lados | None → Input → Output → Both (`DataType.java`) | None → Input → Output → Both (`sides.rs:24-27`) | OK |
| Mapeamento de direções vanilla | Down (0), Up (1), North (2), South (3), West (4), East (5) | Down, Up, North, South, West, East (`sides.rs:53-59`) | OK |
