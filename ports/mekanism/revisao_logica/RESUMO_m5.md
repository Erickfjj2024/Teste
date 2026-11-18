# RESUMO DE EXECUÇÃO: Revisão de Lógica do Fission Reactor

**Tarefa:** T20261109-230337-0z4 (continuação direta de T20261109-220615-jd9 após reinício da VM)  
**Data:** 2026-11-09  
**Status:** CONCLUÍDO

---

## 1. Escopo e Arquivos Modificados
- **Arquivo de código modificado:**
  - `port/mod/src/fission.rs` (977 linhas)
- **Arquivos de documentação gerados em `.saida/T20261109-230337-0z4/`:**
  - `.saida/T20261109-230337-0z4/LOGICA_fission.md` (16.594 bytes)
  - `.saida/T20261109-230337-0z4/FORK_fission.md` (7.649 bytes)
  - `.saida/T20261109-230337-0z4/RESUMO.md` (este documento)

---

## 2. Correções de Lógica Implementadas em `port/mod/src/fission.rs`

1. **BUG 1 — Suporte a `mekanism:reactor_glass` nas paredes externas:**
   - Adicionada constante `GLASS = "mekanism:reactor_glass"` e variante `Part::Glass`.
   - Regra atualizada em `validate`: posições de face (`edges == 1`) aceitam `Part::Glass` junto de `Port` e `Adapter`. Posições de quina/aresta (`edges > 1`) continuam exigindo exclusivamente `Casing`.
2. **BUG 2 — Queima contínua de frações decimais residuais de combustível:**
   - No loop `tick`, a queima de combustível agora processa frações residuais de `burn_remaining` mesmo quando o tanque inteiro atinge 0 mB (`(r.fuel.chemical == FISSILE && r.fuel.amount > 0) || r.burn_remaining > 0.0`), prevenindo o congelamento de frações de combustível.
3. **BUG 3 — Preservação de taxa de queima zero (`rate_limit = 0.0`):**
   - Inicialização padronizada de novos reatores em `DEFAULT_RATE` (0.1) e preservação estrita de `saved.rate_limit >= 0.0` em vez de `> 0.0`, impedindo que reatores desligados ou pausados em taxa 0.0 reacendam sozinhos após descarregar/carregar chunks ou reiniciar o mundo.
4. **BUG 4 — Validação da substância química da água na fervura:**
   - O tanque de água agora valida explicitamente que a substância presente é `minecraft:water` (`r.water.chemical == WATER || r.water.chemical.is_empty()`), evitando reações indevidas com outros líquidos inseridos acidentalmente.
5. **BUG 5 (Sintoma S3) — Bloqueio estrito de portas em estruturas inválidas/desmontadas:**
   - Quando um reator é quebrado ou fica inválido, todas as portas vinculadas têm suas faces desacopladas (`FlowSide::None`), filtros de tanques fixados em `NOTHING`, capacidades aparentes zeradas e menus suprimidos, impedindo que portas isoladas funcionem como supertanques no mundo. O conteúdo permanece armazenado na memória da estrutura para restauração na reforma.
6. **BUG 6 (Sintoma S2) — Notificação de formação de estrutura para jogadores próximos:**
   - Ao transicionar de inválido para válido, o reator envia notificação de sucesso aos jogadores em um raio de 40 blocos na dimensão via `players::tell`.

---

## 3. Testes Unitários e Métricas

### Contagem de Testes em `port/mod/src/fission.rs`
- **Total de funções anotadas com `#[test]`:** 13
- **Testes pré-existentes:** 8
  1. `a_column_of_fuel_under_a_rod_is_a_reactor`
  2. `the_columns_have_to_be_whole`
  3. `burning_fuel_makes_heat_waste_and_steam`
  4. `sodium_becomes_superheated_sodium`
  5. `heat_damages_and_cold_repairs`
  6. `the_buttons_work_within_the_limits`
  7. `a_member_remembers_its_reactor`
  8. `the_roll_is_in_range`
- **Novos testes adicionados nesta rodada:** 5
  9. `reactor_glass_is_allowed_on_walls_but_not_edges` (valida aceitação de vidro em paredes e rejeição em quinas/arestas)
  10. `burn_remaining_burns_to_completion_when_fuel_tank_is_zero` (valida combustão até zerar `burn_remaining` sem combustível novo)
  11. `zero_burn_rate_is_preserved_across_loads` (valida persistência de taxa 0.0)
  12. `reactor_dimensions_minimum_and_maximum_bounds` (valida limites 3..=18 e altura mínima funcional)
  13. `port_cycle_modes_and_names` (valida ciclo dos 3 modos de porta: Input, Output Waste, Output Coolant)

### Medição de Execução de Testes
- **Tempo e execução:** não medido (o ambiente de execução da VM / Cloud Shell Hub não possui toolchain local do Rust/Cargo instalado para compilação WASM).
- **Prova de mutação:** pendente do juiz (a tarefa não forneceu lista prévia de mutações para validação).

---

## 4. Requisitos de Fork do Horizon Documentados (`FORK_fission.md`)
- **FORK-21 (Detalhamento Fissão):** Suporte a portas lógicas (Logic Adapter) com emissão e recepção de redstone digital e emissão analógica de comparador proporcional aos tanques.
- **FORK-29 (Sintoma S1):** Controle condicional de uso de blocos (`on-block-use` retornando `pass-to-item` quando a estrutura for inválida, permitindo colocar blocos sem agachar).
- **FORK-30 (Sintoma S2):** Suporte a spawn de partículas do lado do servidor (`world.spawn-particles`) para faíscas nas arestas da estrutura formada.
- **FORK-31 (Sintoma S4):** Renderização tridimensional de refrigerante e vapor dentro de multiblocos com paredes de vidro estrutural.
- **FORK-32:** Consulta de entidades biológicas vivas em bounding box (`get-entities-in-bounds`) para aplicação de irradiação na zona quente interna do reator.

---

## 5. Pendências e Riscos
- **Compilação Rust:** Todo o código em `port/mod/src/fission.rs` utiliza estritamente os tipos e APIs existentes (`horizon`, `std`, e o submódulo `dynamic`), porém a compilação final em WASM será realizada pelo orquestrador no PC receptor.
- **Comportamento S1 e S4:** A resolução definitiva em jogo dos sintomas S1 (abertura de menu com bloco na mão) e S4 (renderização do fluido interno) depende da implementação das propostas `FORK-29` e `FORK-31` no motor Horizon.
