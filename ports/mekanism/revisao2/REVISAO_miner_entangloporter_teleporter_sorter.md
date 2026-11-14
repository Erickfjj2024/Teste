# Revisão Cruzada: Digital Miner, Quantum Entangloporter, Teleporter e Logistical Sorter

**Escopo da Tarefa**: Revisão cruzada da auditoria anterior (t4).
**Versão de referência Java**: Mekanism 1.21.1-10.7.19.85 (`v1.21.1-10.7.19.85`)
**Conversão de energia**: Joules para Forge Energy ($FE = J \times \frac{2}{5}$ via `crate::fe()`)

---

## 1. Tabela Cruzada de Conferência

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| **Digital Miner - Raio padrão** | `10` (`TileEntityDigitalMiner.java:125`) | `10` (`miner.rs:13`) | OK |
| **Digital Miner - Raio máximo** | `32` (`GeneralConfig.java:227`) | `32` (`miner.rs:15`) | OK |
| **Digital Miner - Raio mínimo** | `0` (`TileEntityDigitalMiner.java:393, 401, 1399`) | `MIN_RADIUS = 0` (`miner.rs:14, 73, 88`) | CONFIRMADA |
| **Digital Miner - Altura padrão (min/max Y)** | `minY = 0, maxY = 60` (`DEFAULT_HEIGHT_RANGE = 60`) (`TileEntityDigitalMiner.java:124, 143, 144`) | `DEFAULT_MIN_Y = 0, DEFAULT_MAX_Y = 60` (`miner.rs:20, 21`) | OK |
| **Digital Miner - Limites de altura Y** | `-64` a `319` (`TileEntityDigitalMiner.java:411, 429`) | `-64` a `319` (`miner.rs:90, 93`) | OK |
| **Digital Miner - Delay de mineração** | `80` ticks (`GeneralConfig.java:229`, `TileEntityDigitalMiner.java:156`) | `80` ticks (`TICKS_PER_MINE = 80`) (`miner.rs:16`) | OK |
| **Digital Miner - Consumo base de energia** | `1_000L` J/t (`UsageConfig.java:58`) | `fe(1_000) = 400` FE/t (`USAGE`) (`miner.rs:17`) | OK |
| **Digital Miner - Capacidade de energia** | `50_000L` J (`StorageConfig.java:65`) | `fe(50_000) = 20_000` FE (`CAPACITY`) (`miner.rs:19`, `lib.rs:304`) | OK |
| **Digital Miner - Fórmula Silk / Raio / Altura** | Silk ×12 (`GeneralConfig.java:225`); raio > 10 escalado por 22.0; altura > 60 escalada por 323.0; arredondamento ceil (`MinerEnergyContainer.java:49-74`) | `SILK_MULTIPLIER = 12`, `energy_usage()` (`miner.rs:18, 134-149, 224`) | CONFIRMADA |
| **Digital Miner - Botão RESET** | Java `reset()` desativa `running = false` (`TileEntityDigitalMiner.java:828`) | Era no-op `RESET => {}`; corrigido para `RESET => self.running = false,` (`miner.rs:97`) | CORRIGIDO |
| **Digital Miner - Ordem da fila de mineração** | Java minera de cima para baixo (`TileEntityDigitalMiner.java:470-472, 933`) para blocos com gravidade não caírem | Era `work.queue.pop()` (LIFO, minerava de baixo para cima); corrigido para `work.queue.remove(0)` (FIFO top-down) (`miner.rs:233-234`) | CORRIGIDO |
| **Digital Miner - Slots e filtros** | 27 buffer + 1 energia (`TileEntityDigitalMiner.java:202-209`, `MekanismBlocks.java:350-353`), filtros virtuais ilimitados | 27 buffer + 5 filtros em slots físicos (total 32 slots) (`miner.rs:23-25`) | FORK (FORK-16) |
| **Digital Miner - Varredura de blocos** | Thread assíncrona `ThreadMinerSearch` (`TileEntityDigitalMiner.java:804, 827`) | 512 blocos/tick síncronos na tick thread (`miner.rs:26-27, 205`) | FORK (FORK-15) |
| **Digital Miner - Substituição e Auto-Pull** | `setReplace` e `pullInventory` (`TileEntityDigitalMiner.java:559-580, 629-634`) | Não implementado (`miner.rs:7`) | FORK |
| **Digital Miner - Estrutura Bounding Block** | 17 blocos de colisão/conexão ao redor do bloco central (3x2x3) (`MekanismBlockTypes.java:334-350`) | Bloco único com exclusão de `bounding_block` e `digital_miner` no filtro (`miner.rs:112`) | FORK |
| **Digital Miner - Upgrades suportados** | Speed, Energy, Anchor, Stone Generator (`MekanismBlockTypes.java:331`, `TileEntityDigitalMiner.java:1116-1120`) | Não implementado (`miner.rs:7`) | FORK |
| **Quantum Entangloporter - Capacidade de energia** | `256_000_000L` J (`GeneralConfig.java:261`, `EnergyCubeTier.java:16`) | `fe(256_000_000) = 102_400_000` FE (`entangloporter.rs:14`) | CONFIRMADA |
| **Quantum Entangloporter - Capacidade de fluido** | `256_000` mB (`GeneralConfig.java:264`, `FluidTankTier.java:12`) | `256_000` mB (`FLUID_CAPACITY`) (`entangloporter.rs:15`) | OK |
| **Quantum Entangloporter - Capacidade de químico** | `8_192_000` mB (`GeneralConfig.java:267`, `ChemicalTankTier.java:15`) | `8_192_000` mB (`CHEMICAL_CAPACITY`) (`entangloporter.rs:16`) | OK |
| **Quantum Entangloporter - Slots de itens** | 1 slot (`InventoryFrequency.java:166`) | 1 stack (`entangloporter.rs:59`, `lib.rs:1924`) | OK |
| **Quantum Entangloporter - Taxas de transferência** | Sem limite artificial por tick (`InventoryFrequency.java:279-305`) | Sem limite artificial (`lib.rs:1920`, `entangloporter.rs:170`) | OK |
| **Quantum Entangloporter - Frequências** | Frequências nomeadas públicas/privadas com UUID (`InventoryFrequency.java:153-161`) | Frequências numéricas públicas 1 a 99 (`entangloporter.rs:18, 45-51`) | FORK (FORK-18) |
| **Quantum Entangloporter - Transporte térmico (calor)** | Buffer de calor compartilhado (`InventoryFrequency.java:168-170`) | Não implementado (meio térmico ausente no Horizon) | FORK (FORK-4/FORK-6) |
| **Quantum Entangloporter - Configuração de lados** | Side config avançado por face e tipo de transmissão (`ItemBlockQuantumEntangloporter.java:28-36`) | Não suportado na GUI do Horizon | FORK (FORK-20) |
| **Teleporter - Custo base de energia** | `1_000L` J (`UsageConfig.java:87`) | `1_000` J = `fe(1_000) = 400` FE (`BASE_COST`) (`teleporter.rs:14, 64`) | OK |
| **Teleporter - Custo por distância** | `10L` J/bloco (`UsageConfig.java:88`) | `10.0` J/bloco = `fe(10) = 4` FE/bloco (`DISTANCE_COST`) (`teleporter.rs:15, 64`) | OK |
| **Teleporter - Penalidade dimensional** | `10_000L` J (`UsageConfig.java:89`) | `10_000` J = `fe(10_000) = 4_000` FE (`DIMENSION_PENALTY`) (`teleporter.rs:16, 64`) | OK |
| **Teleporter - Capacidade de energia** | `5_000_000L` J (`StorageConfig.java:102`) | `fe(5_000_000) = 2_000_000` FE (`CAPACITY`) (`teleporter.rs:17`, `lib.rs:1928`) | OK |
| **Teleporter - Delay de chegada** | `5` ticks (`TileEntityTeleporter.java:344`) | `5` ticks (`DELAY = 5`) (`teleporter.rs:20, 218`) | OK |
| **Teleporter - Disparo do teletransporte** | Instantâneo (0 ticks de espera antes do disparo) (`TileEntityTeleporter.java:327`) | Instantâneo (`teleporter.rs:214`) | OK |
| **Teleporter - Moldura (Frame)** | 9 blocos de moldura ao redor de abertura 1x2 (`TileEntityTeleporter.java:540-598`) | 9 blocos em anel 3x4 (`teleporter.rs:29-59`) | OK |
| **Teleporter - Escala dimensional (Nether)** | 8.0 para Nether, 1.0 para Overworld/End (`TileEntityTeleporter.java:485-486`) | 8.0 para Nether, 1.0 para outros (`teleporter.rs:69`) | OK |
| **Teleporter - Posição de destino** | Abaixo: Y-2; Demais faces: 1 bloco à frente (`TileEntityTeleporter.java:300-307`) | Abaixo: Y-2; Demais faces: 1 bloco à frente (`teleporter.rs:103-110`) | OK |
| **Teleporter - Entidades suportadas** | Mobs, veículos, passageiros com multiplicador de custo (`TileEntityTeleporter.java:283, 452-460, 508`) | Apenas jogadores (`Player`) são teleportados (`teleporter.rs:175-205`) | FORK (FORK-9) |
| **Teleporter - Efeitos audiovisuais** | Som de portal e partículas `PacketPortalFX` (`TileEntityTeleporter.java:321`) | Não implementado | FORK (FORK-19) |
| **Logistical Sorter - Delay / Frequência de puxada** | `10` ticks (`TICKS_PER_HALF_SECOND = 10`) (`TileEntityLogisticalSorter.java:153`) | `10` ticks (`DELAY = 10`) (`sorter.rs:22, 106`) | OK |
| **Logistical Sorter - Quantidade puxada** | 1 item (single) ou até 64 (`ABSOLUTE_MAX_STACK_SIZE`) (`TileEntityLogisticalSorter.java:144`) | 1 item (single) ou 64 (`sorter.rs:109`) | OK |
| **Logistical Sorter - Consumo de energia** | Sem energia (`TileEntityLogisticalSorter.java`) | Sem energia (`sorter.rs`) | OK |
| **Logistical Sorter - Faces de transporte** | Puxa de trás (intake), ejeta na frente (outbox) (`BlockLogisticalSorter.java:40-49`, `TileEntityLogisticalSorter.java:128, 158`) | Face traseira intake, face frontal outbox (`sorter.rs:50-55`) | OK |
| **Logistical Sorter - Slots e filtros** | Filtros virtuais ilimitados; 1 slot visual frontal (`TileEntityLogisticalSorter.java:63, 98`) | 2 slots de trânsito (intake/outbox) + 5 slots de filtro físico no container (`sorter.rs:17-20`) | FORK (FORK-16) |
| **Logistical Sorter - Recursos avançados** | Round-robin, cor padrão, cor por filtro, min/max count, redstone (`TileEntityLogisticalSorter.java:79-82, 132-133, 162`) | Auto-eject toggle, single item toggle, 5 filtros de item simples (`sorter.rs:24-27`) | FORK (FORK-17) |

---

## 2. Análise Detalhada das Correções da Rodada 1

1. **Digital Miner - Raio Mínimo 0**:
   - **CONFIRMADA**. O Java permite configurar raio 0 (`TileEntityDigitalMiner.java:393, 401, 1399`), representando uma coluna 1×1 centrada no minerador. A alteração em `miner.rs:14, 73, 88` está correta.

2. **Digital Miner - Consumo por Silk / Raio / Altura**:
   - **CONFIRMADA**. A fórmula em `MinerEnergyContainer.java:49-74` aplica:
     - Multiplicador de Silk Touch: 12× (`GeneralConfig.java:225`).
     - Custo adicional de raio: $(radius - 10) / 22.0$ para $radius > 10$.
     - Custo adicional de altura: $(height - 60) / 323.0$ para $height > 60$ ($height = maxY - minY$).
     - Multiplicadores compostos $(1 + cost_{raio}) \times (1 + cost_{altura})$, arredondamento superior (`MathUtils.ceilToLong`), e conversão fiel para Forge Energy via `fe()`.
     - Implementação em `miner.rs:18, 134-149, 224` confirmada como fiel.

3. **Quantum Entangloporter - Energia em FE**:
   - **CONFIRMADA**. A capacidade nominal no Java é 256.000.000 Joules (`GeneralConfig.java:261`, `EnergyCubeTier.java:16`). No port, que conta energia em Forge Energy, $256.000.000 \times \frac{2}{5} = 102.400.000\text{ FE}$. A correção em `entangloporter.rs:14` (`crate::fe(256_000_000)`) está correta.

---

## 3. Descobertas e Correções da Rodada 2

1. **Digital Miner - Botão RESET não parava a máquina**:
   - **Problema**: Em `miner.rs:97`, o botão `RESET` executava apenas `RESET => {}`. Embora limpasse a fila em `run_miners`, a flag `s.running` continuava `true`, fazendo a máquina reiniciar a varredura imediatamente em vez de parar.
   - **Java**: No Java, `TileEntityDigitalMiner.java:828` define explicitamente `running = false;` no método `reset()`.
   - **Correção aplicada**: Em `miner.rs:97`, alterado para `RESET => self.running = false,`.

2. **Digital Miner - Inversão da ordem da fila de mineração (LIFO vs FIFO)**:
   - **Problema**: A função `position()` varre a área de cima para baixo (`y = max_y - layer`). Os blocos eram adicionados ao final de `work.queue` com `push()`. No momento de minerar, o código executava `work.queue.pop()`, retirando o último elemento inserido (que ficava no nível inferior de Y), resultando em mineração de baixo para cima (LIFO).
   - **Java**: O Java (`TileEntityDigitalMiner.java:463-470`) minera estritamente do topo para baixo para evitar que areia, cascalho e blocos com gravidade caiam e quebrem ou fiquem fora de alcance.
   - **Correção aplicada**: Em `miner.rs:233-234`, alterado de `while let Some(p) = work.queue.pop()` para `while !work.queue.is_empty() { let p = work.queue.remove(0); ... }`, garantindo consumo em fila (FIFO) do topo para a base.

3. **Digital Miner - Ferramenta de escavação**:
   - No Java, `TileEntityDigitalMiner.java:1303` usa `ItemAtomicDisassembler` totalmente carregado com encantamento Silk Touch (quando ativo) em um fake player (`MekFakePlayer`) emulando o UUID do proprietário. No port, passa `"minecraft:netherite_pickaxe"` para `world::mine_block` devido às capacidades disponíveis no Horizon.
