# Revisão dos módulos de calor contra o Java / Heat modules reviewed against Java

> Módulos: `heat.rs`, `resistive_heater.rs`, `fuelwood_heater.rs`, `laser_tractor.rs` (rodada 3, tarefa e3). Verificados com `tools/verificador/check.sh` (compilação + testes) e contra o Java 1.21.1-10.7.19.85.

## Conferido e igual ao Java / Checked and equal
`HeatAPI` (ambient 300, ar 10 000, capacidade 1, condução 1, isolamento 0, EPSILON), `ConductorTier` (condução 5, capacidade 1, isolamento 10 / 400 / 8 000 / 100 000), Fuelwood Heater (capacidade 100, condução 5, isolamento 10, `heatPerFuelTick` 400, multiplicador 1), Resistive Heater (uso base 100 J, buffer = uso × 400, eficiência 0,6, calor = Joules consumidos × eficiência: a conversão FE→J está certa), Laser Tractor Beam (buffer 5 000 000 000 J, 27 slots de saída 9×3). Ordem de atualização do calor (depois do tick da máquina) igual.

## Erros encontrados e corrigidos / Bugs found and fixed
| # | Arquivo | Problema | Java |
|---|---|---|---|
| 1 | `heat.rs` `HeatNetwork::tick` | Algoritmo **inventado**: perda para o ar pulava faces com vizinho; troca entre condutores usava (T1−T2)/(inv1+inv2)×min(cap). O Java: cada condutor trata o vizinho como "sink" e usa (T − **ambiente**)/(inv do sink + coef. próprio)×capacidade; perda para o ar nas 6 faces sempre; troca entre condutores da mesma rede não conta como transferência adjacente | `ITileHeatHandler.simulateAdjacent/Environment`, `HeatNetwork.onUpdate`, `ThermodynamicConductor.incrementAdjacentTransfer` |
| 2 | `heat.rs` `HeatCapacitor::update` | Zerava o calor pendente ≤ EPSILON e travava o calor em 0 | `BasicHeatCapacitor.update` (acumula até passar de EPSILON; sem limite) |
| 3 | `fuelwood_heater.rs` | Queimava o **dobro**: faltava `getBurnTime / 2`; item que dá 0 não é consumido; balde de lava em pilha > 1 não queima | `FuelInventorySlot.burn` |
| 4 | `fuelwood_heater.rs` | Tabela de combustíveis aceitava madeira do Nether e lajes de pedra; faltavam os combustíveis do Mekanism (`charcoal_block` 16 000, `bio_fuel` 100, `bio_fuel_block` 1 000) | `MekanismDataMapsProvider:52-57` |
| 5 | `fuelwood_heater.rs`, `resistive_heater.rs` | Perda adjacente só somava valores positivos | `ITileHeatHandler.incrementAdjacentTransfer` |
| 6 | `heat.rs` teste | Teste esperava calor já aplicado no sink; o sink é preguiçoso. Corrigido com a conta do Java (= 20) | `simulateAdjacent` |

Testes novos citando o Java: `java_two_conductors_use_ambient_not_the_neighbours_temperature`, `java_capacitor_keeps_small_heat_until_it_passes_epsilon`, `java_burn_time_is_half_and_mekanism_fuels_are_known`.

## Ainda diferente (pedido de fork) / Still different (fork requests)
FORK-4 (meio calor de primeira classe), FORK-7 (dano/laser), FORK-8 (escavação e drops do feixe trator), FORK-20 (campo de texto para o uso do Resistive Heater), **FORK-32** (tempo de queima exato de qualquer item). O `laser_tractor.rs` implementa os dados e o inventário; o traçado de raio, entidades atingidas, escudos e armaduras dependem desses FORK.
`laser_tractor.rs:271` tem um erro de compilação (`ref` na edição 2024) que a tarefa c3 está corrigindo.
