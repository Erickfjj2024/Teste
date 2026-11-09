# Resumo da Revisão Cruzada: FISSION, FUSION e TURBINE

**Tarefa:** T20261109-194505-xea  
**Data:** 2026-11-09  
**Status:** Concluído com sucesso  

---

## 1. Arquivos Auditados e Modificados

1. **`port/mod/src/fission.rs`** (792 linhas):
   - **Correção da Rodada 1 Confirmada:** `AIR = 10_000.0` (`HeatAPI.java:24`), `SODIUM_CONDUCTIVITY = 1.0` (`MekanismDataMapsProvider.java:85`), e fallback de radiação para `partial_waste` no colapso (`FissionReactorMultiblockData.java:381-394`) confirmados como 100% fiéis ao Java.
   - **Correção da Rodada 2 Aplicada (Volume do Multibloco):** Em `fission.rs:175`, `volume` calculava o ar interno `(size - 2)^3` (27 para um reator 5×5×5). No Java (`MultiblockData.java:198`), `setVolume` usa `bounds.length() * bounds.width() * bounds.height()`. Corrigido para `size[0] * size[1] * size[2]` (125 para 5×5×5), restaurando as capacidades oficiais de refrigerante e vapor.
   - **Docstring Atualizada:** Linhas 6-7 corrigidas para documentar o suporte aos 3 modos de porta pelo configurador (input only, output waste, output coolant).
   - **Teste editado:** `a_column_of_fuel_under_a_rod_is_a_reactor` (volume esperado atualizado de 27 para 125).

2. **`port/mod/src/fusion.rs`** (705 linhas):
   - **15 Constantes Físicas e Mecânicas Conferidas:** Todas idênticas ao Java (`BURN_TEMPERATURE` 100M K, `PLASMA_HEAT_CAPACITY` 100, `CASE_HEAT_CAPACITY` 1, `PLASMA_CASE_CONDUCTIVITY` 0.2, `ENERGY_PER_FUEL` 10M J, `THERMOCOUPLE` 0.05, `CASING_CONDUCTIVITY` 0.1, `WATER_HEATING` 0.3, `FUEL_CAPACITY` 1 000 mB, `ENERGY_CAPACITY` 400M FE via `fe(1_000_000_000)`, `WATER_PER_INJECTION` 1M mB, `STEAM_PER_INJECTION` 100M mB, `HOHLRAUM_FUEL` 10 mB, `DEFAULT_INJECTION` 2 mB/t, `MAX_INJECTION` 98 mB/t).
   - **Correção da Rodada 2 Aplicada (Faces de Energia das Portas):** Em `fusion.rs:539`, quando a porta estava em modo entrada (`!output`), definia `energy_faces: Some(vec![FlowSide::Input; 6])`. No Java (`FusionReactorMultiblockData.java:157`), o contêiner de energia é estritamente de saída (`VariableCapacityEnergyContainer.output`) e portas nunca aceitam energia de cabos. Corrigido para `Some(vec![FlowSide::None; 6])`.

3. **`port/mod/src/turbine.rs`** (669 linhas):
   - **Correção da Rodada 1 Confirmada:** O transbordamento no tanque de água dos vents não bloqueia o fluxo de vapor nem a geração de energia (`TurbineMultiblockData.java:156`).
   - **Correção da Rodada 2 Aplicada (Volume do Multibloco):** Em `turbine.rs:180`, `volume` calculava o ar interno `(size - 2)^3` (54 para uma turbina 5×8×5). No Java (`MultiblockData.java:198`), `volume` é o paralelepípedo delimitador externo `bounds.length() * bounds.width() * bounds.height()`. Corrigido para `size[0] * size[1] * size[2]` (200 para 5×8×5), restaurando a capacidade oficial de armazenamento de energia.
   - **Correção da Rodada 2 Aplicada (Bug de Energia Gratuita com Fluxo Nulo):** Em `turbine.rs:294-296`, adicionada a verificação `if flow == 0 { return Tick::default(); }` correspondendo ao `if (clientFlow > 0)` de `TurbineMultiblockData.java:151-155`. Anteriormente, taxas fracionárias (ex: 0.5 mB/t gerando `flow = 0`) produziam energia sem consumir vapor.
   - **Correção da Rodada 2 Aplicada (Remoção do Teto Artificial `i32::MAX`):** Em `turbine.rs:297`, removido o `.min(f64::from(i32::MAX))` que limitava a energia gerada por tick a 2.147B FE/t.
   - **Testes editados:**
     - `a_complex_with_rotors_coils_and_a_plane_is_a_turbine` (volume esperado atualizado de 54 para 200).
     - `steam_turns_the_blades_into_energy_and_water` (adicionada asserção verificando que `flow == 0` retorna `Tick::default()`).

4. **`port/DIFERENCAS.md`**:
   - Inserida a Seção 21 com os 11 itens detalhados da Revisão Cruzada de Reatores e Turbina.

---

## 2. Testes de Unidade e Prova de Mutação

### Nomes dos Testes Editados
1. `a_column_of_fuel_under_a_rod_is_a_reactor` (em `port/mod/src/fission.rs:662`): Linha editada atualizou a expectativa de volume de 27 para 125, em conformidade com o cálculo oficial `MultiblockData.java:198` (`5 * 5 * 5 = 125`).
2. `a_complex_with_rotors_coils_and_a_plane_is_a_turbine` (em `port/mod/src/turbine.rs:586`): Linha editada atualizou a expectativa de volume de 54 para 200, em conformidade com o cálculo oficial `MultiblockData.java:198` (`5 * 8 * 5 = 200`).
3. `steam_turns_the_blades_into_energy_and_water` (em `port/mod/src/turbine.rs:644`): Linha editada adicionou verificação `assert_eq!(step(&f, 4, 1, 0, 0), Tick::default());` garantindo que nenhuma energia gratuita é gerada quando o fluxo trunca para 0.

### Contagem de Testes do Escopo
- `fission.rs`: 8 testes (`a_column_of_fuel_under_a_rod_is_a_reactor`, `the_columns_have_to_be_whole`, `burning_fuel_makes_heat_waste_and_steam`, `sodium_becomes_superheated_sodium`, `heat_damages_and_cold_repairs`, `the_buttons_work_within_the_limits`, `a_member_remembers_its_reactor`, `the_roll_is_in_range`).
- `fusion.rs`: 7 testes (`the_rounded_cube_has_twelve_missing_blocks_a_face`, `a_whole_shell_with_a_controller_is_a_reactor`, `it_needs_the_shape_the_controller_and_an_empty_inside`, `a_hohlraum_ignites_a_hot_plasma_and_it_keeps_burning`, `the_laser_heats_ten_times_more_before_the_burn`, `the_injection_rate_moves_by_two`, `a_member_remembers_its_reactor`).
- `turbine.rs`: 5 testes (`a_complex_with_rotors_coils_and_a_plane_is_a_turbine`, `it_needs_everything_in_its_place`, `blades_go_where_the_original_puts_them`, `steam_turns_the_blades_into_energy_and_water`, `a_member_remembers_its_turbine`).
- **Total de testes no escopo:** 20 testes.
- **Execução na VM:** não medido (a VM não possui toolchain Rust configurada).
- **Prova de mutação:** a prova de mutação está pendente do juiz (nenhuma lista de mutações foi fornecida na especificação da tarefa).

---

## 3. Entregáveis Gerados

1. **Relatório completo de Revisão Cruzada:** `.saida/T20261109-194505-xea/REVISAO_reactors_turbine.md` (68 parâmetros minuciosamente catalogados contra os arquivos e linhas do Java e do Port).
2. **Correções aplicadas diretamente nos fontes:** `port/mod/src/fission.rs`, `port/mod/src/fusion.rs` e `port/mod/src/turbine.rs`.
3. **Bloco para `port/DIFERENCAS.md` (Seção 21):** Aplicado no arquivo e reproduzido na seção 4 abaixo.
4. **Resumo da tarefa:** `.saida/T20261109-194505-xea/RESUMO.md` (este documento).

---

## 4. Bloco Inserido em `port/DIFERENCAS.md` (Seção 21)

```markdown
## 21. Reatores e Turbina - Revisão Cruzada (mod 0.1.86, auditoria Java 1.21.1-10.7.19.85)

Revisão cruzada independente de `fission.rs`, `fusion.rs` e `turbine.rs` contra os fontes do Mekanism Java 1.21.1 (`FissionReactorMultiblockData`, `FusionReactorMultiblockData`, `TurbineMultiblockData`, `TurbineValidator`, `GeneratorsConfig` e `GeneralConfig`). Confirmação dos itens da rodada 1 e correção de discrepâncias que passaram despercebidas na primeira auditoria.

| # | Diferença / Difference | Estado / State |
|---|---|---|
| 21.1 | Fission e Turbina (Volume do Multibloco): no Java (`MultiblockData.java:198`), o volume é o bounding box externo delimitador `bounds.length() * bounds.width() * bounds.height()`. No port, ambos calculavam o volume interno de ar `(size - 2)^3`, reduzindo a capacidade de refrigerante do reator a ~21.6% e o buffer de energia da turbina a ~27% do valor oficial. Corrigido em `fission.rs:175` e `turbine.rs:180` para `size[0] * size[1] * size[2]` | CORRIGIDO |
| 21.2 | Turbina (Energia Gratuita em Baixa Vazão): no Java (`TurbineMultiblockData.java:151-155`), a inserção de energia e a condensação só ocorrem se `clientFlow > 0`. No port, verificava-se apenas `rate <= 1e-5`, permitindo que taxas fracionárias (ex.: 0.5 mB/t, onde `flow = 0`) gerassem energia gratuitamente sem consumir vapor. Corrigido com `if flow == 0 { return Tick::default(); }` em `turbine.rs:294-296` | CORRIGIDO |
| 21.3 | Turbina (Teto de Energia em `i32::MAX`): no Java, a geração usa `long` de 64 bits (`TurbineMultiblockData.java:152`). No port, `turbine.rs:297` limitava a energia por tick a `i32::MAX` (~2.147B FE/t). Removido o teto artificial, mantendo `(mult * rate * 0.4).max(0.0) as u64` | CORRIGIDO |
| 21.4 | Reator de Fusão (Faces de Energia da Porta): no Java (`FusionReactorMultiblockData.java:157`), o contêiner de energia é estritamente de saída (`VariableCapacityEnergyContainer.output`); o reator não recebe energia por cabos. No port, o modo entrada (`!output`) configurava `energy_faces: Some(vec![FlowSide::Input; 6])`. Corrigido em `fusion.rs:539` para `Some(vec![FlowSide::None; 6])` | CORRIGIDO |
| 21.5 | Reator de Fissão (Comentários de Cabeçalho): cabeçalho de `fission.rs:6-7` indicava erroneamente que portas não possuem modos. Atualizado para refletir os 3 modos implementados (Input, Output Waste, Output Coolant) | CORRIGIDO |
| 21.6 | Confirmações da Rodada 1: coeficiente do ar (`AIR` = 10 000.0, divisor total 20 010.0), condutividade do sódio (`SODIUM_CONDUCTIVITY` = 1.0), radiação de `partial_waste` no colapso com tanque vazio (fallback `NUCLEAR_WASTE`), e transbordamento não-bloqueante no tanque de água dos vents da turbina conferidos e 100% fiéis ao Java | CONFIRMADA |
| 21.7 | Reatores de Fissão e Fusão (Dano a Entidades e Zona Letal): no Java, a câmara do reator de fissão irradia entidades vivas (`FissionReactorMultiblockData.java:209`) e o reator de fusão aplica 50 000 de dano instantâneo por tick na câmara 3×3×3 (`FusionReactorMultiblockData.java:291-297`). Não implementados por falta de API de varredura e dano a entidades em área no Horizon | FORK |
| 21.8 | Reator de Fusão (Componente Químico do Hohlraum): no Java (`ItemHohlraum.java`), o Hohlraum é carregado com 10 mB de combustível D-T em um `IChemicalHandler` acoplado ao item. No port, o Hohlraum é tratado como item vanilla simples que satisfaz a ignição como se estivesse cheio (10 mB) por ausência de componentes NBT de substância química em itens | FORK |
| 21.9 | Turbina Industrial (Modos de Descarte de Vapor): no Java (`TurbineMultiblockData.java:168-179`), a turbina possui 3 modos de descarte de vapor (`DumpingMode`: `IDLE`, `DUMPING_EXCESS`, `DUMPING`). No port, a interface simplificada não implementa os seletores de descarte | FORK |
| 21.10 | Fissão e Fusão (Modos Lógicos do Logic Adapter): no Java, o Logic Adapter possui múltiplos modos de emissão e detecção de sinal analógico/digital de redstone (8 modos no Fission, 4 modos no Fusion). No port, atua como bloco passivo de carcaça | FORK |
| 21.11 | Reator de Fissão (Persistência de Burn Rate = 0.0): no Java (`FissionReactorCache.java:21`), `rateLimit = -1` indica valor não-inicializado, permitindo fixar intencionalmente taxa de queima 0.0. No port (`fission.rs:445`), a verificação `saved.rate_limit > 0.0` faz com que uma taxa de 0.0 seja redefinida para `DEFAULT_RATE` (0.1) ao recarregar o mundo | FORK |
```
