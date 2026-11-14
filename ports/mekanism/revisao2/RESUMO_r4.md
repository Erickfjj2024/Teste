# Resumo da Tarefa T20261109-194705-9xd

## Objetivo
Revisão cruzada (rodada 2) de **DIGITAL MINER**, **QUANTUM ENTANGLOPORTER**, **TELEPORTER** e **LOGISTICAL SORTER** (arquivos `miner.rs`, `entangloporter.rs`, `teleporter.rs`, `sorter.rs`) contra a fonte da verdade em Java (Mekanism 1.21.1-10.7.19.85), conferência das correções da rodada 1 e descoberta de novas discrepâncias, e especificação completa das 10 peças do grupo **Utilidades** (Parte B).

---

## 1. Arquivos Auditados e Modificados

### Arquivos no Escopo Auditados
- Java (Fonte da Verdade - Mekanism 1.21.1-10.7.19.85):
  - `java/main/java/mekanism/common/tile/machine/TileEntityDigitalMiner.java`
  - `java/main/java/mekanism/common/capabilities/energy/MinerEnergyContainer.java`
  - `java/main/java/mekanism/common/tile/TileEntityQuantumEntangloporter.java`
  - `java/main/java/mekanism/common/content/entangloporter/InventoryFrequency.java`
  - `java/main/java/mekanism/common/tile/TileEntityTeleporter.java`
  - `java/main/java/mekanism/common/tile/TileEntityLogisticalSorter.java`
  - `java/main/java/mekanism/common/block/basic/BlockLogisticalSorter.java`
  - `java/main/java/mekanism/common/config/GeneralConfig.java`
  - `java/main/java/mekanism/common/config/UsageConfig.java`
  - `java/main/java/mekanism/common/config/StorageConfig.java`
  - `java/main/java/mekanism/common/tier/EnergyCubeTier.java`
  - `java/main/java/mekanism/common/tier/FluidTankTier.java`
  - `java/main/java/mekanism/common/tier/ChemicalTankTier.java`
  - Peças utilitárias: `TileEntityChargepad.java`, `BlockChargepad.java`, `TileEntityDimensionalStabilizer.java`, `BlockCardboardBox.java`, `TileEntityCardboardBox.java`, `TileEntityPersonalBarrel.java`, `TileEntityPersonalChest.java`, `TileEntityPersonalStorage.java`, `TileEntitySeismicVibrator.java`, `TileEntityOredictionificator.java`, `TileEntityFormulaicAssemblicator.java`, `ItemCraftingFormula.java`, `ItemConfigurationCard.java`.
- Port (Rust WASM Horizon):
  - `port/mod/src/miner.rs` (378 linhas) - **MODIFICADO**
  - `port/mod/src/entangloporter.rs` (222 linhas) - **AUDITADO (CONFIRMADO)**
  - `port/mod/src/teleporter.rs` (306 linhas) - **AUDITADO (CONFIRMADO)**
  - `port/mod/src/sorter.rs` (160 linhas) - **AUDITADO (CONFIRMADO)**
  - `port/DIFERENCAS.md` - **MODIFICADO** (adicionada seção 21.4)

---

## 2. Conferência das Correções da Rodada 1

1. **Digital Miner - Raio Mínimo 0**:
   - **CONFIRMADA**. Java permite raio 0 (`TileEntityDigitalMiner.java:393, 401, 1399`), correspondendo a uma coluna 1×1. Valor e lógica conferidos contra `miner.rs:14, 73, 88`.
2. **Digital Miner - Consumo por Silk / Raio / Altura**:
   - **CONFIRMADA**. Base 1.000 J/t (400 FE/t), multiplicador Silk Touch de 12× (`GeneralConfig.java:225`), custo adicional de raio acima de 10 (`(radius - 10) / 22.0`) e altura acima de 60 (`(height - 60) / 323.0`) com arredondamento `ceil` e conversão em FE via `crate::fe()`. Conferido em `MinerEnergyContainer.java:49-74` e `miner.rs:18, 134-149`.
3. **Quantum Entangloporter - Energia em FE**:
   - **CONFIRMADA**. Java especifica buffer de 256.000.000 Joules (`GeneralConfig.java:261`, `EnergyCubeTier.java:16`). No port, o armazenamento é em Forge Energy ($256.000.000 \times \frac{2}{5} = 102.400.000\text{ FE}$). Conferido em `entangloporter.rs:14`.

---

## 3. Correções Aplicadas na Rodada 2

1. **`port/mod/src/miner.rs` (Ação do botão RESET)**:
   - Em `miner.rs:97`, a recepção do botão `RESET` executava `RESET => {}`. Embora limpasse a fila em `run_miners`, a flag `s.running` continuava `true`, fazendo a máquina reiniciar imediatamente a varredura sem parar.
   - No Java, `TileEntityDigitalMiner.java:828` define expressamente `running = false;` em `reset()`.
   - Corrigido para `RESET => self.running = false,`.

2. **`port/mod/src/miner.rs` (Inversão da ordem de mineração LIFO → FIFO)**:
   - Em `miner.rs:233`, a mineração retirava posições com `work.queue.pop()`. Como a função `position()` varre de cima para baixo (`y = max_y - layer`), os blocos do topo eram inseridos primeiro com `push()` e os do fundo por último; ao usar `pop()`, os blocos do fundo eram minerados primeiro (LIFO).
   - No Java (`TileEntityDigitalMiner.java:463-470`), os blocos são minerados estritamente do topo para baixo para impedir a queda de blocos com gravidade (areia, cascalho) antes de serem minerados.
   - Corrigido para `while !work.queue.is_empty() { let p = work.queue.remove(0); ... }`, garantindo ordem FIFO de cima para baixo.

---

## 4. Testes Auditados e Editados

- **`port/mod/src/miner.rs`**:
  - Teste editado: `the_buttons_keep_the_area_sane`.
  - Adicionada validação de parada na ação de reset:
    ```rust
    assert!(s.press(RESET));
    assert!(!s.running);
    ```
- **Mutações**: A prova de mutação está pendente do juiz (nenhuma lista de mutações foi fornecida na especificação da tarefa).
- **Execução na VM**: não medido (a VM não possui toolchain de compilação rustc/cargo configurada, conforme documentado nas instruções do projeto e em `HANDOFF.md`).

---

## 5. Parte B: Especificação Técnica Completa do Grupo Utilidades

Especificadas no arquivo `.saida/T20261109-194705-9xd/ESPEC_utilidades.md` todas as 10 peças requeridas, contendo nomes de registro, classes Java, blockstates, NBT e data components, números e configs em J e FE, inventários e posições na GUI, receitas do datagen, sons, redstone, interações, capacidades e requisitos do Horizon:
1. **Chargepad** (`mekanism:chargepad`): taxa 1.024.000 J/t = 409.600 FE/t, buffer 2.048.000 J = 819.200 FE, recarga de jogadores (1 item/t) e Robits sobre AABB $[y, y+0.4]$.
2. **Dimensional Stabilizer** (`mekanism:dimensional_stabilizer`): 5.000 J/t = 2.000 FE/t por chunk, buffer 40.000 J = 16.000 FE, raio de 1 a 2 (1 a 25 chunks mantidos carregados com centro fixo).
3. **Cardboard Box** (`mekanism:cardboard_box`): empacotamento completo de `BlockState` e `BlockEntity` em componente `BlockData` com supressão de drops; desempacotamento com shift + clique.
4. **Personal Barrel** (`mekanism:personal_barrel`): 54 slots, modos de segurança (PUBLIC, PRIVATE, TRUSTED), persistência total de itens ao quebrar (`persistsToItem`) e abertura como item portátil na mão.
5. **Personal Chest** (`mekanism:personal_chest`): 54 slots, controlador de tampa `ChestLidController`, sons de baú vanilla, persistência ao quebrar e uso como item portátil.
6. **Seismic Vibrator** (`mekanism:seismic_vibrator`): consumo 50 J/t = 20 FE/t, buffer 20.000 J = 8.000 FE, bounding block 1×2×1 acima, pulso a cada 40 ticks para inspeção geológica via Seismic Reader.
7. **Oredictionificator** (`mekanism:oredictionificator`): sem energia, slot de entrada (56, 115) e saída (164, 115), conversão de tags de minérios/itens a 1 item/t via filtros configuráveis.
8. **Formulaic Assemblicator** (`mekanism:formulaic_assemblicator`): 35 slots (1 fórmula, 9 grade 3×3, 18 entrada, 6 saída, 1 energia), base 40 ticks, 100 J/t = 40 FE/t, buffer 40.000 J = 16.000 FE, modos automático e controle de estoque.
9. **Crafting Formula** (`mekanism:crafting_formula`): item portador da grade 3×3 em `FORMULA_HOLDER`, nomes dinâmicos (Encoded / Invalid) e limpeza com shift + clique.
10. **Configuration Card** (`mekanism:configuration_card`): item para cópia e aplicação de configurações de lados, ejetor e redstone entre máquinas (`IConfigCardAccess`), empilhamento de 1.

---

## 6. Entregáveis Gerados

1. Relatório de revisão cruzada detalhado:
   `.saida/T20261109-194705-9xd/REVISAO_miner_entangloporter_teleporter_sorter.md`
2. Bloco formatado para a seção 21 de `port/DIFERENCAS.md`:
   `.saida/T20261109-194705-9xd/BLOCO_DIFERENCAS.md` (e aplicado em `port/DIFERENCAS.md:373-388`)
3. Especificação técnica completa do grupo utilidades:
   `.saida/T20261109-194705-9xd/ESPEC_utilidades.md`
4. Correções de código aplicadas:
   `port/mod/src/miner.rs`
5. Este resumo final:
   `.saida/T20261109-194705-9xd/RESUMO.md`
