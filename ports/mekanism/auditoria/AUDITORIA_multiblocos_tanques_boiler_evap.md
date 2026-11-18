# Relatório de Auditoria: Multiblocos (Dynamic Tank, Thermoelectric Boiler e Thermal Evaporation)

**Referência**: Mekanism 1.21.1-10.7.19.85 (Java)  
**Port**: Horizon Mod WASM Rust (`port/mod/src/dynamic.rs`, `port/mod/src/boiler.rs`, `port/mod/src/evaporation.rs`)  
**Data**: 2026-11-09  
**ID da Tarefa**: T20261109-190810-jtm  

---

## 1. Tabela Comparativa Geral

| Escopo / Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado | Detalhes / Observações |
|---|---|---|---|---|
| **DYNAMIC TANK** | | | | |
| Tamanho mín/máx | 3x3x3 a 18x18x18 (`CuboidStructureValidator.java:34`) | 3x3x3 a 18x18x18 (`dynamic.rs:35`) | OK | Validado via `(3..=18).contains(s)` para cada eixo. |
| Capacidade Fluido por bloco | 350.000 mB/bloco (`GeneralConfig.java:169`) | 350.000 mB/bloco (`dynamic.rs:16`, `dynamic.rs:152`) | CORRIGIDO | Estava incorretamente usando 16.000.000 mB/bloco para fluido e químico. Separado em `FLUID_PER_BLOCK = 350_000` e aplicado nas válvulas, buckets e merge. |
| Capacidade Químico por bloco | 16.000.000 mB/bloco (`GeneralConfig.java:171`) | 16.000.000 mB/bloco (`dynamic.rs:18`, `dynamic.rs:153`) | OK | `CHEMICAL_PER_BLOCK = 16_000 * 1000`. Químicos usam 16.000 buckets/bloco interno. |
| Tipo de conteúdo exclusivo | `MergedTank` fluido OU químico (`TankMultiblockData.java:49`) | `shared.fluid` OU `shared.chemical` (`dynamic.rs:198-200`) | OK | Se ambos ficarem > 0, o químico é descartado em favor do fluido. |
| Regras de Válvula | Face/parede apenas, não em arestas/moldura (`CuboidStructureValidator.java:115`) | Face/parede apenas (`dynamic.rs:51`) | OK | `(e, Some(true)) if e > 1 => return bad`. Válvulas em arestas são rejeitadas. |
| Vidro Estrutural | Permitido em paredes/faces (`TileEntityStructuralGlass.java:21`, `CuboidStructureValidator.java:115`) | Permitido em carcaça (`lib.rs:1942`, `dynamic.rs:38`) | OK / LIMITE | No port, o vidro estrutural é aceito como casing no grupo `dynamic::GROUP`. |
| Capacidade no Menu GUI | Mostra químico por padrão (vazio/químico) ou fluido se presente (`GuiDynamicTank.java:51-60`) | Mostra `chem_cap` quando vazio/químico ou `fluid_cap` quando fluido presente (`dynamic.rs:217-218`) | CORRIGIDO | Anteriormente usava o mesmo `cap` de 16M fixo para ambos. |
| Slots de Balde | 2 slots (input drain/fill e output fill/drain) (`TankMultiblockData.java:60`) | 2 slots gerenciados via `tank::buckets` (`dynamic.rs:208-212`, `lib.rs:1772`) | CORRIGIDO | Capacidade máxima usada na operação de balde corrigida para `fluid_cap`. |
| **THERMOELECTRIC BOILER** | | | | |
| Tamanho mín/máx | 3x3x3 a 18x18x18 (`CuboidStructureValidator.java:34`) | 3x3x3 a 18x18x18 (`boiler.rs:81`) | OK | `(3..=18).contains(s)`. |
| Capacidade Água por bloco | 16.000 mB/bloco (`GeneralConfig.java:286`) | 16.000 mB/bloco (`boiler.rs:40`) | OK | `WATER_PER = 16_000` mB/bloco de volume de água. |
| Capacidade Vapor por bloco | 160.000 mB/bloco (`GeneralConfig.java:288`) | 160.000 mB/bloco (`boiler.rs:41`) | OK | `STEAM_PER = 160_000` mB/bloco de volume de vapor. |
| Capacidade Refrigerante Quente | 256.000 mB/bloco (`GeneralConfig.java:289`) | 256.000 mB/bloco (`boiler.rs:42`) | OK | `COOLANT_PER = 256_000` mB/bloco de volume de água. |
| Capacidade Refrigerante Resfriado | 256.000 mB/bloco (`GeneralConfig.java:290`) | 256.000 mB/bloco (`boiler.rs:42`) | OK | `COOLANT_PER = 256_000` mB/bloco de volume de vapor. |
| Capacidade Térmica da Carcaça | 50.0 por bloco (`BoilerMultiblockData.java:62`) | 50.0 por bloco (`boiler.rs:25`) | OK | `CASING_HEAT = 50.0`. |
| Coeficiente Inverso de Isolamento | 100.000,0 (`BoilerMultiblockData.java:63`) | 100.000,0 (`boiler.rs:26`) | OK | `INSULATION = 100_000.0`. |
| Coeficiente Inverso de Condução | 1,0 (`BoilerMultiblockData.java:64`) | 1,0 (`boiler.rs:27`) | OK | `CONDUCTION = 1.0`. |
| Coeficiente Inverso do Ar | 10.000,0 (`HeatAPI.java:24`) | 10.000,0 (`boiler.rs:28`) | CORRIGIDO | Estava `10.0` no port (diferença de 1000x na taxa de resfriamento com ar ambiente). Corrigido para `10_000.0`. |
| Temperatura Base de Ebulição | 373,15 K (`HeatUtils.java:18`) | 373,15 K (`boiler.rs:30`) | OK | `BOIL = 373.15`. |
| Entalpia Térmica Água/Vapor | 10,0 J/mB (`HeatUtils.java:21`, `GeneralConfig.java:202`) | 10,0 (`boiler.rs:31`) | OK | `ENTHALPY = 10.0` (`maxEnergyPerSteam`). |
| Eficiência Energética do Vapor | 0,2 (`HeatUtils.java:25`) | 0,2 (`boiler.rs:32`) | OK | `EFFICIENCY = 0.2`. |
| Condutividade Térmica da Água | 0,7 (`GeneralConfig.java:291`) | 0,7 (`boiler.rs:33`) | OK | `WATER_CONDUCTIVITY = 0.7`. |
| Transferência por Elemento Superaquecedor | 16.000.000,0 J/t (`GeneralConfig.java:293`) | 16.000.000,0 (`boiler.rs:34`) | OK | `ELEMENT_TRANSFER = 16_000_000.0`. |
| Entalpia do Refrigerante (Sódio) | 5,0 J/mB (`chemical_attribute_heated_coolant.json:6`) | 5,0 (`boiler.rs:36`) | OK | `COOLANT_ENTHALPY = 5.0`. |
| Condutividade do Refrigerante (Sódio) | 0,4 (`HeatedCoolant.java:37`) | 0,4 (`boiler.rs:37`) | OK | `COOLANT_CONDUCTIVITY = 0.4`. |
| Temperatura do Refrigerante (Sódio) | 100.000,0 K (`HeatedCoolant.java:38`) | 100.000,0 K (`boiler.rs:38`) | OK | `COOLANT_TEMP = 100_000.0`. |
| Substância Fria Resultante | `mekanism:sodium` (`chemical_attribute_heated_coolant.json:5`) | `mekanism:sodium` (`boiler.rs:20`) | OK | `COLD = "mekanism:sodium"`. |
| Altura da Câmara de Vapor | `maxPos.getY() - plane` (`BoilerValidator.java:128`) | `max[1] - plane` (`boiler.rs:157`) | CORRIGIDO | Estava `max[1] - 1 - plane`, gerando altura 0 e volume de vapor 0 para caldeiras 3x3x3 com dispersor em y=1. Corrigido para `max[1] - plane`. |
| Modos das Válvulas | Entrada, Saída de Vapor, Saída de Refrigerante (`BoilerValveMode.java`) | Modos 0 (entrada), 1 (saída vapor), 2 (saída refrigerante frio) via sneak click com configurator (`boiler.rs:192-203, 422-430`) | OK | Implementado fielmente com configurator e persistido via `port_modes`. |
| **THERMAL EVAPORATION PLANT** | | | | |
| Base do Multibloco | 4x4 exato (`EvaporationValidator.java:26-27`) | 4x4 exato (`evaporation.rs:55`) | OK | `sx == 4 && sz == 4`. |
| Altura do Multibloco | 3 a 18 blocos (`EvaporationValidator.java:27`) | 3 a 18 blocos (`evaporation.rs:55`) | OK | `(3..=18).contains(&sy)`. |
| Controlador | Exatamente 1, na parede/face (`EvaporationValidator.java:38, 48`) | Exatamente 1, na parede/face (`evaporation.rs:77-82, 93`) | OK | `controllers == 1` e rejeitado em arestas não-topo. |
| Topo da Torre | Centro 2x2 aberto, cantos abertos para solares (`EvaporationValidator.java:49-51`) | Centro 2x2 aberto, cantos abertos (`evaporation.rs:70-76`) | OK | `roof_corner` ignorado, `roof_middle` deve ser ar. |
| Capacidade Fluido Entrada por Nível | 256.000 mB/nível (4 blocos internos x 64.000 mB) (`GeneralConfig.java:311`, `EvaporationMultiblockData.java:222`) | 256.000 mB/nível (`evaporation.rs:25, 236`) | CORRIGIDO | Estava usando `(height - 1) * 64_000` (subtraindo 1 e sem multiplicar pelos 4 blocos da área interna 2x2). Corrigido para `height * 4 * 64_000 = height * 256_000`. |
| Capacidade Fluido Saída | 10.000 mB (`GeneralConfig.java:313`) | 10.000 mB (`evaporation.rs:26`) | OK | `OUTPUT_CAPACITY = 10_000`. |
| Temperatura Ambiente Base | 300,0 K (`HeatAPI.java:20`) | 300,0 K (`evaporation.rs:17`) | OK | `AMBIENT = 300.0`. |
| Temperatura Máxima do Multiplicador | 3.000,0 K (`EvaporationMultiblockData.java:68`) | 3.000,0 K (`evaporation.rs:18`) | OK | `MAX_TEMP = 3_000.0`. |
| Multiplicador de Temperatura | 0,4 (`GeneralConfig.java:306`) | 0,4 (`evaporation.rs:19`) | OK | `TEMP_MULTIPLIER = 0.4`. |
| Multiplicador Solar | 0,2 por painel (`GeneralConfig.java:304`) | 0,2 por painel (`evaporation.rs:20`) | OK | `SOLAR = 0.2`. |
| Dissipação Térmica com Ambiente | 0,02 (`GeneralConfig.java:302`) | 0,02 (`evaporation.rs:21`) | OK | `DISSIPATION = 0.02`. |
| Fórmula do Multiplicador | `(T_eff - 300) * 0.4 * (height / 18)` (`EvaporationMultiblockData.java:158`) | `(T_eff - 300) * 0.4 * (height / 18)` (`evaporation.rs:132`) | OK | `multiplier` idêntico ao Java. |
| Ritmo de Operações | Quando mult < 1, 1 op a cada `ceil(1/mult)` ticks; quando >= 1, `(int) mult` ops/t (`EvaporationMultiblockData.java:274-275`) | Idêntico (`evaporation.rs:136-144`) | OK | `operations(mult, tick)` reproduz com exatidão o scheduler do Java. |
| Detecção de Painéis Solares | 4 cantos do topo (`EvaporationMultiblockData.java:293-298`) | 4 cantos do topo com sol (`evaporation.rs:263-278`) | OK | Coordenadas `[min[0], top, min[2]]` etc. |
| Aquecimento Externo via Válvulas | Resistive Heater / Condutores térmicos (`TileEntityThermalEvaporationValve.java:28`) | Não suportado (rede de calor não portada) | LIMITE | Conexão de calor externo por condutores termodinâmicos não existe no Horizon. |

---

## 2. Detalhamento das Divergências Encontradas e Corrigidas

### 2.1 Dynamic Tank: Capacidade de Fluido (`dynamic.rs`)
- **Problema encontrado**: `dynamic.rs` definia apenas `pub const PER_BLOCK: u64 = 16_000 * 1000;` e utilizava essa constante tanto para químicos quanto para fluidos. No entanto, no Java 10.7.19 (`GeneralConfig.java:169` e `TankMultiblockData.java:134-135`), a capacidade de fluido é de **350 baldes por bloco** (350.000 mB/bloco), enquanto a de químicos é de **16.000 baldes por bloco** (16.000.000 mB/bloco).
- **Correção**: 
  - Declaradas constantes `FLUID_PER_BLOCK = 350 * 1000` e `CHEMICAL_PER_BLOCK = 16_000 * 1000`.
  - Calculadas capacidades separadas `fluid_cap = volume * FLUID_PER_BLOCK` e `chem_cap = volume * CHEMICAL_PER_BLOCK`.
  - Atualizadas as operações de mescla (`merge`), transvase com baldes nos slots das válvulas (`tank::buckets`), filtros de saturação de tanque (`set_tank_filter`) e a exibição de capacidade na tela (`dynamic_menu`).

### 2.2 Thermoelectric Boiler: Coeficiente de Dissipação do Ar (`boiler.rs`)
- **Problema encontrado**: A constante `AIR` estava definida como `10.0`. No Java (`HeatAPI.java:24`), `AIR_INVERSE_COEFFICIENT = 10_000.0`. Isso causava uma perda de calor para o ar ambiente **1000 vezes mais rápida** que a do jogo original.
- **Correção**:
  - `pub const AIR: f64 = 10_000.0;`
  - A impedância térmica total de convecção ambiente agora calcula `AIR + INSULATION + CONDUCTION = 10_000.0 + 100_000.0 + 1.0 = 110_001.0`, correspondendo ao Java `simulateEnvironment()`.
  - Teste `the_air_cools_it_slowly` atualizado para verificar a perda com `110_001.0`.

### 2.3 Thermoelectric Boiler: Altura da Câmara de Vapor (`boiler.rs`)
- **Problema encontrado**: A fórmula da altura da câmara de vapor estava `let steam_height = (max[1] - 1 - plane).max(0);`. No Java (`BoilerValidator.java:128`), a fórmula é `(structure.renderLocation.getY() + structure.height() - 2) - initDisperser.getY()`, o que se simplifica matematicamente para `max[1] - plane`. Por conta do `- 1` extra no port, uma caldeira 3x3x3 com dispersores no plano y=1 obtinha altura de vapor 0 e volume de vapor 0, impossibilitando a geração ou armazenamento de vapor.
- **Correção**:
  - `let steam_height = (max[1] - plane).max(0);`
  - Testes `a_box_with_dispersers_and_an_element_is_a_boiler` e `the_steam_room_is_the_height_above_the_dispersers` ajustados para refletir o volume exato do Java.

### 2.4 Thermal Evaporation Plant: Capacidade do Tanque de Entrada (`evaporation.rs`)
- **Problema encontrado**: O cálculo de capacidade do tanque de entrada estava `let in_cap = (u64::from(found.height) - 1) * PER_LEVEL;` com `PER_LEVEL = 64_000`. No Java (`EvaporationMultiblockData.java:222` e `GeneralConfig.java:311`), `inputTankCapacity = (volume / 4) * evaporationFluidPerTank`. Como a torre tem base 4x4, `volume = 16 * height`, logo `volume / 4 = 4 * height`. Cada nível possui 4 blocos de volume interno (2x2), totalizando `4 * 64.000 = 256.000 mB` por nível de altura da torre, sem subtrair 1 da altura.
- **Correção**:
  - `pub const FLUID_PER_TANK: u64 = 64_000;`
  - `pub const PER_LEVEL: u64 = 4 * FLUID_PER_TANK;` (256.000 mB/nível)
  - `let in_cap = u64::from(found.height) * PER_LEVEL;`
  - Uma torre de altura mínima 3 agora comporta 768.000 mB (em vez de 128.000 mB); altura máxima 18 comporta 4.608.000 mB (em vez de 1.088.000 mB), em conformidade total com o Java.
