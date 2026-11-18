# RESUMO DE EXECUÇÃO: REVISÃO DE LÓGICA DE MULTIBLOCOS (EVAPORATION & SPS)

**ID da Tarefa**: T20261109-230225-dg0  
**Data**: 2026-11-09  
**Status**: CONCLUÍDO

---

## 1. Arquivos Editados no Escopo

- `port/mod/src/evaporation.rs` (679 linhas)
- `port/mod/src/sps.rs` (728 linhas)
- Total de linhas nos arquivos do escopo: 1407 linhas

---

## 2. Entregáveis Gerados

1. `.saida/T20261109-230225-dg0/LOGICA_evaporation_sps.md`:
   - Análise detalhada dos 5 eixos de lógica de multiblocos (forma, formação/desmontagem/cache, estado entre blocos, tick/operação e casos limite).
   - Cobertura explícita dos sintomas S1 a S5 relatados em `.saida/T20261109-230225-dg0/anexos/SINTOMAS_MULTIBLOCOS.md`.
   - Tabela comparativa exaustiva confrontando `java/` (arquivo:linha) e `port/` (arquivo:linha).
   - Inventário dos 7 bugs identificados e corrigidos.
2. `.saida/T20261109-230225-dg0/FORK_evaporation_sps.md`:
   - Especificações completas com assinaturas WIT para os requisitos:
     - FORK-4 (Transferência térmica em válvulas)
     - FORK-7 (Dano letal físico na câmara interna do SPS)
     - FORK-10 (Transformação de mobs e raios cosméticos do SPS)
     - FORK-19 / FORK-31 (Renderização de fluidos internos e núcleo de plasma)
     - FORK-21 (Sinal de comparador analógico por válvulas/portas)
     - FORK-29 (Uso condicional de blocos de multibloco para permitir colocação de bloco sem shift — Sintoma S1)
     - FORK-30 (Sistema de partículas de formação de multibloco — Sintoma S2)
3. `.saida/T20261109-230225-dg0/RESUMO.md` (este documento).

---

## 3. Resumo dos Bugs Encontrados e Corrigidos

1. **Vidro Estrutural em Arestas e Quinas Estruturais** (`evaporation.rs:78-89`, `sps.rs:104-109`):
   - *Causa*: `part_of` agrupava vidro como casing/block normal.
   - *Correção*: Diferenciado `Part::Glass`. Arestas (frame) exigem estritamente `Part::Block` / `Part::Casing`, rejeitando vidro.
2. **Válvula ou Porta Solitária Mantendo Funcionamento Após Desmontagem (Sintoma S3)** (`evaporation.rs:244-255, 290-312`, `sps.rs:374-386, 420-440`):
   - *Causa*: Blocos não formados eram ignorados nas funções de tick e mantinham tanques de registro (`SPEC_CAPACITY`).
   - *Correção*: Membros não formados agora têm faces fechadas (`FlowSide::None`), filtros travados em `NOTHING`, energia zerada e menus suprimidos.
3. **Duplicação de Fluidos por Baldes no Controlador** (`evaporation.rs:309-322`):
   - *Causa*: Baldes eram processados após a gravação de `plant.given`, fazendo `dynamic::merge` registrar 1000 mB a mais no tick seguinte.
   - *Correção*: Processamento de baldes antecipado para antes do espelhamento de dados.
4. **Mistura de Fluidos no Tanque de Entrada por Canos Externos** (`evaporation.rs:410-416`):
   - *Causa*: Filtros anunciavam todas as receitas mesmo com tanque parcialmente cheio de outro líquido.
   - *Correção*: Filtro trava na substância armazenada enquanto houver líquido no tanque.
5. **Deadlock no SPS com Tanque de Antimatéria Quase Cheio** (`sps.rs:195-200`):
   - *Causa*: `process` rejeitava operações se a produção total excedesse o espaço livre.
   - *Correção*: Quantidade de polônio limitada ao espaço restante (`max_polonium_room`).
6. **Consumo de Energia por Portas sem Supercharged Coil no SPS** (`sps.rs:472-477, 502-512`):
   - *Causa*: Portas sem coil aceitavam energia e a canalizavam para o processo.
   - *Correção*: Fechada a entrada de energia (`energy_faces: FlowSide::None`) em portas desprovidas de coil.
7. **Ausência de Emissão de Redstone por Comparador Analógico** (`evaporation.rs:200-208, 451`, `sps.rs:245-253, 497`):
   - *Causa*: Comparadores não recebiam sinal proporcional ao conteúdo do tanque.
   - *Correção*: Implementado `comparator_level` (0..15) integrado ao stub `host_set_comparator_output` (`FORK-21`).

---

## 4. Testes Unitários e Métricas

Total de testes unitários nos arquivos do escopo: **24 testes** (13 em `evaporation.rs`, 11 em `sps.rs`).

### Lista de Testes em `port/mod/src/evaporation.rs` (13 testes):
- `a_tower_with_a_controller_is_a_plant`
- `it_needs_the_size_one_controller_and_a_closed_wall`
- `the_sun_heats_it_until_the_air_takes_as_much`
- `the_temperature_sets_how_many_recipes_run`
- `evaporating_turns_the_input_into_the_output`
- `a_member_remembers_its_plant`
- `evaporation_input_capacity`
- `only_advanced_solar_powers_evaporation`
- `structural_glass_allowed_on_walls_and_rim_not_on_edges`
- `controller_in_roof_corner_causes_conflict`
- `evaporation_comparator_level`
- `evaporation_clamps_stored_fluid_on_rebuild_or_shrink`
- `unformed_valve_locks_tank_faces_and_filters_nothing`

### Lista de Testes em `port/mod/src/sps.rs` (11 testes):
- `the_shape_misses_twelve_blocks_a_face`
- `a_shell_with_a_coil_behind_a_port_is_an_sps`
- `a_coil_has_to_be_behind_a_port`
- `energy_turns_polonium_into_antimatter`
- `a_lot_of_energy_makes_whole_antimatter_at_once`
- `a_member_remembers_its_sps`
- `sps_constants_match_java_config`
- `sps_structural_glass_allowed_at_requirement_2_not_at_frame`
- `sps_process_does_not_deadlock_when_output_nearly_full`
- `sps_death_zone_bounds_and_comparator`
- `unformed_sps_port_locks_faces_and_clears_energy`

### Prova de Mutação
A prova de mutação está pendente do juiz (a tarefa não forneceu lista de mutantes pré-definida).

### Compilação e Trechos mais Arriscados
Ambiente sem toolchain `rustc`/`cargo`. O código foi manualmente revisado contra AST, semântica de empréstimo (borrowing) e tipos do Horizon. Os trechos de maior sensibilidade identificados e validados foram:
1. `evaporation.rs`: Clamping e sincronização em `evaporation_structures` com filtro condicional de jogadores via `players::online()`.
2. `sps.rs`: Resolução de espaço em `process` (`max_polonium_room`) prevenindo underflow em `room - 1`.
3. `sps.rs`: Closure `has_coil` checando limites de coordenadas relativas e conversão segura para `usize`.
