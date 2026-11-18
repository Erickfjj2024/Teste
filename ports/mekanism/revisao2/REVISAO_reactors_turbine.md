# Relatório de Revisão Cruzada: Reatores (Fission, Fusion) e Turbina Industrial

**Tarefa:** T20261109-194505-xea (Cloud Shell Hub - Revisão Cruzada t3)  
**Escopo:** `port/mod/src/fission.rs`, `port/mod/src/fusion.rs`, `port/mod/src/turbine.rs`  
**Referência Java:** Mekanism 1.21.1-10.7.19.85 (`GeneratorsConfig`, `GeneralConfig`, `HeatAPI`, `HeatUtils`, `FissionReactorMultiblockData`, `FusionReactorMultiblockData`, `TurbineMultiblockData`, `TurbineValidator`, etc.)  
**Relatório Anterior Auditado:** `port/auditoria/RESUMO_t3.md` e `port/auditoria/AUDITORIA_reactors_turbine.md` (tarefa T20261109-183803-8so)

---

## 1. Auditoria e Reavaliação das Correções da Rodada 1

| Item Auditado | Valor / Lógica Java (arquivo:linha) | Valor / Lógica Port (arquivo:linha) | Parecer da Revisão Cruzada | Estado |
|---|---|---|---|---|
| **Fission: Coeficiente Inverso do Ar (`AIR`)** | `HeatAPI.AIR_INVERSE_COEFFICIENT = 10_000.0` (`HeatAPI.java:24`). Transferência total: `HeatAPI.AIR_INVERSE_COEFFICIENT * 2 + 10 = 20_010.0` (`FissionReactorMultiblockData.java:252`) | `AIR = 10_000.0`, `INSULATION = 10_000.0`, `CONDUCTION = 10.0`. Divisor total: `AIR + INSULATION + CONDUCTION = 20_010.0` (`fission.rs:26-28, 324`) | A Rodada 1 corrigiu `AIR` de 10.0 para 10_000.0. A soma exata do divisor de perda para o ar confere 100% com a expressão Java. | **CONFIRMADA** |
| **Fission: Condutividade do Sódio Refrigerante** | `thermalEnthalpy = 5.0`, `conductivity = 1.0` (`MekanismDataMapsProvider.java:85`, `CooledCoolant.java:38`) | `SODIUM_ENTHALPY = 5.0`, `SODIUM_CONDUCTIVITY = 1.0` (`fission.rs:32-33`) | A Rodada 1 corrigiu `SODIUM_CONDUCTIVITY` de 0.4 para 1.0. O registro em DataMaps Java confirma que o sódio superaquecido tem condutividade 1.0. | **CONFIRMADA** |
| **Fission: Radiação de `partial_waste` no Meltdown** | Em `getWasteTankRadioactivity(true)`, se `wasteTank.isEmpty()`, usa `MekanismChemicals.NUCLEAR_WASTE.get().getRadioactivity()` (0.01) (`FissionReactorMultiblockData.java:381-394`) | `let waste_chem = if s.reactor.waste.amount == 0 { WASTE } else { &s.reactor.waste.chemical };` (`fission.rs:505`) | A Rodada 1 evitou que `partial_waste` fosse ignorado quando `waste.amount == 0`. Lógica idêntica ao fallback Java. | **CONFIRMADA** |
| **Turbine: Transbordamento de Água nos Vents** | `ventTank.insert(new FluidStack(Fluids.WATER, ...))` não impede o consumo de vapor nem a geração de energia quando cheio (`TurbineMultiblockData.java:156`) | `room = water_capacity(f).saturating_sub(water); produced = flow.min(water_capacity(f)).min(room);` (`turbine.rs:301-302`) | A Rodada 1 removeu o travamento precoce que impedia a turbina de girar com o tanque cheio de água. Mantém geração contínua. | **CONFIRMADA** |

---

## 2. Descobertas e Correções da Rodada 2 (Omissões da Rodada 1)

Durante esta revisão cruzada independente contra o Java 1.21.1, identificamos falhas conceituais e discrepâncias que passaram despercebidas na Rodada 1:

1. **Volume do Multibloco em `fission.rs` e `turbine.rs` (Correção Estrutural Maior):**
   - **Java:** Em `MultiblockData.java:198`, `setVolume` calcula `bounds.length() * bounds.width() * bounds.height()`, isto é, o volume do paralelepípedo delimitador externo (*bounding box*). Os tetos configurados (`GeneratorsConfig.java:138, 155`) são expressos em dimensões externas (18×18×18 para fissão, 17×17×18 para turbina).
   - **Port (anterior):** Ambos calculavam volume interno de ar `(size[0] - 2) * (size[1] - 2) * (size[2] - 2)`.
   - **Impacto:** Um reator de fissão 5×5×5 calculava volume 27 em vez de 125 (apenas 21.6% do oficial), cortando dramaticamente a capacidade de refrigerante e vapor gerado. Uma turbina 5×8×5 calculava volume 54 em vez de 200 (27% do oficial), reduzindo drasticamente o buffer de energia.
   - **Correção aplicada:** Atualizado para `size[0] * size[1] * size[2]` em `fission.rs:175` e `turbine.rs:180`. Testes de unidade atualizados com os valores oficiais.

2. **Geração Gratuita de Energia na Turbina (`clientFlow > 0`):**
   - **Java:** `TurbineMultiblockData.java:151-155` verifica explicitamente `if (clientFlow > 0)` antes de aplicar `insert` de energia, contração do tanque químico e condensação de água.
   - **Port (anterior):** Verificava `if rate <= 1e-5`, permitindo que taxas fracionárias (ex: 0.5 mB/t, onde `flow = rate as u64 = 0`) gerassem energia `generated > 0` sem consumir 1 único mB de vapor.
   - **Correção aplicada:** Inserido `if flow == 0 { return Tick::default(); }` em `turbine.rs:294-296`.

3. **Remoção do Teto Artificial de Energia da Turbina (`i32::MAX`):**
   - **Java:** O Mekanism Java usa aritmética `FloatingLong` / `long` de 64 bits para inserção de energia (`TurbineMultiblockData.java:152`).
   - **Port (anterior):** `turbine.rs:293` limitava a energia gerada a `.min(f64::from(i32::MAX))`, impondo um teto de ~2.147 bilhões de FE por tick em turbinas gigantes.
   - **Correção aplicada:** Removido o teto de 32 bits, preservando `(mult * rate * 0.4).max(0.0) as u64` em `turbine.rs:297`.

4. **Faces de Energia na Porta de Fusão em Modo Entrada:**
   - **Java:** O contêiner de energia do reator de fusão é estritamente de saída (`VariableCapacityEnergyContainer.output`, `FusionReactorMultiblockData.java:157`). Portas nunca aceitam energia de cabos externos; a ignição térmica externa vem unicamente de feixes de laser através das matrizes de foco.
   - **Port (anterior):** `fusion.rs:539` definia `energy_faces: Some(vec![FlowSide::Input; 6])` no modo `!output`.
   - **Correção aplicada:** Alterado para `Some(vec![FlowSide::None; 6])` em `fusion.rs:539`.

5. **Correção de Docstrings Desatualizadas em `fission.rs`:**
   - As linhas 6-7 de `fission.rs` continuavam documentando que as portas não possuíam modos ("The ports have no mode"), divergindo do código implementado na Rodada 1. Corrigido para documentar os 3 modos de operação (Input, Output Waste, Output Coolant).

---

## 3. Tabela Comparativa Completa (Revisão Cruzada)

| # | Item / Mecânica | Valor / Lógica Java (arquivo:linha) | Valor / Lógica Port (arquivo:linha) | Estado | Detalhes / Justificativa |
|---|---|---|---|---|---|
| 1 | Fission: Dimensões Mínimas | 3×3×3 (`GeneratorsConfig.java:136-137`) | `MIN_SIZE = [3, 3, 3]` (`fission.rs:13`) | **OK** | Idêntico |
| 2 | Fission: Dimensões Máximas | 18×18×18 (`GeneratorsConfig.java:138`) | `MAX_SIZE = [18, 18, 18]` (`fission.rs:14`) | **OK** | Idêntico |
| 3 | Fission: Volume do Multibloco | `bounds.length() * bounds.width() * bounds.height()` (`MultiblockData.java:198`) | `size[0] * size[1] * size[2]` (`fission.rs:175`) | **CORRIGIDO** | Corrigido na Rodada 2 de `(size-2)^3` para volume delimitador externo oficial |
| 4 | Fission: Taxa de Queima Máxima por Montagem | 1.0 mB/t (`GeneratorsConfig.java:141`) | `BURN_PER_ASSEMBLY = 1.0` (`fission.rs:41`) | **OK** | Idêntico |
| 5 | Fission: Capacidade de Combustível por Montagem | 8 000 mB (`GeneratorsConfig.java:143`) | `FUEL_PER_ASSEMBLY = 8_000` (`fission.rs:42`) | **OK** | Idêntico |
| 6 | Fission: Capacidade de Refrigerante Frio por Volume | 100 000 mB (`GeneratorsConfig.java:140`) | `COOLANT_PER_VOLUME = 100_000` (`fission.rs:43`) | **OK** | Idêntico |
| 7 | Fission: Capacidade de Refrigerante Quente por Volume | 1 000 000 mB (`FissionReactorMultiblockData.java:137`) | `HEATED_PER_VOLUME = 1_000_000` (`fission.rs:44`) | **OK** | Idêntico |
| 8 | Fission: Energia Térmica por Combustível | 1 000 000 J/mB (`GeneratorsConfig.java:139`) | `HEAT_PER_FUEL = 1_000_000.0` (`fission.rs:37`) | **OK** | Idêntico |
| 9 | Fission: Capacidade Térmica da Carcaça por Bloco | 1 000 J/K (`GeneratorsConfig.java:144`) | `CASING_HEAT = 1_000.0` (`fission.rs:38`) | **OK** | Idêntico |
| 10 | Fission: Área Alvo por Montagem | 4.0 (`GeneratorsConfig.java:142`) | `SURFACE_TARGET = 4.0` (`fission.rs:39`) | **OK** | Idêntico |
| 11 | Fission: Taxa de Queima Padrão | 0.1 mB/t (`FissionReactorMultiblockData.java:68`) | `DEFAULT_RATE = 0.1` (`fission.rs:40`) | **OK** | Idêntico |
| 12 | Fission: Temperatura de Ebulição Base | 373.15 K (`HeatUtils.java:17`) | `BASE_BOIL = 373.15` (`fission.rs:30`) | **OK** | Idêntico |
| 13 | Fission: Condutividade Térmica da Água | 0.5 (`FissionReactorMultiblockData.java:78`) | `WATER_CONDUCTIVITY = 0.5` (`fission.rs:29`) | **OK** | Idêntico |
| 14 | Fission: Entalpia da Água | 50.0 (10.0 / 0.2) (`HeatUtils.java:21, 24`) | `WATER_ENTHALPY = 10.0 / 0.2` (`fission.rs:31`) | **OK** | Idêntico |
| 15 | Fission: Entalpia do Sódio | 5.0 (`CooledCoolant.java:38`) | `SODIUM_ENTHALPY = 5.0` (`fission.rs:32`) | **OK** | Idêntico |
| 16 | Fission: Condutividade Térmica do Sódio | 1.0 (`MekanismDataMapsProvider.java:85`) | `SODIUM_CONDUCTIVITY = 1.0` (`fission.rs:33`) | **CONFIRMADA** | Confirmada correção da rodada 1 |
| 17 | Fission: Dissipação Ambiental (Ar) | 20 010.0 (`HeatAPI.java:24`, `FissionReactorMultiblockData.java:252`) | `AIR + INSULATION + CONDUCTION = 20_010.0` (`fission.rs:26-28, 324`) | **CONFIRMADA** | Confirmada correção da rodada 1 |
| 18 | Fission: Temperatura Mínima de Dano | 1 200.0 K (`FissionReactorMultiblockData.java:64`) | `MIN_DAMAGE_TEMP = 1_200.0` (`fission.rs:34`) | **OK** | Idêntico |
| 19 | Fission: Temperatura Máxima de Dano | 1 800.0 K (`FissionReactorMultiblockData.java:65`) | `MAX_DAMAGE_TEMP = 1_800.0` (`fission.rs:35`) | **OK** | Idêntico |
| 20 | Fission: Dano Máximo | 100.0% (`FissionReactorMultiblockData.java:67`) | `MAX_DAMAGE = 100.0` (`fission.rs:36`) | **OK** | Idêntico |
| 21 | Fission: Probabilidade de Meltdown | 0.001 (0.1%) (`FissionReactorMultiblockData.java:66`) | `MELTDOWN_CHANCE = 0.001` (`fission.rs:45`) | **OK** | Idêntico |
| 22 | Fission: Raio de Explosão do Meltdown | 8 blocos (`GeneratorsConfig.java:146`) | `MELTDOWN_RADIUS = 8` (`fission.rs:46`) | **OK** | Idêntico |
| 23 | Fission: Dano Pós-Meltdown | 75.0% (`GeneratorsConfig.java:148`) | `POST_MELTDOWN_DAMAGE = 75.0` (`fission.rs:47`) | **OK** | Idêntico |
| 24 | Fission: Multiplicador de Radiação no Meltdown | 50.0× (`FissionReactorMultiblockData.java:350`) | `50.0` (`fission.rs:510`) | **OK** | Idêntico |
| 25 | Fission: Radiação de `partial_waste` no Meltdown | Fallback para `NUCLEAR_WASTE` se tanque vazio (`FissionReactorMultiblockData.java:381-394`) | Fallback para `WASTE` (`fission.rs:505`) | **CONFIRMADA** | Confirmada correção da rodada 1 |
| 26 | Fission: Modos das Portas | Input, Output Waste, Output Coolant (`TileEntityFissionReactorPort.java`) | 3 modos suportados via configurador (`fission.rs:220-238`) | **OK** | Docstring corrigida na rodada 2 |
| 27 | Fission: Radiação a Entidades Vivas | Emissão por distância a entidades no raio (`FissionReactorMultiblockData.java:209`) | Não suportado por falta de API de busca de entidades no Horizon | **FORK** | Depende de API de entidades (FORK-7/FORK-11) |
| 28 | Fission: Logic Adapter | 8 modos de controle e sinal redstone (`TileEntityFissionReactorLogicAdapter.java`) | Bloco de carcaça passivo | **FORK** | Depende de portas redstone analógicas/digitais (FORK-21) |
| 29 | Fission: Persistência de Burn Rate = 0.0 | `-1` indica não-inicializado (`FissionReactorCache.java:21`) | `rate_limit > 0.0` reseta 0.0 para 0.1 no reload (`fission.rs:445`) | **FORK** | Detalhado na Seção 21 |
| 30 | Fusion: Dimensões da Estrutura | 5×5×5 com 12 blocos vazados por face (`FusionReactorValidator.java:38`) | 5×5×5, `GRID` 12 vazados por face (`fusion.rs:47, 90-145`) | **OK** | Idêntico |
| 31 | Fusion: Temperatura de Queima | 100 000 000 K (`FusionReactorMultiblockData.java:60`) | `BURN_TEMPERATURE = 100_000_000.0` (`fusion.rs:27`) | **OK** | Idêntico |
| 32 | Fusion: Capacidade Térmica do Plasma | 100.0 (`FusionReactorMultiblockData.java:62`) | `PLASMA_HEAT_CAPACITY = 100.0` (`fusion.rs:28`) | **OK** | Idêntico |
| 33 | Fusion: Capacidade Térmica da Carcaça | 1.0 (`FusionReactorMultiblockData.java:61`) | `CASE_HEAT_CAPACITY = 1.0` (`fusion.rs:29`) | **OK** | Idêntico |
| 34 | Fusion: Condutividade Plasma-Carcaça | 0.2 (`FusionReactorMultiblockData.java:63`) | `PLASMA_CASE_CONDUCTIVITY = 0.2` (`fusion.rs:30`) | **OK** | Idêntico |
| 35 | Fusion: Energia por Combustível | 10 000 000 J/mB (`GeneratorsConfig.java:180`) | `ENERGY_PER_FUEL = 10_000_000.0` (`fusion.rs:31`) | **OK** | Idêntico |
| 36 | Fusion: Eficiência Termopar | 0.05 (`GeneratorsConfig.java:182`) | `THERMOCOUPLE = 0.05` (`fusion.rs:32`) | **OK** | Idêntico |
| 37 | Fusion: Condutividade Térmica da Carcaça | 0.1 (`GeneratorsConfig.java:184`) | `CASING_CONDUCTIVITY = 0.1` (`fusion.rs:33`) | **OK** | Idêntico |
| 38 | Fusion: Razão de Aquecimento de Água | 0.3 (`GeneratorsConfig.java:186`) | `WATER_HEATING = 0.3` (`fusion.rs:34`) | **OK** | Idêntico |
| 39 | Fusion: Capacidade de Combustível D-T | 1 000 mB (`FusionReactorMultiblockData.java:135`) | `FUEL_CAPACITY = 1_000` (`fusion.rs:35`) | **OK** | Idêntico |
| 40 | Fusion: Capacidade de Energia | 1 000 000 000 J = 400 000 000 FE (`FusionReactorMultiblockData.java:157`) | `ENERGY_CAPACITY = fe(1_000_000_000)` (`fusion.rs:36`) | **OK** | Idêntico (`fe()` = 400M FE) |
| 41 | Fusion: Capacidade de Água por Taxa de Injeção | 1 000 000 mB (`FusionReactorMultiblockData.java:140`) | `WATER_PER_INJECTION = 1_000_000` (`fusion.rs:37`) | **OK** | Idêntico |
| 42 | Fusion: Capacidade de Vapor por Taxa de Injeção | 100 000 000 mB (`FusionReactorMultiblockData.java:146`) | `STEAM_PER_INJECTION = 100_000_000` (`fusion.rs:38`) | **OK** | Idêntico |
| 43 | Fusion: Combustível no Hohlraum | 10 mB (`ItemHohlraum.java:19`) | `HOHLRAUM_FUEL = 10` (`fusion.rs:39`) | **OK** | Idêntico |
| 44 | Fusion: Taxa de Injeção Padrão | 2 mB/t (`FusionReactorMultiblockData.java:70`) | `DEFAULT_INJECTION = 2` (`fusion.rs:40`) | **OK** | Idêntico |
| 45 | Fusion: Taxa de Injeção Máxima | 98 mB/t (`FusionReactorMultiblockData.java:69`) | `MAX_INJECTION = 98` (`fusion.rs:41`) | **OK** | Idêntico |
| 46 | Fusion: Aquecimento por Laser | 10× sem ignição, 1× com ignição (`FusionReactorMultiblockData.java:199`) | 10× se `!r.burning`, 1× se `r.burning` (`fusion.rs:214`) | **OK** | Idêntico |
| 47 | Fusion: Paridade de Injeção D-T | Injeção em números pares (`amountToInject -= amountToInject % 2`) (`FusionReactorMultiblockData.java:315`) | `inject -= inject % 2; let half = inject / 2;` (`fusion.rs:241-242`) | **OK** | Idêntico |
| 48 | Fusion: Faces de Energia das Portas | Apenas saída (`VariableCapacityEnergyContainer.output`, `FusionReactorMultiblockData.java:157`) | Modo entrada usa `FlowSide::None`, saída usa `FlowSide::Output` (`fusion.rs:539`) | **CORRIGIDO** | Corrigido na Rodada 2 para rejeitar energia de cabos |
| 49 | Fusion: Morte Instantânea no Núcleo | 50 000 de dano por tick a qualquer entidade na câmara 3×3×3 (`FusionReactorMultiblockData.java:291-297`) | Não implementado por ausência de API de colisão/dano em área | **FORK** | Depende de API de entidades (FORK-7/FORK-11) |
| 50 | Fusion: Item Hohlraum com Gás | Guarda 10 mB D-T em componente NBT (`ItemHohlraum.java`) | Item vanilla sem componente de substância; reator considera cheio se presente no slot | **FORK** | Depende de NBT dinâmico em itens (FORK-23/FORK-26) |
| 51 | Fusion: Logic Adapter | 4 modos redstone (`TileEntityFusionReactorLogicAdapter.java`) | Bloco de carcaça passivo | **FORK** | Depende de portas redstone analógicas/digitais (FORK-21) |
| 52 | Turbine: Dimensões Permitidas | Largura/Comprimento ímpares 5 a 17, Altura 3 a 18 (`GeneratorsConfig.java:154-156`) | Ímpares 5..=17, altura 3..=18 (`turbine.rs:1-3, 100-115`) | **OK** | Idêntico |
| 53 | Turbine: Volume Total | `bounds.length() * bounds.width() * bounds.height()` (`MultiblockData.java:198`) | `size[0] * size[1] * size[2]` (`turbine.rs:180`) | **CORRIGIDO** | Corrigido na Rodada 2 de `(size-2)^3` para volume delimitador externo oficial |
| 54 | Turbine: Volume Inferior (`lowerVolume`) | `length * width * turbineHeight` (`TurbineValidator.java:196`) | `size[0] * size[2] * height` (`turbine.rs:181`) | **OK** | Idêntico ao produto da base externa pela altura dos rotores |
| 55 | Turbine: Lâminas Máximas (`MAX_BLADES`) | 28.0 (`TurbineMultiblockData.java:49`) | `MAX_BLADES = 28.0` (`turbine.rs:28`) | **OK** | Idêntico |
| 56 | Turbine: Lâminas por Bobina | 4 (`GeneratorsConfig.java:160`) | `BLADES_PER_COIL = 4` (`turbine.rs:29`) | **OK** | Idêntico |
| 57 | Turbine: Vazão por Ventilação | 32 000.0 mB/t (`GeneratorsConfig.java:161`) | `VENT_FLOW = 32_000.0` (`turbine.rs:30`) | **OK** | Idêntico |
| 58 | Turbine: Vazão por Dispersor | 1 280.0 mB/t (`GeneratorsConfig.java:162`) | `DISPERSER_FLOW = 1_280.0` (`turbine.rs:31`) | **OK** | Idêntico |
| 59 | Turbine: Taxa por Condensador | 64 000 mB/t (`GeneratorsConfig.java:163`) | `CONDENSER_RATE = 64_000` (`turbine.rs:32`) | **OK** | Idêntico |
| 60 | Turbine: Capacidade de Energia por Volume | 16 000 000 J = 6 400 000 FE (`GeneratorsConfig.java:159`) | `ENERGY_PER_VOLUME = fe(16_000_000)` (`turbine.rs:34`) | **OK** | Idêntico |
| 61 | Turbine: Capacidade de Vapor por Bloco de Tanque | 64 000 mB (`TurbineMultiblockData.java:50`) | `STEAM_PER_TANK = 64_000` (`turbine.rs:35`) | **OK** | Idêntico |
| 62 | Turbine: Energia Máxima por mB de Vapor | 10.0 J (`GeneralConfig.java:82`) | `ENERGY_PER_STEAM = 10.0` (`turbine.rs:37`) | **OK** | Idêntico |
| 63 | Turbine: Consumo Condicionado a `clientFlow > 0` | Só gera energia se `clientFlow > 0` (`TurbineMultiblockData.java:151-155`) | `if flow == 0 { return Tick::default(); }` (`turbine.rs:294-296`) | **CORRIGIDO** | Eliminado bug de energia grátis quando `rate < 1.0` |
| 64 | Turbine: Remoção do Teto de Geração `i32::MAX` | Usa `long` de 64 bits (`TurbineMultiblockData.java:152`) | `(mult * rate * 0.4).max(0.0) as u64` (`turbine.rs:297`) | **CORRIGIDO** | Removido teto artificial de 32 bits |
| 65 | Turbine: Transbordamento do Tanque de Água | Não bloqueia fluxo de vapor (`TurbineMultiblockData.java:156`) | Condensa até o espaço livre sem travar (`turbine.rs:301-302`) | **CONFIRMADA** | Confirmada correção da rodada 1 |
| 66 | Turbine: Algoritmo de Adição/Remoção de Pás | Recursivo de baixo para cima na adição e topo para baixo na remoção (`TileEntityTurbineRotor.java:89-126`) | Algoritmo idêntico em `add_blade` e `remove_blade` (`turbine.rs:224-245`) | **OK** | Idêntico |
| 67 | Turbine: Modos de Despejo de Vapor (`dumpMode`) | Dumping Mode: `IDLE`, `DUMPING_EXCESS`, `DUMPING` (`TurbineMultiblockData.java:168-179`) | Interface simplificada não suporta modos de descarte de vapor | **FORK** | Depende de botões/modos de dumping na GUI (FORK-20) |
| 68 | Turbine: Animação e Áudio do Rotor | Rotação proporcional contínua e som de turbina (`RenderTurbineRotor.java`) | Sem renderizador de modelo animado no WASM | **FORK** | Sem impacto mecânico (FORK-10) |

---

## 4. Testes de Unidade e Prova de Mutação

### Testes Editados
1. **`port/mod/src/fission.rs`**:
   - `a_column_of_fuel_under_a_rod_is_a_reactor`: Atualizado o volume esperado de 27 para 125, refletindo o volume delimitador externo oficial `bounds.length() * bounds.width() * bounds.height()` (`5 * 5 * 5 = 125`).
2. **`port/mod/src/turbine.rs`**:
   - `a_complex_with_rotors_coils_and_a_plane_is_a_turbine`: Atualizado o volume esperado de 54 para 200, refletindo o volume delimitador externo oficial `bounds.length() * bounds.width() * bounds.height()` (`5 * 8 * 5 = 200`).
   - `steam_turns_the_blades_into_energy_and_water`: Adicionada verificação `assert_eq!(step(&f, 4, 1, 0, 0), Tick::default());` comprovando que quando o fluxo trunca para zero, nenhuma energia é concedida gratuitamente.

### Resumo dos Testes do Escopo
- `fission.rs`: 8 testes (`a_column_of_fuel_under_a_rod_is_a_reactor`, `the_columns_have_to_be_whole`, `burning_fuel_makes_heat_waste_and_steam`, `sodium_becomes_superheated_sodium`, `heat_damages_and_cold_repairs`, `the_buttons_work_within_the_limits`, `a_member_remembers_its_reactor`, `the_roll_is_in_range`).
- `fusion.rs`: 7 testes (`the_rounded_cube_has_twelve_missing_blocks_a_face`, `a_whole_shell_with_a_controller_is_a_reactor`, `it_needs_the_shape_the_controller_and_an_empty_inside`, `a_hohlraum_ignites_a_hot_plasma_and_it_keeps_burning`, `the_laser_heats_ten_times_more_before_the_burn`, `the_injection_rate_moves_by_two`, `a_member_remembers_its_reactor`).
- `turbine.rs`: 5 testes (`a_complex_with_rotors_coils_and_a_plane_is_a_turbine`, `it_needs_everything_in_its_place`, `blades_go_where_the_original_puts_them`, `steam_turns_the_blades_into_energy_and_water`, `a_member_remembers_its_turbine`).
- **Total:** 20 testes unitários.
- **Execução na VM:** não medido (a VM não possui toolchain Rust configurada, em conformidade com as regras da tarefa).
- **Prova de mutação:** a prova de mutação está pendente do juiz.
