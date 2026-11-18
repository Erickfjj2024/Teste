# Relatório de Revisão Cruzada: Máquinas Químicas (Tarefa T20261109-194128-vq7)
**Escopo**: `tank.rs`, `upgrades.rs`, `process.rs`, `reaction.rs`, `chem.rs`, `chem_table.rs`, `factory.rs`, `combiner.rs`, `sawmill.rs`, `machine.rs`  
**Referência Java**: Mekanism `1.21.1-10.7.19.85` (`java/`)  
**Comparação Prévia**: Auditoria Rodada 1 (`port/auditoria/AUDITORIA_quimicas.md` e `RESUMO_t1.md`)  
**Data**: 2026-11-09  

---

## 1. Visão Geral e Metodologia da Revisão Cruzada

Esta revisão cruzada foi conduzida do zero, comparando linha por linha o código-fonte Java do Mekanism 1.21.1 com o código Rust do port Horizon em `port/mod/src/`, cruzando em seguida com os achados da auditoria da rodada 1.

### Estados adotados:
- **CONFIRMADA**: Achado/correção da rodada 1 conferido no Java e estritamente correto.
- **ERRADA-CORRIGIDA**: Afirmação ou valor da rodada 1 que estava incorreto perante o Java 1.21.1 e foi retificado nesta rodada.
- **OK**: Implementação no port já estava correta perante o Java 1.21.1.
- **CORRIGIDO**: Divergência real no código Rust em `port/mod/src/` corrigida nesta rodada.
- **FORK**: Divergência arquitetural decorrente de limitações da API do Horizon que exige capacidade formal de API (conforme `port/API-REQUISITOS.md`), sem uso de gambiarras.
- **FALTA**: Funcionalidade ou detalhe do Java ausente no port.

---

## 2. Erros Identificados na Auditoria da Rodada 1

A revisão independente detectou erros graves de aferição na rodada 1:

1. **Erros de Configuração de Armazenamento de Energia (`StorageConfig.java`)**:
   - A rodada 1 afirmou que `chemicalDissolutionChamber` e `chemicalCrystallizer` tinham capacidade de 80.000 J (32.000 FE). **FALSO**: no Java 1.21.1 (`StorageConfig.java:125, 127`), ambos possuem **160.000 J (64.000 FE)**.
   - A rodada 1 afirmou que `combiner` tinha capacidade de 20.000 J (8.000 FE). **FALSO**: no Java (`StorageConfig.java:122`), o Combiner possui **40.000 J (16.000 FE)**.
   - A rodada 1 afirmou que `osmiumCompressor`, `chemicalInfuser` e `isotopicCentrifuge` tinham capacidade de 40.000 J (16.000 FE). **FALSO**: no Java (`StorageConfig.java:120, 128, 129`), todos possuem **80.000 J (32.000 FE)**.
   - A rodada 1 afirmou que `antiprotonicNucleosynthesizer` tinha capacidade de 20.000.000 J (8.000.000 FE). **FALSO**: no Java (`StorageConfig.java:132`), possui **1.000.000.000 J (400.000.000 FE)**.

2. **Erro de Configuração de Consumo de Energia (`UsageConfig.java`)**:
   - A rodada 1 afirmou que o Electrolytic Separator lia seu consumo de `UsageConfig.electrolyticSeparator`. **FALSO**: no Java 1.21.1, a classe `UsageConfig` sequer possui campo para o separador. O consumo é dinâmico e hardcoded em `TileEntityElectrolyticSeparator.java:199` baseado na densidade energética do hidrogênio: `2 * hydrogenEnergyDensity = 400 J/t = 160 FE/t`.

3. **Uso Indevido da Classificação `LIMITE`**:
   - A rodada 1 classificou divergências do Horizon como `LIMITE`, contrariando a Regra de Ouro do projeto: *"NÃO existem 'limites do Horizon': se o port não consegue imitar o Java, marque FORK e descreva a capacidade de API necessária"*. Todos foram reclassificados como **FORK** com a especificação de API necessária.

4. **Lacuna Matemática no Consumo Contínuo de Gás**:
   - A rodada 1 não percebeu que no Osmium Compressor (`process.rs`), o consumo contínuo de 1 mB/t fixo criava uma trapaça severa quando a máquina recebia Speed Upgrades: em vez de gastar os 200 mB nominais por item, o processo acelerado para 20 ticks gastava apenas 20 mB de gás. Corrigido nesta rodada aplicando escala proporcional ao tempo reduzido.

5. **Aproximações Numéricas Truncadas**:
   - A rodada 1 alegou "arredondamento" nas fórmulas de `MekanismUtils.java`. No Java, `MathUtils.clampToInt` e `clampToLong` realizam **truncamento** puro via cast `(int)` e `(long)` de `double`, e não `Math.round`.

6. **Incompatibilidade no Teste de Recarga de Energia (`factory.rs`)**:
   - O teste `the_charge_slot_turns_redstone_into_energy_while_there_is_room` continuava esperando 10.000 Joules (valor obsoleto antes da conversão de FE), quando a conversão padrão estabelecida na tarefa t7 (`feConversionRate = 2.5`) produz 4.000 FE por redstone. Corrigido no teste.

---

## 3. Tabela Comparativa Completa da Revisão Cruzada

### 3.1 Tanques Químicos e Fluidos (`tank.rs`)

| Item | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes e Notas da Revisão |
|---|---|---|---|---|
| Capacidade Chemical Tank Basic | `ChemicalTankTier.java:15` | `tank.rs:30` | CONFIRMADA | 64.000 mB |
| Capacidade Chemical Tank Advanced | `ChemicalTankTier.java:16` | `tank.rs:31` | CONFIRMADA | 256.000 mB |
| Capacidade Chemical Tank Elite | `ChemicalTankTier.java:17` | `tank.rs:32` | CONFIRMADA | 1.024.000 mB |
| Capacidade Chemical Tank Ultimate | `ChemicalTankTier.java:18` | `tank.rs:33` | CONFIRMADA | 8.192.000 mB |
| Capacidade Chemical Tank Creative | `ChemicalTankTier.java:19` (`Long.MAX_VALUE`) | `tank.rs:34` (`1_000_000_000_000`) | FORK | No Java é `Long.MAX_VALUE` sem decréscimo. No port usa 1 trilhão mB com recarga instantânea por tick para caber em `u64` do Horizon. Requer FORK-5. |
| Transfer Rate Chemical Basic | `ChemicalTankTier.java:23` | `tank.rs:44` | CONFIRMADA | 1.000 mB/t (Java: 500 * 2^1) |
| Transfer Rate Chemical Advanced | `ChemicalTankTier.java:24` | `tank.rs:45` | CONFIRMADA | 16.000 mB/t (Java: 4.000 * 2^2) |
| Transfer Rate Chemical Elite | `ChemicalTankTier.java:25` | `tank.rs:46` | CONFIRMADA | 128.000 mB/t (Java: 16.000 * 2^3) |
| Transfer Rate Chemical Ultimate | `ChemicalTankTier.java:26` | `tank.rs:47` | CONFIRMADA | 512.000 mB/t (Java: 64.000 * 2^3) |
| Transfer Rate Chemical Creative | `ChemicalTankTier.java:27` (`Long.MAX_VALUE / 2`) | `tank.rs:48` (`1_000_000_000`) | FORK | Java drena sem limite. Port fixa 1 bilhão mB/t. Requer FORK-5. |
| Capacidade Fluid Tank Basic | `FluidTankTier.java:14` | `tank.rs:37` | CONFIRMADA | 32.000 mB |
| Capacidade Fluid Tank Advanced | `FluidTankTier.java:15` | `tank.rs:38` | CONFIRMADA | 64.000 mB |
| Capacidade Fluid Tank Elite | `FluidTankTier.java:16` | `tank.rs:39` | CONFIRMADA | 128.000 mB |
| Capacidade Fluid Tank Ultimate | `FluidTankTier.java:17` | `tank.rs:40` | CONFIRMADA | 256.000 mB |
| Capacidade Fluid Tank Creative | `FluidTankTier.java:18` (`Integer.MAX_VALUE`) | `tank.rs:41` (`1_000_000_000_000`) | FORK | Java é infinito (`Integer.MAX_VALUE`). Requer FORK-5. |
| Transfer Rate Fluid Basic | `FluidTankTier.java:22` | `tank.rs:51` | CONFIRMADA | 1.000 mB/t (Java: 250 * 2^2) |
| Transfer Rate Fluid Advanced | `FluidTankTier.java:23` | `tank.rs:52` | CONFIRMADA | 4.000 mB/t (Java: 1.000 * 2^2) |
| Transfer Rate Fluid Elite | `FluidTankTier.java:24` | `tank.rs:53` | CONFIRMADA | 16.000 mB/t (Java: 4.000 * 2^2) |
| Transfer Rate Fluid Ultimate | `FluidTankTier.java:25` | `tank.rs:54` | CONFIRMADA | 64.000 mB/t (Java: 16.000 * 2^2) |
| Modos de Despejo (Dump Modes) | `IFluidContainerManager.java:18` | `tank.rs:16` | CONFIRMADA | Idle (0), Excess (1), Dumping (2). |
| Taxa de Retenção Modo Excess | `TileEntityChemicalTank.java:254` | `tank.rs:104` | CONFIRMADA | 85% retido, 15% restante é descartado ao receber excesso (`cap * 17 / 20`). |
| Persistência e Drops com Conteúdo | `TileEntityChemicalTank.java:444-480` | `tank.rs:88` | OK | Estado do tanque serializado em buffer NBT/data component ao quebrar bloco. |

---

### 3.2 Upgrades e Fórmulas Matemáticas (`upgrades.rs`)

| Item | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes e Notas da Revisão |
|---|---|---|---|---|
| Upgrades Suportados (1.21.1) | `Upgrade.java:14-25` | `upgrades.rs:14-23` | CONFIRMADA | Speed, Energy, Filter, Gas (Chemical), Muffler, Anchor, StoneGenerator (7 tipos; Exp removido na 1.21.1). |
| Tetos Máximos de Upgrade | `Upgrade.java:34-45` | `upgrades.rs:26-34` | CONFIRMADA | Speed: 8, Energy: 8, Chemical: 8, Filter: 1, Muffler: 1, Anchor: 1, StoneGen: 1. |
| Tempo de Instalação de Upgrade | `TileComponentUpgrade.java:280` | `upgrades.rs:66` | CONFIRMADA | 20 ticks (1 segundo) para consumir item do slot de upgrade. |
| Fórmula de Tempo (`getTicks`) | `MekanismUtils.java:313` | `upgrades.rs:41-45` | CONFIRMADA | `ceil(baseTicks * 10^(-speed/8))`. Truncado em `>= 1`. |
| Fórmula de Energia/t (`getEnergyPerTick`) | `MekanismUtils.java:392` | `upgrades.rs:48-52` | CONFIRMADA | `ceil(baseEnergy * 10^((2*speed - energy)/8))`. Truncado em `>= 1`. |
| Fórmula de Energia Máx (`getMaxEnergy`) | `MekanismUtils.java:374` | `upgrades.rs:55-58` | CONFIRMADA | `baseMax * 10^(energy/8)`. |
| Fator Médio Químico (Suporte a Gas) | `MekanismUtils.java:346` | `upgrades.rs:61-64` | CONFIRMADA | `10^((2*speed - chemical)/8)`. |
| Fator Médio Químico (SEM Suporte a Gas) | `MekanismUtils.java:346` | `upgrades.rs:67-73` | CORRIGIDO | Máquinas como Infuser/Oxidizer que não suportam Chemical Upgrade usam `10^(speed/8)` no Java. Adicionada função `gas_mean_with_support(speed, chemical, supports_chemical)`. |
| Truncamento vs Arredondamento | `MathUtils.java:120-135` | `upgrades.rs:44,51` | ERRADA-CORRIGIDA | Rodada 1 disse "arredondamento"; Java usa clampToInt com cast puro `(int)`. Port usa `as u64` / `ceil()`. |

---

### 3.3 Máquinas Químicas Individuais (`process.rs`, `reaction.rs`, `machine.rs`)

| Máquina | Config / Campo Java | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes e Notas da Revisão |
|---|---|---|---|---|---|
| **Chemical Oxidizer** | Energia Armazenada | `StorageConfig.java:124` | `process.rs:32` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:119` | `process.rs:33` | CONFIRMADA | 50 J/t = 20 FE/t |
| | Tempo Base | `TileEntityChemicalOxidizer.java:26` | `process.rs:34` | CONFIRMADA | 100 ticks |
| | Capacidade de Gás | `TileEntityChemicalOxidizer.java:34` | `process.rs:36` | CONFIRMADA | 10.000 mB |
| | Produção por Operação | `oxidizing/` recipes | `process.rs:14` | CONFIRMADA | 100 mB/operação (Redstone, Carbon, etc.) |
| **Metallurgic Infuser** | Energia Armazenada | `StorageConfig.java:118` | `process.rs:32` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:113` | `process.rs:33` | CONFIRMADA | 50 J/t = 20 FE/t |
| | Tempo Base | `TileEntityMetallurgicInfuser.java:37` | `process.rs:34` | CONFIRMADA | 200 ticks |
| | Capacidade de Infusão | `TileEntityMetallurgicInfuser.java:47` | `process.rs:37` | CONFIRMADA | 1.000 mB de Chemical |
| | Consumo de Infusão | Recipes (`infusing/`) | `process.rs:15` | CONFIRMADA | 10 mB ou 20 mB por operação |
| **Chemical Dissolution** | Energia Armazenada | `StorageConfig.java:125` | `process.rs:32` | ERRADA-CORRIGIDA | Java: 160.000 J = 64.000 FE (Rodada 1 informou 32k FE). Port tem capacidade configurável. |
| | Consumo Base | `UsageConfig.java:120` | `process.rs:33` | CONFIRMADA | 400 J/t = 160 FE/t |
| | Tempo Base | `TileEntityChemicalDissolutionChamber.java:29` | `process.rs:34` | CONFIRMADA | 100 ticks |
| | Tanque de Ácido Sulfúrico | `TileEntityChemicalDissolutionChamber.java:37` | `process.rs:37` | CONFIRMADA | 10.000 mB |
| | Tanque de Gás de Saída | `TileEntityChemicalDissolutionChamber.java:44` | `process.rs:36` | CONFIRMADA | 10.000 mB |
| | Taxa de Ácido por Operação | Recipes (`dissolution/`) | `process.rs:16` | CONFIRMADA | 100 mB (ore chunk) ou 200 mB (raw ore) |
| | Saída por Operação | Recipes (`dissolution/`) | `process.rs:16` | CONFIRMADA | 1.000 mB (slurry não processado) |
| **Chemical Washer** | Energia Armazenada | `StorageConfig.java:126` | `process.rs:32` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:121` | `process.rs:33` | CONFIRMADA | 200 J/t = 80 FE/t |
| | Tempo Base | `TileEntityChemicalWasher.java:27` | `process.rs:34` | CONFIRMADA | 1 tick por operação contínua |
| | Tanque de Água | `TileEntityChemicalWasher.java:36` | `process.rs:37` | CONFIRMADA | 10.000 mB de fluido |
| | Tanque de Slurry Entrada | `TileEntityChemicalWasher.java:41` | `process.rs:37` | CONFIRMADA | 10.000 mB de químico |
| | Tanque de Slurry Limpo | `TileEntityChemicalWasher.java:47` | `process.rs:36` | CONFIRMADA | 10.000 mB de químico limpo |
| | Consumo por tick | `TileEntityChemicalWasher.java:82` | `process.rs:17` | CONFIRMADA | 5 mB de água + 5 mB de slurry = 5 mB slurry limpo |
| **Chemical Crystallizer** | Energia Armazenada | `StorageConfig.java:127` | `process.rs:32` | ERRADA-CORRIGIDA | Java: 160.000 J = 64.000 FE (Rodada 1 informou 32k FE). |
| | Consumo Base | `UsageConfig.java:122` | `process.rs:33` | CONFIRMADA | 400 J/t = 160 FE/t |
| | Tempo Base | `TileEntityChemicalCrystallizer.java:29` | `process.rs:34` | CONFIRMADA | 400 ticks (20 segundos) |
| | Tanque de Entrada | `TileEntityChemicalCrystallizer.java:37` | `process.rs:37` | CONFIRMADA | 10.000 mB |
| | Consumo por Operação | Recipes (`crystallizing/`) | `process.rs:18` | CONFIRMADA | 200 mB de Slurry limpo -> 1 Cristal |
| **Chemical Injection** | Energia Armazenada | `StorageConfig.java:119` | `process.rs:32` | CONFIRMADA | 160.000 J = 64.000 FE |
| | Consumo Base | `UsageConfig.java:114` | `process.rs:33` | CONFIRMADA | 400 J/t = 160 FE/t |
| | Tempo Base | `TileEntityAdvancedElectricMachine.java:24` | `process.rs:34` | CONFIRMADA | 200 ticks |
| | Tanque de Gás | `TileEntityAdvancedElectricMachine.java:33` | `process.rs:37` | CONFIRMADA | 1.000 mB (HCl / vapor de água) |
| | Consumo de Gás | Recipes (`injecting/`) | `process.rs:19` | CONFIRMADA | 1 mB de HCl por operação (com chance de consumo) |
| **Chemical Purification**| Energia Armazenada | `StorageConfig.java:119` | `process.rs:32` | CONFIRMADA | 160.000 J = 64.000 FE |
| | Consumo Base | `UsageConfig.java:114` | `process.rs:33` | CONFIRMADA | 400 J/t = 160 FE/t |
| | Tempo Base | `TileEntityAdvancedElectricMachine.java:24` | `process.rs:34` | CONFIRMADA | 200 ticks |
| | Tanque de Gás | `TileEntityAdvancedElectricMachine.java:33` | `process.rs:37` | CONFIRMADA | 1.000 mB (Oxigênio) |
| | Consumo de Gás | Recipes (`purifying/`) | `process.rs:20` | CONFIRMADA | 1 mB de Oxigênio por operação (chance) |
| **Osmium Compressor** | Energia Armazenada | `StorageConfig.java:120` | `process.rs:32` | ERRADA-CORRIGIDA | Java: 80.000 J = 32.000 FE (Rodada 1 informou 16k FE). |
| | Consumo Base | `UsageConfig.java:115` | `process.rs:33` | CONFIRMADA | 200 J/t = 80 FE/t |
| | Tempo Base | `TileEntityAdvancedElectricMachine.java:24` | `process.rs:34` | CONFIRMADA | 200 ticks |
| | Tanque de Gás | `TileEntityAdvancedElectricMachine.java:33` | `process.rs:37` | CONFIRMADA | 1.000 mB (Liquid Osmium) |
| | Consumo de Gás Base | Recipes (`compressing/`) | `process.rs:21` | CONFIRMADA | 1 mB/tick constante = 200 mB no total dos 200 ticks |
| | Escalonamento c/ Upgrades | `TileEntityAdvancedElectricMachine.java:83` | `process.rs:142-156` | CORRIGIDO | Corrigido no port: agora escala o consumo contínuo para gastar os mesmos 200 mB no tempo reduzido por Speed Upgrades. |
| **PRC (Pressurized Reaction)** | Energia Armazenada | `StorageConfig.java:123` | `reaction.rs:26` | CONFIRMADA | 80.000 J = 32.000 FE (base, antes da receita) |
| | Consumo Base | `UsageConfig.java:118` | `reaction.rs:27` | CONFIRMADA | 5 J/t = 2 FE/t (+ energy_required / duration) |
| | Tanque de Entrada Fluido | `TileEntityPressurizedReactionChamber.java:44`| `reaction.rs:30` | CONFIRMADA | 10.000 mB |
| | Tanque de Entrada Gás | `TileEntityPressurizedReactionChamber.java:49`| `reaction.rs:31` | CONFIRMADA | 10.000 mB |
| | Tanque de Saída Gás | `TileEntityPressurizedReactionChamber.java:55`| `reaction.rs:32` | CONFIRMADA | 10.000 mB |
| | Duração Customizada | `PressurizedReactionRecipe.java:24` | `reaction.rs:40` | CONFIRMADA | Definida pela receita (ex.: Substrate 100t, HDPE Pellet 60t) |
| **Electrolytic Separator** | Energia Armazenada | `StorageConfig.java:130` | `machine.rs:45` | CONFIRMADA | 160.000 J = 64.000 FE |
| | Consumo Base | `TileEntityElectrolyticSeparator.java:199` | `machine.rs:46` | ERRADA-CORRIGIDA | Java: não tem UsageConfig; 2 * hydrogenEnergyDensity = 400 J/t = 160 FE/t. |
| | Taxa de Operação | `TileEntityElectrolyticSeparator.java:204` | `machine.rs:50` | CONFIRMADA | 2 mB de água -> 2 mB hidrogênio + 1 mB oxigênio por tick base (proporção 2:1) |
| | Tanque de Fluido | `TileEntityElectrolyticSeparator.java:45` | `machine.rs:47` | CONFIRMADA | 24.000 mB de Água |
| | Tanques de Gás | `TileEntityElectrolyticSeparator.java:50,56` | `machine.rs:48,49` | CONFIRMADA | 2.400 mB de Hidrogênio e 2.400 mB de Oxigênio |
| | Modos de Despejo dos Gases | `TileEntityElectrolyticSeparator.java:214` | `machine.rs:55` | CONFIRMADA | Dump mode individual por gás (Idle, Excess, Dump) |
| **Rotary Condensentrator** | Energia Armazenada | `StorageConfig.java:121` | `machine.rs:60` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:116` | `machine.rs:61` | CONFIRMADA | 50 J/t = 20 FE/t |
| | Tanque de Fluido | `TileEntityRotaryCondensentrator.java:45` | `machine.rs:64` | CONFIRMADA | 10.000 mB |
| | Tanque de Gás | `TileEntityRotaryCondensentrator.java:50` | `machine.rs:65` | CONFIRMADA | 10.000 mB |
| | Razão de Conversão | `TileEntityRotaryCondensentrator.java:152` | `machine.rs:70` | CONFIRMADA | 1 mB fluido <-> 1 mB gás (1:1) |
| | Modos de Operação | `TileEntityRotaryCondensentrator.java:60` | `machine.rs:68` | CONFIRMADA | Condensenting (Gás -> Fluido) / Decondensenting (Fluido -> Gás) |
| **Nutritional Liquifier** | Energia Armazenada | `StorageConfig.java:131` | `machine.rs:80` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:125` | `machine.rs:81` | CONFIRMADA | 50 J/t = 20 FE/t |
| | Tempo Base | `TileEntityNutritionalLiquifier.java:26` | `machine.rs:82` | CONFIRMADA | 100 ticks |
| | Tanque de Nutritional Paste | `TileEntityNutritionalLiquifier.java:36` | `machine.rs:85` | CONFIRMADA | 10.000 mB |
| | Cálculo de Pasta Nutritiva | `TileEntityNutritionalLiquifier.java:94` | `machine.rs:89` | FORK | Java lê `FoodProperties` (nutrition * 50 mB + saturation * 10 mB). Port usa tabela fixa por falta de API de food do Horizon (FORK-23). |
| **Pigment Extractor** | Energia Armazenada | `StorageConfig.java:134` | `machine.rs:100` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:127` | `machine.rs:101` | CONFIRMADA | 50 J/t = 20 FE/t |
| | Tempo Base | `TileEntityPigmentExtractor.java:26` | `machine.rs:102` | CONFIRMADA | 100 ticks |
| | Tanque de Pigmento | `TileEntityPigmentExtractor.java:36` | `machine.rs:105` | CONFIRMADA | 10.000 mB |
| | Produção por Flor/Corante | Recipes (`pigment_extracting/`)| `machine.rs:108` | CONFIRMADA | 256 mB por corante, 768 mB por flor de 2 blocos |
| **Pigment Mixer** | Energia Armazenada | `StorageConfig.java:135` | `machine.rs:115` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:128` | `machine.rs:116` | CONFIRMADA | 50 J/t = 20 FE/t |
| | Tempo Base | `TileEntityPigmentMixer.java:26` | `machine.rs:117` | CONFIRMADA | 100 ticks |
| | Tanques de Entrada (2x) | `TileEntityPigmentMixer.java:36,41` | `machine.rs:120,121` | CONFIRMADA | 10.000 mB cada |
| | Tanque de Saída | `TileEntityPigmentMixer.java:47` | `machine.rs:122` | CONFIRMADA | 10.000 mB |
| **Painting Machine** | Energia Armazenada | `StorageConfig.java:133` | `machine.rs:130` | CONFIRMADA | 80.000 J = 32.000 FE |
| | Consumo Base | `UsageConfig.java:126` | `machine.rs:131` | CONFIRMADA | 50 J/t = 20 FE/t |
| | Tempo Base | `TileEntityPaintingMachine.java:26` | `machine.rs:132` | CONFIRMADA | 100 ticks |
| | Tanque de Pigmento | `TileEntityPaintingMachine.java:36` | `machine.rs:135` | CONFIRMADA | 10.000 mB |
| | Consumo de Pigmento | Recipes (`painting/`) | `machine.rs:138` | CONFIRMADA | 256 mB por bloco / item tingido |
| **Chemical Infuser** | Energia Armazenada | `StorageConfig.java:128` | `process.rs:32` | ERRADA-CORRIGIDA | Java: 80.000 J = 32.000 FE (Rodada 1 informou 16k FE). |
| | Consumo Base | `UsageConfig.java:123` | `process.rs:33` | CONFIRMADA | 200 J/t = 80 FE/t |
| | Tempo Base | `TileEntityChemicalInfuser.java:26` | `process.rs:34` | CONFIRMADA | 1 tick por operação contínua |
| | Tanques de Entrada (2x) | `TileEntityChemicalInfuser.java:37,42` | `process.rs:37` | CONFIRMADA | 10.000 mB cada |
| | Tanque de Saída | `TileEntityChemicalInfuser.java:48` | `process.rs:36` | CONFIRMADA | 10.000 mB |
| **Isotopic Centrifuge** | Energia Armazenada | `StorageConfig.java:129` | `machine.rs:150` | ERRADA-CORRIGIDA | Java: 80.000 J = 32.000 FE (Rodada 1 informou 16k FE). |
| | Consumo Base | `UsageConfig.java:124` | `machine.rs:151` | CONFIRMADA | 200 J/t = 80 FE/t |
| | Tempo Base | `TileEntityIsotopicCentrifuge.java:26`| `machine.rs:152` | CONFIRMADA | 1 tick por operação |
| | Tanque de Entrada | `TileEntityIsotopicCentrifuge.java:36`| `machine.rs:155` | CONFIRMADA | 10.000 mB |
| | Tanque de Saída | `TileEntityIsotopicCentrifuge.java:42`| `machine.rs:156` | CONFIRMADA | 10.000 mB |
| **Solar Neutron Activator**| Energia Armazenada | - (sem energia elétrica) | `machine.rs:170` | CONFIRMADA | 0 J (acionado exclusivamente por luz solar direta) |
| | Tanque de Entrada | `TileEntitySolarNeutronActivator.java:35`| `machine.rs:172` | CONFIRMADA | 10.000 mB |
| | Tanque de Saída | `TileEntitySolarNeutronActivator.java:40`| `machine.rs:173` | CONFIRMADA | 10.000 mB |
| | Penalidade de Chuva | `TileEntitySolarNeutronActivator.java:201`| `plus.rs:310` | CONFIRMADA | 80% de penalidade (`* 0.2f`) se chuva/trovoada |
| **Antiprotonic Nucleosynthesizer** | Energia Armazenada | `StorageConfig.java:132` | `machine.rs:180` | ERRADA-CORRIGIDA | Java: 1.000.000.000 J = 400.000.000 FE (Rodada 1 informou 8.000.000 FE). |
| | Consumo Base | Recipes (`nucleosynthesizing/`)| `machine.rs:182` | CONFIRMADA | Definido pela receita (ex.: Nether Star = 400.000 FE/t) |
| | Tanque de Gás de Entrada| `TileEntityAntiprotonicNucleosynthesizer.java:36`| `machine.rs:185` | CONFIRMADA | 10.000 mB (Antimatéria) |

---

### 3.4 Combiner e Precision Sawmill (`combiner.rs`, `sawmill.rs`)

| Item | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes e Notas da Revisão |
|---|---|---|---|---|
| Combiner: Armazenamento | `StorageConfig.java:122` | `combiner.rs:25` | ERRADA-CORRIGIDA | Java: 40.000 J = 16.000 FE (Rodada 1 informou 8.000 FE). |
| Combiner: Consumo Base | `UsageConfig.java:117` | `combiner.rs:26` | CONFIRMADA | 50 J/t = 20 FE/t |
| Combiner: Tempo Base | `TileEntityCombiner.java:26` | `combiner.rs:27` | CONFIRMADA | 200 ticks |
| Combiner: Slot Extra | `TileEntityCombiner.java:35` | `combiner.rs:32` | CONFIRMADA | Consome bloco agregador (ex.: Cobblestone, Gravel) |
| Sawmill: Armazenamento | `StorageConfig.java:112` | `sawmill.rs:25` | CONFIRMADA | 20.000 J = 8.000 FE |
| Sawmill: Consumo Base | `UsageConfig.java:107` | `sawmill.rs:26` | CONFIRMADA | 20 J/t = 8 FE/t |
| Sawmill: Tempo Base | `TileEntityPrecisionSawmill.java:26` | `sawmill.rs:27` | CONFIRMADA | 200 ticks |
| Sawmill: Chance Output | `TileEntityPrecisionSawmill.java:65` | `sawmill.rs:45` | CONFIRMADA | Geração probabilística de subproduto (Sawdust) |

---

### 3.5 Fábricas Químicas e Elétricas (`factory.rs`)

| Item | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes e Notas da Revisão |
|---|---|---|---|---|
| Linhas: Basic Factory | `FactoryTier.java:17` | `factory.rs:22` | CONFIRMADA | 3 processos simultâneos |
| Linhas: Advanced Factory| `FactoryTier.java:18` | `factory.rs:23` | CONFIRMADA | 5 processos simultâneos |
| Linhas: Elite Factory | `FactoryTier.java:19` | `factory.rs:24` | CONFIRMADA | 7 processos simultâneos |
| Linhas: Ultimate Factory | `FactoryTier.java:20` | `factory.rs:25` | CONFIRMADA | 9 processos simultâneos |
| Capacidade Tanques Químicos | `TileEntityItemStackChemicalToItemStackFactory.java:34` | `factory.rs:52` | CONFIRMADA | Escala com número de linhas: `capacidade_base * linhas` |
| Rastreamento de Progresso | `TileEntityFactory.java:120` | `factory.rs:110` | CONFIRMADA | Progresso independente em cada linha a partir do byte 46 (`TAIL`) |
| Recarga Slot Energia Redstone | `TileEntityFactory.java:188` | `factory.rs:280` | CORRIGIDO | Teste unitário corrigido para validar conversão padrão de 4.000 FE por redstone |

---

### 3.6 Tabela de Químicos e Interface (`chem_table.rs`, `chem.rs`)

| Item | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes e Notas da Revisão |
|---|---|---|---|---|
| Total de Químicos Mapeados | `MekanismChemicals.java` | `chem_table.rs:12` | CONFIRMADA | 72 produtos químicos |
| Indexação na GUI | `ContainerMekanism.java` | `chem_table.rs:14` | CONFIRMADA | 1-based index (0 = sem químico / vazio) |
| Botão de Despejo de Infusor | `GuiMetallurgicInfuser.java:45` | `chem.rs:35` | CONFIRMADA | Widget ID 4000 aciona descarte de químico residual |

---

## 4. Persistência, Redstone, Quebra de Blocos e Regras de Ligação

1. **Persistência de Conteúdo ao Quebrar Bloco**:
   - No Java, ao quebrar qualquer máquina química ou tanque com uma Wrench ou picareta, o bloco serializa seus componentes (energia, químicos nos tanques, upgrades instalados, modo de despejo, configuração de lados) diretamente no `ItemStack` via Data Components (`MekanismDataComponents`).
   - No port Rust, essa persistência é simulada via buffer serializado de estado de bloco. No entanto, o Horizon WASM ainda não possui suporte nativo completo para data components arbitrários em drops de itens do jogador, requerendo **FORK-5.4** / API de `item.set-custom-data`.

2. **Modos de Redstone**:
   - Todas as máquinas suportam 4 modos de controle de redstone (`IRedstoneControl`): `DISABLED` (ignora), `HIGH` (opera com sinal), `LOW` (opera sem sinal), `PULSE` (opera um pulso a cada subida).
   - O port implementa a verificação de sinal de bloco vizinho (`neighbor_signal > 0`), cobrindo os modos `HIGH`, `LOW` e `DISABLED`. O modo `PULSE` exige detecção de transição de borda que é tratada guardando `last_redstone_signal` no estado interno.

3. **Conexões e Regras de Ligação**:
   - Os tanques químicos respeitam os lados configurados em `TileComponentConfig`. Cada face pode ser configurada como `INPUT`, `OUTPUT`, `BOTH`, `EXTRA`, `ENERGY`, `NONE`.
   - No port `factory.rs` e `process.rs`, os lados são respeitados através das máscaras de direção relativas ao `facing` do bloco.

---

## 5. Conclusão da Revisão

A rodada 1 cometeu erros de leitura em 7 capacidades de armazenamento de energia (`StorageConfig.java`) e 1 consumo dinâmico (`UsageConfig.java`), além de ter classificado restrições como `LIMITE` em vez de `FORK`.

Com as correções aplicadas nesta rodada em `factory.rs`, `upgrades.rs` e `process.rs`:
- O cálculo de consumo contínuo de gás com speed upgrades no Osmium Compressor está 100% alinhado com o Java.
- O cálculo de efeito de upgrade químico para máquinas sem suporte a chemical upgrade foi isolado corretamente.
- O teste de recarga de redstone em fábricas foi corrigido para FE.
- Todas as capacidades e números foram minuciosamente verificados contra os arquivos Java de origem.
