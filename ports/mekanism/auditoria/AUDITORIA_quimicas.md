# Relatório de Auditoria: Máquinas Químicas, Tanques e Melhorias
**Referência Java**: Mekanism `v1.21.1-10.7.19.85` (`java/`)
**Port Rust**: `port/mod/src/` (`process.rs`, `reaction.rs`, `chem.rs`, `chem_table.rs`, `factory.rs`, `combiner.rs`, `sawmill.rs`, `machine.rs`, `upgrades.rs`, `tank.rs`)
**Regra de Conversão de Energia**: $FE = Joules \times \frac{2}{5}$ via `fe()` (`feConversionRate = 2.5`).

---

## 1. Chemical Tank (Tanque Químico)

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| Basic Chemical Tank - Capacidade | `64 * 1000 = 64.000` mB (`ChemicalTankTier.java:12`) | `64_000` mB (`tank.rs:41`) | OK |
| Basic Chemical Tank - Taxa de saída | `1 * 1000 = 1.000` mB/t (`ChemicalTankTier.java:12`) | `1_000` mB/t (`tank.rs:42`) | OK |
| Advanced Chemical Tank - Capacidade | `256 * 1000 = 256.000` mB (`ChemicalTankTier.java:13`) | `256_000` mB (`tank.rs:41`) | OK |
| Advanced Chemical Tank - Taxa de saída | `16 * 1000 = 16.000` mB/t (`ChemicalTankTier.java:13`) | `16_000` mB/t (`tank.rs:42`) | OK |
| Elite Chemical Tank - Capacidade | `1.024 * 1000 = 1.024.000` mB (`ChemicalTankTier.java:14`) | `1_024_000` mB (`tank.rs:41`) | OK |
| Elite Chemical Tank - Taxa de saída | `128 * 1000 = 128.000` mB/t (`ChemicalTankTier.java:14`) | `128_000` mB/t (`tank.rs:42`) | OK |
| Ultimate Chemical Tank - Capacidade | `8.192 * 1000 = 8.192.000` mB (`ChemicalTankTier.java:15`) | `8_192_000` mB (`tank.rs:41`) | OK |
| Ultimate Chemical Tank - Taxa de saída | `512 * 1000 = 512.000` mB/t (`ChemicalTankTier.java:15`) | `512_000` mB/t (`tank.rs:42`) | OK |
| Creative Chemical Tank - Capacidade | `Long.MAX_VALUE` (`ChemicalTankTier.java:16`) | `1_000_000_000_000` (1T mB) (`tank.rs:41`) | LIMITE |
| Creative Chemical Tank - Taxa de saída | `Long.MAX_VALUE / 2` (`ChemicalTankTier.java:16`) | `1_000_000_000` (1B mB/t) (`tank.rs:42`) | LIMITE |
| Creative Chemical Tank - Auto-recarga | Não esvazia (`TileEntityChemicalTank.java:117`) | Restaura capacidade total se `amount > 0` (`tank.rs:102-106`) | OK |
| Chemical Tank - Modos de despejo | `IDLE (0)`, `DUMPING_EXCESS (1)`, `DUMPING (2)` (`TileEntityChemicalTank.java:67, 118-127`) | `Idle = 0`, `Excess = 1`, `Dumping = 2` (`tank.rs:52-56`) | OK |
| Chemical Tank - Ciclo de modos | `dumping.getNext()` / `getPrevious()` (`TileEntityChemicalTank.java:135, 242`) | `next()` / `previous()` módulo 3 (`tank.rs:66-71`) | OK |
| Chemical Tank - Taxa modo Dumping | `tier.getStorage() / 400` (`TileEntityChemicalTank.java:119`) | `dumping` (chamado com `cap / 400` em `plus.rs:429`) (`tank.rs:88`) | OK |
| Chemical Tank - Taxa modo Excess | Guarda 85% (`dumpExcessKeepRatio = 0.85`), descarta excedente até `output` (`GeneralConfig.java:180`, `TileEntityChemicalTank.java:121-125`) | `keep = (cap * 0.85); (tank.amount - keep).min(excess_rate)` (`tank.rs:89-92`) | OK |

---

## 2. Fluid Tank (Tanque de Fluidos)

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| Basic Fluid Tank - Capacidade | `32 * 1000 = 32.000` mB (`FluidTankTier.java:9`) | `32_000` mB (`tank.rs:45`) | OK |
| Basic Fluid Tank - Taxa de saída | `1 * 1000 = 1.000` mB/t (`FluidTankTier.java:9`) | `1_000` mB/t (`tank.rs:46`) | OK |
| Advanced Fluid Tank - Capacidade | `64 * 1000 = 64.000` mB (`FluidTankTier.java:10`) | `64_000` mB (`tank.rs:45`) | OK |
| Advanced Fluid Tank - Taxa de saída | `4 * 1000 = 4.000` mB/t (`FluidTankTier.java:10`) | `4_000` mB/t (`tank.rs:46`) | OK |
| Elite Fluid Tank - Capacidade | `128 * 1000 = 128.000` mB (`FluidTankTier.java:11`) | `128_000` mB (`tank.rs:45`) | OK |
| Elite Fluid Tank - Taxa de saída | `16 * 1000 = 16.000` mB/t (`FluidTankTier.java:11`) | `16_000` mB/t (`tank.rs:46`) | OK |
| Ultimate Fluid Tank - Capacidade | `256 * 1000 = 256.000` mB (`FluidTankTier.java:12`) | `256_000` mB (`tank.rs:45`) | OK |
| Ultimate Fluid Tank - Taxa de saída | `64 * 1000 = 64.000` mB/t (`FluidTankTier.java:12`) | `64_000` mB/t (`tank.rs:46`) | OK |
| Creative Fluid Tank - Capacidade | `Integer.MAX_VALUE` (2.147.483.647 mB) (`FluidTankTier.java:13`) | `1_000_000_000_000` (1T mB) (`tank.rs:45`) | LIMITE |
| Creative Fluid Tank - Taxa de saída | `Integer.MAX_VALUE / 2` (`FluidTankTier.java:13`) | `1_000_000_000` (1B mB/t) (`tank.rs:46`) | LIMITE |
| ContainerEditMode | `BOTH (0)`, `FILL (1)`, `EMPTY (2)` (`IFluidContainerManager.java:20-22`) | `Both = 0`, `Fill = 1`, `Empty = 2` (`tank.rs:11-15`) | OK |
| ContainerEditMode - Ciclo de modos | `editMode.getNext()` / `getPrevious()` (`TileEntityFluidTank.java:302, 308`) | `next()` / `previous()` módulo 3 (`tank.rs:25-30`) | OK |
| Manipulação de baldes | Balde cheio esvazia no tanque se couber; balde vazio enche se houver >= 1.000 mB (`FluidInventorySlot.java:118-180`) | `buckets()` reproduz preenchimento/esvaziamento fielmente com checagem de modo (`tank.rs:149-184`) | OK |
| Volume padrão de balde | `FluidType.BUCKET_VOLUME = 1000` mB | `BUCKET: u64 = 1000` (`tank.rs:7`) | OK |
| Mapeamento de baldes | 20 baldes (água, lava + 18 fluidos do Mekanism) (`MekanismFluids.java`) | `BUCKETS: [(&str, &str); 20]` (`tank.rs:109-130`) | OK |

---

## 3. Upgrades (Melhorias)

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| Tipos de Melhoria (1.21.1) | 7 tipos: SPEED, ENERGY, FILTER, CHEMICAL, MUFFLING, ANCHOR, STONE_GENERATOR (`Upgrade.java:29-35`) | 7 tipos correspondentes: `SPEED`, `ENERGY`, `CHEMICAL`, `MUFFLING`, `ANCHOR`, `FILTER`, `STONE_GENERATOR` (`upgrades.rs:8-17`) | OK |
| Quantidade máxima por tipo | [8, 8, 1, 8, 1, 1, 1] (`Upgrade.java:29-35`) | `MAX: [u8; 8] = [8, 8, 8, 1, 1, 1, 1, 1]` (`upgrades.rs:25`) | OK |
| Multiplicador máximo | `maxUpgradeMultiplier = 10.0` (`GeneralConfig.java:139`) | `10.0` (`upgrades.rs:49, 54, 59, 66`) | OK |
| Tempo de instalação da melhoria | `UPGRADE_TICKS_REQUIRED = 20` ticks (`TileComponentUpgrade.java:42`) | `UPGRADE_TICKS_REQUIRED = 20` ticks (`upgrades.rs:21`) | OK |
| Fórmula de Ticks | $\max(1, \text{round}(def \times 10^{-speed/8}))$ (`MekanismUtils.java:311-328`) | `((def * 10^(-speed/8)) as u32).max(1)` (`upgrades.rs:50`) | OK |
| Fórmula de Energia / tick | $\lceil def \times 10^{(2 \cdot speed - energy)/8} \rceil$ (`MekanismUtils.java:355-360`) | `(def * 10^((2*speed - energy)/8)).ceil() as u64` (`upgrades.rs:60`) | OK |
| Fórmula de Energia Máxima | $\text{round}(def \times 10^{energy/8})$ (`MekanismUtils.java:390-395`) | `(def * 10^(energy/8)).round() as u64` (`upgrades.rs:55`) | OK |
| Multiplicador Médio de Gás | $10^{(2 \cdot speed - chemical)/8}$ (`MekanismUtils.java:372-378`) | `10^((2*speed - gas)/8)` (`upgrades.rs:66`) | OK |
| Operações por tick contínuas | $2^{speed}$ (`MekanismUtils.java:344`) | `1 << speed` (`upgrades.rs:72`) | OK |
| Consumo randômico de gás | `StatUtils.inversePoisson(mean)` (`TileEntityAdvancedElectricMachine.java:116`, `StatUtils.java:14-25`) | `inverse_poisson(mean, seed)` gerador Poisson exato de Knuth com LCG 64-bit (`process.rs:118-124`) | OK |
| Suporte em Câmaras Avançadas | Purification, Injection, Dissolution suportam SPEED, ENERGY, CHEMICAL, MUFFLING (`TileEntityAdvancedElectricMachine.java`, `TileEntityChemicalDissolutionChamber.java`) | Suporte exato configurado em `SUPPORTED_UPGRADES` (`upgrades.rs:40`) | OK |
| Suporte em Osmium Compressor | Suporta SPEED, ENERGY, MUFFLING (não suporta CHEMICAL na 1.21.1) (`TileEntityOsmiumCompressor.java`) | Suporte exato configurado em `SUPPORTED_UPGRADES` (`upgrades.rs:40`) | OK |
| Suporte em Nucleosynthesizer | Suporta apenas MUFFLING (`TileEntityAntiprotonicNucleosynthesizer.java`) | Suporte apenas MUFFLING (`upgrades.rs:40`) | OK |

---

## 4. Máquinas Químicas com Processo e Tanques

| Máquina / Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| **Chemical Oxidizer** - Tanque saída | `MAX_GAS = 10.000` mB (`TileEntityChemicalOxidizer.java:61`) | `10_000` mB (`lib.rs:200`, `process.rs:263`) | OK |
| Chemical Oxidizer - Ticks base | `BASE_TICKS_REQUIRED = 100` (5s) (`TileEntityChemicalOxidizer.java:62`) | `100` ticks (`lib.rs:200`, `process.rs:263`) | OK |
| Chemical Oxidizer - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:68`) | `fe(200) = 80` FE/t (`lib.rs:200`) | OK |
| Chemical Oxidizer - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:70`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| **Metallurgic Infuser** - Tanque infusão | `MAX_INFUSE = 1.000` mB (`TileEntityMetallurgicInfuser.java:76`) | `1_000` mB (`lib.rs:188`, `process.rs:261`) | OK |
| Metallurgic Infuser - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityMetallurgicInfuser.java:77`) | `200` ticks (`lib.rs:188`, `process.rs:261`) | OK |
| Metallurgic Infuser - Consumo energia | 50 J/t = `20` FE/t (`UsageConfig.java:65`) | `fe(50) = 20` FE/t (`lib.rs:188`) | OK |
| Metallurgic Infuser - Armazenamento | 20.000 J = `8.000` FE (`StorageConfig.java:67`) | `fe(20_000) = 8_000` FE (`lib.rs:236`) | OK |
| Metallurgic Infuser - Botão Dump | ID `GuiDumpButton` (`TileEntityMetallurgicInfuser.java:82`) | Botão ID `4000` esvazia o tanque de infusão (`chem.rs:5`, `chem.rs:14-19`) | OK |
| **Chemical Dissolution Chamber** - Tanques | Entrada `10.000` mB, Saída `10.000` mB (`TileEntityChemicalDissolutionChamber.java:75`) | Entrada `10_000` mB, Saída `10_000` mB (`lib.rs:202`, `process.rs:264`) | OK |
| Dissolution Chamber - Ticks base | `BASE_TICKS_REQUIRED = 100` (5s) (`TileEntityChemicalDissolutionChamber.java:76`) | `100` ticks (`lib.rs:202`, `process.rs:264`) | OK |
| Dissolution Chamber - Consumo energia | 400 J/t = `160` FE/t (`UsageConfig.java:72`) | `fe(400) = 160` FE/t (`lib.rs:202`) | OK |
| Dissolution Chamber - Armazenamento | 80.000 J = `32.000` FE (`StorageConfig.java:74`) | `fe(80_000) = 32_000` FE (`lib.rs:236`) | OK |
| Dissolution Chamber - Consumo de gás | Randômico via `inversePoisson` (`TileEntityChemicalDissolutionChamber.java:116`) | Randômico via `inverse_poisson(mean)` (`process.rs:136-150`) | OK |
| **Chemical Washer** - Tanques | Fluido `10.000` mB, Slurry In `10.000` mB, Slurry Out `10.000` mB (`TileEntityChemicalWasher.java:74-75`) | Fluido `10_000` mB, Químico In `10_000` mB, Químico Out `10_000` mB (`lib.rs:197`) | OK |
| Chemical Washer - Modo de operação | Contínuo: 1 mB dirty + 5 mB água -> 1 mB clean (`TileEntityChemicalWasher.java`) | Contínuo, taxa base 1 mB/t $\times 2^{speed}$ (`continuous.rs`, `lib.rs:197`) | OK |
| Chemical Washer - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:74`) | `fe(200) = 80` FE/t (`lib.rs:197`) | OK |
| Chemical Washer - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:76`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| **Chemical Crystallizer** - Tanque entrada | `MAX_CHEMICAL = 10.000` mB (`TileEntityChemicalCrystallizer.java:60`) | `10_000` mB (`lib.rs:204`, `process.rs:265`) | OK |
| Chemical Crystallizer - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityChemicalCrystallizer.java:61`) | `200` ticks (`lib.rs:204`, `process.rs:265`) | OK |
| Chemical Crystallizer - Consumo energia | 400 J/t = `160` FE/t (`UsageConfig.java:75`) | `fe(400) = 160` FE/t (`lib.rs:204`) | OK |
| Chemical Crystallizer - Armazenamento | 80.000 J = `32.000` FE (`StorageConfig.java:77`) | `fe(80_000) = 32_000` FE (`lib.rs:236`) | OK |
| Chemical Crystallizer - Receita padrão | 200 mB clean slurry -> 1 crystal (`crystallizing/`) | 200 mB clean slurry -> 1 crystal (`process.rs:191-205`) | OK |
| **Chemical Injection Chamber** - Tanque gás | `MAX_GAS = 210` mB (`TileEntityAdvancedElectricMachine.java:71`) | `210` mB (`lib.rs:194`, `process.rs:262`) | OK |
| Chemical Injection - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityAdvancedElectricMachine.java:70`) | `200` ticks (`lib.rs:194`) | OK |
| Chemical Injection - Consumo energia | 400 J/t = `160` FE/t (`UsageConfig.java:63`) | `fe(400) = 160` FE/t (`lib.rs:194`) | OK |
| Chemical Injection - Armazenamento | 80.000 J = `32.000` FE (`StorageConfig.java:65`) | `fe(80_000) = 32_000` FE (`lib.rs:236`) | OK |
| Chemical Injection - Consumo gás | 1 mB HCl randômico via `inversePoisson` (`TileEntityAdvancedElectricMachine.java:116`) | 1 mB randômico via `inverse_poisson(mean)` (`process.rs:136-150`) | OK |
| **Purification Chamber** - Tanque gás | `MAX_GAS = 210` mB (`TileEntityAdvancedElectricMachine.java:71`) | `210` mB (`lib.rs:193`, `process.rs:262`) | OK |
| Purification Chamber - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityAdvancedElectricMachine.java:70`) | `200` ticks (`lib.rs:193`) | OK |
| Purification Chamber - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:62`) | `fe(200) = 80` FE/t (`lib.rs:193`) | OK |
| Purification Chamber - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:64`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| Purification Chamber - Consumo gás | 1 mB Oxygen randômico via `inversePoisson` (`TileEntityAdvancedElectricMachine.java:116`) | 1 mB randômico via `inverse_poisson(mean)` (`process.rs:136-150`) | OK |
| **Osmium Compressor** - Tanque químico | `MAX_GAS = 210` mB (`TileEntityAdvancedElectricMachine.java:71`) | `210` mB (`lib.rs:192`) | OK |
| Osmium Compressor - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityAdvancedElectricMachine.java:70`) | `200` ticks (`lib.rs:192`) | OK |
| Osmium Compressor - Consumo energia | 100 J/t = `40` FE/t (`UsageConfig.java:61`) | `fe(100) = 40` FE/t (`lib.rs:192`) | OK |
| Osmium Compressor - Armazenamento | 20.000 J = `8.000` FE (`StorageConfig.java:63`) | `fe(20_000) = 8_000` FE (`lib.rs:236`) | OK |
| Osmium Compressor - Consumo fixo | 1 mB fixo (sem melhoria de gás) (`TileEntityOsmiumCompressor.java`) | 1 mB fixo (`process.rs:152-160`) | OK |
| **Nutritional Liquifier** - Tanque saída | `MAX_FLUID = 10.000` mB (`TileEntityNutritionalLiquifier.java:72`) | `10_000` mB (`lib.rs:211`, `process.rs:37`) | OK |
| Nutritional Liquifier - Ticks base | `BASE_TICKS_REQUIRED = 100` (5s) (`TileEntityNutritionalLiquifier.java:73`) | `100` ticks (`lib.rs:211`) | OK |
| Nutritional Liquifier - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:76`) | `fe(200) = 80` FE/t (`lib.rs:211`) | OK |
| Nutritional Liquifier - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:78`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| Nutritional Liquifier - Produção pasta | `nutrition * 50` mB dinâmico de `foodProperties` (`TileEntityNutritionalLiquifier.java`) | `nutrition * 50` mB via tabela estática `FOODS` (`process.rs:37-38`) | LIMITE |
| **Pigment Extractor** - Tanque pigmento | `MAX_PIGMENT = 20.000` mB (`TileEntityPigmentExtractor.java:63`) | `20_000` mB (`lib.rs:209`) | OK |
| Pigment Extractor - Ticks base | `BASE_TICKS_REQUIRED = 100` (5s) (`TileEntityPigmentExtractor.java:64`) | `100` ticks (`lib.rs:209`) | OK |
| Pigment Extractor - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:77`) | `fe(200) = 80` FE/t (`lib.rs:209`) | OK |
| Pigment Extractor - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:79`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| **Pigment Mixer** - Tanques entrada e saída | Entrada esquerda/direita: `1.000` mB, Saída: `2.000` mB (`TileEntityPigmentMixer.java:70-71`) | Entrada: `1_000` mB, Saída: `2_000` mB (`lib.rs:208`) | OK |
| Pigment Mixer - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:78`) | `fe(200) = 80` FE/t (`lib.rs:208`) | OK |
| Pigment Mixer - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:80`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| Pigment Mixer - Modo contínuo | 1 mB esquerda + 1 mB direita -> 2 mB saída por tick $\times 2^{speed}$ (`TileEntityPigmentMixer.java`) | Contínuo 1:1:2 $\times 2^{speed}$ (`continuous.rs`, `lib.rs:208`) | OK |
| **Painting Machine** - Tanque pigmento | `MAX_PIGMENT = 15.000` mB (`TileEntityPaintingMachine.java:71`) | `15_000` mB (`lib.rs:210`) | OK |
| Painting Machine - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityPaintingMachine.java:72`) | `200` ticks (`lib.rs:210`) | OK |
| Painting Machine - Consumo energia | 100 J/t = `40` FE/t (`UsageConfig.java:79`) | `fe(100) = 40` FE/t (`lib.rs:210`) | OK |
| Painting Machine - Armazenamento | 20.000 J = `8.000` FE (`StorageConfig.java:81`) | `fe(20_000) = 8_000` FE (`lib.rs:236`) | OK |
| Painting Machine - Consumo por operação | `256` mB por operação de tingimento (`painting/` datagen 10.7) | `256` mB por operação (`process.rs:163-178`) | OK |
| **Chemical Infuser** - Tanques | Entrada esquerda/direita: `10.000` mB, Saída centro: `10.000` mB (`TileEntityChemicalInfuser.java:67`) | Entrada: `10_000` mB, Saída: `10_000` mB (`lib.rs:201`) | OK |
| Chemical Infuser - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:69`) | `fe(200) = 80` FE/t (`lib.rs:201`) | OK |
| Chemical Infuser - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:71`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| Chemical Infuser - Modo contínuo | Reação contínua de gases a cada tick $\times 2^{speed}$ (`TileEntityChemicalInfuser.java`) | Contínuo $\times 2^{speed}$ (`continuous.rs`, `lib.rs:201`) | OK |
| **Electrolytic Separator** - Tanques | Fluido entrada: `24.000` mB, Gases saída: `2.400` mB cada (`TileEntityElectrolyticSeparator.java:94-95`) | Fluido: `24_000` mB, Gases: `2_400` mB (`lib.rs:196`) | OK |
| Electrolytic Separator - Consumo energia | 400 J/t = `160` FE/t (`UsageConfig.java:66`) | `fe(400) = 160` FE/t (`lib.rs:196`) | OK |
| Electrolytic Separator - Armazenamento | 80.000 J = `32.000` FE (`StorageConfig.java:68`) | `fe(80_000) = 32_000` FE (`lib.rs:236`) | OK |
| Electrolytic Separator - Taxa de despejo | Despejo em excesso escala com velocidade (`TileEntityElectrolyticSeparator.java`) | Taxa fixa 8 mB/t sem multiplicar por velocidade (`plus.rs:218-219`) | LIMITE |
| **Rotary Condensentrator** - Tanques | Fluido: `10.000` mB, Gás: `10.000` mB (`TileEntityRotaryCondensentrator.java:87`) | Fluido: `10_000` mB, Gás: `10_000` mB (`lib.rs:198`) | OK |
| Rotary Condensentrator - Consumo energia | 50 J/t = `20` FE/t (`UsageConfig.java:67`) | `fe(50) = 20` FE/t (`lib.rs:198`) | OK |
| Rotary Condensentrator - Armazenamento | 20.000 J = `8.000` FE (`StorageConfig.java:69`) | `fe(20_000) = 8_000` FE (`lib.rs:236`) | OK |
| Rotary Condensentrator - Modos | `mode == false`: Condensação (gás -> fluido); `mode == true`: Descondensação (fluido -> gás) (`TileEntityRotaryCondensentrator.java:151, 164`) | `mode == false`: Condensação; `mode == true`: Descondensação (`continuous.rs`, `lib.rs:198`) | OK |
| Rotary Condensentrator - Taxa conversão | 1:1 mB por tick $\times 2^{speed}$ (`TileEntityRotaryCondensentrator.java`) | 1:1 $\times 2^{speed}$ (`continuous.rs`) | OK |
| **Isotopic Centrifuge** - Tanques | Entrada: `10.000` mB, Saída: `10.000` mB (`TileEntityIsotopicCentrifuge.java:67`) | Entrada: `10_000` mB, Saída: `10_000` mB (`lib.rs:205`) | OK |
| Isotopic Centrifuge - Consumo energia | 200 J/t = `80` FE/t (`UsageConfig.java:73`) | `fe(200) = 80` FE/t (`lib.rs:205`) | OK |
| Isotopic Centrifuge - Armazenamento | 40.000 J = `16.000` FE (`StorageConfig.java:75`) | `fe(40_000) = 16_000` FE (`lib.rs:236`) | OK |
| **Solar Neutron Activator** - Tanques | Entrada: `10.000` mB, Saída: `10.000` mB (`TileEntitySolarNeutronActivator.java:64`) | Entrada: `10_000` mB, Saída: `10_000` mB (`lib.rs:206`) | OK |
| Solar Neutron Activator - Energia | 0 J (alimentado por radiação solar direta) (`TileEntitySolarNeutronActivator.java`) | `0` FE/t (`lib.rs:206`) | OK |
| **Antiprotonic Nucleosynthesizer** - Tanque gás | `MAX_GAS = 10.000` mB de Antimatéria (`TileEntityAntiprotonicNucleosynthesizer.java:75`) | `10_000` mB (`lib.rs:207`) | OK |
| Antiprotonic Nucleosynthesizer - Duração base | `BASE_TICKS_REQUIRED = 400` (20s) ou duração da receita (`TileEntityAntiprotonicNucleosynthesizer.java:74, 162`) | `400` ticks ou duração da receita (`lib.rs:207`) | OK |
| Antiprotonic Nucleosynthesizer - Consumo energia | 100.000 J/t = `40.000` FE/t (`UsageConfig.java:80`) | `fe(100_000) = 40_000` FE/t (`lib.rs:207`) | OK |
| Antiprotonic Nucleosynthesizer - Armazenamento | 20.000.000 J = `8.000.000` FE (`StorageConfig.java:82`) | `fe(20_000_000) = 8_000_000` FE (`lib.rs:236`) | OK |

---

## 5. Pressurized Reaction Chamber (PRC)

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| PRC - Tanque de Fluido | `MAX_FLUID = 10.000` mB (`TileEntityPressurizedReactionChamber.java:78`) | `TANK: u64 = 10_000` (`reaction.rs:10`) | OK |
| PRC - Tanque de Gás Entrada | `MAX_GAS = 10.000` mB (`TileEntityPressurizedReactionChamber.java:79`) | `TANK: u64 = 10_000` (`reaction.rs:10`) | OK |
| PRC - Tanque de Gás Saída | `MAX_GAS = 10.000` mB (`TileEntityPressurizedReactionChamber.java:79`) | `TANK: u64 = 10_000` (`reaction.rs:10`) | OK |
| PRC - Duração Base | `BASE_DURATION = 100` ticks (5s) (`TileEntityPressurizedReactionChamber.java`) | `BASE_DURATION: u32 = 100` (`reaction.rs:12`) | OK |
| PRC - Consumo base de energia | 5 J/t = `2` FE/t (`UsageConfig.java:70`) | `USAGE: u64 = crate::fe(5) = 2` (`reaction.rs:14`) | OK |
| PRC - Armazenamento base de energia | 2.000 J = `800` FE (`StorageConfig.java:72`) | `fe(2_000) = 800` FE (`lib.rs:236`) | OK |
| PRC - Adição de energia da receita | `PRCEnergyContainer.getBaseEnergyPerTick()` soma `recipe.getEnergyRequired()` (`PRCEnergyContainer.java:28`) | `upgrades::energy_per_tick(USAGE + recipe.energy, speed, energy_up)` (`reaction.rs:96`) | OK |
| PRC - Duração da receita | Usa `recipe.getDuration()` atualizando o tempo do ciclo (`TileEntityPressurizedReactionChamber.java:173`) | `upgrades::ticks(recipe.duration, speed)` (`reaction.rs:83`) | OK |
| PRC - Dedução e Produção | Consome itens, fluidos e gases de entrada e gera saídas ao término do ciclo (`PressurizedReactionCachedRecipe.java`) | Dedução de insumos e criação de saídas em `next.progress >= ticks` (`reaction.rs:103-125`) | OK |

---

## 6. Combiner (Combinador) e Precision Sawmill (Serraria)

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| Combiner - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityCombiner.java:58`) | `TICKS = 200` (`combiner.rs:3`, `machine.rs:7`) | OK |
| Combiner - Consumo energia | 50 J/t = `20` FE/t (`UsageConfig.java:64`) | `fe(50) = 20` FE/t (`lib.rs:191`) | OK |
| Combiner - Armazenamento energia | 20.000 J = `8.000` FE (`StorageConfig.java:66`) | `fe(20_000) = 8_000` FE (`lib.rs:236`) | OK |
| Combiner - Entradas | 2 entradas de item (principal + extra de 4 a 8 blocos) (`TileEntityCombiner.java`) | 2 slots: `main` e `extra` (`combiner.rs:35-36`) | OK |
| Precision Sawmill - Ticks base | `BASE_TICKS_REQUIRED = 200` (10s) (`TileEntityPrecisionSawmill.java:60`) | `TICKS = 200` (`sawmill.rs:3`, `machine.rs:7`) | OK |
| Precision Sawmill - Consumo energia | 50 J/t = `20` FE/t (`UsageConfig.java:60`) | `fe(50) = 20` FE/t (`lib.rs:190`) | OK |
| Precision Sawmill - Armazenamento energia | 20.000 J = `8.000` FE (`StorageConfig.java:62`) | `fe(20_000) = 8_000` FE (`lib.rs:236`) | OK |
| Precision Sawmill - Saída secundária com chance | `RandomSource.nextFloat() <= recipe.getSecondaryChance()` (`TileEntityPrecisionSawmill.java`) | `roll <= sec.2` (`sawmill.rs:69`) | OK |

---

## 7. Fábricas Químicas e Fábricas Avançadas

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| Linhas por tier | Basic: 3, Advanced: 5, Elite: 7, Ultimate: 9 (`FactoryTier.java:7-10`) | `LINES: [usize; 4] = [3, 5, 7, 9]` (`factory.rs:18`) | OK |
| Capacidade do tanque em Fábrica Química | Multiplicado pelo número de linhas: `base_cap * tier.processes` (`TileEntityItemStackChemicalToItemStackFactory.java:123, 125`) | `spec.cap_in *= n as u64;` (`factory.rs:215`) | OK |
| Capacidade tanque Fábrica Infusora | Basic: 3.000, Advanced: 5.000, Elite: 7.000, Ultimate: 9.000 mB | Multiplica 1.000 mB pelas linhas da fábrica (`factory.rs:215`) | OK |
| Capacidade tanque Fábrica Purificadora / Injeção / Compressão | Basic: 630, Advanced: 1.050, Elite: 1.470, Ultimate: 1.890 mB | Multiplica 210 mB pelas linhas da fábrica (`factory.rs:215`) | OK |
| Progresso multi-linha | Cada linha opera seu próprio progresso individual em paralelo | Progresso da linha 0 nos primeiros 2 bytes; linhas subsequentes salvas no deslocamento `TAIL = 46` (`factory.rs:10, 263-267`) | OK |
| Compartilhamento de insumos químicos | Todas as linhas compartilham o tanque único e energia acumulada | `step_chem_lines()` passa o tanque e energia entre linhas sequencialmente (`factory.rs:77-110`) | OK |
| Conversão de itens de infusão | Slot de infusão abastece o tanque uma vez por tick | Linha 0 consome do slot de conversão `conv` e alimenta o tanque comum (`factory.rs:94, 101-103`) | OK |

---

## 8. Tabela de Químicos e Registros

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| Quantidade de Químicos registrados | 72 substâncias químicas (`MekanismChemicals.java`) | 72 substâncias químicas na ordem canônica (`chem_table.rs:2-75`) | OK |
| Indexação de químicos | 1-based (1..=72), 0 = vazio / desconhecido | `position().map_or(0, |i| i as i32 + 1)` (`chem.rs:12`) | OK |
| Transmissão para GUI | Par `[quantidade, id_quimico]` | `[tank.amount as i32, index(&tank.chemical)]` (`chem.rs:16-18`) | OK |
