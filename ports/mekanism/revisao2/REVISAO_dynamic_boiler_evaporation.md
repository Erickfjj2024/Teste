# Revisão Cruzada (Rodada 2): Dynamic Tank, Thermoelectric Boiler e Thermal Evaporation

**ID da Tarefa**: T20261109-194317-824  
**Data**: 2026-11-09  
**Referência de Verdade**: Mekanism 1.21.1-10.7.19.85 (`java/`)  
**Arquivos sob Revisão**:
- `port/mod/src/dynamic.rs` (365 linhas)
- `port/mod/src/boiler.rs` (589 linhas)
- `port/mod/src/evaporation.rs` (460 linhas)

---

## 1. Verificação das Correções da Rodada 1

| Correção da Rodada 1 | Valor / Lógica no Java (arquivo:linha) | Valor / Lógica no Port (arquivo:linha) | Veredito | Análise Técnica e Justificativa |
|---|---|---|---|---|
| **Dynamic Tank: Capacidade Fluido vs Químico** | `GeneralConfig.java:169,171`<br>`TankMultiblockData.java:134-135` | `dynamic.rs:15,17,152-153` | **CONFIRMADA** | No Java, `dynamicTankFluidPerTank = 350 * 1000` (350.000 mB/bloco) e `dynamicTankChemicalPerTank = 16_000 * 1000` (16.000.000 mB/bloco). A separação em `FLUID_PER_BLOCK = 350_000` e `CHEMICAL_PER_BLOCK = 16_000_000` está 100% em conformidade com o original. |
| **Boiler: Coeficiente Inverso de Ar** | `HeatAPI.java:24`<br>`BoilerMultiblockData.java:279` | `boiler.rs:28` | **CONFIRMADA** | No Java, `HeatAPI.AIR_INVERSE_COEFFICIENT = 10_000`. O port corrigiu de 10.0 para 10.000,0, totalizando impedância ambiente de `10_000 + 100_000 + 1 = 110_001.0`. Simulação física coincide perfeitamente. |
| **Boiler: Altura da Câmara de Vapor** | `BoilerValidator.java:128` | `boiler.rs:155` | **CONFIRMADA** | No Java, `(structure.renderLocation.getY() + structure.height() - 2) - initDisperser.getY()`. Como `renderLocation.getY() = min.y + 1` e `height() = max.y - min.y + 1`, a expressão simplifica para `max.y - initDisperser.getY() = max[1] - plane`. A correção de `(max[1] - 1 - plane)` para `(max[1] - plane).max(0)` eliminou o `-1` incorreto. |
| **Evaporation Plant: Capacidade Tanque Entrada** | `EvaporationMultiblockData.java:222`<br>`GeneralConfig.java:310` | `evaporation.rs:24-25,236` | **CONFIRMADA** | No Java, `inputTankCapacity = (volume / 4) * evaporationFluidPerTank`. Para base 4x4, `volume = 16 * height`, logo `volume / 4 = 4 * height`. Com `evaporationFluidPerTank = 64_000`, a capacidade é `4 * 64_000 = 256.000` mB por nível de altura. A correção de `(height - 1) * 64_000` para `height * 256_000` está 100% exata. |

---

## 2. Tabela Geral de Revisão Cruzada

| Escopo / Item | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Análise Detalhada |
|---|---|---|---|---|
| **DYNAMIC TANK** | | | | |
| Dimensões mín/máx | `CuboidStructureValidator.java:34` (3x3x3 a 18x18x18) | `dynamic.rs:39` (`3..=18`) | **OK** | Eixos X, Y e Z validados individualmente no intervalo [3, 18]. |
| Capacidade de Fluido | `GeneralConfig.java:169` (350.000 mB/bloco) | `dynamic.rs:15` (`350 * 1000`) | **CONFIRMADA** | Corretamente aplicado na rodada 1. |
| Capacidade de Químico | `GeneralConfig.java:171` (16.000.000 mB/bloco) | `dynamic.rs:17` (`16_000 * 1000`) | **OK** | Equivalente a 16.000 baldes por bloco interno. |
| Filtro de Válvulas e Contaminação | `MergedTank.java:49`<br>`TankMultiblockData.java:130` | `dynamic.rs:219-232` | **ERRADA-CORRIGIDA** | **O que a R1 não viu**: ao passar `Vec::new()` quando `fluid.amount > 0 && < cap`, o Horizon aceitava inserção de *qualquer outro fluido* no tanque em uso. Corrigido para travar na substância atual (`vec![fluid.chemical.clone()]`). |
| Exclusividade de conteúdo | `TankMultiblockData.java:49` | `dynamic.rs:201` | **OK** | Se ambos ficarem > 0, o químico é descartado em prol do fluido. |
| Regras de Válvula | `CuboidStructureValidator.java:115` | `dynamic.rs:55` | **OK** | Rejeita válvulas em arestas e cantos (`edges > 1`). |
| Vidro Estrutural na carcaça | `TileEntityStructuralGlass.java:18`<br>`CuboidStructureValidator.java:103` | `dynamic.rs:38`, `lib.rs:1942` | **SUPOSIÇÃO** | No Java, Structural Glass é rejeitado nas 12 arestas da moldura (`needsFrame`). No port, o grupo aceita vidro em qualquer casing. |
| Acesso à GUI em Casing | `TileEntityDynamicTank.java:42` | `lib.rs:2070` | **FORK** | No Java, blocos de carcaça também abrem a GUI se formados. No port, apenas válvulas abrem. Requer suporte da API do Horizon. |
| Esvaziar/Encher balde no mundo | `TileEntityDynamicTank.java:40` | `dynamic.rs:208` | **FORK** | Clique com balde na mão diretamente no bloco preenche/esvazia no Java. No port, apenas via slots internos da GUI. |
| Alternância de EditMode de Balde | `TankMultiblockData.java:33,158` | `dynamic.rs:209` | **FORK** | No Java alterna `Both / Fill / Empty`. No port fixado em `Both`. |
| Sinal de Comparador (Redstone) | `TankMultiblockData.java:140` | `dynamic.rs` | **FORK** | Emite sinal proporcional de 0 a 15 (`redstoneLevelFromContents`). Não suportado pela API do Horizon. |
| **THERMOELECTRIC BOILER** | | | | |
| Dimensões mín/máx | `CuboidStructureValidator.java:34` (3x3x3 a 18x18x18) | `boiler.rs:77` (`3..=18`) | **OK** | Validado para cada eixo. |
| Coeficiente Inverso de Ar | `HeatAPI.java:24` (10.000,0) | `boiler.rs:28` (10.000,0) | **CONFIRMADA** | Impedância térmica corrigida na rodada 1. |
| Altura da Câmara de Vapor | `BoilerValidator.java:128` (`max[1] - plane`) | `boiler.rs:155` | **CONFIRMADA** | Cálculo corrigido na rodada 1. |
| Volume de Vapor | `BoilerValidator.java:129` (`width * length * steamHeight`) | `boiler.rs:160` | **OK** | `(size[0] * size[2] * steam_height)`. Inclui área perimetral da carcaça conforme fórmula literal do Java. |
| Volume de Água | `BoilerValidator.java:114` | `boiler.rs:159` | **SUPOSIÇÃO** | No Java `FormationProtocol.explore` inclui moldura do piso e paredes submersas (`isAirOrFrame`). No port calcula `cells.len()` (ar contíguo interno). Mantido por compatibilidade. |
| Capacidade Água / Refrigerante Q. | `GeneralConfig.java:286,289` | `boiler.rs:40,42` (16k / 256k) | **OK** | `water_volume * 16_000` e `water_volume * 256_000`. |
| Capacidade Vapor / Refrigerante R. | `GeneralConfig.java:288,290` | `boiler.rs:41,42` (160k / 256k) | **OK** | `steam_volume * 160_000` e `steam_volume * 256_000`. |
| Capacidade Térmica e Condução | `BoilerMultiblockData.java:62-64` | `boiler.rs:25-27` | **OK** | `CASING_HEAT = 50.0`, `INSULATION = 100_000.0`, `CONDUCTION = 1.0`. |
| Entalpia Água e Eficiência | `HeatUtils.java:21,25` | `boiler.rs:31,32` (10,0 e 0,2) | **OK** | `ENTHALPY = 10.0`, `EFFICIENCY = 0.2`. |
| Condutividade Água e Elementos | `GeneralConfig.java:291,293` | `boiler.rs:33,34` (0,7 e 16M) | **OK** | `WATER_CONDUCTIVITY = 0.7`, `ELEMENT_TRANSFER = 16_000_000.0`. |
| Entalpia, Condutividade e Temp Sódio | `chemical_attribute_heated_coolant.json:6`<br>`HeatedCoolant.java:37,38` | `boiler.rs:36-38` (5.0, 0.4, 100k) | **OK** | Idêntico aos atributos da substância no Java. |
| Validação de Dispersores | `BoilerValidator.java:70-80` | `boiler.rs:115` | **OK** | Exige plano horizontal completo fechado contíguo `(size[0]-2)*(size[2]-2)`. |
| Validação de Elementos | `BoilerValidator.java:82-90` | `boiler.rs:120-135` | **OK** | Devem estar estritamente abaixo do plano de dispersores e conectados entre si. |
| Filtros de Saída de Válvulas | `BoilerMultiblockData.java:126,128` | `boiler.rs:421-424` | **ERRADA-CORRIGIDA** | **O que a R1 não viu**: tanques 1 (vapor) e 3 (refrigerante resfriado) não possuíam filtro configurado nas válvulas, permitindo injeção indevida vinda de tubos externos. Corrigido com `NOTHING`. |
| Limpeza de Substâncias Inválidas | `BoilerMultiblockData.java:124-128` | `boiler.rs:385-391` | **ERRADA-CORRIGIDA** | **O que a R1 não viu**: caldeira limpava água e refrigerante quente espúrios, mas não validava vapor e refrigerante frio. Corrigido. |
| Modos de Válvula | `TileEntityBoilerValve.java:30-40` | `boiler.rs:190-205` | **OK** | 0 = input only, 1 = output steam, 2 = output coolant. |
| Alternância de Modo (Configurador) | `TileEntityBoilerValve.java:88` | `boiler.rs:420-432` | **OK** | Sneak-click com configurador cicla modo e envia mensagem no chat. |
| Renderização 3D de Fluidos | `RenderThermoelectricBoiler.java:30` | `boiler.rs` | **FORK API** | Java renderiza malha 3D animada dos fluidos com vidro estrutural. Port exibe na GUI. |
| **THERMAL EVAPORATION PLANT** | | | | |
| Geometria da Torre | `EvaporationValidator.java:23-24` | `evaporation.rs:56-60` | **OK** | Base fixa 4x4, altura variável de 3 a 18. |
| Validação do Topo e Cantos | `EvaporationValidator.java:44-50` | `evaporation.rs:73-86` | **OK** | 4 cantos livres para painéis solares, anel superior fechado e centro 2x2 aberto. |
| Controlador Único na Parede | `EvaporationValidator.java:30-36` | `evaporation.rs:80-84` | **OK** | Exatamente 1 controlador, proibido em arestas ou no topo. |
| Capacidade Tanque Entrada | `GeneralConfig.java:310` (64k mB/bloco) | `evaporation.rs:24-25` (256k mB/nível) | **CONFIRMADA** | Corretamente computado como `height * 256_000` mB. |
| Capacidade Tanque Saída | `GeneralConfig.java:312` (10.000 mB) | `evaporation.rs:26` (10.000) | **OK** | Capacidade fixa independente da altura. |
| Parâmetros Térmicos | `EvaporationMultiblockData.java:46`<br>`GeneralConfig.java:302-308` | `evaporation.rs:17-21` | **OK** | `MAX_TEMP = 3000.0`, `TEMP_MULTIPLIER = 0.4`, `SOLAR = 0.2`, `DISSIPATION = 0.02`. |
| Multiplicador de Velocidade | `EvaporationMultiblockData.java:158` | `evaporation.rs:132` | **OK** | `(min(3000, T) - 300) * 0.4 * (height / 18)`. |
| Escalonador de Operações | `EvaporationMultiblockData.java:274` | `evaporation.rs:136-144` | **OK** | `tick % ceil(1/mult) == 0` para sub-tick; `(int)mult` para >= 1 op/t. |
| Detecção de Painéis Solares | `GeneratorsTileEntityTypes.java:47`<br>`EvaporationMultiblockData.java:293` | `evaporation.rs:277` | **ERRADA-CORRIGIDA** | **O que a R1 não viu**: Java implementa `IEvaporationSolar` **apenas** no `Advanced Solar Generator`. O port aceitava incorretamente o `solar_generator` comum. Corrigido. |
| Filtro de Tanque de Saída | `EvaporationMultiblockData.java:120` | `evaporation.rs:335` | **ERRADA-CORRIGIDA** | **O que a R1 não viu**: tanque de saída não possuía filtro de bloqueio contra inserção externa de fluidos por tubos. Corrigido com `NOTHING`. |
| Slots de Balde no Controlador | `EvaporationMultiblockData.java:123-126` | `evaporation.rs:310-325` | **OK** | 4 slots: 2 para entrada (fill) e 2 para saída (drain). |
| Aquecimento Externo via Válvulas | `TileEntityThermalEvaporationValve.java:28` | `evaporation.rs` | **FORK** | Java aceita aquecimento por condutores térmicos e aquecedor resistivo. Requer rede de calor (FORK-4). |
| Sinal analógico de redstone | `EvaporationMultiblockData.java:312` | `evaporation.rs` | **FORK** | Válvulas emitem sinal analógico proporcional ao nível do tanque de entrada. Requer API. |

---

## 3. Resumo das Correções Aplicadas na Rodada 2

1. **`port/mod/src/dynamic.rs`**:
   - Isolamento de substância no tanque dinâmico: quando `amount > 0`, a válvula agora recebe como filtro unicamente o identificador da substância corrente (`vec![fluid.chemical.clone()]` / `vec![chemical.chemical.clone()]`), impedindo contaminação por outros fluidos ou gases quando parcialmente preenchido.
   - Adicionado teste unitário `tank_filter_locks_substance`.

2. **`port/mod/src/boiler.rs`**:
   - Proteção de tanques de saída: adicionado filtro `NOTHING` nos índices 1 (vapor) e 3 (refrigerante resfriado) para impedir que tubulações externas empurrem substâncias em portas de saída.
   - Higienização de substâncias no `Boiler`: validação estrita para zerar tanques de vapor e refrigerante frio caso contenham produtos divergentes de `STEAM` e `COLD`.
   - Adicionado teste unitário `boiler_cleans_invalid_substances`.

3. **`port/mod/src/evaporation.rs`**:
   - Restrição estrita de aquecimento solar: eliminada permissão indevida de `solar_generator` básico, mantendo unicamente `mekanism:advanced_solar_generator` conforme verificado em `GeneratorsTileEntityTypes.java:47`.
   - Proteção do tanque de saída: configurado filtro `NOTHING` no tanque 1 (salmoura/lítio) para assegurar que seja estritamente de extração.
   - Adicionado teste unitário `only_advanced_solar_powers_evaporation`.
