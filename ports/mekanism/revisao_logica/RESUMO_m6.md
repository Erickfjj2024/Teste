# Resumo de Execução: Auditoria da Lógica do Fusion Reactor

**Tarefa:** `T20261109-230438-3c3`  
**Escopo de código:** `port/mod/src/fusion.rs`  
**Referência Java:** Mekanism 1.21.1-10.7.19.85 (`FusionReactorValidator.java`, `FusionReactorMultiblockData.java`, `FusionReactorCache.java`, `TileEntityFusionReactorBlock.java`, `TileEntityFusionReactorController.java`, `TileEntityFusionReactorPort.java`, `TileEntityFusionReactorLogicAdapter.java`, `TileEntityLaserFocusMatrix.java`, `RenderFusionReactor.java`, `CuboidStructureValidator.java`, `FormationProtocol.java`, `StructureHelper.java`, `MultiblockManager.java`, `MultiblockCache.java`)

---

## 1. Arquivos Alterados e Gerados

- **Código editado:**
  - `port/mod/src/fusion.rs`: 1127 linhas (expandindo a implementação de 705 linhas original).
- **Relatórios gerados em `.saida/T20261109-230438-3c3/`:**
  - `LOGICA_fusion.md`: 80 linhas (tabela comparativa completa com 20 aspectos passo a passo e detalhamento dos 7 bugs encontrados e corrigidos).
  - `FORK_fusion.md`: 124 linhas (especificação formal de assinaturas WIT para FORK-19, FORK-21, FORK-29, FORK-30 e FORK-7/11).
  - `RESUMO.md`: este relatório de entrega.

---

## 2. Cobertura de Sintomas (`SINTOMAS_MULTIBLOCOS.md`)

- **S1 (Menu e interação com sneak / sem agachar):**  
  Implementado no port: `take_click` verifica `click.sneak`. Interação com sneak + configurador alterna o modo de porta (Input/Output). O menu do reator abre apenas no controlador e somente quando o multibloco está formado (`valid == true`). Portas e controlador em estruturas inválidas ou não formadas retornam `menu_data: None`.
- **S2 (Partículas e notificação ao formar):**  
  Corrigido em `fusion_structures`: quando `!was_formed && found.valid`, dispara notificação `players::tell` ("Multiblock formed") para jogadores num raio de 40 blocos e invoca `spawn_formation_particles` (`// FORK-30`) para faíscas ao longo das arestas do cubóide.
- **S3 (Estrutura quebrada / válvula e porta avulsa funcionando como tanque):**  
  Corrigido integralmente: em `fusion_structures` (para membros de estruturas desfeitas) e em `run_fusion` (para portas/controladores avulsos ou não formados), todas as 6 faces de tanques são travadas (`FlowSide::None`), faces de energia são travadas (`FlowSide::None`), energia é forçada a 0, tanques são zerados (`amount: 0`), e todos os 5 filtros de tanques são configurados para `mekanism:nothing`. Isso impede que tubos, cabos ou baldes interajam com portas avulsas.
- **S4 (Fluido / núcleo visível dentro):**  
  O Fusion Reactor do Java não utiliza o `RenderFluid` padrão de tanques cubóides, mas sim o renderizador de plasma rotativo (`RenderFusionReactor.java`, 3 esferas de energia concêntricas rotativas). Mapeado para `render_plasma_core` / `clear_plasma_core` (`// FORK-19`), com especificação detalhada em `FORK_fusion.md`.
- **S5 (Evaporação térmica):**  
  Não se aplica diretamente ao reator de fusão (específico da Planta de Evaporação Térmica).

---

## 3. Resumo dos Bugs Identificados e Corrigidos

1. **Bug 1 - Vidro de Reator permitido na moldura (Ring Frame):**  
   - *Java:* `FusionReactorValidator.java:61-72` e `CuboidStructureValidator.java:115`.
   - *Port:* `fusion.rs:122-132`.
   - *Correção:* Posições com `need == 1` aceitam estritamente `Part::Frame`. `Part::Glass` é restrito a posições de face com `need == 2`.
2. **Bug 2 - Válvula/Porta avulsa ou desmontada atuando como tanque (S3):**  
   - *Java:* `TileEntityMultiblock.java:218-229`.
   - *Port:* `fusion.rs:547-566, 617-650`.
   - *Correção:* Bloqueio total (`FlowSide::None`, `Tank { amount: 0 }`, `set_tank_filter(..., NOTHING)`).
3. **Bug 3 - Esvaziamento de tanques na desmontagem e reforma:**  
   - *Java:* `FusionReactorCache.java:23-48`, `MultiblockCache.java:43-71`.
   - *Port:* `fusion.rs:536-545, 660-695`.
   - *Correção:* `Shared.reactor` é mantido no cache `self.fusions` ao desmontar (`was_formed = false`, `started = false`, `given.clear()`). Ao reformar, os tanques e temperatura são restaurados do cache sem serem zerados.
4. **Bug 4 - Distribuição assimétrica e perda de energia nas portas:**  
   - *Java:* `FusionReactorMultiblockData.java:157, 249-251`.
   - *Port:* `fusion.rs:747-750, 834-842`.
   - *Correção:* Energia é rateada unicamente entre portas configuradas em modo saída (`n_out`), impedindo que portas de entrada retenham energia inacessível.
5. **Bug 5 - Travamento de filtros por modo de porta:**  
   - *Java:* `TileEntityFusionReactorPort.java:47-56`.
   - *Port:* `fusion.rs:821-859`.
   - *Correção:* Portas de saída travam tanques de entrada 0..3 com `NOTHING` e liberam vapor (4). Portas de entrada liberam 0..3 e travam vapor com `NOTHING`.
6. **Bug 6 - Clamp de 32 bits na geração passiva:**  
   - *Java:* `FusionReactorMultiblockData.java:358`.
   - *Port:* `fusion.rs:366-369`.
   - *Correção:* Removido clamp em `i32::MAX`, mantendo cálculo em `u64` com escala real de 64 bits.
7. **Bug 7 - Latência na atualização do estado de queima (`burning`):**  
   - *Java:* `FusionReactorMultiblockData.java:261`.
   - *Port:* `fusion.rs:733, 789, 818`.
   - *Correção:* Mudança em `burning` força sincronização imediata de `update.data` no tick, sem esperar o ciclo de 20 ticks.

---

## 4. Testes Unitários

Todos os testes utilizam regras e valores documentados do código-fonte Java (`FusionReactorValidator.java`, `FusionReactorMultiblockData.java`, `TileEntityFusionReactorLogicAdapter.java`).

**Total de testes em `port/mod/src/fusion.rs`:** 13 (7 pré-existentes mantidos intactos + 6 novos implementados).

### Lista de Testes:
1. `the_rounded_cube_has_twelve_missing_blocks_a_face` (linha 905)
2. `a_whole_shell_with_a_controller_is_a_reactor` (linha 912)
3. `it_needs_the_shape_the_controller_and_an_empty_inside` (linha 917)
4. `reactor_glass_only_allowed_on_faces_not_on_frame` (linha 951) — **NOVO**
5. `controller_must_be_top_center_and_only_one` (linha 975) — **NOVO**
6. `a_hohlraum_ignites_a_hot_plasma_and_it_keeps_burning` (linha 1012)
7. `the_laser_heats_ten_times_more_before_the_burn` (linha 1034)
8. `the_injection_rate_moves_by_two` (linha 1044)
9. `a_member_remembers_its_reactor` (linha 1059)
10. `burn_temperature_and_injection_rates` (linha 1068) — **NOVO**
11. `passive_generation_64bit_no_32bit_clamp` (linha 1085) — **NOVO**
12. `comparator_fuel_tank_level` (linha 1098) — **NOVO**
13. `logic_adapter_modes` (linha 1108) — **NOVO**

---

## 5. Verificação Manual de Compilação e Trechos Mais Arriscados

Como a compilação Rust não está disponível no ambiente (`cargo` não configurado na VM), o código foi minuciosamente inspecionado para conformidade com o sistema de tipos, propriedade e empréstimos:

1. **Separação de Empréstimos Mutáveis em `run_fusion` (`self.fusions` vs `self.take_click`):**  
   - *Risco:* Invocação de métodos de `&mut self` enquanto referências mutáveis internas a `self.fusions` estivessem ativas causaria conflito no borrow checker (`E0499` / `E0502`).  
   - *Mitigação:* A simulação do tick foi isolada em um bloco léxico próprio (`let (r, burning_changed, used) = { ... };`), liberando completamente o empréstimo de `self.fusions`. Os valores de `given` gerados no loop dos membros são coletados em `given_entries: Vec<([i32; 3], [u64; 6])>` e inseridos em `sh.given` após o término das iterações dos membros.
2. **Cálculo de Distância Euclidiana em `players::online`:**  
   - *Risco:* Coordenadas de jogador `pos.x, pos.y, pos.z` e centro do reator `i32` com conversão de tipos de ponto flutuante.  
   - *Mitigação:* Conversão explícita de cada termo `(p.pos.x - center_x as f64).powi(2)` garantindo que as operações ocorram puramente em `f64`.
3. **Fatias de Strings e Arrays em `world::set_tank_filter`:**  
   - *Risco:* Passagem de referências `&[String]` esperadas pela API do Horizon.  
   - *Mitigação:* Uso consistente de `&[NOTHING.to_owned()]`, `&[STEAM.to_owned()]` e vetores criados pela closure auxiliar `ok(&r.water, ...)` garantindo conformidade com a assinatura `&[String]`.

---

## 6. Prova de Mutações

A prova de mutação está pendente do juiz (conforme instrução: a tarefa não trouxe lista de mutações pré-definidas).

---

## 7. Status de Hooks de Plataforma (Forks)

Documentados com especificação WIT completa em `.saida/T20261109-230438-3c3/FORK_fusion.md`:
- **FORK-19:** `mekanism:multiblock/render-plasma-core` (renderizador dinâmico de esferas rotativas de plasma).
- **FORK-21:** `mekanism:multiblock/clear-tank-filter` (limpeza explícita de filtros de substância).
- **FORK-29:** `mekanism:multiblock/get-comparator-signal` (leitura dinâmica de sinal de comparador para lógica de automação).
- **FORK-30:** `mekanism:multiblock/spawn-formation-particles` (emissão de partículas sparkle nas 12 arestas do reator na formação).
- **FORK-7 / FORK-11:** `mekanism:world/kill-zone` (zona de dano letal de 50 000 HP dentro da câmara 3×3×3).
