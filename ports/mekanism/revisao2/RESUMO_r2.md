# Resumo da Revisão Cruzada (Rodada 2) e Especificação

**ID da Tarefa**: T20261109-194317-824  
**Data**: 2026-11-09  
**Referência de Verdade**: Mekanism Java 1.21.1-10.7.19.85 (`java/`)  
**Arquivos Auditados e Modificados no Escopo**:
- `port/mod/src/dynamic.rs` (365 linhas)
- `port/mod/src/boiler.rs` (589 linhas)
- `port/mod/src/evaporation.rs` (460 linhas)
- `port/DIFERENCAS.md` (398 linhas)

---

## 1. Veredito das Correções da Rodada 1

Todas as correções da rodada 1 foram verificadas minuciosamente contra o código fonte Java oficial:

1. **Dynamic Tank: Capacidade de Fluido vs Químico (`dynamic.rs`)**:
   - *Java*: `GeneralConfig.java:169, 171` e `TankMultiblockData.java:134-135`.
   - *Valores*: `350.000` mB/bloco de fluido (`350 * FluidType.BUCKET_VOLUME`) e `16.000.000` mB/bloco de químico (`16_000 * FluidType.BUCKET_VOLUME`).
   - *Veredito*: **CONFIRMADA**.

2. **Thermoelectric Boiler: Coeficiente Inverso de Ar (`boiler.rs`)**:
   - *Java*: `HeatAPI.java:24` define `AIR_INVERSE_COEFFICIENT = 10_000`.
   - *Valor*: `AIR: f64 = 10_000.0`. Impedância total ambiente $= 10\_000 + 100\_000 + 1 = 110\_001.0$.
   - *Veredito*: **CONFIRMADA**.

3. **Thermoelectric Boiler: Altura da Câmara de Vapor (`boiler.rs`)**:
   - *Java*: `BoilerValidator.java:128`: `(renderLocation.getY() + height - 2) - initDisperser.getY()`.
   - *Cálculo*: Como `renderLocation.getY() = min.y + 1` e `height = max.y - min.y + 1`, simplifica para `max.y - initDisperser.getY() = max[1] - plane`.
   - *Veredito*: **CONFIRMADA**.

4. **Thermal Evaporation Plant: Capacidade do Tanque de Entrada (`evaporation.rs`)**:
   - *Java*: `EvaporationMultiblockData.java:222` e `GeneralConfig.java:310`: `inputTankCapacity = (volume / 4) * evaporationFluidPerTank`.
   - *Cálculo*: Torre base 4x4 $\rightarrow$ `volume = 16 * height`, logo `volume / 4 = 4 * height`. Com `64.000` mB/bloco interno, totaliza `4 * 64.000 = 256.000` mB por nível de altura.
   - *Veredito*: **CONFIRMADA**.

---

## 2. O Que a Rodada 1 Não Viu (Correções da Rodada 2)

Durante a revisão profunda das rotinas de tick, filtros e regras de ligação, foram encontradas e corrigidas as seguintes divergências:

1. **`port/mod/src/dynamic.rs` — Contaminação de Tanques por Filtro Vazio**:
   - *Problema*: ao enviar `Vec::new()` como filtro quando `fluid.amount > 0 && < cap`, a API do Horizon aceitava inserção de *qualquer outro fluido* no tanque já em uso, permitindo mistura indevida.
   - *Correção*: o filtro da válvula agora trava na substância atualmente armazenada (`vec![fluid.chemical.clone()]` / `vec![chemical.chemical.clone()]`) quando `amount > 0`, aceitando qualquer tipo apenas se estiver estritamente vazio (`amount == 0`).
   - *Teste Adicionado*: `tank_filter_locks_substance`.

2. **`port/mod/src/boiler.rs` — Proteção de Portas de Saída e Limpeza de Químicos**:
   - *Problema*: os tanques de saída 1 (vapor) e 3 (refrigerante resfriado) não possuíam filtro configurado nas válvulas, permitindo que tubos externos empurrassem gases para dentro deles. Além disso, o struct `Boiler` não limpava substâncias inválidas nos tanques de vapor e refrigerante frio.
   - *Correção*: registrados filtros `NOTHING` nos tanques 1 e 3 para impedir inserção externa; adicionada higienização para zerar tanques de `steam` e `cold` caso recebam substâncias divergentes de `STEAM` e `COLD`.
   - *Teste Adicionado*: `boiler_cleans_invalid_substances`.

3. **`port/mod/src/evaporation.rs` — Painéis Solares e Tanque de Saída**:
   - *Problema*: a detecção de painéis solares aceitava `mekanism:solar_generator` comum. No Java (`GeneratorsTileEntityTypes.java:47`), apenas `TileEntityAdvancedSolarGenerator` implementa `IEvaporationSolar` e expõe `Capabilities.EVAPORATION_SOLAR`. Além disso, o tanque de saída 1 (salmoura/lítio) não tinha filtro bloqueando injeção externa.
   - *Correção*: restrita a detecção nos 4 cantos unicamente a `mekanism:advanced_solar_generator`; adicionado filtro `NOTHING` no tanque 1 para garantir comportamento de saída pura (`VariableCapacityFluidTank.output`).
   - *Teste Adicionado*: `only_advanced_solar_powers_evaporation`.

---

## 3. Contagem e Relação de Testes Unitários

- **Testes por Arquivo**:
  - `port/mod/src/dynamic.rs`: **6** testes
    1. `a_hollow_box_is_a_tank`
    2. `it_has_to_be_hollow_closed_and_sized_right`
    3. `what_valves_gain_and_lose_is_merged`
    4. `a_member_remembers_its_structure`
    5. `tank_capacity_fluid_vs_chemical`
    6. `tank_filter_locks_substance` (adicionado na rodada 2)
  - `port/mod/src/boiler.rs`: **7** testes
    1. `a_box_with_dispersers_and_an_element_is_a_boiler`
    2. `it_needs_a_whole_plane_and_an_element_under_it`
    3. `the_steam_room_is_the_height_above_the_dispersers`
    4. `heated_coolant_boils_the_water`
    5. `the_air_cools_it_slowly`
    6. `a_member_remembers_its_boiler`
    7. `boiler_cleans_invalid_substances` (adicionado na rodada 2)
  - `port/mod/src/evaporation.rs`: **8** testes
    1. `a_tower_with_a_controller_is_a_plant`
    2. `it_needs_the_size_one_controller_and_a_closed_wall`
    3. `the_sun_heats_it_until_the_air_takes_as_much`
    4. `the_temperature_sets_how_many_recipes_run`
    5. `evaporating_turns_the_input_into_the_output`
    6. `a_member_remembers_its_plant`
    7. `evaporation_input_capacity`
    8. `only_advanced_solar_powers_evaporation` (adicionado na rodada 2)
- **Total de Testes no Escopo**: **21** testes.
- **Compilação**: não medida (ambiente sem toolchain Horizon/Rust SDK configurada; dependência `horizon-mod-sdk` aponta para `@SDK@`).

---

## 4. Prova de Mutação

- A tarefa não trouxe lista pré-definida de mutações. Conforme as instruções do orquestrador, a prova de mutação está pendente do juiz.

---

## 5. Parte B: Especificação do Grupo Calor e Transporte

Especificação completa, autossuficiente e exaustiva documentada em `.saida/T20261109-194317-824/ESPEC_calor_transporte.md` (404 linhas):
1. **Thermodynamic Conductor (Rede de Calor Completa)**: 4 tiers (Basic, Advanced, Elite, Ultimate), condutividade inversa 5.0, isolamento inverso (10, 400, 8.000, 100.000), algoritmos de convecção com ar ambiente (`simulateEnvironment`) e condução adjacente (`simulateAdjacent`), upgrade in-world com ligas, cálculo de eficiência percentual e requisitos FORK-4.
2. **Resistive Heater**: buffer dinâmico de 400 ticks (`usage * 400`), eficiência de 60% (`resistiveHeaterEfficiency = 0.6`), consumo padrão de 100 J/t (40 FE/t) gerando 60 J/t de calor, interface gráfica com campo de texto digital (`GuiTextField`), pacotes `PacketGuiSetEnergy`, luz 8 e derretimento de gelo.
3. **Fuelwood Heater**: queima de qualquer combustível vanilla, geração de 400 J de calor por tick (`heatPerFuelTick`), multiplicador de queima 1 (`fuelwoodTickMultiplier`), animação de chama na GUI, partículas de fumaça e faíscas.
4. **Laser Tractor Beam**: buffer de 5.000.000.000 J (2.000.000.000 FE), custo de escavação de 100.000 J (40.000 FE) por dureza, absorção de blocos quebrados e `ItemEntity` em trânsito para 27 slots internos de saída, transbordamento com vetor de impulso e sinal de comparador.
5. **Restrictive Transporter**: custo de roteamento A* fixado em 1.000 (transbordamento logístico), velocidade Basic de 50 ticks/bloco.
6. **Diversion Transporter**: modos independentes por face (`DiversionControl`: DISABLED, HIGH, LOW) acionados por clique e dependentes de sinal de redstone vizinho, reavaliação dinâmica de rota e desconexão de capability.

---

## 6. Arquivos Gerados na Saída

1. `.saida/T20261109-194317-824/REVISAO_dynamic_boiler_evaporation.md` (90 linhas)
2. `.saida/T20261109-194317-824/ESPEC_calor_transporte.md` (404 linhas)
3. `.saida/T20261109-194317-824/BLOCO_DIFERENCAS_t2.md` (11 linhas)
4. `.saida/T20261109-194317-824/RESUMO.md` (este documento)
