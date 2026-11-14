# Auditoria de Transmissores e Armazenamento (Mekanism 1.21.1-10.7.19.85 vs Port Horizon)

Escopo: `transmitter.rs`, `induction.rs`, `bin.rs`, `cube.rs`, `continuous.rs`, `sides.rs`, `bounding.rs`.

Regra de energia: A referência oficial é o release **Mekanism 1.21.1-10.7.19.85**. No Java, a energia interna é em Joules (J). O port no Horizon utiliza Forge Energy (FE), com taxa de conversão `feConversionRate = 2.5`, ou seja, `FE = J * 2 / 5` via `fe()`.

---

## 1. Tabela Geral de Auditoria

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| **Cabos Universais (Universal Cable)** | | | |
| Basic Universal Cable (capacidade) | 8.000 J (`CableTier.java:12`) | `fe(8_000)` = 3.200 FE (`lib.rs:332`) | OK |
| Advanced Universal Cable (capacidade) | 128.000 J (`CableTier.java:13`) | `fe(128_000)` = 51.200 FE (`lib.rs:332`) | OK |
| Elite Universal Cable (capacidade) | 1.024.000 J (`CableTier.java:14`) | `fe(1_024_000)` = 409.600 FE (`lib.rs:332`) | OK |
| Ultimate Universal Cable (capacidade) | 8.192.000 J (`CableTier.java:15`) | `fe(8_192_000)` = 3.276.800 FE (`lib.rs:332`) | OK |
| Cable taxa de bombeamento / extração | Não possui (rede contínua) (`CableTier.java`) | Não aplicável | OK |
| **Tubos Pressurizados (Pressurized Tube)** | | | |
| Basic Pressurized Tube (capacidade) | 4.000 mB (`TubeTier.java:10`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Basic Pressurized Tube (taxa extração/pull) | 750 mB/t (`TubeTier.java:10`) | 750 mB/t (`lib.rs:333`, `TUBE_RATES[0]`) | OK |
| Advanced Pressurized Tube (capacidade) | 16.000 mB (`TubeTier.java:11`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Advanced Pressurized Tube (taxa extração/pull) | 2.000 mB/t (`TubeTier.java:11`) | 2.000 mB/t (`lib.rs:333`, `TUBE_RATES[1]`) | OK |
| Elite Pressurized Tube (capacidade) | 256.000 mB (`TubeTier.java:12`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Elite Pressurized Tube (taxa extração/pull) | 64.000 mB/t (`TubeTier.java:12`) | 64.000 mB/t (`lib.rs:333`, `TUBE_RATES[2]`) | OK |
| Ultimate Pressurized Tube (capacidade) | 1.024.000 mB (`TubeTier.java:13`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Ultimate Pressurized Tube (taxa extração/pull) | 256.000 mB/t (`TubeTier.java:13`) | 256.000 mB/t (`lib.rs:333`, `TUBE_RATES[3]`) | OK |
| **Canos Mecânicos (Mechanical Pipe)** | | | |
| Basic Mechanical Pipe (capacidade) | 2.000 mB (`PipeTier.java:10`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Basic Mechanical Pipe (taxa extração/pull) | 250 mB/t (`PipeTier.java:10`) | 250 mB/t (`lib.rs:334`, `PIPE_RATES[0]`) | OK |
| Advanced Mechanical Pipe (capacidade) | 8.000 mB (`PipeTier.java:11`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Advanced Mechanical Pipe (taxa extração/pull) | 1.000 mB/t (`PipeTier.java:11`) | 1.000 mB/t (`lib.rs:334`, `PIPE_RATES[1]`) | OK |
| Elite Mechanical Pipe (capacidade) | 32.000 mB (`PipeTier.java:12`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Elite Mechanical Pipe (taxa extração/pull) | 8.000 mB/t (`PipeTier.java:12`) | 8.000 mB/t (`lib.rs:334`, `PIPE_RATES[2]`) | OK |
| Ultimate Mechanical Pipe (capacidade) | 128.000 mB (`PipeTier.java:13`) | Não modelada no `TransmitterSpec` (`lib.rs:1971`) | LIMITE |
| Ultimate Mechanical Pipe (taxa extração/pull) | 32.000 mB/t (`PipeTier.java:13`) | 32.000 mB/t (`lib.rs:334`, `PIPE_RATES[3]`) | OK |
| **Transportadores Logísticos (Logistical Transporter)** | | | |
| Basic Logistical Transporter (throughput/pull) | 1 item/pull (`TransporterTier.java:9`) | 1 item/pull (`lib.rs:336`, `TRANSPORTER_RATES[0]`) | OK |
| Basic Logistical Transporter (speed/delay) | 5 ticks (`TransporterTier.java:9`) | Não modelado na API do Horizon | LIMITE |
| Advanced Logistical Transporter (throughput/pull) | 16 itens/pull (`TransporterTier.java:10`) | 16 itens/pull (`lib.rs:336`, `TRANSPORTER_RATES[1]`) | OK |
| Advanced Logistical Transporter (speed/delay) | 10 ticks (`TransporterTier.java:10`) | Não modelado na API do Horizon | LIMITE |
| Elite Logistical Transporter (throughput/pull) | 32 itens/pull (`TransporterTier.java:11`) | 32 itens/pull (`lib.rs:336`, `TRANSPORTER_RATES[2]`) | OK |
| Elite Logistical Transporter (speed/delay) | 20 ticks (`TransporterTier.java:11`) | Não modelado na API do Horizon | LIMITE |
| Ultimate Logistical Transporter (throughput/pull) | 64 itens/pull (`TransporterTier.java:12`) | 64 itens/pull (`lib.rs:336`, `TRANSPORTER_RATES[3]`) | OK |
| Ultimate Logistical Transporter (speed/delay) | 50 ticks (`TransporterTier.java:12`) | Não modelado na API do Horizon | LIMITE |
| Restrictive Transporter | Custo 1.000, Tier Basic (`RestrictiveTransporter.java:14`) | Não registrado (`transmitter.rs:15-16`) | LIMITE |
| Diversion Transporter | Modos redstone por face (`DiversionTransporter.java:38`) | Não registrado (`transmitter.rs:15-16`) | LIMITE |
| Cores do transportador (EnumColor) | 18 cores (`EnumColor.java:28-46`) | 18 cores + None (19 estados) (`transmitter.rs:80-83`) | CORRIGIDO |
| **Condutores Termodinâmicos (Thermodynamic Conductor)** | | | |
| Inverse Conduction (todos os tiers) | 5,0 (`ConductorTier.java:11-14`) | Não implementado (sem sistema de calor) | LIMITE |
| Heat Capacity (todos os tiers) | 1,0 (`HeatAPI.java:28`, `ConductorTier.java:11-14`) | Não implementado (sem sistema de calor) | LIMITE |
| Basic Conduction Insulation | 10,0 (`ConductorTier.java:11`) | Não implementado (sem sistema de calor) | LIMITE |
| Advanced Conduction Insulation | 400,0 (`ConductorTier.java:12`) | Não implementado (sem sistema de calor) | LIMITE |
| Elite Conduction Insulation | 8.000,0 (`ConductorTier.java:13`) | Não implementado (sem sistema de calor) | LIMITE |
| Ultimate Conduction Insulation | 100.000,0 (`ConductorTier.java:14`) | Não implementado (sem sistema de calor) | LIMITE |
| **Bins (`bin.rs`)** | | | |
| Basic Bin (capacidade) | 4.096 itens (`BinTier.java:8`) | 4.096 itens (`bin.rs:12`) | OK |
| Advanced Bin (capacidade) | 8.192 itens (`BinTier.java:9`) | 8.192 itens (`bin.rs:13`) | OK |
| Elite Bin (capacidade) | 32.768 itens (`BinTier.java:10`) | 32.768 itens (`bin.rs:14`) | OK |
| Ultimate Bin (capacidade) | 262.144 itens (`BinTier.java:11`) | 262.144 itens (`bin.rs:15`) | OK |
| Creative Bin (capacidade) | `Integer.MAX_VALUE` = 2.147.483.647 (`BinTier.java:12`) | `u64::MAX / 2` (`bin.rs:16`) | LIMITE |
| Bin Auto-eject ritmo | 10 ticks = 0,5s (`TileEntityBin.java:110`) | 10 ticks (`bin.rs:22`, `EJECT_EVERY`) | OK |
| Bin Slots | 1 slot saída (bottom stack), 1 entrada (`BinInventorySlot.java:29`) | `OUTPUT: 0`, `INPUT: 1` (`bin.rs:18-19`) | OK |
| **Cubos de Energia (Energy Cubes - `cube.rs`, `lib.rs`)** | | | |
| Basic Energy Cube (capacidade) | 4.000.000 J (`EnergyCubeTier.java:13`) | `fe(4_000_000)` = 1.600.000 FE (`lib.rs:312`) | OK |
| Basic Energy Cube (taxa de saída) | 4.000 J/t (`EnergyCubeTier.java:13`) | `fe(4_000)` = 1.600 FE/t (`lib.rs:312`) | OK |
| Advanced Energy Cube (capacidade) | 16.000.000 J (`EnergyCubeTier.java:14`) | `fe(16_000_000)` = 6.400.000 FE (`lib.rs:313`) | OK |
| Advanced Energy Cube (taxa de saída) | 16.000 J/t (`EnergyCubeTier.java:14`) | `fe(16_000)` = 6.400 FE/t (`lib.rs:313`) | OK |
| Elite Energy Cube (capacidade) | 64.000.000 J (`EnergyCubeTier.java:15`) | `fe(64_000_000)` = 25.600.000 FE (`lib.rs:314`) | OK |
| Elite Energy Cube (taxa de saída) | 64.000 J/t (`EnergyCubeTier.java:15`) | `fe(64_000)` = 25.600 FE/t (`lib.rs:314`) | OK |
| Ultimate Energy Cube (capacidade) | 256.000.000 J (`EnergyCubeTier.java:16`) | `fe(256_000_000)` = 102.400.000 FE (`lib.rs:315`) | OK |
| Ultimate Energy Cube (taxa de saída) | 256.000 J/t (`EnergyCubeTier.java:16`) | `fe(256_000)` = 102.400 FE/t (`lib.rs:315`) | OK |
| Creative Energy Cube (capacidade) | `Long.MAX_VALUE` J (`EnergyCubeTier.java:17`) | 1.000.000.000.000 FE (`lib.rs:309`) | LIMITE |
| Creative Energy Cube (taxa de saída) | `Long.MAX_VALUE` J/t (`EnergyCubeTier.java:17`) | 1.000.000.000 FE/t (`lib.rs:1958`) | LIMITE |
| Descarga de Redstone (slot discharge) | 10.000 J = 4.000 FE (`redstone.json:7`) | 4.000 FE (`cube.rs:15`, antes 10.000 J) | CORRIGIDO |
| Descarga de Bloco de Redstone (slot discharge) | 90.000 J = 36.000 FE (`redstone_block.json:7`) | 36.000 FE (`cube.rs:16`, antes 90.000 J) | CORRIGIDO |
| Configuração inicial de lados (FRONT_OUT_EJECT) | Frente = Output, outras = Input (`TileEntityEnergyCube.java:62`) | Frente = Output, outras = Input (`sides.rs:63-67`) | OK |
| **Matriz de Indução (Induction Matrix - `induction.rs`)** | | | |
| Basic Induction Cell (capacidade) | 8.000.000.000 J (`InductionCellTier.java:11`) | `fe(8_000_000_000)` = 3.200.000.000 FE (`induction.rs:16`) | OK |
| Advanced Induction Cell (capacidade) | 64.000.000.000 J (`InductionCellTier.java:12`) | `fe(64_000_000_000)` = 25.600.000.000 FE (`induction.rs:17`) | OK |
| Elite Induction Cell (capacidade) | 512.000.000.000 J (`InductionCellTier.java:13`) | `fe(512_000_000_000)` = 204.800.000.000 FE (`induction.rs:18`) | OK |
| Ultimate Induction Cell (capacidade) | 4.000.000.000.000 J (`InductionCellTier.java:14`) | `fe(4_000_000_000_000)` = 1.600.000.000.000 FE (`induction.rs:19`) | OK |
| Basic Induction Provider (taxa transferência) | 256.000 J/t (`InductionProviderTier.java:11`) | `fe(256_000)` = 102.400 FE/t (`induction.rs:22`) | OK |
| Advanced Induction Provider (taxa transferência) | 2.048.000 J/t (`InductionProviderTier.java:12`) | `fe(2_048_000)` = 819.200 FE/t (`induction.rs:23`) | OK |
| Elite Induction Provider (taxa transferência) | 16.384.000 J/t (`InductionProviderTier.java:13`) | `fe(16_384_000)` = 6.553.600 FE/t (`induction.rs:24`) | OK |
| Ultimate Induction Provider (taxa transferência) | 131.072.000 J/t (`InductionProviderTier.java:14`) | `fe(131_072_000)` = 52.428.800 FE/t (`induction.rs:25`) | OK |
| Dimensões mínimas e máximas da Matrix | 3 a 18 por lado (`CuboidStructureValidator.java:34`) | 3 a 18 por lado (`induction.rs:66`) | OK |
| Casca válida da Matrix | Casing, Port e Structural Glass (`MatrixValidator.java`, `TileEntityStructuralGlass.java:16`) | Casing, Port e Structural Glass (`induction.rs:52`) | OK |
| Alternância de modo da porta (Port Mode) | Sneak right-click: Input / Output (`TileEntityInductionPort.java:60-64`) | Sneak click: Input / Output (`induction.rs:231-237`) | OK |
| **Máquinas Contínuas (`continuous.rs`, `lib.rs`)** | | | |
| Electrolytic Separator tanques | Água 24.000 mB, Gases 2.400 mB (`TileEntityElectrolyticSeparator.java:94-95`) | `[24_000, 2_400, 2_400]` (`lib.rs:202`) | OK |
| Chemical Washer tanques | Água 10.000 mB, Lodo 10.000 mB (`TileEntityChemicalWasher.java:74-75`) | `[10_000, 10_000, 10_000]` (`lib.rs:199`) | OK |
| Chemical Infuser tanques | Gases 10.000 mB cada (`TileEntityChemicalInfuser.java`) | `[10_000, 10_000, 10_000]` (`lib.rs:197`) | OK |
| Rotary Condensentrator tanques | Fluido 10.000 mB, Gás 10.000 mB (`TileEntityRotaryCondensentrator.java`) | `[10_000, 10_000, 0]` (`lib.rs:205`) | OK |
| Pigment Mixer tanques | Pigmentos 1.000 mB / Saída 2.000 mB (`TileEntityPigmentMixer.java:70-71`) | `[1_000, 1_000, 2_000]` (`lib.rs:214`) | OK |
| Isotopic Centrifuge tanques | Gás 10.000 mB entrada e saída (`MekanismBlockTypes.java:557`) | `[10_000, 10_000, 0]` (`lib.rs:218`) | OK |
| Solar Neutron Activator tanques | Gás 10.000 mB entrada e saída (`MekanismBlockTypes.java:388`) | `[10_000, 10_000, 0]` (`lib.rs:220`) | OK |
| **Bounding Blocks (`bounding.rs`)** | | | |
| Advanced Solar Generator offsets | `[0, 1, 0]` + plano 3x3 em `y=2` (`GeneratorsBlockTypes.java:142-150`) | `[0, 1, 0]` + plano 3x3 em `y=2` (`bounding.rs:21-28`) | OK |
| Wind Generator offsets | 4 blocos acima (`y=1..=4`) (`GeneratorsBlockTypes.java:108-113`) | 4 blocos acima (`y=1..=4`) (`bounding.rs:45`) | OK |
| Pigment Mixer, Centrifuge, Activator | `[0, 1, 0]` (`MekanismBlockTypes.java:393, 562, 605`) | `[0, 1, 0]` (`bounding.rs:19`) | OK |
| Digital Miner offsets | Cubóide 3x3x2 excluindo origem (`MekanismBlockTypes.java:338-342`) | Cubóide 3x3x2 excluindo origem (`bounding.rs:33-42`) | OK |
| **Configuração de Lados (`sides.rs`)** | | | |
| Modos de energia suportados | None (0), Input (1), Output (2), Both (3) (`DataType.java`) | None, Input, Output, Both (`sides.rs:7-12`) | OK |
| Ciclo de modos de lados | None → Input → Output → Both (`DataType.java`) | None → Input → Output → Both (`sides.rs:24-27`) | OK |
| Direções vanilla | Down (0), Up (1), North (2), South (3), West (4), East (5) | Down, Up, North, South, West, East (`sides.rs:53-59`) | OK |
