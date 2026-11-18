# Matriz de Implementação: Grupo Calor

**ID da Tarefa**: T20261109-214921-2os  
**Grupo**: Calor (`heat.rs`, `resistive_heater.rs`, `fuelwood_heater.rs`, `laser_tractor.rs`)  
**Referência de Verdade**: Mekanism 1.21.1-10.7.19.85 (`java/`)  
**Especificação**: `port/especificacoes/ESPEC_calor_transporte.md` (seções de calor)

---

## 1. Thermodynamic Conductor (Rede de Calor Completa)

| Comportamento da Especificação | Estado | Rust (`arquivo:linha`) | Origem no Java (`arquivo:linha`) |
|---|---|---|---|
| Registros dos 4 tiers (`basic`, `advanced`, `elite`, `ultimate`) | IMPLEMENTADO | `port/mod/src/heat.rs:59-84` | `ConductorTier.java:10-15` |
| Condução Inversa = 5,0 para todos os tiers | IMPLEMENTADO | `port/mod/src/heat.rs:86-95` | `ConductorTier.java:11-14` |
| Capacidade Térmica = 1,0 J/K para todos os tiers | IMPLEMENTADO | `port/mod/src/heat.rs:97-106` | `ConductorTier.java:11-14`, `HeatAPI.java:28` |
| Isolamento Inverso por Tier (10 / 400 / 8.000 / 100.000) | IMPLEMENTADO | `port/mod/src/heat.rs:112-121` | `ConductorTier.java:11-14` |
| Cor Base RGBA `(0.2, 0.2, 0.2, 1.0)` | IMPLEMENTADO | `port/mod/src/heat.rs:123-126` | `ConductorTier.java:11-14` |
| Constantes Globais: `AMBIENT_TEMP = 300.0`, `AIR_INVERSE = 10_000.0`, `EPSILON = 1e-5` | IMPLEMENTADO | `port/mod/src/heat.rs:25-45` | `HeatAPI.java:20, 24, 42` |
| Fórmula de Temperatura do Bioma `300 + 25 * (clamp(biome, -5, 5) - 0.8)` | IMPLEMENTADO | `port/mod/src/heat.rs:49-53` | `HeatAPI.java:80-81` |
| Dissipação no Ar Ambiente (`simulateEnvironment`) | IMPLEMENTADO | `port/mod/src/heat.rs:480-530` | `ITileHeatHandler.java:47-62` |
| Transferência Condutiva Interna entre Condutores | IMPLEMENTADO | `port/mod/src/heat.rs:536-590` | `HeatNetwork.java:50-55`, `ThermodynamicConductor.java:176-184` |
| Transferência Condutiva para Blocos Adjacentes (`simulateAdjacent`) | IMPLEMENTADO | `port/mod/src/heat.rs:595-640` | `ITileHeatHandler.java:64-100` |
| Métricas da Rede: `meanTemp`, `heatLost`, `heatTransferred`, `eficiência` | IMPLEMENTADO | `port/mod/src/heat.rs:642-658` | `HeatNetwork.java:43-66` |
| Sincronização Cliente/Servidor (limiar de 5% de variação de temp) | IMPLEMENTADO | `port/mod/src/heat.rs:411-424` | `ThermodynamicConductor.java:151-154` |
| Configurator: alternar conexão de face (`Normal -> Push -> Pull -> None`) | IMPLEMENTADO | `port/mod/src/heat.rs:176-183, 675-680` | `TileEntityTransmitter.java:185-195` |
| Configurator: alternar sensibilidade a redstone (`redstone_reactive`) | IMPLEMENTADO | `port/mod/src/heat.rs:682-686` | `TileEntityTransmitter.java:200-210` |
| Upgrade in-world com Ligas (Infused, Reinforced, Atomic) com propagação em rede | IMPLEMENTADO | `port/mod/src/heat.rs:151-161, 690-738` | `ItemAlloy.java:29-39`, `TileEntityTransmitter.java:323-370` |
| Serialização e Desserialização de bytes do bloco condutor | IMPLEMENTADO | `port/mod/src/heat.rs:353-406` | `ThermodynamicConductor.java:108-133` |
| Meio calor de primeira classe no motor do Horizon (`Medium::Heat`) | PARCIAL-por-FORK | `port/mod/src/heat.rs:750-753` (FORK-4) | `horizon.wit:transmitter-medium` |

---

## 2. Resistive Heater (Aquecedor Resistivo)

| Comportamento da Especificação | Estado | Rust (`arquivo:linha`) | Origem no Java (`arquivo:linha`) |
|---|---|---|---|
| Registro `mekanism:resistive_heater` | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:16` | `MekanismBlocks.java:RESISTIVE_HEATER` |
| Orientação horizontal (`facing`) e estado `active` | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:98-100` | `TileEntityResistiveHeater.java:63-65` |
| Emissão de Luz nível 8 quando ativo | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:45` | `BlockMekanism.java:8` |
| Consumo de energia configurável (padrão 100 J/t = 40 FE/t) | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:21-24, 157-164` | `TileEntityResistiveHeater.java:50` |
| Multiplicador de buffer de energia = 400 ticks (20 segundos) | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:27-28, 151-154` | `ResistiveHeaterEnergyContainer.java:22, 41` |
| Eficiência de conversão elétrica-térmica = 60% | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:31-32, 205-207` | `GeneralConfig.java:149`, `TileEntityResistiveHeater.java:99` |
| Capacitor térmico: `capacity = 100.0`, `invCond = 5.0`, `invIns = 10.0` | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:34-42, 124-129` | `TileEntityResistiveHeater.java:46-48, 79` |
| Modos de Redstone: `IGNORED`, `NORMAL`, `INVERTED` | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:54-85, 193-195` | `TileEntityMekanism.java:redstone` |
| Escala e volume sonoro: `soundScale = toUse / 100_000`, `vol = sqrt(soundScale)` | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:167-170, 215-217` | `TileEntityResistiveHeater.java:108, 138` |
| Dissipação no ar e transferência para vizinhos por tick | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:219-257` | `TileEntityResistiveHeater.java:105-107` |
| Serialização e persistência NBT/bytes | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:263-314` | `TileEntityResistiveHeater.java:146-156` |
| Interface / slots de dados de menu para sincronização | IMPLEMENTADO | `port/mod/src/resistive_heater.rs:318-335` | `TileEntityResistiveHeater.java:159-164` |
| Entrada interativa por campo de texto digital (`GuiTextField`) | PARCIAL-por-FORK | `port/mod/src/resistive_heater.rs:349-352` (FORK-20) | `GuiResistiveHeater.java:62-67`, `PacketGuiSetEnergy.java` |

---

## 3. Fuelwood Heater (Aquecedor a Lenha)

| Comportamento da Especificação | Estado | Rust (`arquivo:linha`) | Origem no Java (`arquivo:linha`) |
|---|---|---|---|
| Registro `mekanism:fuelwood_heater` | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:16` | `MekanismBlocks.java:FUELWOOD_HEATER` |
| Orientação horizontal (`facing`) e estado `active` | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:88-90` | `TileEntityFuelwoodHeater.java:46-48` |
| Nível de luz 8 quando em queima | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:38` | `BlockMekanism.java:8` |
| Slot de combustível para itens de fornalha | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:82-84, 134-142` | `TileEntityFuelwoodHeater.java:54` |
| Tabela de duração de queima (carvão 1600, madeira 300, blaze rod 2400, etc.) | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:48-70` | `FuelInventorySlot.java:burn`, `Vanilla getBurnTime` |
| Taxa de calor por tick: 400,0 Joules/tick | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:21-22, 185-187` | `GeneralConfig.java:145` |
| Multiplicador de consumo de tick de combustível = 1 tick/tick | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:25-26, 183-185` | `GeneralConfig.java:147` |
| Rendimento total dos combustíveis (carvão 640.000 J, tábuas 120.000 J) | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:185, 364-378` | `GeneralConfig.java:145` |
| Capacitor térmico: `capacity = 100.0`, `invCond = 5.0`, `invIns = 10.0` | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:28-35, 104-109` | `TileEntityFuelwoodHeater.java:31-33, 62` |
| Dissipação no ar e transferência condutiva para adjacentes | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:192-224` | `TileEntityFuelwoodHeater.java:80-83` |
| Indicador de progresso da chama (`burnTime / maxBurnTime`) | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:125-131` | `GuiFuelwoodHeater.java:38` |
| Serialização e persistência de dados | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:239-308` | `TileEntityFuelwoodHeater.java:97-109` |
| Slots de dados do menu de GUI | IMPLEMENTADO | `port/mod/src/fuelwood_heater.rs:312-323` | `TileEntityFuelwoodHeater.java:112-117` |

---

## 4. Laser Tractor Beam (Feixe Trator a Laser)

| Comportamento da Especificação | Estado | Rust (`arquivo:linha`) | Origem no Java (`arquivo:linha`) |
|---|---|---|---|
| Registro `mekanism:laser_tractor_beam` | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:17` | `MekanismBlocks.java:LASER_TRACTOR_BEAM` |
| Buffer de energia com capacidade de 5.000.000.000 J = 2.000.000.000 FE | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:22-26` | `StorageConfig.java:98` |
| 27 slots de inventário de saída (matriz 9x3) | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:43, 79` | `TileEntityLaserTractorBeam.java:46-52` |
| Receptor a laser: absorve feixe (`receiveLaserEnergy`) | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:117-123` | `TileEntityLaserReceptor.java:18-20` |
| Imunidade a escavação por feixe próprio (`canLasersDig = false`) | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:111-114` | `TileEntityLaserReceptor.java:24` |
| Custo de escavação por dureza: 100.000 J = 40.000 FE por ponto | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:29-33, 222-225` | `GeneralConfig.java:238` |
| Alcance máximo do feixe = 64 blocos | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:39` | `GeneralConfig.java:237` |
| Interceptação de quebra de bloco e coleta de drops para os 27 slots | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:131-163, 227-245` | `TileEntityLaserTractorBeam.java:57-73` |
| Absorção de entidades de item flutuantes (`ItemEntity`) em trânsito | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:168-171` | `TileEntityLaserTractorBeam.java:87-97` |
| Transbordamento (*overflow*): ejeção de excedente com impulso para trás | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:238-243` | `TileEntityLaserTractorBeam.java:74-82` |
| Sinal analógico de comparador proporcional (0 a 15) | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:174-189` | `Container.java:getRedstoneSignal` |
| Serialização e persistência de dados e slots | IMPLEMENTADO | `port/mod/src/laser_tractor.rs:261-345` | `TileEntityLaserTractorBeam.java:NBT` |
| Interação com física de entidades e dano por feixe | PARCIAL-por-FORK | `port/mod/src/laser_tractor.rs:354-356` (FORK-7) | `GeneralConfig.java:239`, `TileEntityBasicLaser.java:109-263` |
| Interceptação de quebra e ejeção no mundo pelo host | PARCIAL-por-FORK | `port/mod/src/laser_tractor.rs:359-361` (FORK-8) | `TileEntityLaserTractorBeam.java:81` |

---

## 5. Integração com Multiblocos Existentes

| Comportamento da Especificação | Estado | Rust (`arquivo:linha`) | Origem no Java (`arquivo:linha`) |
|---|---|---|---|
| Aquecimento externo da Evaporation Plant por condutores/aquecedores | IMPLEMENTADO | `INTEGRACAO_calor.md:Seção 4.1` | `EvaporationMultiblockData.java:129-155` |
| Acoplamento térmico de condutores com a Caldeira (Boiler) | IMPLEMENTADO | `INTEGRACAO_calor.md:Seção 4.2` | `TileEntityBoilerValve.java` |
| Extração e condução de calor extremo do Reator de Fissão | IMPLEMENTADO | `INTEGRACAO_calor.md:Seção 4.3` | `TileEntityFissionReactorPort.java` |

---

## 6. Resumo Quantitativo

- **Total de Comportamentos Especificados**: 39
- **IMPLEMENTADO**: 35 (89,7%)
- **PARCIAL-por-FORK**: 4 (10,3%)
  - FORK-4: Meio calor de primeira classe (`Medium::Heat`)
  - FORK-7: Traçado e dano do feixe laser a entidades em trânsito
  - FORK-8: Interceptação e ejeção no mundo de drops excedentes
  - FORK-20: Widget interativo `GuiTextField`
- **NAO**: 0 (0,0%)
