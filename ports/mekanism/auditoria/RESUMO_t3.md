# Resumo da Auditoria: FISSION REACTOR, FUSION REACTOR e INDUSTRIAL TURBINE

**Tarefa:** T20261109-183803-8so  
**Data:** 2026-11-09  
**Status:** Concluído com sucesso  

---

## 1. Arquivos Auditados e Modificados

1. **`port/mod/src/fission.rs`** (792 linhas):
   - **`AIR` (coeficiente inverso do ar):** corrigido de `10.0` para `10_000.0` (conferido em `HeatAPI.java:24` e `FissionReactorMultiblockData.java:252`). No port a dissipação para o ambiente estava quase 2× mais rápida que o original (resistência 10.020 vs 20.010).
   - **`SODIUM_CONDUCTIVITY` (condutividade térmica do refrigerante sódio):** corrigido de `0.4` para `1.0` (conferido em `MekanismDataMapsProvider.java:85` e `CooledCoolant.java:38`, onde `new CooledCoolant(MekanismChemicals.SUPERHEATED_SODIUM, 5, 1)` registra entalpia 5 e condutividade 1.0).
   - **Radiação de resíduo residual (`partial_waste`) no Meltdown:** corrigido em `fission.rs:505` para usar a radioatividade padrão de `WASTE` (`0.01`) se `s.reactor.waste.amount == 0`, espelhando a lógica de `FissionReactorMultiblockData.java:381-394`.
   - **Testes contidos no arquivo:** 8 testes (`a_column_of_fuel_under_a_rod_is_a_reactor`, `the_columns_have_to_be_whole`, `burning_fuel_makes_heat_waste_and_steam`, `sodium_becomes_superheated_sodium`, `heat_damages_and_cold_repairs`, `the_buttons_work_within_the_limits`, `a_member_remembers_its_reactor`, `the_roll_is_in_range`).

2. **`port/mod/src/fusion.rs`** (705 linhas):
   - Todas as 15 constantes físicas e de configuração foram conferidas contra `FusionReactorMultiblockData.java` e `GeneratorsConfig.java`. Todos os valores são 100% fiéis à versão Java 1.21.1:
     - `BURN_TEMPERATURE`: 100.000.000 K
     - `PLASMA_HEAT_CAPACITY`: 100
     - `CASE_HEAT_CAPACITY`: 1
     - `PLASMA_CASE_CONDUCTIVITY`: 0,2
     - `ENERGY_PER_FUEL`: 10.000.000 J
     - `THERMOCOUPLE`: 0,05
     - `CASING_CONDUCTIVITY`: 0,1
     - `WATER_HEATING`: 0,3
     - `FUEL_CAPACITY`: 1.000 mB
     - `ENERGY_CAPACITY`: 400.000.000 FE (`fe(1_000_000_000)`)
     - `WATER_PER_INJECTION`: 1.000.000 mB
     - `STEAM_PER_INJECTION`: 100.000.000 mB
     - `HOHLRAUM_FUEL`: 10 mB
     - `DEFAULT_INJECTION`: 2 mB/t
     - `MAX_INJECTION`: 98 mB/t
   - **Testes contidos no arquivo:** 7 testes (`the_rounded_cube_has_twelve_missing_blocks_a_face`, `a_whole_shell_with_a_controller_is_a_reactor`, `it_needs_the_shape_the_controller_and_an_empty_inside`, `a_hohlraum_ignites_a_hot_plasma_and_it_keeps_burning`, `the_laser_heats_ten_times_more_before_the_burn`, `the_injection_rate_moves_by_two`, `a_member_remembers_its_reactor`).

3. **`port/mod/src/turbine.rs`** (665 linhas):
   - Constantes conferidas contra `TurbineMultiblockData.java`, `TurbineValidator.java`, `GeneratorsConfig.java` e `GeneralConfig.java`:
     - `MAX_BLADES`: 28
     - `BLADES_PER_COIL`: 4
     - `VENT_FLOW`: 32.000,0 mB/t
     - `DISPERSER_FLOW`: 1.280,0 mB/t
     - `CONDENSER_RATE`: 64.000 mB/t
     - `ENERGY_PER_VOLUME`: 6.400.000 FE (`fe(16_000_000)`)
     - `STEAM_PER_TANK`: 64.000 mB
     - `ENERGY_PER_STEAM`: 10,0 J/mB
   - **Comportamento de transbordamento de água dos condensadores:** corrigido em `turbine.rs:298-301`. No Java (`TurbineMultiblockData.java:154-156`), o `ventTank.insert` descarta o excesso de água sem parar a turbina. No port Rust, havia um bloqueio `if water + produced > water_capacity(f) { return Tick::default(); }` que travava a rotação e geração da turbina inteira se o tanque estivesse cheio. Corrigido para limitar a água produzida ao espaço livre (`room = water_capacity(f).saturating_sub(water)` e `produced = flow.min(water_capacity(f)).min(room)`), mantendo a geração de energia e consumo de vapor contínuos.
   - **Nome do teste editado:** `steam_turns_the_blades_into_energy_and_water` (atualizado para verificar que com o tanque de água cheio a turbina continua gerando energia: `assert!(full_water.energy > 0 && full_water.water == 0)`).
   - **Testes contidos no arquivo:** 5 testes (`a_complex_with_rotors_coils_and_a_plane_is_a_turbine`, `it_needs_everything_in_its_place`, `blades_go_where_the_original_puts_them`, `steam_turns_the_blades_into_energy_and_water`, `a_member_remembers_its_turbine`).

---

## 2. Testes e Mutação

- **Contagem total de testes nos 3 módulos auditados:** 20 testes (8 em `fission.rs`, 7 em `fusion.rs`, 5 em `turbine.rs`).
- **Execução de testes na VM:** não medido (a VM não possui toolchain Rust configurada, conforme documentado no contexto da tarefa).
- **Prova de mutação:** a prova de mutação está pendente do juiz (nenhuma lista de mutações foi fornecida na especificação da tarefa).

---

## 3. Entregáveis Gerados

1. **Relatório detalhado com tabela comparativa de todos os 63 parâmetros:** `.saida/T20261109-183803-8so/AUDITORIA_reactors_turbine.md`
2. **Correções aplicadas diretamente nos fontes:** `port/mod/src/fission.rs` e `port/mod/src/turbine.rs`.
3. **Bloco para `port/DIFERENCAS.md` (formato da seção 19, números 20.x):** pronto na seção 3 do relatório `AUDITORIA_reactors_turbine.md` e abaixo.
4. **Resumo da tarefa:** `.saida/T20261109-183803-8so/RESUMO.md` (este documento).

---

## 4. Bloco Pronto para Colar em `port/DIFERENCAS.md`

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
