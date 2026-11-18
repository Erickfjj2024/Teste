# Resumo da Auditoria: Máquinas Químicas (Tarefa T20261109-190722-kmt)

## 1. Escopo e Execução
- **Objetivo**: Auditoria de números e algoritmos das MÁQUINAS QUÍMICAS (tanques, quantidades de gás/químico por operação, taxas de transferência, tempos de processo, melhorias) entre Mekanism Java `v1.21.1-10.7.19.85` e o port Horizon em Rust.
- **Arquivos Auditados no Port (`port/mod/src/`)**:
  - `tank.rs`
  - `upgrades.rs`
  - `process.rs`
  - `reaction.rs`
  - `chem.rs`
  - `chem_table.rs`
  - `factory.rs`
  - `combiner.rs`
  - `sawmill.rs`
  - `machine.rs`
- **Arquivos Java de Referência (`java/`)**:
  - Tiers: `ChemicalTankTier.java`, `FluidTankTier.java`, `FactoryTier.java`
  - Máquinas e blocos: `TileEntityChemicalTank.java`, `TileEntityFluidTank.java`, `IFluidContainerManager.java`, `TileEntityChemicalOxidizer.java`, `TileEntityMetallurgicInfuser.java`, `TileEntityChemicalDissolutionChamber.java`, `TileEntityChemicalWasher.java`, `TileEntityChemicalCrystallizer.java`, `TileEntityAdvancedElectricMachine.java`, `TileEntityPressurizedReactionChamber.java`, `PRCEnergyContainer.java`, `TileEntityElectrolyticSeparator.java`, `TileEntityRotaryCondensentrator.java`, `TileEntityNutritionalLiquifier.java`, `TileEntityPigmentExtractor.java`, `TileEntityPigmentMixer.java`, `TileEntityPaintingMachine.java`, `TileEntityChemicalInfuser.java`, `TileEntityIsotopicCentrifuge.java`, `TileEntitySolarNeutronActivator.java`, `TileEntityAntiprotonicNucleosynthesizer.java`, `TileEntityCombiner.java`, `TileEntityPrecisionSawmill.java`, `TileEntityElectricMachine.java`, `TileEntityProgressMachine.java`, `TileEntityFactory.java`, `TileEntityItemStackChemicalToItemStackFactory.java`
  - Configurações e utilitários: `UsageConfig.java`, `StorageConfig.java`, `GeneralConfig.java`, `MekanismUtils.java`, `StatUtils.java`, `TileComponentUpgrade.java`, `Upgrade.java`
  - Receitas Datagen: `java/datagen/generated/mekanism/data/mekanism/recipe/`

---

## 2. Testes Unitários dos Arquivos no Escopo
Contagem real de testes (`#[test]`) inspecionados diretamente nos arquivos do escopo:
- `port/mod/src/process.rs`: 9 testes
- `port/mod/src/reaction.rs`: 4 testes
- `port/mod/src/chem.rs`: 1 teste
- `port/mod/src/chem_table.rs`: 0 testes
- `port/mod/src/factory.rs`: 7 testes
- `port/mod/src/combiner.rs`: 2 testes
- `port/mod/src/sawmill.rs`: 4 testes
- `port/mod/src/machine.rs`: 5 testes
- `port/mod/src/upgrades.rs`: 5 testes
- `port/mod/src/tank.rs`: 6 testes
- **Total**: 42 testes unitários.
- **Execução na VM**: não medido (sem toolchain Rust/Horizon configurada no ambiente Cloud Shell).
- **Prova de Mutação**: a prova de mutação está pendente do juiz (a tarefa não trouxe lista de mutações).

---

## 3. Síntese dos Resultados da Auditoria

1. **Chemical Tank & Fluid Tank**:
   - Capacidades das tiers Basic (64k / 32k), Advanced (256k / 64k), Elite (1.024k / 128k), Ultimate (8.192k / 256k) e taxas de transferência (1k, 16k, 128k, 512k mB/t para químico; 1k, 4k, 16k, 64k mB/t para fluido) conferem perfeitamente.
   - Tanques criativos usam 1 trilhão de mB e taxa de 1 bilhão mB/t no port (LIMITE para acomodação em tipos da GUI/Horizon sem overflow), com recarga instantânea por tick em `creative()`.
   - Modos de despejo (`Idle`, `Excess`, `Dumping`) e razão de retenção em excesso (85% da capacidade) idênticos ao Java.

2. **Melhorias (Upgrades)**:
   - 7 tipos da 1.21.1 (sem upgrade de experiência).
   - Teto de upgrades [8, 8, 8, 1, 1, 1, 1, 1] e tempo de instalação de 20 ticks idênticos.
   - Fórmulas de tempo ($10^{-speed/8}$), consumo de energia ($10^{(2 \cdot speed - energy)/8}$), capacidade máxima de energia ($10^{energy/8}$) e multiplicador médio de gás ($10^{(2 \cdot speed - chemical)/8}$) 100% idênticas ao `MekanismUtils.java`.
   - Consumo probabilístico de gás implementado via algoritmo exato de Poisson inverso de Knuth em `process.rs`.

3. **Máquinas Químicas**:
   - Oxidizer, Infuser, Dissolution, Washer, Crystallizer, Injection, Purification, Compressor, PRC, Separator, Rotary, Liquifier, Pigment Extractor, Pigment Mixer, Painting, Chemical Infuser, Centrifuge, Solar Neutron Activator e Nucleosynthesizer tiveram capacidades de tanques, ticks base, consumo em FE ($J \times \frac{2}{5}$) e armazenamento rigorosamente validados contra `UsageConfig`, `StorageConfig` e `TileEntity*`.
   - No PRC, a taxa base (2 FE/t) e adição de energia da receita (`energy_required`), bem como a duração customizada por receita, conferem com `PRCEnergyContainer.java` e `PressurizedReactionRecipe.java`.
   - Nas fábricas químicas (3, 5, 7, 9 linhas), a multiplicação da capacidade dos tanques pelo número de linhas (`spec.cap_in *= n`) e o rastreamento individual do progresso por linha a partir do byte 46 (`TAIL`) conferem com `TileEntityItemStackChemicalToItemStackFactory.java`.

4. **Tabela de Químicos**:
   - 72 produtos químicos mapeados na ordem canônica em `chem_table.rs` (1-based index na interface). Botão de descarte do infusor (ID 4000) conferido.

---

## 4. Entregáveis Produzidos
1. `.saida/T20261109-190722-kmt/AUDITORIA_quimicas.md`: Tabela completa e detalhada de auditoria contendo item, valor Java (com arquivo e linha), valor port (com arquivo e linha) e estado (`OK` / `LIMITE`).
2. `.saida/T20261109-190722-kmt/DIFERENCAS_bloco.md`: Bloco no formato da Seção 19 de `port/DIFERENCAS.md`, enumerado sob a Seção 20 (`20.1` a `20.5`), documentando os limites e decisões do port.
3. `.saida/T20261109-190722-kmt/RESUMO.md`: Este relatório final de auditoria.
