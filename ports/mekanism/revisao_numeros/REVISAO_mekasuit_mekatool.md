# Revisão do MekaSuit e do Meka-Tool contra o Java / MekaSuit and Meka-Tool reviewed against Java

> Módulos: `modules.rs`, `mekasuit.rs`, `meka_tool.rs` (rodada 3, tarefas e6 e e7). Verificados com `tools/verificador/check.sh` (410 testes passam) e contra o Java 1.21.1-10.7.19.85.

## Conferido e igual ao Java / Checked and equal
- **Todos os valores de `GearConfig`** do Meka-Tool e do MekaSuit (29 de energia: capacidade 16 000 000 J, carga 100 000 J, uso base 10 J / silk 100 / arma 2 000 / enxada, pá, machado, tosquia 10 / teleporte 1 000; MekaSuit: carga de inventário 10 000, solar 500, dano 100 000, magia 1 000, queda 50, salto 1 000, elytra 32 000, poção 40 000, sprint 100, gravitacional 1 000, visão 500, hidrostático 500, nutricional 20 000, atração 250) + dano base 4, velocidade de ataque −2,4, eficiência base 4, mineração estendida ligada, alcance de teleporte 100, `miningCount` 128.
- **Tamanho máximo de pilha dos 31 módulos** do Mekanism (`MekanismModules`) e as flags de modo (9 módulos) e de HUD (10).
- **Módulo de energia:** `base × 2^instalados` (`ModuleEnergyUnit.getEnergyValue`); 8 módulos = ×256.
- **Tabelas de modos:** ataque 0/4/8/16/24/32, escavação 0/4/16/32/64/128, explosão 0–4, vein mining 0/2/4/6/8, fazenda 0/1/3/5/7.
- **Atributos das 4 peças:** defesa Netherite (3, 8, 6, 3), dureza 3, resistência a empurrão 0,1, absorção (0,15 / 0,40 / 0,30 / 0,15), dissipação e refração de laser por peça, blindagem de radiação.
- **Capacidades:** jetpack 24 000 mB × módulos instalados, pasta nutritiva 128 000 mB, taxas 256.
- **Dano do Meka-Tool:** `2 000 J × (dano/4)` e o dano parcial com pouca energia (`divideToLevel`).

## Erros encontrados e corrigidos / Bugs found and fixed
| # | Arquivo | Problema | Java |
|---|---|---|---|
| 1 | `modules.rs` | O módulo **Elytra** nascia desligado. No Java ele é ligado; só a **tecla de modo** vem desligada (`modeChangeDisabledByDefault`). Só a visão noturna é `disabledByDefault` | `MekanismModules:139,181`, `ModuleData:517` |
| 2 | `meka_tool.rs` | Custo de escavação multiplicava em FE e forçava mínimo de 1 FE. Com a escavação desligada (eficiência 0) o Java cobra **0**. Agora o cálculo é em Joules (`long × float`) e só depois vira FE | `ItemMekaTool:147-156,335-343` |
| 3 | `meka_tool.rs` | Vein mining: o limite era 128 no total; no Java é `iniciais + 128 × tipos rastreados`. A ordem de varredura dos vizinhos era x→y→z invertida (o `BlockPos.betweenClosed` varia x mais rápido, z mais lento). Reescrito com fila FIFO, bloco contado ao sair da fila e `TraversalDistance` | `ModuleVeinMiningUnit:136-177,227-249` |
| 4 | `mekasuit.rs` | Absorção de dano: percorria capacete→botas; o Java percorre **botas, calças, peitoral, capacete** (`Player.getArmorSlots`). A fase dos módulos pulava peças sem energia, mas o módulo geotérmico custa 0 e absorve mesmo com a calça descarregada. O sonic boom é absorvido a 0,75 | `ItemMekaSuitArmor:408-477`, `MekanismDataMapsProvider` |
| 5 | `mekasuit.rs` | **Queda:** cobrava 20 FE fixos por queda. No Java o custo é `ceil(50 J × dano de queda × razão)` (por ponto de dano: 15 de dano = 300 FE), com absorção parcial se faltar energia | `CommonPlayerTickHandler:237-250` |
| 6 | `meka_tool.rs`, `robit.rs`, `mekasuit.rs` | (compilação) 4 `const fn` com `.min()`, campo inexistente, 7 empréstimos em testes, um `i32` estourando | — |

Testes novos citando o Java: `java_vein_limit_grows_with_the_tracked_block_types`, `java_vein_neighbours_go_x_then_y_then_z`, `java_excavation_unit_off_costs_nothing`, `java_absorption_goes_feet_legs_chest_head`, `java_geothermal_unit_absorbs_fire_even_without_energy`, `java_sonic_boom_is_absorbed_at_three_quarters`, e o de queda reescrito.

## Ainda não conferido / Not yet checked
Esta revisão cobriu **constantes, tabelas e as fórmulas de energia e absorção**. **Não** li contra o Java: jetpack (empuxo, hover, consumo de hidrogênio), módulo gravitacional, injeção nutritiva (conversão de pasta em comida), respiração eletrolítica, locomotive boosting, repulsor hidrostático, elytra, distribuição de carga, atração magnética, teletransporte (alcance e custo por distância), explosão (`IBlastingItem`), fazenda, tosquia, o HUD, e o `Robit`, `Electric Bow` e `HDPE Elytra`. As chamadas ao host (partículas, raio de busca, quebra de blocos, rádio) estão como FORK (`fork_equipamento.rs`).
