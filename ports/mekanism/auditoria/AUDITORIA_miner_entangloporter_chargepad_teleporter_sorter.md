# Auditoria: Digital Miner, Quantum Entangloporter, Chargepad, Teleporter e Logistical Sorter

**Versão de referência**: Mekanism 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`)
**Conversão de energia**: Joules para Forge Energy (FE = J × 2 / 5 via `crate::fe()`)

---

## 1. Tabela Comparativa de Auditoria

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| **Digital Miner - Raio padrão** | `10` (`TileEntityDigitalMiner.java:125`) | `10` (`miner.rs:13`) | OK |
| **Digital Miner - Raio máximo** | `32` (`GeneralConfig.java:227`) | `32` (`miner.rs:15`) | OK |
| **Digital Miner - Raio mínimo** | `0` (`TileEntityDigitalMiner.java:393, 401, 1399`) | Era `1` (`miner.rs:69, 81`); corrigido para `0` (`miner.rs:14, 73, 88`) | CORRIGIDO |
| **Digital Miner - Altura padrão (min/max Y)** | `minY = 0, maxY = 60` (`DEFAULT_HEIGHT_RANGE = 60`) (`TileEntityDigitalMiner.java:124, 143, 144`) | `DEFAULT_MIN_Y = 0, DEFAULT_MAX_Y = 60` (`miner.rs:20, 21`) | OK |
| **Digital Miner - Limites de altura Y** | `-64` a `319` (`TileEntityDigitalMiner.java:411, 429`) | `-64` a `319` (`miner.rs:90, 93`) | OK |
| **Digital Miner - Delay de mineração** | `80` ticks (`GeneralConfig.java:229`, `TileEntityDigitalMiner.java:156`) | `80` ticks (`TICKS_PER_MINE = 80`) (`miner.rs:16`) | OK |
| **Digital Miner - Consumo base de energia** | `1_000L` J/t (`UsageConfig.java:58`) | `fe(1_000) = 400` FE/t (`USAGE`) (`miner.rs:17`) | OK |
| **Digital Miner - Capacidade de energia** | `50_000L` J (`StorageConfig.java:65`) | `fe(50_000) = 20_000` FE (`CAPACITY`) (`miner.rs:19`, `lib.rs:304`) | OK |
| **Digital Miner - Fórmula de energia / Silk multiplier** | Base 1 000 J/t; Silk ×12 (`GeneralConfig.java:225`); raio > 10 e altura > 60 escalam até 4× total (`MinerEnergyContainer.java:49-74`) | Era flat 400 FE/t; corrigido com `SILK_MULTIPLIER = 12` e `energy_usage(&Settings)` (`miner.rs:18, 134-149, 223`) | CORRIGIDO |
| **Digital Miner - Slots e filtros** | 27 slots buffer + 1 energia (`TileEntityDigitalMiner.java:202-209`); filtros virtuais ilimitados | 27 buffer + 5 filtros em slots físicos (total 32 slots) (`miner.rs:23-25`, `lib.rs:1949`) | LIMITE |
| **Digital Miner - Varredura e visual** | Thread assíncrona `ThreadMinerSearch`; substituição de blocos minerados; caixa visual | 512 blocos/tick síncronos na tick thread (`miner.rs:26-27`); sem substituição/visual | LIMITE |
| **Quantum Entangloporter - Capacidade de energia** | `256_000_000L` J (`GeneralConfig.java:261`, `EnergyCubeTier.java:18`) | Era `256_000_000` (J direto em FE); corrigido para `fe(256_000_000) = 102_400_000` FE (`entangloporter.rs:14`) | CORRIGIDO |
| **Quantum Entangloporter - Capacidade de fluido** | `256_000` mB (`GeneralConfig.java:264`, `FluidTankTier.java:11`) | `256_000` mB (`FLUID_CAPACITY`) (`entangloporter.rs:15`) | OK |
| **Quantum Entangloporter - Capacidade de químico/gás** | `8_192_000` mB (`GeneralConfig.java:267`, `ChemicalTankTier.java:19`) | `8_192_000` mB (`CHEMICAL_CAPACITY`) (`entangloporter.rs:16`) | OK |
| **Quantum Entangloporter - Slots de itens** | 1 slot (`InventoryFrequency.java:166`) | 1 stack (`entangloporter.rs:56`, `lib.rs:1924`) | OK |
| **Quantum Entangloporter - Taxas de transferência** | Sem limite artificial (até capacidade do buffer por tick se receptor aceitar) (`InventoryFrequency.java:279-305`) | Sem limite artificial (até capacidade do buffer por tick) (`lib.rs:1920`, `entangloporter.rs:170`) | OK |
| **Quantum Entangloporter - Calor** | Buffer de calor (`InventoryFrequency.java:168`) | Não implementado (rede térmica ausente no port) | LIMITE |
| **Quantum Entangloporter - Frequências** | Frequências nomeadas públicas/privadas com UUID (`InventoryFrequency.java:158`) | Frequências numéricas públicas 1 a 99 (`entangloporter.rs:18, 38`) | LIMITE |
| **Chargepad - Bloco e TileEntity** | `BlockChargepad.java`, `TileEntityChargepad.java`, `MekanismBlocks.CHARGEPAD` | Não existe no port (`port/mod/src/`) | FALTA |
| **Chargepad - Taxa de carga (consumo)** | `1_024_000L` J/t (`UsageConfig.java:60`) = 409.600 FE/t | Não existe no port | FALTA |
| **Chargepad - Capacidade de armazenamento** | `2_048_000L` J (`StorageConfig.java:69`) = 819.200 FE | Não existe no port | FALTA |
| **Chargepad - Conexões e comportamento** | Traseira e base (`TileEntityChargepad.java:49`); carrega `Player` (1 item/t) e `EntityRobit`; som de placa de pressão | Não existe no port | FALTA |
| **Teleporter - Custo base de energia** | `1_000L` J (`UsageConfig.java:87`) | `1_000` J = `fe(1_000) = 400` FE (`BASE_COST`) (`teleporter.rs:14, 64`) | OK |
| **Teleporter - Custo por distância** | `10L` J/bloco (`UsageConfig.java:88`) | `10.0` J/bloco = `fe(10) = 4` FE/bloco (`DISTANCE_COST`) (`teleporter.rs:15, 64`) | OK |
| **Teleporter - Penalidade interdimensional** | `10_000L` J (`UsageConfig.java:89`) | `10_000` J = `fe(10_000) = 4_000` FE (`DIMENSION_PENALTY`) (`teleporter.rs:16, 64`) | OK |
| **Teleporter - Capacidade de energia** | `5_000_000L` J (`StorageConfig.java:102`) | `fe(5_000_000) = 2_000_000` FE (`CAPACITY`) (`teleporter.rs:17`, `lib.rs:1928`) | OK |
| **Teleporter - Cooldown / Delay após chegada** | `5` ticks (`teleDelay = 5`) (`TileEntityTeleporter.java:344`) | `5` ticks (`DELAY = 5`) (`teleporter.rs:20`) | OK |
| **Teleporter - Tempo de teletransporte** | Instantâneo (0 ticks de espera antes do disparo) (`TileEntityTeleporter.java:327`) | Instantâneo (0 ticks de espera antes do disparo) (`teleporter.rs:216`) | OK |
| **Teleporter - Moldura (Frame)** | 9 blocos de moldura ao redor de abertura 1x2 (`TileEntityTeleporter.java:540-560`) | 9 blocos em anel 3x4 (`teleporter.rs:30-58`) | OK |
| **Teleporter - Escala dimensional (Nether)** | 8.0 para Nether, 1.0 para Overworld/End (`TileEntityTeleporter.java:485-486`) | 8.0 para Nether, 1.0 para outros (`teleporter.rs:65-67`) | OK |
| **Teleporter - Posição de destino** | Abaixo: Y-2; Demais faces: 1 bloco à frente (`TileEntityTeleporter.java:300-307`) | Abaixo: Y-2; Demais faces: 1 bloco à frente (`teleporter.rs:103-110`) | OK |
| **Teleporter - Entidades suportadas** | Todas as entidades e passageiros (`TileEntityTeleporter.java:283, 452-460`) com custo multiplicado | Apenas jogadores (`Player`) são teleportados (`teleporter.rs:205`) | LIMITE |
| **Logistical Sorter - Delay / Frequência de puxada** | `10` ticks (`TICKS_PER_HALF_SECOND = 10`) (`TileEntityLogisticalSorter.java:153`) | `10` ticks (`DELAY = 10`) (`sorter.rs:22`) | OK |
| **Logistical Sorter - Quantidade puxada por ciclo** | 1 item (single) ou até 64 (`ABSOLUTE_MAX_STACK_SIZE`) (`TileEntityLogisticalSorter.java:144`) | 1 item (single) ou 64 (`sorter.rs:100`) | OK |
| **Logistical Sorter - Consumo de energia** | Sem energia (`TileEntityLogisticalSorter.java`) | Sem energia (`sorter.rs`) | OK |
| **Logistical Sorter - Slots e filtros** | Filtros virtuais ilimitados; 1 slot visual interno na frente (`TileEntityLogisticalSorter.java:98`) | 2 slots de trânsito (intake/outbox) + 5 slots de filtro (total 7) (`sorter.rs:18-20`) | LIMITE |
| **Logistical Sorter - Recursos avançados** | Round-robin, cor padrão, cor por filtro, min/max count, redstone | Auto-eject toggle, single item toggle, 5 filtros de item (`sorter.rs:24-27`) | LIMITE |

---

## 2. Detalhes das Correções Aplicadas

### 2.1 `port/mod/src/entangloporter.rs`
- **Energia em FE**: `ENERGY_CAPACITY` foi corrigido de `256_000_000` para `crate::fe(256_000_000)` (102.400.000 FE).
- **Consistência de registro**: Em `lib.rs:1920`, `EnergySpec` do Entangloporter consome diretamente `entangloporter::ENERGY_CAPACITY`, garantindo capacidade, recebimento e extração máxima em 102.400.000 FE.

### 2.2 `port/mod/src/miner.rs`
- **Raio Mínimo Paritário**: Adicionada constante `pub const MIN_RADIUS: u8 = 0;`. Atualizado `decode()` para `.clamp(MIN_RADIUS, MAX_RADIUS)` e `press(RADIUS_DOWN)` para `.max(MIN_RADIUS)`. No Java, o raio 0 é permitido e representa uma coluna 1×1 `(0*2 + 1)`.
- **Cálculo de Consumo Dinâmico**: Adicionada constante `pub const SILK_MULTIPLIER: u64 = 12;` e função `pub fn energy_usage(s: &Settings) -> u64`, replicando fielmente o cálculo de `MinerEnergyContainer.java`:
  - Base: 1.000 J/t (400 FE/t)
  - Silk Touch: multiplicador 12× (`12_000 J/t = 4_800 FE/t`)
  - Escala de raio: `(radius - 10) / 22.0` se `radius > 10`
  - Escala de altura: `(height - 60) / 323.0` se `height > 60`
  - Total em Joules arredondado para cima (`ceil`) e convertido via `crate::fe()`.
- **Execução na Mineração**: Atualizado `run_miners()` para descontar `usage = energy_usage(&s)` em vez do valor estático `USAGE`.
- **Testes**: Atualizado o teste `the_buttons_keep_the_area_sane` para verificar redução até `MIN_RADIUS` e adicionado o teste `energy_usage_scales_with_silk_radius_and_height`.

---

## 3. Especificação Completa do Chargepad (FALTA)

Como o Chargepad não existe no port, aqui estão as especificações exatas definidas no Mekanism Java 1.21.1 para futura implementação:
- **Identificadores**: `mekanism:chargepad`, classe `TileEntityChargepad` e bloco `BlockChargepad`.
- **Consumo / Taxa de Operação**: `1_024_000L` J/t (`UsageConfig.java:60`), correspondendo a `409_600` FE/t.
- **Capacidade de Armazenamento**: `2_048_000L` J (`StorageConfig.java:69`), correspondendo a `819_200` FE.
- **Lados de Conexão**: Aceita energia pela face traseira (`RelativeSide.BACK`) e inferior (`RelativeSide.BOTTOM`).
- **Detecção de Entidades**: Bounding box `AABB(x, y, z, x + 1.0, y + 0.4, z + 1.0)`. Filtra entidades não-espectadoras do tipo `Player` ou `EntityRobit`.
- **Comportamento de Carga**:
  - Para `Player`: insere até `1_024_000` J/t em 1 item recarregável por tick do inventário do jogador (e Curios, se presente).
  - Para `EntityRobit`: insere até `1_024_000` J/t diretamente no robô.
- **Áudio / Visual**:
  - Emite som de placa de pressão: `SoundEvents.STONE_PRESSURE_PLATE_CLICK_ON` (pitch 0.8) ao ativar e `STONE_PRESSURE_PLATE_CLICK_OFF` (pitch 0.7) ao desativar.
  - Partículas de poeira de redstone (`DustParticleOptions.REDSTONE`) na base.
- **Sem Upgrades e Sem GUI**: O bloco não possui tela própria nem suporte a upgrades.
