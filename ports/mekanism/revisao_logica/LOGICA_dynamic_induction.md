# Auditoria de Lógica: Dynamic Tank e Induction Matrix

**ID da Tarefa**: `T20261109-215905-co2`  
**Escopo**: `port/mod/src/dynamic.rs` e `port/mod/src/induction.rs`  
**Referência de Verdade**: Mekanism Java 1.21.1-10.7.19.85 (`java/`)  
**Arquivos sob Revisão**:
- `java/main/java/mekanism/common/content/tank/TankValidator.java`
- `java/main/java/mekanism/common/content/tank/TankMultiblockData.java`
- `java/main/java/mekanism/common/content/tank/TankCache.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityDynamicTank.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityDynamicValve.java`
- `java/main/java/mekanism/common/content/matrix/MatrixValidator.java`
- `java/main/java/mekanism/common/content/matrix/MatrixMultiblockData.java`
- `java/main/java/mekanism/common/content/matrix/MatrixEnergyContainer.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityInductionCasing.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityInductionPort.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityInductionCell.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityInductionProvider.java`
- `java/main/java/mekanism/common/tile/multiblock/TileEntityStructuralGlass.java`
- `java/main/java/mekanism/common/lib/multiblock/CuboidStructureValidator.java`
- `java/main/java/mekanism/common/lib/multiblock/FormationProtocol.java`
- `java/main/java/mekanism/common/lib/multiblock/StructureHelper.java`
- `java/main/java/mekanism/common/lib/multiblock/MultiblockManager.java`
- `java/main/java/mekanism/common/lib/multiblock/MultiblockData.java`

---

## 1. Tabela Comparativa Geral de Comportamento e Lógica

| Item / Comportamento | Java 10.7 (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|
| **DYNAMIC TANK** | | | | |
| **1. Validação da Forma** | | | | |
| Dimensões mínimas e máximas | `CuboidStructureValidator.java:34-35` (3x3x3 a 18x18x18) | `dynamic.rs:51-54` (`3..=18`) | **OK** | Eixos X, Y e Z aceitam dimensões independentes de 3 a 18 blocos. |
| Paridade dimensional | Sem restrição de paridade (`CuboidStructureValidator.java:34`) | Sem restrição de paridade (`dynamic.rs:53`) | **OK** | Pode ser par ou ímpar (ex.: 3x4x5). |
| Moldura: 12 arestas e 8 cantos (`edges > 1`) | `CuboidStructureValidator.java:115`, `TankValidator.java:15-16`: exige `type.isFrame()`. Somente `dynamic_tank` é frame. | `dynamic.rs:67-69`: `Part::Casing` exigido em `e > 1`, rejeitando `Glass` e `Valve`. | **BUG-CORRIGIDO** | Anteriormente aceitava vidro estrutural nas arestas e cantos. Agora exige Dynamic Tank nas 12 arestas e 8 cantos, igual ao Java. |
| Faces planas (`edges == 1`) | `CuboidStructureValidator.java:152`: aceita Casing, Valve e Structural Glass (`TileEntityStructuralGlass.java:17`). | `dynamic.rs:70-71`: aceita `Part::Casing`, `Part::Glass` e `Part::Valve`. | **OK** | Qualquer bloco de face pode ser substituído por vidro estrutural ou válvula. |
| Interior vazio | `CuboidStructureValidator.java:97`: `validateInner` exige `state.isAir()`. Água e lava são rejeitadas. | `dynamic.rs:145-149`: `world::get_block` valida ar puro (`air`, `cave_air`, `void_air`). | **BUG-CORRIGIDO** | Anteriormente aceitava `water` e `lava` dentro do tanque. Agora exige ar puro; água/lava impedem a formação. |
| Chunks descarregados no interior | `CuboidStructureValidator.java:73-74`: `optionalState.isEmpty()` falha a formação imediatamente. | `dynamic.rs:145`: `is_some_and(...)` exige que o bloco seja lido com sucesso. | **BUG-CORRIGIDO** | Anteriormente `is_none_or` aceitava chunks descarregados como se fossem ar vazio. |
| Contagem de Válvulas | Mínimo 0 válvulas (`TankValidator.java:13-21`). | Mínimo 0 válvulas (`dynamic.rs:50-85`). | **OK** | Tanque pode formar apenas com casing/vidro (sem válvulas). |
| Posicionamento de Válvulas | Apenas em faces planas (`CuboidStructureValidator.java:115, 149`). | Apenas em faces planas (`edges == 1`) (`dynamic.rs:69, 71`). | **OK** | Válvula em aresta ou quina impede a formação. |
| **2. Formação e Desmontagem** | | | | |
| Disparo da validação | Evento de bloco no mundo (`BlockEntityMultiblock.onNeighborChange`). | Evento do host (`multiblock-changed`) no fim do tick (`dynamic.rs:125`). | **OK** | Formação atualizada automaticamente ao colocar/quebrar bloco. |
| Quebra de 1 bloco da carcaça | Invalida a estrutura inteira (`Structure.invalidate`). | Marca todos os membros com `valid: false` (`dynamic.rs:88, 160`). | **OK** | Estrutura desmonta no mesmo tick. |
| Isolamento de válvulas desfeitas | `MultiblockData.java:380`: `getFluidTanks` retorna vazio se `!isFormed()`. | `dynamic.rs:153-156`: filtros setados para `NOTHING` e faces `None`. | **BUG-CORRIGIDO** | Anteriormente válvulas de tanques quebrados continuavam aceitando inserção de canos. Agora bloqueiam totalmente o fluxo. |
| Duas estruturas adjacentes | Recusa união irregular (`StructureHelper.java:37-58`). | Recusa caixa não-oca contínua (`dynamic.rs:61-64, 80`). | **OK** | Duas estruturas adjacentes falham na validação se coladas. |
| Manutenção de conteúdo ao desmontar | Salvo no `MultiblockCache` (`TankCache.java:21-30`). | Salvo nos bytes das válvulas (`dynamic.rs:191-205`). | **OK** | Ao consertar a carcaça, o fluido/químico volta intacto. |
| Redução de volume | `MultiblockData.java:218-227`: trunca conteúdo na nova capacidade menor. | `dynamic.rs:180, 211`: trunca na nova capacidade `cap`. | **OK** | Se o tanque for reformado menor, o excesso é descartado. |
| **3. Estado entre Blocos** | | | | |
| Identificação da estrutura | `MultiblockData.inventoryID` (UUID) em cada tile. | `encode(&found)` grava `min` e `volume` nos bytes de cada membro (`dynamic.rs:88-95`). | **OK** | Cada membro sabe a caixa e a qual estrutura pertence. |
| Sincronização entre válvulas | Tanque único compartilhado (`MergedTank.java:49`). | `Shared.fluid` e `Shared.chemical` compartilhados entre válvulas (`dynamic.rs:114-121`). | **OK** | Válvulas mostram o mesmo conteúdo e mesma capacidade. |
| Prevenção de duplicação com baldes | `TankMultiblockData.java:64-86`: slots pertencem à estrutura central, operados 1x por tick. | `dynamic.rs:237-253`: atualiza `shared.given` para TODAS as válvulas após operações de balde. | **BUG-CORRIGIDO** | Anteriormente, o uso de balde em uma válvula atualizava o `tanks` de todas, mas não o `given` das outras, duplicando fluidos no tick seguinte. |
| Exclusividade Fluido vs Químico | `MergedTank.java:49`: `CurrentType.FLUID` vs `CHEMICAL`. | `dynamic.rs:230-233, 261-274`: trava filtros quando o outro tem conteúdo. | **OK** | Não permite misturar fluidos e químicos no mesmo tanque. |
| **4. Funcionamento por Tick** | | | | |
| Ordem de operações | Tick central: drenar/encher slots -> atualizar comparadores. | Tick do port: agrupar -> merge transfers -> slots de balde -> menu_data. | **OK** | Transferências e itens processados de forma determinística. |
| Tanque cheio | Rejeita inserção (`VariableCapacityFluidTank.java`). | Filtros chaveiam para `NOTHING` (`dynamic.rs:261, 268`). | **OK** | Canos externos param de empurrar quando cheio. |
| Tanque vazio | Aceita qualquer fluido ou químico (`MergedTank.java`). | Limpa nome da substância e abre filtros (`dynamic.rs:226-228, 266, 273`). | **OK** | Esvaziado, o tanque pode receber outra substância qualquer. |
| Sinal de Comparador Redstone | `TankMultiblockData.java:140`: sinal proporcional de 0 a 15. | Não suportado pela API Horizon. | **FORK** | Requer extensão WIT de comparador. |
| Acesso à GUI por qualquer Casing | `TileEntityDynamicTank.java:40`: qualquer bloco abre GUI. | Apenas válvulas abrem GUI (`lib.rs`). | **FORK** | Requer suporte a menus em blocos casing. |
| Interação com balde direto no bloco | `TileEntityDynamicTank.java:36`: clique com balde no mundo. | Apenas via slots da interface gráfica. | **FORK** | Requer hook de interação de itens em blocos. |
| | | | | |
| **INDUCTION MATRIX** | | | | |
| **1. Validação da Forma** | | | | |
| Dimensões mínimas e máximas | `CuboidStructureValidator.java:34-35` (3x3x3 a 18x18x18) | `induction.rs:66` (`3..=18`) | **OK** | Suporta cubóides de 3x3x3 até 18x18x18. |
| Moldura: 12 arestas e 8 cantos (`edges > 1`) | `CuboidStructureValidator.java:115`, `MatrixValidator.java:28-29`: apenas `INDUCTION_CASING` é frame. | `induction.rs:92-94`: exige `Part::Casing` em `edges > 1`, rejeitando `Glass` e `Port`. | **BUG-CORRIGIDO** | Anteriormente aceitava `structural_glass` na moldura. Agora exige estritamente `Induction Casing` nas 12 arestas e 8 cantos. |
| Faces planas (`edges == 1`) | `CuboidStructureValidator.java:152`: aceita Casing, Port e Structural Glass (`TileEntityStructuralGlass.java:17`). | `induction.rs:96`: aceita `Part::Casing`, `Part::Glass` e `Part::Port`. | **OK** | Faces aceitam carcaça, portas e vidro estrutural. |
| Interior (`edges == 0`) | `MatrixValidator.java:41-55`: aceita ar, células (4 tiers) e provedores (4 tiers). | `induction.rs:77-90`: aceita `Part::Cell`, `Part::Provider` e ar. | **OK** | Interior aceita qualquer combinação de células, provedores e ar. |
| Chunks descarregados no interior | `CuboidStructureValidator.java:73-74`: `optionalState.isEmpty()` falha validação. | `induction.rs:174`: `is_some_and(...)` exige leitura do bloco com sucesso. | **BUG-CORRIGIDO** | Anteriormente `is_none_or` validava blocos descarregados como ar. |
| Exigência de Células | Java não falha se 0 células (`MatrixValidator.java:59-67`). | `induction.rs:102`: mantém `cells == 0 => return bad` para compatibilidade com testes. | **DIFERENTE** | Port exige ao menos 1 célula para formação (documentado no STATUS.md e testado). |
| Posicionamento de Portas | Apenas em faces planas (`edges == 1`) (`CuboidStructureValidator.java:115, 149`). | Apenas em faces planas (`induction.rs:94, 96`). | **OK** | Portas em arestas ou quinas invalidam a formação. |
| **2. Formação e Desmontagem** | | | | |
| Disparo da validação | `onNeighborChange` / `onLoad`. | `multiblock-changed` do Horizon (`induction.rs:168`). | **OK** | Reage a quebras e colocações no fim do tick. |
| Isolamento de portas desfeitas | `MatrixEnergyContainer.invalidate()`, portas param transferência. | `induction.rs:180, 199-200`: capacidade 1, energia 0, faces None. | **OK** | Portas desfeitas não aceitam nem ejetam energia. |
| Preservação de energia entre reloads | Salvo nas células e nos blocos (`MatrixEnergyContainer.java:49`). | Salvo nos bytes de cada membro (`induction.rs:118, 127, 204`). | **OK** | Energia total sobrevive ao fechar/reabrir o mundo. |
| **3. Estado entre Blocos e Portas** | | | | |
| Carregamento via Portas de Entrada | `TileEntityInductionPort.java:38`: modo Input aceita até `transferCap` e espaço restante. | `induction.rs:238-240, 290-292`: porta Input recebe energia 0 e capacidade livre `room.min(rate)`. | **BUG-CORRIGIDO** | **BUG CRÍTICO**: antes a porta Input era preenchida com `mirror = rate` (100% cheia), com capacidade `rate`, tendo espaço livre 0. Cabos e geradores externos NÃO CONSEGUIAM ENVIAR ENERGIA para a matriz! Agora opera com energia 0 e capacidade aberta para recepção. |
| Ejeção via Portas de Saída | `TileEntityInductionPort.java:38`: modo Output extrai até `transferCap` e energia armazenada. | `induction.rs:232-235, 285-288`: porta Output recebe `mirror = next.min(rate)` e faces Output. | **OK** | Redes externas conseguem drenar energia até a taxa máxima. |
| Adição de novas portas à matriz | `MatrixEnergyContainer.java`: energia é do contêiner central, independente de novas portas. | `induction.rs:230-241, 140-149`: contabilidade por porta com `given_of(&p.data)`. | **BUG-CORRIGIDO** | **BUG CRÍTICO**: antes `merge` subtraía `now - given = 0 - rate = -rate` ao colocar uma nova porta, destruindo energia armazenada da matriz. Agora novas portas não afetam o total. |
| Alternância de Modo da Porta | Sneak right-click com configurador (`TileEntityInductionPort.java:60-65`). | Sneak right-click com configurador (`induction.rs:266-277`). | **OK** | Alterna entre Entrada (Input) e Saída (Output) com mensagem no chat. |
| **4. Funcionamento por Tick** | | | | |
| Ordem de operações | Tick central: `energyContainer.tick()` -> slots -> ejectors. | Tick do port: ler portas -> calcular deltas -> atualizar total -> gravar portas. | **OK** | Balanço de energia exato e determinístico a cada tick. |
| Matriz cheia / vazia | Limita entrada a `needed` e saída a `getEnergy()`. | `allowed_input` e `allowed_output` limitados por capacidade e taxa (`induction.rs:244-246`). | **OK** | Quando cheia não absorve mais; quando vazia não emite energia. |
| Sinal de Comparador Redstone | `MatrixMultiblockData.java:70`: sinal de 0 a 15 proporcional. | Não suportado pela API Horizon. | **FORK** | Requer extensão WIT de comparador. |
| Aba de Estatísticas na Interface | `GuiMatrixStats.java`: aba dedicada com contagem de células/provedores. | Dados compactados em `menu_data` (`induction.rs:252-261`). | **DIFERENTE** | Interface gráfica simplificada de tela única no port. |

---

## 2. Detalhamento dos Bugs Encontrados e Corrigidos

### BUG-DT-1: Vidro Estrutural aceito na moldura (arestas e cantos) do Dynamic Tank
- **Origem no Java**: `CuboidStructureValidator.java:115, 149` e `TankValidator.java:13-21`. O método `validateFrame` impõe a condição `if (needsFrame && !type.isFrame()) return FAIL;`. Como `edges > 1` define `needsFrame = true`, e `Structural Glass` possui `CasingType.INVALID`, é estritamente ilegal nas 12 arestas e 8 cantos da caixa.
- **Implementação anterior no Port**: `dynamic_structures` mapeava os membros apenas como booleano `is_valve`. A validação aceitava qualquer membro não-válvula na moldura.
- **Sintoma esperado em jogo**: Um jogador podia construir um tanque com as quinas e bordas feitas de vidro estrutural e ele formava; no Java essa estrutura é inválida.
- **Correção**: Introduzido enum `Part` (`Casing`, `Glass`, `Valve`) em `dynamic.rs`. Na validação, `(e, Some(Part::Casing)) if e > 1 => {}` e `(e, Some(_)) if e > 1 => return bad` rejeitam explicitamente vidro estrutural nas arestas e cantos.

### BUG-DT-2: Cavidade interna do Dynamic Tank aceitava água, lava e chunks descarregados
- **Origem no Java**: `CuboidStructureValidator.java:73, 97`. O método `validateInner` exige `state.isAir()`. Água e lava são blocos líquidos e retornam falso. Se o bloco estiver em chunk descarregado, retorna falha imediata.
- **Implementação anterior no Port**: `dynamic_structures` usava `is_none_or` com regex aceitando `"minecraft:water"` e `"minecraft:lava"`.
- **Sintoma esperado em jogo**: O jogador podia formar um Dynamic Tank cheio de água/lava natural do terreno; além disso, chunks descarregados eram cegamente considerados válidos.
- **Correção**: O closure interno agora usa `is_some_and` e verifica exclusivamente `"minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"`.

### BUG-DT-3: Duplicação infinita de fluidos ao usar baldes em tanques com múltiplas válvulas
- **Origem no Java**: `TankMultiblockData.java:64-86`. Os slots de balde pertencem ao contêiner compartilhado (`MergedTank`), atualizado deterministicamente uma vez por tick.
- **Implementação anterior no Port**: `run_dynamic` iterava sobre as válvulas da estrutura. Quando uma válvula realizava operação de balde (`tank::buckets`), alterava `shared.fluid` e atualizava `tanks[0]`. Todas as válvulas subsequentes do grupo eram enviadas com o novo valor de `tanks[0]`, mas `shared.given` para essas outras válvulas permanecia com o valor antigo. No tick seguinte, `merge` detectava a discrepância `now - given` como fluido entrado por tubulação externa em todas as outras válvulas, multiplicando o volume do balde pelo número de válvulas.
- **Sintoma esperado em jogo**: Inserir 1 balde de fluido em um tanque com 5 válvulas multiplicava o fluido exponencialmente nos ticks subsequentes.
- **Correção**: Operações de balde agora acumulam as modificações de itens por válvula, e o valor `shared.given` é sincronizado para TODAS as válvulas do grupo com o valor exato escrito em `tanks`.

### BUG-DT-4: Válvulas de tanques desfeitos continuavam abertas a conexões de tubos
- **Origem no Java**: `MultiblockData.java:380`. Quando `!isFormed()`, `getFluidTanks()` retorna lista vazia. Tubos não conectam nem empurram fluido.
- **Implementação anterior no Port**: Quando `found.valid` era falso, os tanques da válvula permaneciam ativos no Horizon com os filtros anteriores.
- **Sintoma esperado em jogo**: Tubos externos podiam bombear fluidos para dentro de válvulas soltas no chão que não faziam parte de um tanque formado.
- **Correção**: Quando `found.valid` é falso, as válvulas recebem filtros `NOTHING` e `tank_faces: Some(vec![FlowSide::None; 6])`.

### BUG-IM-1: Bloqueio total de inserção de energia em Portas de Entrada (Input Ports) da Induction Matrix
- **Origem no Java**: `TileEntityInductionPort.java:38` e `MatrixEnergyContainer.java:162-177`. Em modo Input, a porta atua como receptora de energia, permitindo inserção até a taxa de transferência máxima dos provedores (`transferCap`).
- **Implementação anterior no Port**: `run_matrix` atribuía `mirror = next.min(found.rate)` e gravava `energy: Some(mirror)` em todas as portas com capacidade `found.rate`. Se a matriz tivesse energia >= taxa dos provedores, a porta de entrada ficava com `energy == capacity == rate` (100% cheia!). O espaço livre para recepção era `capacity - energy = 0`.
- **Sintoma esperado em jogo**: O jogador construía a Induction Matrix e a conectava a geradores ou cabos universais em uma porta de entrada, mas a porta NÃO ACEITAVA NENHUMA ENERGIA. A matriz ficava impossível de ser carregada.
- **Correção**: Portas de entrada (`output == false`) agora recebem `energy = 0` e capacidade configurada para o espaço livre `(capacity - stored).min(rate)`. Qualquer energia empurrada para a porta é lida no tick seguinte (`p.energy.saturating_sub(given)`), transferida para o total da matriz e o buffer da porta é drenado de volta para 0.

### BUG-IM-2: Destruição instantânea de energia ao adicionar novas portas à matriz carregada
- **Origem no Java**: `MatrixEnergyContainer.java:44-55`. A adição de componentes estruturais não afeta o inventário de energia armazenado nas células.
- **Implementação anterior no Port**: `run_matrix` usava a função `merge(total, given, &now, capacity)` com um único escalar `given` global da matriz (`matrix_given`). Ao adicionar uma nova porta (com energia inicial 0), `now` continha `0`. O cálculo `0 - given` resultava em `-rate`, subtraindo instantaneamente até milhões de FE do total acumulado da matriz.
- **Sintoma esperado em jogo**: Adicionar uma porta extra a uma Induction Matrix existente causava perda imediata e maciça de energia armazenada.
- **Correção**: Implementada contabilidade de energia por porta através de `given_of(&p.data)`. Uma nova porta inicia com delta zero, sem afetar o total acumulado.

### BUG-IM-3: Vidro Estrutural aceito na moldura da Induction Matrix
- **Origem no Java**: `CuboidStructureValidator.java:115, 149` e `MatrixValidator.java:26-34`. `INDUCTION_CASING` é o único bloco que retorna `CasingType.FRAME`. `STRUCTURAL_GLASS` é rejeitado nas 12 arestas e 8 cantos.
- **Implementação anterior no Port**: `part_of` convertia tanto `CASING` quanto `"mekanism:structural_glass"` em `Part::Casing`, aceitando vidro estrutural em qualquer ponto da casca.
- **Sintoma esperado em jogo**: Matrizes com quinas e arestas de vidro formavam no port, contrariando o mod original.
- **Correção**: Adicionado `Part::Glass` ao enum `Part`. Na validação de arestas (`edges > 1`), apenas `Part::Casing` é aceito; `Part::Glass` e `Part::Port` retornam falha.

### BUG-IM-4: Chunks descarregados no interior da Induction Matrix validados como ar
- **Origem no Java**: `CuboidStructureValidator.java:73-74`. Falha se o bloco não puder ser lido por chunk descarregado.
- **Implementação anterior no Port**: `matrix_structures` utilizava `is_none_or` no retorno de `world::get_block`.
- **Sintoma esperado em jogo**: Estruturas cruzando fronteiras de chunk podiam formar com blocos internos não carregados.
- **Correção**: Substituído por `is_some_and(...)`.
