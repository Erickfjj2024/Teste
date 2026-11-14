# RESUMO DA TAREFA T20261109-230537-ppj

**Tarefa:** Revisão do Mecanismo Comum dos Multiblocos e Bounding Blocks (Mekanism Java 1.21.1-10.7.19.85 vs Horizon WASM Port)  
**Data:** 2026-11-09  
**Status:** CONCLUÍDO  

---

## 1. Arquivos Modificados no Escopo

Apenas os arquivos permitidos no escopo da tarefa foram alterados:
- `port/mod/src/lib.rs`
- `port/mod/src/bounding.rs`

Nenhum outro arquivo do código-fonte foi alterado.

---

## 2. Entregáveis Gerados

1. `.saida/T20261109-230537-ppj/LOGICA_multiblock_comum.md`
   - Tabela de conferência cobrindo as 5 seções requeridas:
     1. Validação da forma (tamanhos mín/máx, paridade, WallRelative, arestas/faces/interior, contagem de controladores/válvulas, vidro estrutural);
     2. Formação e desmontagem (gatilhos de eventos, isolamento de válvulas órfãs, preservação de cache no desmonte e reforma);
     3. Estado entre blocos e delimitadores (sincronização de tanques e energia entre válvulas, dono do estado, remoção e colocação mútua de bounding blocks);
     4. Funcionamento por tick e interação do jogador (ordem de operações, cliques de blocos não formados sem agachar, configurador, comparadores);
     5. Casos limite e resiliência (limites de tamanho, fronteiras de chunks, explosões, corpos estranhos no interior).
   - Lista detalhada de todos os bugs encontrados e os sintomas em jogo esperados.

2. `.saida/T20261109-230537-ppj/FORK_multiblock_comum.md`
   - Especificação técnica com assinaturas WIT para o SDK e engine do Horizon:
     - `FORK-29`: Uso condicional de bloco e interceptação dinâmica de menu (`on-block-use` / `menu-mode: dynamic`).
     - `FORK-30`: Disparo de partículas no mundo pelo mod (`world.spawn-particles`).
     - `FORK-31`: Renderização volumétrica 3D de fluidos no interior de multiblocos translúcidos (`render.set-multiblock-fluid`).
     - `FORK-32`: Vínculo e redirecionamento de cliques nativo para Bounding Blocks (`bounding-spec`).
     - `FORK-33`: Suporte a múltiplos grupos por bloco em `multiblock-spec` (split de grupos separados por vírgula).
     - `FORK-34`: Mascaramento dinâmico de capacidades de blocos (`set-capabilities-enabled`).

3. `.saida/T20261109-230537-ppj/RESUMO.md` (este documento).

---

## 3. Resolução dos Sintomas Reportados em Jogo

- **S1 (Só dava para colocar bloco com shift):**
  - **Causa:** Válvulas tinham `menu: true` estático e carcaças consumiam interação.
  - **Correção:** Em `item_used` (`lib.rs:2297-2307`), se o bloco é de multibloco e a estrutura não está formada ou o jogador está agachando, a função retorna `false` (passando o clique para a engine colocar o bloco da mão). Proposto `FORK-29`.
- **S2 (Sem partículas ao formar):**
  - **Causa:** O mod não rastreava transição de estado unformed -> formed e não notificava jogadores.
  - **Correção:** `lib.rs:2647-2666` rastreia transições em `formed_structures`. Na transição para formado, jogadores num raio de 40 blocos (1600 distância quadrática, conforme `TileEntityMultiblock.java:297-312`) recebem a notificação em índigo (`§9Multiblock Formed`) e gancho para faíscas `FORK-30`.
- **S3 (Válvula órfã continuava funcionando como tanque):**
  - **Causa:** Registro estático dava enorme capacidade às válvulas, e runners ignoravam válvulas de estruturas inválidas sem selá-las.
  - **Correção:** Em `multiblock_changed` e `block_entities_tick` (`lib.rs:2680-2692,2705-2735`), qualquer válvula ou porta com estrutura inválida tem seus tanques selados com filtro `NOTHING` (`mekanism:nothing`), `tank_faces` forçado para `None` e `energy_faces` para `None`.
- **S4 (Fluido não aparece fisicamente dentro do tanque):**
  - **Causa:** Falta de suporte da engine do Horizon para desenhar malhas/volumes de fluido translúcidos no interior de estruturas com vidro estrutural.
  - **Correção:** Mapeado e especificado no `FORK-31` (`render.multiblock-fluid`), com hooks preparados em `multiblock_changed`.
- **S5 (Thermal Evaporation Plant não produzia salmoura):**
  - **Causa:** Tipos `evaporating`, `activating` e `centrifuging` ausentes na lista de conversão de receitas do Python (`MACHINE_TYPES`). Confirmado que foram adicionados pelo orquestrador.

---

## 4. Testes Unitários e Prova de Mutação

### Testes Unitários Editados e Criados (Contagem Real e Nomes Exatos)

#### Em `port/mod/src/bounding.rs` (Total: 5 testes):
1. `big_machines_take_the_blocks_their_models_fill`
2. `a_bounding_block_remembers_its_machine`
3. `modification_station_bounding_offsets_by_facing` (Novo: validação de 3 offsets cardeais da Modification Station conforme `MekanismBlockTypes.java:538-548`)
4. `digital_miner_takes_twenty_six_bounding_blocks` (Novo: validação de 26 blocos delimitadores do Digital Miner conforme `MekanismBlockTypes.java:334-345`)
5. `wind_generator_has_pole_and_top_parts` (Novo: validação de 4 blocos verticais do Wind Generator com mastro e topo conforme `GeneratorsBlockTypes.java:104-112`)

#### Em `port/mod/src/lib.rs` (Total: 4 testes):
1. `multiblock_block_classification` (Novo: classificação estrita de carcaças, válvulas, portas, rotores e blocos de máquinas conforme `TileEntityMultiblock.java:218-229`)
2. `multiblock_formation_sparkle_distance_threshold` (Novo: validação do limiar de 40 blocos / 1600 distância quadrática para partículas de formação conforme `TileEntityMultiblock.java:297-312`)
3. `multiblock_max_size_constant` (Novo: validação da constante `MAX_SIZE = 18` conforme `FormationProtocol.java:37`)
4. `multiblock_on_activate_pass_through_conditions` (Novo: validação da regra booleana de fall-through de interação conforme `TileEntityMultiblock.java:218-229`)

- **Total de testes no escopo:** 9 testes.
- **Execução via Cargo:** não medido (o ambiente do container retorna: `error: rustup could not choose a version of rustc to run, because one wasn't specified explicitly, and no default is configured`).
- **Prova de Mutação:** A prova de mutação está pendente do juiz (nenhuma lista de mutações foi fornecida na tarefa).

---

## 5. Avaliação de Riscos de Compilação

Como o compilador Rust não está funcional nesta máquina virtual, todos os trechos alterados foram revisados manualmente com rigor estrito em tipos, assinaturas, imports e empréstimos:
1. `port/mod/src/bounding.rs`:
   - Risco baixo: As funções `offsets_with_facing` e `parts_with_facing` utilizam tipos estáticos idênticos aos já existentes (`Vec<[i32; 3]>` e `Vec<([i32; 3], &'static str)>`). O módulo `modification_station` já existia no projeto com a função `bounding_offsets(&str) -> [[i32; 3]; 3]`.
2. `port/mod/src/lib.rs`:
   - Risco baixo/médio:
     - Adição dos campos `formed_structures: HashSet<(String, String, [i32; 3])>` e `formed_blocks: HashSet<(String, i32, i32, i32)>` na struct `Mekanism` com inicialização limpa no `init()`. `HashSet` já estava importado no topo do arquivo.
     - `.on_blocks()` habilitado em `Registration` no `init()`. Verificado no patch `0008-Mod-messages-API-block-8.patch:446` que `pub fn on_blocks(mut self) -> Self` existe no SDK do Horizon.
     - `world::mine_block` em `block_broken`: verificado que a chamada passa exatamente os 10 argumentos esperados pelo import do Horizon (`dimension`, `x`, `y`, `z`, `tool`, `silk_touch`, `drop_x`, `drop_y`, `drop_z`, `face`), idêntica à linha 1145 existente no arquivo.
     - `multiblock_changed` e `block_entities_tick`: manipulações de `FlowSide` e `BlockEntityUpdate` utilizam os enums exportados pelo SDK sem alocações inválidas.
