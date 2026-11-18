# Auditoria FISSION REACTOR, FUSION REACTOR e INDUSTRIAL TURBINE

**Escopo:** `port/mod/src/fission.rs`, `port/mod/src/fusion.rs`, `port/mod/src/turbine.rs`  
**Referência:** Mekanism 1.21.1-10.7.19.85 (`java/`)  
**Conversão de energia:** $1\text{ FE} = 2.5\text{ J}$ ($J \times 2/5$), via `fe()` em `port/mod/src/lib.rs`.

---

## 1. Tabela Comparativa Completa

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| **FISSION REACTOR** | | | |
| Limites de dimensão (min/max) | 3×3×3 a 18×18×18 (`CuboidStructureValidator.java:34`) | 3..=18 em x, y, z (`fission.rs:89`) | OK |
| Capacidade de calor da carcaça (`casingHeatCapacity`) | 1.000 J/K por bloco (`GeneratorsConfig.java:146`, `FissionReactorMultiblockData.java:177`) | 1.000,0 (`fission.rs:38`, `fission.rs:266`) | OK |
| Calor gerado por combustível (`energyPerFissionFuel`) | 1.000.000 J/mB (`GeneratorsConfig.java:144`, `FissionReactorMultiblockData.java:479`) | 1.000.000,0 (`fission.rs:37`, `fission.rs:279`) | OK |
| Taxa de queima padrão (`defaultBurnRate`) | 0,1 mB/t (`GeneratorsConfig.java:150`, `FissionReactorMultiblockData.java:128`) | 0,1 (`fission.rs:40`) | OK |
| Queima máx por montagem (`burnPerAssembly`) | 1 mB/t (`GeneratorsConfig.java:152`, `FissionReactorMultiblockData.java:475`) | 1,0 (`fission.rs:41`, `fission.rs:241`) | OK |
| Alvo de área superficial (`surfaceAreaTarget`) | 4,0 (`GeneratorsConfig.java:148`, `FissionReactorMultiblockData.java:566`) | 4,0 (`fission.rs:39`, `fission.rs:248`) | OK |
| Capacidade combustível por montagem (`maxFuelPerAssembly`) | 8.000 mB (`GeneratorsConfig.java:154`, `FissionReactorMultiblockData.java:582`) | 8.000 mB (`fission.rs:42`, `fission.rs:252`) | OK |
| Capacidade refrigerante frio por vol (`cooledCoolantPerTank`) | 100.000 mB (`GeneratorsConfig.java:157`, `FissionReactorMultiblockData.java:590`) | 100.000 mB (`fission.rs:43`, `fission.rs:256`) | OK |
| Capacidade refrigerante quente por vol (`heatedCoolantPerTank`) | 1.000.000 mB (`GeneratorsConfig.java:159`, `FissionReactorMultiblockData.java:591`) | 1.000.000 mB (`fission.rs:44`, `fission.rs:260`) | OK |
| Capacidade de resíduo (`wasteTank`) | `fuelCapacity` = assemblies × 8.000 mB (`FissionReactorMultiblockData.java:161`) | `fuel_capacity(&found)` (`fission.rs:440`) | OK |
| Temperatura base de ebulição (`BASE_BOIL_TEMP`) | 373,15 K (273,15 + 100) (`HeatUtils.java:18`) | 373,15 K (`fission.rs:30`) | OK |
| Entalpia térmica da água (`waterThermalEnthalpy`) | 50,0 J/mB (10,0 / 0,2) (`HeatUtils.java:21-25`, `GeneralConfig.java:202`) | 50,0 (10,0 / 0,2) (`fission.rs:31`) | OK |
| Condutividade térmica da água (`waterConductivity`) | 0,5 (`FissionReactorMultiblockData.java:73`) | 0,5 (`fission.rs:29`) | OK |
| Entalpia térmica do sódio (`thermal_enthalpy`) | 5,0 J/mB (`MekanismDataMapsProvider.java:85`) | 5,0 (`fission.rs:32`) | OK |
| Condutividade do sódio (`conductivity`) | 1,0 (`MekanismDataMapsProvider.java:85`, `CooledCoolant.java:38`) | Era 0,4; corrigido para 1,0 (`fission.rs:33`) | CORRIGIDO |
| Coeficiente inverso de isolamento (`INVERSE_INSULATION`) | 10.000 (`FissionReactorMultiblockData.java:70`) | 10.000,0 (`fission.rs:26`) | OK |
| Coeficiente inverso de condução (`INVERSE_CONDUCTION`) | 10 (`FissionReactorMultiblockData.java:71`) | 10,0 (`fission.rs:27`) | OK |
| Coeficiente inverso do ar (`AIR_INVERSE_COEFFICIENT`) | 10.000 (`HeatAPI.java:24`, `FissionReactorMultiblockData.java:252`) | Era 10,0; corrigido para 10.000,0 (`fission.rs:28`) | CORRIGIDO |
| Temperatura mínima para dano (`MIN_DAMAGE_TEMPERATURE`) | 1.200 K (`FissionReactorMultiblockData.java:75`) | 1.200,0 K (`fission.rs:34`) | OK |
| Temperatura máxima de escalonamento de dano (`MAX_DAMAGE_TEMP`) | 1.800 K (`FissionReactorMultiblockData.java:76`) | 1.800,0 K (`fission.rs:35`) | OK |
| Dano máximo antes do colapso (`MAX_DAMAGE`) | 100 (`FissionReactorMultiblockData.java:77`) | 100,0 (`fission.rs:36`) | OK |
| Chance de colapso por tick (`fissionMeltdownChance`) | 0,001 (`GeneratorsConfig.java:169`, `FissionReactorMultiblockData.java:320`) | 0,001 (`fission.rs:45`, `fission.rs:333`) | OK |
| Raio do colapso (`fissionMeltdownRadius`) | 8 blocos (`GeneratorsConfig.java:167`, `FissionReactorMultiblockData.java:338`) | 8 blocos (`fission.rs:46`, `fission.rs:607`) | OK |
| Multiplicador de radiação do colapso | 50× (`GeneratorsConfig.java:171`, `FissionReactorMultiblockData.java:350`) | 50× (`fission.rs:509`) | OK |
| Dano pós-colapso (`fissionPostMeltdownDamage`) | 75,0 (0,75 × MAX_DAMAGE) (`GeneratorsConfig.java:173`, `FissionReactorMultiblockData.java:359`) | 75,0 (`fission.rs:47`, `fission.rs:345`) | OK |
| Radiação do resíduo com tanque vazio no meltdown | Usa radioatividade padrão de `NUCLEAR_WASTE` para `partialWaste` (`FissionReactorMultiblockData.java:381-394`) | Corrigido para aplicar `WASTE` como fallback se `waste.amount == 0` (`fission.rs:505`) | CORRIGIDO |
| Radiação por resíduo excedente em operação normal | `(new_waste - kept) * radioactivity` (`FissionReactorMultiblockData.java:488-494`) | `(new_waste - kept) * radioactivity(WASTE)` (`fission.rs:292`) | OK |
| Radiação em entidades na zona quente | `IRadiationManager.radiate(entity, magnitude)` (`FissionReactorMultiblockData.java:515`) | Não suportado por limitações da API WASM do Horizon | LIMITE |
| Modos de porta do reator | 3 modos (Input, Output Waste, Output Coolant) via configurador (`TileEntityFissionReactorPort.java:70`) | 3 modos configuráveis via `with_mode` / configurator (`fission.rs:218-238, 567-587`) | OK |
| Logic Adapter (sinais de redstone / computador) | Modos Damage, Critical, Temperature, Excess Waste, Scram, etc. (`TileEntityFissionReactorLogicAdapter.java:50`) | Bloco estrutural sem lógica de redstone | LIMITE |
| **FUSION REACTOR** | | | |
| Geometria e dimensões | Cuboide 5×5×5 arredondado (72 blocos vazios, 12 por face), controlador no topo central (2,4,2), interior 3×3×3 oco (`FusionReactorValidator.java:24-80`) | Validação com `GRID` idêntica, controlador central superior (`fusion.rs:47, 85-142`) | OK |
| Temperatura mínima de queima (`burnTemperature`) | 100.000.000 K (`FusionReactorMultiblockData.java:71`) | 100.000.000,0 K (`fusion.rs:27`) | OK |
| Capacidade de calor do plasma (`plasmaHeatCapacity`) | 100 (`FusionReactorMultiblockData.java:74`) | 100,0 (`fusion.rs:28`) | OK |
| Capacidade de calor da carcaça (`caseHeatCapacity`) | 1 (`FusionReactorMultiblockData.java:75`) | 1,0 (`fusion.rs:29`) | OK |
| Condutividade plasma-carcaça (`plasmaCaseConductivity`) | 0,2 (`FusionReactorMultiblockData.java:78`) | 0,2 (`fusion.rs:30`) | OK |
| Energia por combustível de fusão (`energyPerFusionFuel`) | 10.000.000 J (`GeneratorsConfig.java:179`, `FusionReactorMultiblockData.java:325`) | 10.000.000,0 J (`fusion.rs:31`) | OK |
| Eficiência do termopar (`fusionThermocoupleEfficiency`) | 0,05 (`GeneratorsConfig.java:181`, `FusionReactorMultiblockData.java:358`) | 0,05 (`fusion.rs:32`) | OK |
| Condutividade térmica da carcaça (`casingThermalConductivity`) | 0,1 (`GeneratorsConfig.java:183`, `FusionReactorMultiblockData.java:355`) | 0,1 (`fusion.rs:33`) | OK |
| Razão de aquecimento de água (`fusionWaterHeatingRatio`) | 0,3 (`GeneratorsConfig.java:185`, `FusionReactorMultiblockData.java:338`) | 0,3 (`fusion.rs:34`) | OK |
| Capacidade dos tanques de combustível (`fusionFuelCapacity`) | 1.000 mB por tanque (Deutério, Trítio, D-T) (`GeneratorsConfig.java:187`, `FusionReactorMultiblockData.java:149-153`) | 1.000 mB (`fusion.rs:35`, `fusion.rs:406`) | OK |
| Capacidade do buffer de energia (`fusionEnergyCapacity`) | 1.000.000.000 J (`GeneratorsConfig.java:189`, `FusionReactorMultiblockData.java:157`) | 400.000.000 FE (`fe(1_000_000_000)`) (`fusion.rs:36`) | OK |
| Água por taxa de injeção (`fusionWaterPerInjection`) | 1.000.000 mB (`GeneratorsConfig.java:192`, `FusionReactorMultiblockData.java:408`) | 1.000.000 mB (`fusion.rs:37`, `fusion.rs:201`) | OK |
| Vapor por taxa de injeção (`fusionSteamPerInjection`) | 100.000.000 mB (`GeneratorsConfig.java:194`, `FusionReactorMultiblockData.java:409`) | 100.000.000 mB (`fusion.rs:38`, `fusion.rs:205`) | OK |
| Capacidade de gás do Hohlraum (`hohlraumMaxGas`) | 10 mB D-T Fuel (`GeneratorsConfig.java:79`) | 10 mB (`fusion.rs:39`) | OK |
| Taxa de injeção padrão (`injectionRate`) | 2 mB/t (`FusionReactorMultiblockData.java:125`) | 2 (`fusion.rs:40`) | OK |
| Taxa de injeção máxima (`MAX_INJECTION`) | 98 mB/t (`FusionReactorMultiblockData.java:69`) | 98 (`fusion.rs:41`) | OK |
| Aquecimento por laser (`addTemperatureFromEnergyInput`) | `energy / plasmaHeatCapacity` (10× pré-ignição) (`FusionReactorMultiblockData.java:195-201`) | `(energy * 2.5 / 100)` (10× pré-ignição) (`fusion.rs:210-216`) | OK |
| Ajuste de taxa de injeção | Passos de ±2, clamp 0..98 (`FusionReactorMultiblockData.java:406-420`) | Passos de ±2 via botões (`fusion.rs:317-326`) | OK |
| Cálculo de temperatura máx do plasma (`getMaxPlasmaTemperature`) | `injectionRate * energyPerFuel / conductivity * ...` (`FusionReactorMultiblockData.java:462-468`) | Fórmula idêntica em `max_plasma` (`fusion.rs:312-315`) | OK |
| Consumo e queima do Hohlraum | Ignição com Hohlraum cheio no slot quando plasma >= 100M K (`FusionReactorMultiblockData.java:224-226, 300-309`) | Consome Hohlraum e inicia queima (`fusion.rs:222-235`) | OK |
| Hohlraum com capacidade química dinâmica (NBT) | O item possui capacidade para 10 mB e pode ser enchido gradualmente (`ItemHohlraum.java`) | Item estático considerado cheio (limitação da stack Horizon) | LIMITE |
| Dano letal no núcleo (`kill`) | 50.000 de dano a entidades dentro da câmara (`FusionReactorMultiblockData.java:291-297`) | Ausência de API de dano a entidades de área | LIMITE |
| **INDUSTRIAL TURBINE** | | | |
| Limites de dimensão | L e C ímpares 5..17, Altura 3..18 (`TurbineValidator.java:37, 64-66`) | L e C ímpares 5..=17, Altura 3..=18 (`turbine.rs:85-87`) | OK |
| Lâminas máximas (`MAX_BLADES`) | 28 (`TurbineValidator.java:34`) | 28,0 (`turbine.rs:28`) | OK |
| Lâminas por bobina (`turbineBladesPerCoil`) | 4 (`GeneratorsConfig.java:128`) | 4 (`turbine.rs:29`) | OK |
| Vazão por ventilação (`turbineVentChemicalFlow`) | 32.000 mB/t (`GeneratorsConfig.java:130`) | 32.000,0 (`turbine.rs:30`) | OK |
| Vazão por dispersor (`turbineDisperserChemicalFlow`) | 1.280 mB/t (`GeneratorsConfig.java:132`) | 1.280,0 (`turbine.rs:31`) | OK |
| Taxa de condensação (`condenserRate`) | 64.000 mB/t (`GeneratorsConfig.java:134`) | 64.000 (`turbine.rs:32`) | OK |
| Capacidade de energia por volume (`turbineEnergyCapacityPerVolume`) | 16.000.000 J (`GeneratorsConfig.java:136`, `TurbineMultiblockData.java:246`) | 6.400.000 FE (`fe(16_000_000)`) (`turbine.rs:34`) | OK |
| Capacidade de vapor por volume inferior (`turbineChemicalPerTank`) | 64.000 mB (`GeneratorsConfig.java:140`, `TurbineMultiblockData.java:235`) | 64.000 (`turbine.rs:35`) | OK |
| Energia máxima por vapor (`maxEnergyPerSteam`) | 10 J/mB (`GeneralConfig.java:202`, `TurbineMultiblockData.java:141`) | 10,0 (`turbine.rs:37`) | OK |
| Multiplicador de energia | `(10 / 28) * min(blades, coils * 4)` (`TurbineMultiblockData.java:141-142`) | `(10.0 / 28.0) * min(blades, coils * 4)` (`turbine.rs:253-255`) | OK |
| Vazão máxima (`max_flow`) | `min(lowerVolume * dispersers * 1280, vents * 32000)` (`TurbineMultiblockData.java:146-147`) | `min(lower_volume * dispersers * 1280, vents * 32000)` (`turbine.rs:249-251`) | OK |
| Comportamento de transbordamento de água dos condensadores | Turbina não desliga se o tanque de ventilação estiver cheio; excesso de água não cabe e é descartado (`TurbineMultiblockData.java:154-156`) | Port parava a turbina se `water + produced > cap`; corrigido para limitar água gerada sem travar a rotação/geração (`turbine.rs:298-303`) | CORRIGIDO |
| Adição e remoção de lâminas no rotor | Máximo 2 lâminas por rotor; preenche de baixo para cima, remove de cima para baixo (`TileEntityTurbineRotor.java:80-137`) | Algoritmo idêntico em `add_blade` / `remove_blade` (`turbine.rs:224-246`) | OK |
| Modos de despejo de vapor (Dumping Mode) | Idle, Dumping, Dumping Excess (`TurbineMultiblockData.java:168-179`) | Sem suporte a modos de despejo (DIFERENCAS 4.5.1) | LIMITE |

---

## 2. Detalhamento das Correções Realizadas

### 2.1 `port/mod/src/fission.rs`
1. **`AIR` (Coeficiente inverso do ar):**
   - No Java (`HeatAPI.java:24`): `AIR_INVERSE_COEFFICIENT = 10_000`.
   - No port Rust estava configurado como `10.0`, o que fazia a perda térmica para o ambiente ser quase duas vezes mais rápida do que no original (`10.020` vs `20.010` de resistência térmica).
   - Alterado para `pub const AIR: f64 = 10_000.0;`.
2. **`SODIUM_CONDUCTIVITY` (Condutividade térmica do refrigerante sódio):**
   - No Java 1.21.1 (`MekanismDataMapsProvider.java:85` e `CooledCoolant.java:38`), o sódio possui entalpia 5,0 e condutividade 1,0.
   - No port Rust estava como `0.4` (suposição herdada de versões legadas).
   - Alterado para `pub const SODIUM_CONDUCTIVITY: f64 = 1.0;`.
3. **Radiação de resíduo no Meltdown quando o tanque está vazio:**
   - No Java (`FissionReactorMultiblockData.java:381-394`), se o tanque de resíduo está vazio (`stored.isEmpty()`), a radioatividade do lixo nuclear padrão (`MekanismChemicals.NUCLEAR_WASTE` = 0.01) é usada para multiplicar o resíduo residual (`partialWaste`).
   - No Rust (`fission.rs:505`), se `s.reactor.waste.amount == 0`, a string química ficava vazia (`""`), resultando em radioatividade 0 para `partial_waste`.
   - Corrigido para selecionar `WASTE` como fallback caso o tanque esteja vazio.

### 2.2 `port/mod/src/turbine.rs`
1. **Comportamento do tanque de água condensada (`step`):**
   - No Java (`TurbineMultiblockData.java:154-156`), o `ventTank.insert` recebe a água condensada e, caso o tanque não tenha espaço suficiente, o excedente é simplesmente descartado; a geração de energia e o consumo de vapor **não** são interrompidos.
   - No Rust (`turbine.rs:299-302`), havia uma checagem restritiva `if water + produced > water_capacity(f) { return Tick::default(); }`, o que paralisava a turbina inteira se os reservatórios de água estivessem cheios.
   - Corrigido para calcular `room = water_capacity(f).saturating_sub(water)` e `produced = flow.min(water_capacity(f)).min(room)`, permitindo que a turbina continue gerando energia continuamente mesmo com tanque de água cheio.
   - O teste `steam_turns_the_blades_into_energy_and_water` foi atualizado para validar que um tanque de água cheio não impede a geração de energia (`full_water.energy > 0 && full_water.water == 0`).

---

## 3. Bloco para `port/DIFERENCAS.md` (Seção 20)

```markdown
## 20. Reatores e Turbina (mod 0.1.86, auditoria Java 1.21.1-10.7.19.85)

Auditoria completa de constantes, capacidades, condutividades e algoritmos de `fission.rs`, `fusion.rs` e `turbine.rs` contra os fontes do Mekanism Java 1.21.1 (`GeneratorsConfig`, `FissionReactorMultiblockData`, `FusionReactorMultiblockData`, `TurbineMultiblockData` e registros de DataMaps).

| # | Diferença | Estado |
|---|---|---|
| 20.1 | Fission: constante `AIR` corrigida de 10,0 para 10.000,0 (`HeatAPI.AIR_INVERSE_COEFFICIENT`); condutividade do sódio corrigida de 0,4 para 1,0 (`MekanismDataMapsProvider` `CooledCoolant`); fallback de radiação para `partial_waste` no colapso | CORRIGIDO |
| 20.2 | Turbina: acúmulo de água no tanque das ventilações (`ventTank`) não interrompe mais o funcionamento da turbina; vapor continua gerando energia e água é condensada até o limite de espaço livre | CORRIGIDO |
| 20.3 | Fusion: todas as 15 constantes físicas e de configuração conferidas e 100% idênticas ao release (temperatura de queima 100M K, capacidades térmicas 100/1, condutividade plasma-carcaça 0,2, energia 10M J/mB, etc.) | OK |
| 20.4 | Fission e Fusion: dano letal a entidades na câmara do reator (zona quente do fission e 50.000 de dano na câmara de fusão) não implementados por falta de API de colisão/dano em área no Horizon | LIMITE |
| 20.5 | Fusion: Hohlraum continua operando como item simples com combustível embutido (10 mB) consumido ao ignificar, por não haver química individualizada em itens na stack atual | LIMITE |
| 20.6 | Turbina: modos de descarte de vapor (Dumping Mode: Idle, Dumping, Dumping Excess) ausentes na interface simplificada | ABERTA |
| 20.7 | Fission: portas possuem 3 modos pelo configurador (input, output waste, output coolant); Logic Adapter atua como bloco passivo sem portas lógicas de redstone/computador | ABERTA |
```
