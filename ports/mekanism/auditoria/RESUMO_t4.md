# Resumo da Tarefa T20261109-183859-tvo

## Objetivo
Auditoria completa de **DIGITAL MINER**, **QUANTUM ENTANGLOPORTER**, **CHARGEPAD**, **TELEPORTER** e **LOGISTICAL SORTER**: conferência de raio mín/máx, delay, consumo de energia por bloco (`minerEnergyUsage`), slots e filtros, capacidade e taxas do entangloporter (energia, fluido, gás, itens), custo e tempo do teleporter, velocidade do sorter e levantamento das definições do chargepad ausente.

---

## 1. Arquivos Auditados e Modificados

### Arquivos no Escopo Auditados
- Java (Fonte da Verdade - Mekanism 1.21.1-10.7.19.85):
  - `java/main/java/mekanism/common/tile/machine/TileEntityDigitalMiner.java`
  - `java/main/java/mekanism/common/capabilities/energy/MinerEnergyContainer.java`
  - `java/main/java/mekanism/common/tile/TileEntityQuantumEntangloporter.java`
  - `java/main/java/mekanism/common/content/entangloporter/InventoryFrequency.java`
  - `java/main/java/mekanism/common/tile/TileEntityChargepad.java`
  - `java/main/java/mekanism/common/block/basic/BlockChargepad.java`
  - `java/main/java/mekanism/common/tile/TileEntityTeleporter.java`
  - `java/main/java/mekanism/common/tile/TileEntityLogisticalSorter.java`
  - `java/main/java/mekanism/common/config/GeneralConfig.java`
  - `java/main/java/mekanism/common/config/UsageConfig.java`
  - `java/main/java/mekanism/common/config/StorageConfig.java`
  - `java/main/java/mekanism/common/tier/EnergyCubeTier.java`
  - `java/main/java/mekanism/common/tier/FluidTankTier.java`
  - `java/main/java/mekanism/common/tier/ChemicalTankTier.java`
- Port (Rust WASM Horizon):
  - `port/mod/src/miner.rs` (375 linhas) - **MODIFICADO**
  - `port/mod/src/entangloporter.rs` (222 linhas) - **MODIFICADO**
  - `port/mod/src/teleporter.rs` (306 linhas) - **AUDITADO (OK)**
  - `port/mod/src/sorter.rs` (160 linhas) - **AUDITADO (OK)**
  - Chargepad - **AUDITADO (FALTA no port)**

---

## 2. Correções Aplicadas

1. **`port/mod/src/entangloporter.rs`**:
   - `ENERGY_CAPACITY` corrigido de `256_000_000` para `crate::fe(256_000_000)` (102.400.000 FE). No Java, o buffer é 256.000.000 Joules (`GeneralConfig.java:261`), correspondente ao Ultimate Energy Cube; no port, a energia é armazenada em Forge Energy (J × 2 / 5).

2. **`port/mod/src/miner.rs`**:
   - Raio Mínimo: adicionada a constante `pub const MIN_RADIUS: u8 = 0;`, atualizado `decode()` para `.clamp(MIN_RADIUS, MAX_RADIUS)` e `press(RADIUS_DOWN)` para `.max(MIN_RADIUS)`. No Java (`TileEntityDigitalMiner.java:393, 401, 1399`), raio 0 é permitido (representando coluna 1×1).
   - Consumo de Energia Dinâmico: adicionada a constante `pub const SILK_MULTIPLIER: u64 = 12;` e a função `pub fn energy_usage(s: &Settings) -> u64`, aplicando a fórmula fiel de `MinerEnergyContainer.java`:
     - Base: 1.000 J/t (400 FE/t)
     - Silk Touch: multiplicador 12×
     - Raio > 10: `(radius - 10) / 22.0`
     - Altura > 60: `(height - 60) / 323.0`
     - Total em Joules arredondado para cima (`ceil`) e convertido via `crate::fe()`.
   - Dedução no Tick: `run_miners()` agora debita `let usage = energy_usage(&s);` a cada tick em vez de valor estático.

---

## 3. Testes Auditados e Editados

- **`port/mod/src/miner.rs`**:
  - Teste editado: `the_buttons_keep_the_area_sane` (atualizado para testar decremento do raio até `MIN_RADIUS = 0`).
  - Teste adicionado: `energy_usage_scales_with_silk_radius_and_height` (valida consumo base, multiplicador Silk Touch e cálculo de posição com raio 0).
- **Mutações**: A prova de mutação está pendente do juiz (nenhuma lista de mutações foi fornecida na especificação da tarefa).
- **Execução na VM**: não medido (a VM não possui toolchain de compilação rustc/cargo configurada, conforme documentado nas instruções da tarefa e em `HANDOFF.md`).

---

## 4. Levantamento do Chargepad (FALTA)

O Chargepad não existe no port (`port/mod/src/`). O Java define:
- Identificadores: `BlockChargepad.java`, `TileEntityChargepad.java`, `mekanism:chargepad`.
- Consumo / taxa de recarga: `1_024_000L` J/t (`UsageConfig.java:60`) = 409.600 FE/t.
- Armazenamento: `2_048_000L` J (`StorageConfig.java:69`) = 819.200 FE.
- Faces de entrada de energia: `RelativeSide.BACK` e `RelativeSide.BOTTOM`.
- Entidades atendidas: `Player` (1 item recarregável por tick) e `EntityRobit` situados em `AABB(x, y, z, x+1, y+0.4, z+1)`.
- Áudio/visual: som de placa de pressão stone ao ativar/desativar e partículas de poeira de redstone. Sem tela (GUI) e sem upgrades.

---

## 5. Entregáveis Gerados

1. Relatório de auditoria detalhado:
   `.saida/T20261109-183859-tvo/AUDITORIA_miner_entangloporter_chargepad_teleporter_sorter.md`
2. Bloco formatado para inclusão na seção 20 de `port/DIFERENCAS.md`:
   `.saida/T20261109-183859-tvo/BLOCO_DIFERENCAS.md`
3. Correções aplicadas no código-fonte do port:
   `port/mod/src/miner.rs` e `port/mod/src/entangloporter.rs`
4. Este relatório de resumo:
   `.saida/T20261109-183859-tvo/RESUMO.md`
