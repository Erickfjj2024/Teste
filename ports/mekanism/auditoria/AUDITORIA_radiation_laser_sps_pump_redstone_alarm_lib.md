# Relatório de Auditoria: Radiação, Laser, SPS, Electric Pump, Fluidic Plenisher, Redstone/Alarm e Constantes Gerais

**Referência Java de verdade:** Mekanism 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`) em `java/`  
**Escopo do Port:** `port/mod/src/radiation.rs`, `laser.rs`, `sps.rs`, `pump.rs`, `redstone.rs`, `alarm.rs`, `lib.rs`, `config.rs`

---

## 1. Tabela Completa de Auditoria

| Item / Constante | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado | Detalhes / Observações |
|---|---|---|---|---|
| **Radiação: Baseline** | `0.000_000_100` (100 nSv/h) (`RadiationManager.java:80`) | `BASELINE: f64 = 0.000_000_100` (`radiation.rs:10`) | OK | Nível base de radiação no ambiente e nas entidades |
| **Radiação: Magnitude Mínima** | `0.000_010` (10 µSv/h) (`RadiationManager.java:81`) | `MIN_MAGNITUDE: f64 = 0.000_010` (`radiation.rs:11`) | OK | Menor magnitude com efeito mensurável |
| **Radiação: Decay Fontes** | `0.9995D` (`GeneralConfig.java:211`) | `DECAY: f64 = 0.9995` (`radiation.rs:12`) | OK | Taxa de decaimento por segundo de fontes ativas |
| **Radiação: Decay Alvo/Dose** | `0.9995D` (`GeneralConfig.java:213`) | `DECAY: f64 = 0.9995` (`radiation.rs:12`) | OK | Taxa de decaimento da dose acumulada no jogador |
| **Radiação: Efeitos Negativos Min Severity** | `0.1D` (`GeneralConfig.java:215`) | `MIN_SEVERITY: f64 = 0.1` (`radiation.rs:13`) | OK | Severidade mínima para disparar efeitos negativos aleatórios |
| **Radioatividade: Nuclear Waste** | `0.01` Sv/h/mB (`chemical_attribute_radioactivity.json:4`) | `0.01` (`radiation.rs:17`) | OK | Radioatividade por mB de resíduo nuclear |
| **Radioatividade: Spent Nuclear Waste** | `0.01` Sv/h/mB (`chemical_attribute_radioactivity.json:13`) | `0.01` (`radiation.rs:17`) | OK | Radioatividade por mB de resíduo nuclear consumido |
| **Radioatividade: Plutônio** | `0.02` Sv/h/mB (`chemical_attribute_radioactivity.json:7`) | `0.02` (`radiation.rs:18`) | OK | Radioatividade por mB de plutônio |
| **Radioatividade: Polônio** | `0.05` Sv/h/mB (`chemical_attribute_radioactivity.json:10`) | `0.05` (`radiation.rs:19`) | OK | Radioatividade por mB de polônio |
| **Proteção Hazmat: Máscara** | `0.25` (HELMET) (`ItemHazmatSuitArmor.java:22`) | `0.25` (slot 39) (`radiation.rs:26`) | OK | 25% de atenuação da dose |
| **Proteção Hazmat: Jaleco/Gown** | `0.4` (CHESTPLATE) (`ItemHazmatSuitArmor.java:23`) | `0.4` (slot 38) (`radiation.rs:27`) | OK | 40% de atenuação da dose |
| **Proteção Hazmat: Calça/Pants** | `0.2` (LEGGINGS) (`ItemHazmatSuitArmor.java:24`) | `0.2` (slot 37) (`radiation.rs:28`) | OK | 20% de atenuação da dose |
| **Proteção Hazmat: Botas** | `0.15` (BOOTS) (`ItemHazmatSuitArmor.java:25`) | `0.15` (slot 36) (`radiation.rs:29`) | OK | 15% de atenuação da dose (conjunto total = 1.0 = 100%) |
| **RadiationScale: Limites de Escala** | `LOG_BASELINE = log10(10µSv/h)`, `LOG_MAX = log10(100)` (`RadiationScale.java:55-57`) | `MIN_MAGNITUDE.log10()`, `100f64.log10()` (`radiation.rs:44`) | OK | Normalização da dose em escala logarítmica (0.0 a 1.0) |
| **RadiationScale: Tiers / Sons** | Enum NONE, LOW, MEDIUM, ELEVATED, HIGH, EXTREME (`RadiationScale.java:23-35`) | Não há enum no port; severidade calculada via `severity()` (`radiation.rs:40-47`) | LIMITE | Sons do contador Geiger do original não existem na API do WASM |
| **Radiação: Raio de Verificação** | 5 chunks = 80 blocos (`GeneralConfig.java:209`, `RadiationLevelData.java:18`) | Soma sobre todas as fontes carregadas da dimensão (`radiation.rs:50-59`) | LIMITE | Java indexa por cuboid `MAX_BLOCK_RANGE` (80 blocos); port calcula por distância quadrática |
| **Radiação: Taxa de Acumulação** | `magnitude / 3600.0` (Sv/h -> Sv/s) (`PlayerExposure.java:62`) | `level / 3600.0 * (1.0 - resistance.min(1.0))` (`radiation.rs:88`) | OK | Conversão correta de taxa horária para dose por segundo |
| **Radiação: Dano por Dose Alta** | `max(1, log1p(radiation))` (`RadiationEntity.java:55`) | `(dose.ln_1p() as f32).max(1.0)` (`radiation.rs:100`) | OK | Dano mágico aplicado proporcional ao logaritmo natural da dose |
| **Radiação: Barril de Resíduos Radioativos** | Capacidade 512 000 mB, decaimento 1 mB/20t (`GeneralConfig.java:216-220`) | Não implementado no port | FALTA | Bloco `radioactive_waste_barrel` ainda não portado |
| **Laser: Alcance Máximo** | `64` blocos (`GeneralConfig.java:237`) | `RANGE: i32 = 64` (`laser.rs:9`) | OK | Distância máxima do feixe do laser |
| **Laser: Consumo / Disparo por Tick** | `10_000L` J/t (`UsageConfig.java:72`) | `FIRE: u64 = fe(10_000)` = 4 000 FE/t (`laser.rs:11`) | OK | Disparo por tick em FE |
| **Laser: Capacidade de Energia** | `2_000_000L` J (`StorageConfig.java:95`) | `LASER_CAPACITY: u64 = fe(2_000_000)` = 800 000 FE (`laser.rs:13`) | OK | Capacidade do bloco Laser |
| **Laser Amplifier: Capacidade** | `5_000_000_000L` J (`StorageConfig.java:97`) | `AMPLIFIER_CAPACITY: u64 = fe(5_000_000_000)` = 2 000 000 000 FE (`laser.rs:12`) | OK | Capacidade do Laser Amplifier |
| **Laser Amplifier: Delay Padrão** | `0` ticks (`TileEntityLaserAmplifier.java:54`) | `delay: 0` (`laser.rs:88`) | OK | Intervalo em ticks entre pulsos |
| **Laser Amplifier: Thresholds Padrão** | Mínimo `0L`, Máximo `5_000_000_000L` J (`TileEntityLaserAmplifier.java:51-52`) | Mínimo `0`, Máximo `AMPLIFIER_CAPACITY` (`laser.rs:88`) | OK | Limiares de disparo do amplificador |
| **Laser: Energia por Dureza (Escavação)** | `100_000L` J por dureza (`GeneralConfig.java:238`) | Não implementado | LIMITE | Escavação de blocos por laser não portada no WASM |
| **Laser: Energia por Dano** | `2_500L` J por 1 HP (meio coração) (`GeneralConfig.java:239`) | Não implementado | LIMITE | Dano/queima de entidades por laser não portado |
| **Laser Tractor Beam** | `5_000_000_000L` J (`StorageConfig.java:99`) | Não implementado | FALTA | Bloco Laser Tractor Beam ainda não portado |
| **Laser Amplifier: Modos Redstone** | OFF, ENTITY_DETECTION, ENERGY_CONTENTS (`TileEntityLaserAmplifier.java:368`) | Não implementado | LIMITE | Modos de saída redstone do amplificador ausentes |
| **SPS: Polônio por Antimatéria** | `1_000` mB (`GeneralConfig.java:317`) | `INPUT_PER_ANTIMATTER: u64 = 1_000` (`sps.rs:20`) | OK | Taxa de conversão de polônio em antimatéria |
| **SPS: Capacidade do Tanque de Saída** | `1_000L` mB (`GeneralConfig.java:319`) | `OUTPUT_CAPACITY: u64 = 1_000` (`sps.rs:21`) | OK | Armazenamento de antimatéria no SPS |
| **SPS: Capacidade do Tanque de Entrada** | `2_000L` mB (`spsInputPerAntimatter * 2L`) (`SPSMultiblockData.java:82`) | `input_capacity() = 2_000` (`sps.rs:140`) | OK | Armazenamento de polônio no SPS |
| **SPS: Energia por Entrada (Polônio)** | `1_000_000L` J/mB (`GeneralConfig.java:320`) | `ENERGY_PER_INPUT: u64 = fe(1_000_000)` = 400 000 FE (`sps.rs:22`) | OK | Energia por mB de polônio processado |
| **SPS Port: Capacidade de Energia** | `1_000_000_000L` J (`StorageConfig.java:119`) | `PORT_ENERGY: u64 = fe(1_000_000_000)` = 400 000 000 FE (`sps.rs:23`) | OK | Armazenamento por porta do SPS |
| **SPS: Estrutura Multibloco** | Cubo 7x7x7 com cantos arredondados (`SPSValidator.java`) | Grid 7x7x7 idêntico (`sps.rs:25-33`) | OK | Validação e peças (casing, port, coil) |
| **SPS: Dano na Death Zone** | `lastReceivedEnergy / 1000F` dano/tick (`SPSMultiblockData.java:227`) | Não implementado | LIMITE | Dano a entidades no interior do acelerador não implementado |
| **Electric Pump: Consumo de Energia** | `100L` J/t (`UsageConfig.java:59`) | `USAGE: u64 = crate::fe(100)` = 40 FE/t (`pump.rs:8`) | OK | Consumo por tick da bomba |
| **Electric Pump: Armazenamento** | `40_000L` J (`StorageConfig.java:68`) | `storage_of("mekanism:electric_pump")` = `fe(40_000)` = 16 000 FE (`lib.rs:298`) | OK | Armazenamento de energia da bomba |
| **Electric Pump: Ticks por Operação** | `19` ticks (`TileEntityElectricPump.java:71`) | `BASE_TICKS: u32 = 19` (`pump.rs:10`) | OK | Ticks de ciclo base da bomba |
| **Electric Pump: Tanque Interno** | `10_000` mB (10 baldes) (`TileEntityElectricPump.java:72`) | `TANK: u64 = 10_000` (`pump.rs:12`) | OK | Capacidade do tanque de fluidos |
| **Electric Pump: Alcance Máximo** | `80` blocos (`GeneralConfig.java:249`) | `RANGE: i32 = 80` (`pump.rs:14`) | OK | Alcance de busca de fontes de fluido |
| **Electric Pump: Heavy Water Extraction** | `10` mB por balde de água (`GeneralConfig.java:253`) | Não implementado | LIMITE | Extração de água pesada com Filter Upgrade não portada |
| **Fluidic Plenisher: Consumo de Energia** | `100L` J/t (`UsageConfig.java:71`) | `USAGE: u64 = crate::fe(100)` = 40 FE/t (`pump.rs:8`, `lib.rs:154`) | OK | Consumo por tick do plenisher |
| **Fluidic Plenisher: Armazenamento** | `40_000L` J (`StorageConfig.java:93`) | `storage_of("mekanism:fluidic_plenisher")` = `fe(40_000)` = 16 000 FE (`lib.rs:298`) | OK | Armazenamento de energia do plenisher |
| **Fluidic Plenisher: Ticks por Operação** | `20` ticks (`TICKS_PER_SECOND`) (`TileEntityFluidicPlenisher.java:62`) | `20` ticks (`pump.rs:219`) | OK | Ticks de ciclo base do plenisher |
| **Fluidic Plenisher: Tanque Interno** | `10_000` mB (10 baldes) (`TileEntityFluidicPlenisher.java:63`) | `TANK: u64 = 10_000` (`pump.rs:12`) | OK | Capacidade do tanque do plenisher |
| **Fluidic Plenisher: Max Plenisher Nodes** | `4_000` (`GeneralConfig.java:255`) | Era `4096`; corrigido para `4000` (`pump.rs:56`) | CORRIGIDO | Quantidade máxima de nós preenchidos pelo plenisher |
| **Redstone: Modos de Controle** | DISABLED (0), HIGH (1), LOW (2), PULSE (3) (`IRedstoneControl.java:41-44`) | Disabled (0), High (1), Low (2), Pulse (3) (`redstone.rs:8-13`) | OK | Enum de controle por sinal redstone |
| **Redstone: Lógica de Ativação** | Disabled: true; High: powered; Low: !powered; Pulse: powered && !lastTick (`TileEntityMekanism.java:1121-1126`) | Idêntico em `can_function()` (`redstone.rs:32-38`) | OK | Condição booleana de operação por sinal |
| **Industrial Alarm: Bloco e Estado Lit** | `setActive(isPowered())` (`TileEntityIndustrialAlarm.java:20`) | `lit` atualizado pelo sinal (`alarm.rs:29-32`) | OK | Ativação e iluminação por sinal de redstone |
| **Industrial Alarm: Áudio e Feixe** | Efeitos de som e feixe giratório (`RenderIndustrialAlarm.java`) | Não implementado | LIMITE | Restrição do ambiente WASM |
| **Conversão FE/J** | `forgeConversionRate = 2.5` (`GeneralConfig.java:195`) | `joules * 2 / 5` (`lib.rs:287`) | OK | 2.5 Joules = 1 Forge Energy |
| **Capacidade Cubo Criativo** | `Long.MAX_VALUE` (`EnergyCubeTier.java:16`) | `CREATIVE_CUBE = 1_000_000_000_000` (`lib.rs:309`) | OK | Representação finita segura de energia ilimitada |
| **Energy Cube Tiers (Armazenamento / Saída)** | Basic (4M/4k), Advanced (16M/16k), Elite (64M/64k), Ultimate (256M/256k) em Joules (`EnergyCubeTier.java:12-15`) | Convertidos por `fe()` para FE: Basic (1.6M/1.6k), Adv (6.4M/6.4k), Elite (25.6M/25.6k), Ult (102.4M/102.4k) (`lib.rs:312-315`) | OK | Todos os 4 tiers em conformidade com o Java |
| **Universal Cable Capacidades** | Basic 8kJ, Adv 128kJ, Elite 1 024kJ, Ult 8 192kJ (`CableTier.java:12-15`) | Convertidos por `fe()`: 3.2k, 51.2k, 409.6k, 3 276.8k FE (`lib.rs:332`) | OK | Vazão/capacidade por tier dos cabos |
| **Pressurized Tube Vazões** | Basic 750, Adv 2 000, Elite 64 000, Ult 256 000 mB/t (`TubeTier.java:11-14`) | Idêntico: `[750, 2_000, 64_000, 256_000]` (`lib.rs:333`) | OK | Taxa de extração/transporte de tubos químicos |
| **Mechanical Pipe Vazões** | Basic 250, Adv 1 000, Elite 8 000, Ult 32 000 mB/t (`PipeTier.java:11-14`) | Idêntico: `[250, 1_000, 8_000, 32_000]` (`lib.rs:334`) | OK | Taxa de extração/transporte de canos de fluido |
| **Logistical Transporter Taxas** | Basic 1, Adv 16, Elite 32, Ult 64 itens (`TransporterTier.java:9-12`) | Idêntico: `[1, 16, 32, 64]` (`lib.rs:336`) | OK | Itens puxados por operação do transportador |
| **Fluid Tank Tiers (Capacidade / Saída)** | Basic (32k/1k), Adv (64k/4k), Elite (128k/16k), Ult (256k/64k) mB (`FluidTankTier.java:10-14`) | Idêntico: 32k/1k, 64k/4k, 128k/16k, 256k/64k (`lib.rs:271-274`) | OK | Capacidades e vazões dos tanques de fluido |
| **Chemical Tank Tiers (Capacidade / Saída)** | Basic (64k/1k), Adv (256k/16k), Elite (1 024k/128k), Ult (8 192k/512k) mB (`ChemicalTankTier.java:12-16`) | Idêntico: 64k/1k, 256k/16k, 1 024k/128k, 8 192k/512k (`lib.rs:276-279`) | OK | Capacidades e vazões dos tanques químicos |
| **Armazenamento de Máquinas (`storage_of`)** | 20kJ / 40kJ / 80kJ / 160kJ / 50kJ / 2kJ / 1GJ (`StorageConfig.java:44-121`) | 26 máquinas mapeadas e convertidas por `fe()` (`lib.rs:290-308`) | OK | Todas as máquinas do match conferidas contra StorageConfig |
| **Consumo de Máquinas (`MACHINES`)** | Crusher 50J, PRC 5J, Crystallizer 400J, etc. (`UsageConfig.java:49-79`) | Convertidos por `fe()` (`lib.rs:181-269`) | OK | Todos os 78 registros da tabela conferidos |
| **Nutritional Liquifier: Taxa de Pasta** | `50` mB por ponto de nutrição (`GeneralConfig.java:281`) | `nutrition * 50` mB (`lib.rs:643`) | OK | Conversão de comida em pasta nutricional |
| **Nutritional Liquifier: Alimentos** | 44 alimentos vanilla com nutrição de 0 a 10 (`Foods.java` do Minecraft 1.21.1) | 44 entradas mapeadas em `FOODS` (`lib.rs:624-636`) | OK | Valores nutricionais idênticos aos do Vanilla 1.21.1 |
| **Altura dos Olhos** | Em pé: 1.62; Agachado: 1.27 (`Player.java` Vanilla) | `EYE = 1.62`, `EYE_SNEAKING = 1.27` (`lib.rs:501-502`) | OK | Constantes de altura dos olhos do jogador |
| **Side Config: DataType Ordinais** | 0 a 9 (None, Input, Input1, Input2, Output, Output1, Output2, InputOutput, Energy, Extra) (`DataType.java:21-30`) | Idêntico: ordinais 0 a 9 (`config.rs:8-19`) | OK | Tipos de dados de conectividade de faces |
| **Side Config: Transmissions** | Items, Energy, Fluids, Chemicals, Heat (`TransmissionType.java`) | Transmissions 0 a 4 (`config.rs:68-74`) | OK | Redes configuráveis nas máquinas |
| **Configurator: Modos** | Items, Fluids, Chemicals, Energy, Heat, Empty, Rotate, Wrench (`ConfiguratorMode`) | Modos 0 a 7 (`config.rs:105-114`) | OK | Modos de operação do configurator |
| **Side Config: RelativeSide** | FRONT (0), LEFT (1), RIGHT (2), BACK (3), TOP (4), BOTTOM (5) (`RelativeSide.java:17-23`) | `FRONT..BOTTOM: usize = 0..5` (`config.rs:507-512`) | OK | Mapeamento de lados relativos |
| **Direções Espaciais** | DOWN (0), UP (1), NORTH (2), SOUTH (3), WEST (4), EAST (5) (`Direction.java` Vanilla) | `DOWN..EAST: usize = 0..5` (`config.rs:872-877`, `laser.rs:24-29`) | OK | Mapeamento de faces cardeais e verticais |

---

## 2. Resumo das Correções Aplicadas no Código

1. **`port/mod/src/pump.rs`**:
   - `MAX_NODES`: corrigido de `4096` para `4000` para coincidir com `MekanismConfig.general.maxPlenisherNodes` (`GeneralConfig.java:255`).
   - Adicionado teste unitário `max_nodes_matches_java_config` validando `assert_eq!(MAX_NODES, 4000)`.
2. **`port/mod/src/laser.rs`**:
   - Atualizado o comentário do cabeçalho que ainda citava "20 FE a tick" (valor da 26.3) para "4,000 FE a tick", consistente com `fe(10_000)` do Mekanism 1.21.1.
