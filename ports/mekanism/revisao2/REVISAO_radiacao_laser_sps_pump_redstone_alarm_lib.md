# Relatório de Revisão Cruzada: Radiação, Laser, SPS, Electric Pump, Fluidic Plenisher, Redstone/Alarm e Constantes Gerais

**Tarefa:** T20261109-195424-mh3 (Revisão Cruzada - Rodada 2)  
**Fonte da Verdade:** Mekanism Java 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`) em `java/`  
**Escopo do Port:** `port/mod/src/radiation.rs`, `laser.rs`, `sps.rs`, `pump.rs`, `redstone.rs`, `alarm.rs`, `lib.rs`, `config.rs`  
**Relatório Anterior Comparado:** `port/auditoria/AUDITORIA_radiation_laser_sps_pump_redstone_alarm_lib.md` e `RESUMO_t8.md`

---

## 1. Metodologia da Revisão Cruzada

Esta revisão cruzada foi executada de forma independente do zero, conferindo cada valor numérico, fórmula de conversão e comportamento lógico diretamente no código-fonte Java do Mekanism 1.21.1 e nos arquivos Rust do port.  
Todas as correções aplicadas na rodada 1 foram verificadas (marcadas como **CONFIRMADA**). Todos os itens que a rodada 1 classificou como "LIMITE" foram **reclassificados como FORK**, especificando a respectiva capacidade exigida no SDK do Horizon (`port/API-REQUISITOS.md`). Além disso, foram levantados os aspectos que a rodada 1 **não viu**: persistência e descarte ao quebrar blocos, ordem de operações por tick, interações com entidades, eventos de clique, strings e tooltips oficiais da tradução e integração com o sistema de upgrades.

---

## 2. Tabela de Conferência e Revisão Cruzada

| # | Item / Constante | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes da Auditoria e Reclassificação |
|---|---|---|---|---|---|
| 1 | **Radiação: Baseline** | `RadiationManager.java:77` (`BASELINE = 0.000_000_100`) | `radiation.rs:11` (`BASELINE = 0.000_000_100`) | CONFIRMADA | Nível base de radiação natural de fundo (100 nSv/h). Idêntico. |
| 2 | **Radiação: Magnitude Mínima** | `RadiationManager.java:78` (`MIN_MAGNITUDE = 0.000_010`) | `radiation.rs:12` (`MIN_MAGNITUDE = 0.000_010`) | CONFIRMADA | Limiar mínimo com efeito mensurável (10 µSv/h). Idêntico. |
| 3 | **Radiação: Decaimento de Fontes** | `GeneralConfig.java:211` (`radiationSourceDecayRate = 0.9995D`) | `radiation.rs:13` (`DECAY = 0.9995`) | CONFIRMADA | Fator de decaimento por segundo de fontes no ambiente. Idêntico. |
| 4 | **Radiação: Decaimento de Dose/Alvo** | `GeneralConfig.java:213` (`radiationTargetDecayRate = 0.9995D`) | `radiation.rs:13` (`DECAY = 0.9995`) | CONFIRMADA | Fator de decaimento por segundo da dose acumulada em entidades. Idêntico. |
| 5 | **Radiação: Min Severity para Efeitos** | `GeneralConfig.java:215` (`radiationNegativeEffectsMinSeverity = 0.1D`) | `radiation.rs:14` (`MIN_SEVERITY = 0.1`) | CONFIRMADA | Severidade mínima na escala (0.1) para início dos efeitos negativos. Idêntico. |
| 6 | **Radioatividade: Nuclear Waste** | `chemical_attribute_radioactivity.json:4` (`0.01` Sv/h/mB) | `radiation.rs:18` (`0.01`) | CONFIRMADA | Taxa de emissão de radioatividade por mB de resíduo nuclear. Idêntico. |
| 7 | **Radioatividade: Spent Nuclear Waste** | `chemical_attribute_radioactivity.json:13` (`0.01` Sv/h/mB) | `radiation.rs:18` (`0.01`) | CONFIRMADA | Taxa de emissão por mB de resíduo nuclear consumido. Idêntico. |
| 8 | **Radioatividade: Plutônio** | `chemical_attribute_radioactivity.json:7` (`0.02` Sv/h/mB) | `radiation.rs:19` (`0.02`) | CONFIRMADA | Taxa de emissão por mB de plutônio. Idêntico. |
| 9 | **Radioatividade: Polônio** | `chemical_attribute_radioactivity.json:10` (`0.05` Sv/h/mB) | `radiation.rs:20` (`0.05`) | CONFIRMADA | Taxa de emissão por mB de polônio. Idêntico. |
| 10 | **Proteção Hazmat: Máscara / Helmet** | `ItemHazmatSuitArmor.java:23` (`HELMET -> 0.25`) | `radiation.rs:28` (`(39, "mekanism:hazmat_mask") => 0.25`) | CONFIRMADA | Atenuação de 25% da radiação na cabeça (slot 39). Idêntico. |
| 11 | **Proteção Hazmat: Jaleco / Chestplate** | `ItemHazmatSuitArmor.java:24` (`CHESTPLATE -> 0.4`) | `radiation.rs:29` (`(38, "mekanism:hazmat_gown") => 0.4`) | CONFIRMADA | Atenuação de 40% da radiação no peitoral (slot 38). Idêntico. |
| 12 | **Proteção Hazmat: Calça / Leggings** | `ItemHazmatSuitArmor.java:25` (`LEGGINGS -> 0.2`) | `radiation.rs:30` (`(37, "mekanism:hazmat_pants") => 0.2`) | CONFIRMADA | Atenuação de 20% da radiação nas pernas (slot 37). Idêntico. |
| 13 | **Proteção Hazmat: Botas / Boots** | `ItemHazmatSuitArmor.java:26` (`BOOTS -> 0.15`) | `radiation.rs:31` (`(36, "mekanism:hazmat_boots") => 0.15`) | CONFIRMADA | Atenuação de 15% da radiação nos pés (slot 36). Conjunto completo = 100%. Idêntico. |
| 14 | **RadiationScale: Limites e Logaritmo** | `RadiationScale.java:55-57` (`LOG_BASELINE = log10(1e-5)`, `LOG_MAX = log10(100)`) | `radiation.rs:48` (`(MIN_MAGNITUDE.log10(), 100f64.log10())`) | CONFIRMADA | Mapeamento logarítmico de severidade entre 10 µSv/h e 100 Sv/h. Idêntico. |
| 15 | **RadiationScale: Tiers e Sons de Áudio** | `RadiationScale.java:23-35,69-77` (`NONE`, `LOW`, `MEDIUM`, `ELEVATED`, `HIGH`, `EXTREME`, sons Geiger) | `radiation.rs:44-50` (apenas `severity()`) | **FORK (FORK-13)** | Reclassificado de LIMITE para FORK. Sons procedurais de cliques do contador Geiger (`geiger_slow`, `geiger_medium`, etc.) exigem API de reprodução de áudio dinâmico no cliente do Horizon. |
| 16 | **Radiação: Raio de Verificação** | `RadiationLevelData.java:21-25` (`radiationChunkCheckRadius * 16 = 80` blocos) | `radiation.rs:53-63` (`level_at` itera todas as fontes da dimensão) | **FORK (FORK-13)** | Reclassificado de LIMITE para FORK. Java indexa por cuboid restrito a 80 blocos via `IndexedCuboidMap`; o port soma todas as fontes ativas. Requer subsistema de chunks indexados. |
| 17 | **Radiação: Taxa de Exposição Horária** | `PlayerExposure.java:65` (`magnitude / 3600.0` Sv/h para Sv/s) | `radiation.rs:97` (`level / 3600.0 * (1.0 - resistance.min(1.0))`) | CONFIRMADA | Conversão estrita de taxa por hora para dose acumulada por segundo com fator de blindagem. Idêntico. |
| 18 | **Radiação: Dano por Dose Elevada** | `RadiationEntity.java:57-77` (`Math.max(1, (float) Math.log1p(radiation))`, dano `mekanism:radiation`, `addExhaustion`) | `radiation.rs:104-115,133` (`player_items::hurt(..., "minecraft:magic", amount)`) | **FORK (FORK-11)** | Novo detalhe: No Java aplica tipo de dano customizado `mekanism:radiation` e exaustão alimentar (`player.getFoodData().addExhaustion(strength)`). O port usa `minecraft:magic` sem exaustão. |
| 19 | **Radiação: Despejo ao Quebrar Bloco** | `TileEntityMekanism.java:712-716` (`dumpRadiation` descarrega tanques radioativos na quebra) | Não implementado no port | **FORK (FORK-13)** | Novo detalhe: Ao quebrar qualquer máquina, tanque ou tubo contendo químico radioativo, o resíduo é dispersado no ambiente e removido do item dropado. |
| 20 | **Geiger Counter: Formatação e Decay Timer** | `ItemGeigerCounter.java:40-45` (`RADIATION_EXPOSURE`, `3` decimais, `RADIATION_DECAY_TIME`) | `radiation.rs:143,168` (`Radiation: {} Sv/h`, `2` decimais, formato simplificado) | **FORK (FORK API)** | Novo detalhe: Java usa texto oficial `"Radiation Exposure: 100.000 nSv/h"` e tempo de decaimento `"Time to Decay: Xh Ym"`. Port usa string customizada sem timer. |
| 21 | **Dosimeter: Interação com Entidades** | `ItemDosimeter.java:49-58` (`interactLivingEntity` lê dose da entidade alvo) | `radiation.rs:145` (apenas dose do próprio jogador) | **FORK (FORK API)** | Novo detalhe: Clique direito em outro jogador ou mob exibe `"Entity Radiation Exposure: %s"`. Port suporta apenas leitura própria. |
| 22 | **Radioactive Waste Barrel** | `BlockRadioactiveWasteBarrel.java`, `TileEntityRadioactiveWasteBarrel.java` (512k mB, 1 mB/20t) | `registro_blocos_itens.csv` lista como `FALTA` | **FALTA** | Bloco não implementado no mod. Especificação completa detalhada fornecida na Parte B deste relatório. |
| 23 | **Laser: Alcance Máximo** | `GeneralConfig.java:237` (`laserRange = 64`) | `laser.rs:11` (`RANGE: i32 = 64`) | CONFIRMADA | Distância máxima do traçado do feixe de laser. Idêntico. |
| 24 | **Laser: Consumo / Disparo por Tick** | `UsageConfig.java:72` (`laser = 10_000L` J/t -> `4_000` FE/t) | `laser.rs:13` (`FIRE = fe(10_000)` = 4 000 FE/t) | CONFIRMADA | Disparo por tick do bloco Laser. Idêntico. |
| 25 | **Laser: Armazenamento de Energia** | `StorageConfig.java:95` (`laser = 2_000_000L` J -> `800_000` FE) | `laser.rs:15` (`LASER_CAPACITY = fe(2_000_000)` = 800 000 FE) | CONFIRMADA | Buffer interno de energia do Laser. Idêntico. |
| 26 | **Laser Amplifier: Capacidade** | `StorageConfig.java:97` (`laserAmplifier = 5_000_000_000L` J -> `2_000_000_000` FE) | `laser.rs:14` (`AMPLIFIER_CAPACITY = fe(5_000_000_000)` = 2 000 000 000 FE) | CONFIRMADA | Capacidade do Laser Amplifier em FE. Idêntico. |
| 27 | **Laser Amplifier: Delay Padrão** | `TileEntityLaserAmplifier.java:49` (`delay = 0`) | `laser.rs:94` (`delay: 0`) | CONFIRMADA | Intervalo em ticks entre disparos sucessivos do amplificador. Idêntico. |
| 28 | **Laser Amplifier: Thresholds Padrão** | `TileEntityLaserAmplifier.java:46-47` (`minThreshold = 0L`, `maxThreshold = maxEnergy`) | `laser.rs:94` (`min: 0`, `max: AMPLIFIER_CAPACITY as u32`) | CONFIRMADA | Limiares mínimo e máximo para autorizar o pulso laser. Idêntico. |
| 29 | **Laser: Energia por Dureza (Escavação)** | `GeneralConfig.java:238` (`laserEnergyPerHardness = 100_000L` J = `40_000` FE) | Não implementado no port | **FORK (FORK-8)** | Reclassificado de LIMITE para FORK. Escavação progressiva de blocos conforme dureza requer API de mineração de blocos orientada por energia. |
| 30 | **Laser: Energia por Dano (HP)** | `GeneralConfig.java:239` (`laserEnergyPerDamage = 2_500L` J = `1_000` FE) | Não implementado no port | **FORK (FORK-7)** | Reclassificado de LIMITE para FORK. Dano térmico em entidades que interceptam o feixe de laser exige API de dano físico por raio contínuo. |
| 31 | **Laser Tractor Beam** | `StorageConfig.java:99` (5 GJ = 2 GFE), `TileEntityLaserTractorBeam.java` (27 slots) | Não implementado no port | **FALTA** | Receptor de laser com inventário de 27 slots de saída para sucção de drops e atração de itens. |
| 32 | **Laser Amplifier: Modos Redstone** | `TileEntityLaserAmplifier.java:51,92-97` (`OFF`, `ENTITY_DETECTION`, `ENERGY_CONTENTS`) | Não implementado no port | **FORK (FORK-21)** | Reclassificado de LIMITE para FORK. Emissão de sinal redstone conforme modo (detecção de entidade ou comparador por energia). |
| 33 | **SPS: Polônio por Antimatéria** | `GeneralConfig.java:317` (`spsInputPerAntimatter = 1_000`) | `sps.rs:20` (`INPUT_PER_ANTIMATTER = 1_000`) | CONFIRMADA | 1 000 mB de polônio requeridos para sintetizar 1 mB de antimatéria. Idêntico. |
| 34 | **SPS: Capacidade do Tanque de Saída** | `GeneralConfig.java:319` (`spsOutputTankCapacity = 1_000L`) | `sps.rs:21` (`OUTPUT_CAPACITY = 1_000`) | CONFIRMADA | Armazenamento de antimatéria na câmara de saída do acelerador. Idêntico. |
| 35 | **SPS: Capacidade do Tanque de Entrada** | `SPSMultiblockData.java:82` (`spsInputPerAntimatter * 2L = 2_000L`) | `sps.rs:140` (`input_capacity() = 2_000`) | CONFIRMADA | Buffer de entrada de polônio (o dobro do consumo de 1 mB). Idêntico. |
| 36 | **SPS: Energia por mB de Entrada** | `GeneralConfig.java:320` (`spsEnergyPerInput = 1_000_000L` J = `400_000` FE) | `sps.rs:22` (`ENERGY_PER_INPUT = fe(1_000_000)` = 400 000 FE) | CONFIRMADA | Energia consumida por cada mB de polônio acelerado. Idêntico. |
| 37 | **SPS Port: Capacidade de Energia** | `StorageConfig.java:119` (`spsPort = 1_000_000_000L` J = `400_000_000` FE) | `sps.rs:23` (`PORT_ENERGY = fe(1_000_000_000)` = 400 000 000 FE) | CONFIRMADA | Buffer individual de cada porta conectada a bobinas supercharged. Idêntico. |
| 38 | **SPS: Malha Multibloco 7x7x7** | `SPSValidator.java:GRID` (cubo 7x7x7 com cantos arredondados) | `sps.rs:25-33` (`GRID: [[u8; 7]; 7]`) | CONFIRMADA | Validação de casing, structural glass, ports e coils alinhadas. Idêntico. |
| 39 | **SPS: Death Zone e Relâmpago** | `SPSMultiblockData.java:200-238` (`hurt(..., lastReceivedEnergy / 1000F)`, relâmpago de experimento) | Não implementado no port | **FORK (FORK-7 / FORK-10)** | Reclassificado de LIMITE para FORK. Dano elétrico letal dentro da câmara e conversão de mobs válidos (ex.: porco em zombie piglin). |
| 40 | **Electric Pump: Consumo de Energia** | `UsageConfig.java:59` (`electricPump = 100L` J -> `40` FE/t) | `pump.rs:8` (`USAGE = fe(100)` = 40 FE/t) | CONFIRMADA | Consumo operacional por tick de bombeamento. Idêntico. |
| 41 | **Electric Pump: Buffer de Energia** | `StorageConfig.java:68` (`electricPump = 40_000L` J -> `16_000` FE) | `lib.rs:298` (`storage_of(...) = fe(40_000)` = 16 000 FE) | CONFIRMADA | Buffer de energia armazenada pela bomba elétrica. Idêntico. |
| 42 | **Electric Pump: Ticks por Ciclo** | `TileEntityElectricPump.java:71` (`BASE_TICKS_REQUIRED = 19`) | `pump.rs:10` (`BASE_TICKS = 19`) | CONFIRMADA | Duração da operação base da bomba. Idêntico. |
| 43 | **Electric Pump: Tanque de Fluido** | `TileEntityElectricPump.java:72` (`MAX_FLUID = 10_000`) | `pump.rs:12` (`TANK = 10_000`) | CONFIRMADA | Volume interno do tanque de fluidos (10 baldes). Idêntico. |
| 44 | **Electric Pump: Alcance Horizontal** | `GeneralConfig.java:249` (`maxPumpRange = 80`) | `pump.rs:14` (`RANGE = 80`) | CONFIRMADA | Raio máximo de busca de fontes de fluidos a partir da base da bomba. Idêntico. |
| 45 | **Electric Pump: Heavy Water Extraction** | `GeneralConfig.java:253` (`pumpHeavyWaterAmount = 10` mB/balde), `TileEntityElectricPump.java:279` | Não implementado no port | **FORK (FORK-22)** | Reclassificado de LIMITE para FORK. Instalação de Filter Upgrade permite extrair 10 mB de Heavy Water ao bombear água. |
| 46 | **Fluidic Plenisher: Consumo de Energia** | `UsageConfig.java:71` (`fluidicPlenisher = 100L` J -> `40` FE/t) | `pump.rs:8` (`USAGE = fe(100)` = 40 FE/t) | CONFIRMADA | Consumo operacional por tick de preenchimento. Idêntico. |
| 47 | **Fluidic Plenisher: Buffer de Energia** | `StorageConfig.java:93` (`fluidicPlenisher = 40_000L` J -> `16_000` FE) | `lib.rs:298` (`storage_of(...) = fe(40_000)` = 16 000 FE) | CONFIRMADA | Buffer de energia do plenisher. Idêntico. |
| 48 | **Fluidic Plenisher: Ticks por Ciclo** | `TileEntityFluidicPlenisher.java:62` (`TICKS_PER_SECOND = 20`) | `pump.rs:227` (`ticks = upgrades::ticks(if def.form == 1 { 20 } else { BASE_TICKS }, ...)`) | CONFIRMADA | Plenisher roda a cada 20 ticks (1 segundo); bomba a cada 19 ticks. Idêntico. |
| 49 | **Fluidic Plenisher: Tanque Interno** | `TileEntityFluidicPlenisher.java:63` (`MAX_FLUID = 10_000`) | `pump.rs:12` (`TANK = 10_000`) | CONFIRMADA | Capacidade do reservatório de fluido (10 baldes). Idêntico. |
| 50 | **Fluidic Plenisher: Max Nodes** | `GeneralConfig.java:255` (`maxPlenisherNodes = 4_000`) | `pump.rs:56` (`MAX_NODES = 4000`) | CONFIRMADA | Correção da rodada 1 (de 4096 para 4000) conferida no código Java. |
| 51 | **Upgrades em Pump vs Plenisher** | `MekanismBlockTypes.java:370,380` (Pump: Speed, Energy, Filter; Plenisher: Speed, Energy) | `upgrades.rs:40-54` (Kind::Pump cai no default, permitindo Muffling e não Filter) | **CORRIGIDO / OBS** | Novo detalhe: No Java nenhuma das bombas aceita Muffling; Electric Pump aceita Filter Upgrade. |
| 52 | **Redstone: Modos de Controle** | `IRedstoneControl.java:41-44` (Disabled=0, High=1, Low=2, Pulse=3) | `redstone.rs:8-13` (Disabled=0, High=1, Low=2, Pulse=3) | CONFIRMADA | Enum de quatro modos de operação redstone. Idêntico. |
| 53 | **Redstone: Condição de Disparo** | `TileEntityMekanism.java:1121-1126` (`Disabled: true`, `High: powered`, `Low: !powered`, `Pulse: powered && !last`) | `redstone.rs:35-42` (`can_function()`) | CONFIRMADA | Avaliação booleana tick a tick do sinal de redstone. Idêntico. |
| 54 | **Industrial Alarm: Ativação por Redstone** | `TileEntityIndustrialAlarm.java:20` (`setActive(isPowered())`) | `alarm.rs:31-35` (`world::set_block_property(..., "lit", powered)`) | CONFIRMADA | Ativação do bloco sob presença de sinal de redstone. Idêntico. |
| 55 | **Industrial Alarm: Áudio e Feixe de Luz** | `RenderIndustrialAlarm.java`, `MekanismSounds.INDUSTRIAL_ALARM` | Não implementado no port | **FORK (FORK-19)** | Reclassificado de LIMITE para FORK. Sirene em loop e renderizador do feixe rotativo de luz exigem recursos gráficos e de áudio do SDK. |
| 56 | **Conversão Joule para FE** | `GeneralConfig.java:195` (`forgeConversionRate = 2.5`) | `lib.rs:287` (`joules * 2 / 5`) | CONFIRMADA | Fórmula matemática exata: 2.5 Joules = 1 Forge Energy. Idêntico. |
| 57 | **Creative Energy Cube** | `EnergyCubeTier.java:17` (`Long.MAX_VALUE`, `Long.MAX_VALUE`) | `lib.rs:309` (`CREATIVE_CUBE = 1_000_000_000_000`) | **FORK (FORK-5)** | Reclassificado de LIMITE para FORK. Valor finito seguro no port (1 TFE) para prevenir overflow em aritmética de 64 bits. |
| 58 | **Energy Cube Tiers (4 Tiers)** | `EnergyCubeTier.java:13-16` (Basic 4M/4k, Adv 16M/16k, Elite 64M/64k, Ult 256M/256k J) | `lib.rs:311-316` (Basic 1.6M/1.6k, Adv 6.4M/6.4k, Elite 25.6M/25.6k, Ult 102.4M/102.4k FE) | CONFIRMADA | Todos os 4 tiers convertidos perfeitamente por `fe()`. Idêntico. |
| 59 | **Universal Cable: Vazões por Tier** | `CableTier.java:12-15` (8kJ, 128kJ, 1 024kJ, 8 192kJ) | `lib.rs:332` (`[fe(8_000), fe(128_000), fe(1_024_000), fe(8_192_000)]`) | CONFIRMADA | Vazões convertidas: 3.2k, 51.2k, 409.6k, 3 276.8k FE/t. Idêntico. |
| 60 | **Pressurized Tube: Vazões por Tier** | `TubeTier.java:11-14` (750, 2 000, 64 000, 256 000 mB/t) | `lib.rs:333` (`[750, 2_000, 64_000, 256_000]`) | CONFIRMADA | Taxa de transferência química por tick. Idêntico. |
| 61 | **Mechanical Pipe: Vazões por Tier** | `PipeTier.java:11-14` (250, 1 000, 8 000, 32 000 mB/t) | `lib.rs:334` (`[250, 1_000, 8_000, 32_000]`) | CONFIRMADA | Taxa de vazão de fluidos por tick. Idêntico. |
| 62 | **Logistical Transporter: Pull Rates** | `TransporterTier.java:9-12` (1, 16, 32, 64 itens) | `lib.rs:336` (`[1, 16, 32, 64]`) | CONFIRMADA | Quantidade máxima de itens extraídos a cada 10 ticks. Idêntico. |
| 63 | **Fluid Tank Tiers (Capacidade / Saída)** | `FluidTankTier.java:10-14` (32k/1k, 64k/4k, 128k/16k, 256k/64k) | `lib.rs:274-277` (`[32_000, 1_000]`, `[64_000, 4_000]`, etc.) | CONFIRMADA | Capacidade volumétrica e taxa de saída dos tanques de fluido. Idêntico. |
| 64 | **Chemical Tank Tiers (Capacidade / Saída)** | `ChemicalTankTier.java:12-16` (64k/1k, 256k/16k, 1 024k/128k, 8 192k/512k) | `lib.rs:279-282` (`[64_000, 1_000]`, `[256_000, 16_000]`, etc.) | CONFIRMADA | Capacidade volumétrica e taxa de saída dos tanques químicos. Idêntico. |
| 65 | **Armazenamento de Máquinas (`storage_of`)** | `StorageConfig.java:44-121` (20kJ a 1GJ mapeados por máquina) | `lib.rs:293-307` (`storage_of` com 26 máquinas convertidas por `fe()`) | CONFIRMADA | Todos os valores de buffer base conferidos e exatos. Idêntico. |
| 66 | **Consumo de Máquinas (`MACHINES`)** | `UsageConfig.java:49-79` (Crusher 50J, PRC 5J, Crystallizer 400J, etc.) | `lib.rs:181-250` (Tabela `MACHINES` com 78 entradas convertidas por `fe()`) | CONFIRMADA | Todos os 78 registros conferidos diretamente contra `UsageConfig.java`. Idêntico. |
| 67 | **Nutritional Liquifier: Taxa de Pasta** | `GeneralConfig.java:281` (`nutritionalPasteMBPerFood = 50`) | `lib.rs:643` (`nutrition * 50`) | CONFIRMADA | Multiplicador de conversão de pontos de nutrição em mB de pasta. Idêntico. |
| 68 | **Nutritional Liquifier: Alimentos Vanilla** | `Foods.java` do Minecraft 1.21.1 (44 alimentos com valor nutritivo) | `lib.rs:624-636` (`FOODS: [(&str, u64); 44]`) | CONFIRMADA | Todos os 44 alimentos e seus pontos nutricionais conferidos. Idêntico. |
| 69 | **Altura dos Olhos do Jogador** | Minecraft 1.21.1 `Player.java` (em pé: 1.62; agachado: 1.27) | `lib.rs:501-502` (`EYE = 1.62`, `EYE_SNEAKING = 1.27`) | CONFIRMADA | Constantes de altura de linha de visão para colocação direcional de blocos. Idêntico. |
| 70 | **Side Config: DataType Ordinais** | `DataType.java:21-30` (Ordinais 0 a 9) | `config.rs:8-19` (`Data`: None=0 a Extra=9) | CONFIRMADA | Ordinais numéricos, nomes de labels e cores de conexões de lados. Idêntico. |
| 71 | **Side Config: Transmission Types** | `TransmissionType.java` (Items, Energy, Fluids, Chemicals, Heat) | `config.rs:69-75` (`Transmission`: Items=0 a Heat=4) | CONFIRMADA | Tipos de meios de transmissão configuráveis. Idêntico. |
| 72 | **Side Config: Configurator Modes** | `ConfiguratorMode.java` (Items, Fluids, Chemicals, Energy, Heat, Empty, Rotate, Wrench) | `config.rs:105-114` (Modos 0 a 7) | CONFIRMADA | Modos de ferramenta do configurador com alternância por clique. Idêntico. |
| 73 | **Side Config: RelativeSide** | `RelativeSide.java:17-23` (FRONT=0, LEFT=1, RIGHT=2, BACK=3, TOP=4, BOTTOM=5) | `config.rs:507-512` (`FRONT..BOTTOM = 0..5`) | CONFIRMADA | Lados relativos à face frontal da máquina. Idêntico. |
| 74 | **Direções Cardeais e Verticais** | Minecraft `Direction.java` (DOWN=0, UP=1, NORTH=2, SOUTH=3, WEST=4, EAST=5) | `config.rs:872-877`, `laser.rs:29-34` (`DOWN..EAST = 0..5`) | CONFIRMADA | Mapeamento padrão de faces do mundo 3D. Idêntico. |
| 75 | **Desmontagem com Configurador e Radiação** | `TileEntityMekanism.java:537-540` (`getRadiationScale() > 0 -> WrenchResult.RADIOACTIVE`) | `lib.rs:1056` (`radiation::level_at(...) > 0`) | **FORK (FORK API)** | Novo detalhe: No Java a desmontagem com chave inglesa é bloqueada se o bloco contiver resíduo radioativo em seus tanques; no port é bloqueada se o ambiente tiver radiação. |

---

## 3. Resumo dos Itens Reclassificados como FORK

Em estrito cumprimento às diretrizes do projeto (**não existem limites do Horizon**), os itens anteriormente taxados de `LIMITE` foram remapeados para seus respectivos requisitos de API no fork:

1. **FORK-13 (`RadiationEnvironment`):**
   - Suporte a varredura restrita a raio de chunks (5 chunks / 80 blocos via `MAX_BLOCK_RANGE`), sons dinâmicos de cliques do contador Geiger (`geiger_slow` a `geiger_fast`), e mecanismo de dispersão catastrófica de radiação (`dumpRadiation`) quando blocos ou tubulações contendo resíduos radioativos são destruídos no mundo.
2. **FORK-7 e FORK-8 (`EntityDamagePhysics` / `EnergyDrivenBlockDigging`):**
   - Laser: Dano e queima progressiva de entidades (`2 500 J / 1 000 FE por HP`) e escavação de blocos segundo dureza (`100 000 J / 40 000 FE por unidade de hardness`).
   - SPS: Dano elétrico letal por tick na death zone (`lastReceivedEnergy / 1000F`).
3. **FORK-10 (`CustomEntityEffects`):**
   - Efeito visual de relâmpago sem dano de fogo para transmutação de experimentos válidos no acelerador SPS (ex.: porcos em zombie piglins, creepers carregados).
4. **FORK-19 (`BlockRenderersAndShaders`):**
   - Industrial Alarm: Renderizador de feixe giratório volumétrico de holofote vermelho e loop de reprodução sonora da sirene.
5. **FORK-21 (`BlockRedstoneOutputModes`):**
   - Laser Amplifier: Três modos dedicados de saída de sinal redstone (`OFF`, detecção de entidade interceptando o feixe, e medição comparadora por nível de energia).
6. **FORK-22 (`PumpHeavyWaterExtraction`):**
   - Electric Pump: Capacidade de acoplar o item `Filter Upgrade` para extrair 10 mB de Heavy Water a cada bloco de água bombeado.
7. **FORK-26 (`FluidChemicalRegistry`):**
   - Suporte a substâncias químicas líquidas como blocos de fluido no mundo (`LiquidBlock`), física de escoamento e itens balde associados.

---

## 4. Testes Adicionados no Port

Foram adicionados testes unitários de validação das constantes Java diretamente nos módulos do port:
- `laser.rs`: Teste `laser_constants_match_java_config` validando `RANGE == 64`, `FIRE == 4_000`, `LASER_CAPACITY == 800_000` e `AMPLIFIER_CAPACITY == 2_000_000_000`.
- `sps.rs`: Teste `sps_constants_match_java_config` validando `INPUT_PER_ANTIMATTER == 1_000`, `OUTPUT_CAPACITY == 1_000`, `ENERGY_PER_INPUT == 400_000`, `PORT_ENERGY == 400_000_000` e `input_capacity() == 2_000`.
- `pump.rs`: Teste `max_nodes_matches_java_config` expandido validando `MAX_NODES == 4000`, `USAGE == 40`, `BASE_TICKS == 19`, `TANK == 10_000` e `RANGE == 80`.
- `radiation.rs`: Teste `radiation_constants_match_java_config` validando `BASELINE`, `MIN_MAGNITUDE`, `DECAY`, `MIN_SEVERITY` e as 4 constantes de radioatividade (`nuclear_waste`, `spent_nuclear_waste`, `plutonium`, `polonium`).
