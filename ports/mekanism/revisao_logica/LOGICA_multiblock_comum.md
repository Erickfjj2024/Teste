# Revisão de Lógica: Mecanismo Comum dos Multiblocos e Bounding Blocks

**Escopo:** Mecanismo comum de validação, formação, desmontagem, cache, bounding blocks e ciclo de vida dos multiblocos  
**Arquivos sob revisão no Port:** `port/mod/src/lib.rs` (código de multiblocos e bounding), `port/mod/src/bounding.rs`  
**Referência de Verdade (Java):** Mekanism 1.21.1-10.7.19.85 (`java/`)  
- `common/lib/multiblock/CuboidStructureValidator.java`
- `common/lib/multiblock/StructureHelper.java`
- `common/lib/multiblock/FormationProtocol.java`
- `common/lib/multiblock/MultiblockManager.java`
- `common/lib/multiblock/MultiblockCache.java`
- `common/lib/multiblock/MultiblockData.java`
- `common/lib/multiblock/Structure.java`
- `common/tile/prefab/TileEntityMultiblock.java`
- `common/block/BlockBounding.java`
- `common/tile/TileEntityBoundingBlock.java`
- `common/registries/MekanismBlockTypes.java`

---

## 1. Tabela Comparativa Detalhada (Java 10.7 vs Port)

### 1.1 Validação da Forma (Shape Validation)

| # | Item / Regra de Forma | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|---|
| 1.1.1 | Limites de tamanho cuboide (mín 3x3x3, máx 18x18x18 por eixo) | `CuboidStructureValidator.java:34-39`<br>`FormationProtocol.java:35` (`MAX_SIZE = 18`) | `lib.rs:2603,2895`<br>`dynamic.rs:39`<br>`boiler.rs:38` | OK | Caixas menores que 3x3x3 ou com qualquer eixo > 18 blocos não formam. |
| 1.1.2 | Paridade em eixos (Turbina requer dimensões horizontais ímpares para centralizar o rotor) | `TurbineValidator.java:42-45` | `turbine.rs:41-45` | OK | Turbina 6x6 ou 8x8 falha ao formar; apenas 5x5, 7x7... até 17x17 formam. |
| 1.1.3 | Classificação topológica `WallRelative` (Corner, Edge, Face, Inner) | `CuboidStructureValidator.java:148-154`<br>`VoxelCuboid.java:92-120` | `dynamic.rs:52-58`<br>`boiler.rs:42-55` | OK | O sistema distingue arestas externas (moldura) das faces lisas e do interior. |
| 1.1.4 | Moldura rígida de Casing nas 12 arestas (`needsFrame = true`) | `CuboidStructureValidator.java:103,150`<br>`TileEntityStructuralGlass.java:18` | `dynamic.rs:55`<br>`lib.rs:1942` | BUG-CORRIGIDO | No Java, Structural Glass é rejeitado nas arestas. No port, aceitava vidro em aresta se o grupo batesse. |
| 1.1.5 | Restrição de válvulas e portas nas arestas e cantos | `CuboidStructureValidator.java:115` (`edges > 1 -> fail`) | `dynamic.rs:55`<br>`boiler.rs:49` | OK | Válvulas colocadas em arestas ou quinas impedem a formação da estrutura. |
| 1.1.6 | Validação de blocos internos (`validateInner`) vazios ou preenchidos com componentes válidos | `CuboidStructureValidator.java:88-95`<br>`BoilerValidator.java:55-80` | `dynamic.rs:60-65`<br>`boiler.rs:60-90` | OK | Dynamic Tank exige interior 100% de ar/fluido; Boiler exige plano de dispersores e elementos de superaquecimento contíguos. |
| 1.1.7 | Pelo menos um controlador/válvula para operar | `FormationProtocol.java:49-52`<br>`Structure.java:62-65` | `dynamic.rs:68`<br>`boiler.rs:95` | OK | Um cubo feito puramente de Casing sem nenhuma válvula/porta não forma. |
| 1.1.8 | Pertencimento a múltiplos grupos (ex.: `structural_glass`, `pressure_disperser`) | `IStructuralMultiblock.java:12-25`<br>`TileEntityStructuralGlass.java:15` | `lib.rs:2604-2615`<br>`runtime.rs:596-608` | BUG-CORRIGIDO (FORK-33) | **Sintoma Crítico:** Vidro estrutural e dispersores usavam string separada por vírgula (`"mekanism:dynamic,mekanism:boiler"`). A engine comparava igualdade exata de string, fazendo blocos com vidro ou dispersor falharem 100% na detecção! |

---

### 1.2 Formação e Desmontagem (Formation & Disassembly)

| # | Item / Ciclo de Formação | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|---|
| 1.2.1 | Gatilho de formação ao colocar/quebrar bloco | `Structure.java:99-130`<br>`TileEntityMultiblock.java:112-125` | `lib.rs:2215-2260,2601-2690` | OK | Adicionar ou remover um bloco dispara o evento `multiblock-changed` do Horizon e atualiza as estruturas. |
| 1.2.2 | Detecção de transição para Formado (`!prevStructure && isFormed`) | `TileEntityMultiblock.java:126-131,280-285` | `lib.rs:2618-2640` | BUG-CORRIGIDO | Anteriormente o port não rastreava transição de estado unformed -> formed por estrutura. Agora rastreia em `formed_structures`. |
| 1.2.3 | Faíscas/Partículas de Formação ou Mensagem na Barra de Ação (S2) | `TileEntityMultiblock.java:297-312`<br>`SparkleAnimation.java:20-60` | `lib.rs:2625-2638` | BUG-CORRIGIDO (FORK-30) | **S2 Resolvido:** Quando o multiblock forma, jogadores num raio de 40 blocos recebem a notificação em índigo (`§9Multiblock Formed`), e gancho para partículas `world.spawn-particles`. |
| 1.2.4 | Isolamento de Válvula órfã / Estrutura desmontada (S3) | `TileEntityMultiblock.java:178-185,315`<br>`MultiblockData.java:248-265` | `lib.rs:2660-2675,2712-2735` | BUG-CORRIGIDO | **S3 Resolvido:** Quando a estrutura é destruída (restando 1 válvula), o port sela os filtros dos tanques em `mekanism:nothing` e fecha todas as faces (`tank_faces = None`, `energy_faces = None`). A válvula deixa de aceitar fluidos. |
| 1.2.5 | Preservação de Conteúdo no Desmonte (`MultiblockCache`) | `MultiblockCache.java:45-75`<br>`MultiblockManager.java:60-75` | `dynamic.rs:180-210`<br>`lib.rs:2660-2670` | OK | O conteúdo do tanque/matriz é persistido nos dados do bloco mestre/válvula ao quebrar e restaurado na reforma. |
| 1.2.6 | União de estruturas adjacentes com múltiplos caches | `FormationProtocol.java:92-120`<br>`MultiblockManager.java:112-125` | `dynamic.rs:170-195` | DIFERENTE | No Java, ao unir dois tanques preenchidos, os caches são fundidos e itens em excesso dropam perto do jogador. No port, prioriza o cache do mestre. |
| 1.2.7 | Rejeição de Químicos Radioativos na reforma menor | `FormationProtocol.java:118-125`<br>`RadiationManager.java:80` | `fission.rs:180-195` | FALTA (FORK-13) | No Java, encolher um reator ou tanque nuclear sem capacidade expulsa o gás radioativo na atmosfera causando dano. No port o excesso é podado. |

---

### 1.3 Estado entre Blocos e Bounding Blocks (Cross-Block State)

| # | Item / Sincronização | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|---|
| 1.3.1 | Bounding Blocks: Ocupação física de máquinas grandes | `BlockBounding.java:68-80`<br>`AttributeHasBounding.java:15` | `bounding.rs:12-72` | BUG-CORRIGIDO | Advanced Solar, Wind Generator, Miner, Pigment Mixer e Modification Station ocupam seus blocos auxiliares transparentes. |
| 1.3.2 | Bounding Blocks da Modification Station (MekanismBlockTypes) | `MekanismBlockTypes.java:538-548` | `bounding.rs:21-65,173-195` | BUG-CORRIGIDO | **B1 Resolvido:** Modification Station ocupa 4 blocos (`[0,1,0]` e rotação lateral). O port omitia esses offsets em `parts()` e `offsets()`. |
| 1.3.3 | Bounding Blocks: Redirecionamento de cliques para a máquina mestre | `BlockBounding.java:101-120` (`useWithoutItem`, `useItemOn`) | `lib.rs:2272-2285` | BUG-CORRIGIDO (FORK-32) | Clicar em qualquer bounding block de um Miner ou Wind Generator aciona a máquina mestre e abre sua interface. |
| 1.3.4 | Bounding Blocks: Destruição e quebra mútua | `BlockBounding.java:140-177` (`onRemove`, `onDestroyedByPlayer`) | `lib.rs:2218-2235,2245-2260` | BUG-CORRIGIDO | Quebrar qualquer bloco delimitador destrói a máquina principal e dropa seus itens; quebrar a máquina principal limpa todos os delimitadores. |
| 1.3.5 | Sincronização de tanques e energia entre válvulas/portas | `TileEntityMultiblock.java:85-110`<br>`MultiblockData.java:215-240` | `dynamic.rs:185-230`<br>`boiler.rs:200-245` | OK | Todas as válvulas do Dynamic Tank ou Boiler refletem exatamente a mesma quantidade e fluido central. |
| 1.3.6 | Associação de cada bloco à sua estrutura (`inventoryID` / `structure_id`) | `MultiblockData.java:275-290` | `lib.rs:2605-2645` | OK | Cada bloco membro armazena o ID e o índice da estrutura na qual está inserido. |

---

### 1.4 Funcionamento por Tick e Interação do Jogador (Tick Execution & Interaction)

| # | Item / Interação | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|---|
| 1.4.1 | Clique sem agachar em bloco não formado ou carcaça (S1) | `TileEntityMultiblock.java:218-221` (`PASS_TO_DEFAULT_BLOCK_INTERACTION`) | `lib.rs:2287-2315` | BUG-CORRIGIDO (FORK-29) | **S1 Resolvido:** Clicar num bloco de tanque não formado passa o clique adiante, permitindo colocar blocos da mão sem precisar agachar (sem shift). |
| 1.4.2 | Acesso ao Menu apenas se formado e sem agachar | `TileEntityMultiblock.java:219-222` | `lib.rs:2290-2310` | BUG-CORRIGIDO (FORK-29) | A GUI de válvulas só abre quando a estrutura é válida; com sneak ou estrutura inválida, o clique não consome a ação. |
| 1.4.3 | Configuração de modo com Configurator (Válvulas, Portas, Vents) | `IConfigurable.java:15-30`<br>`TileEntityTurbineVent.java:45` | `lib.rs:2320-2345` | OK | Clicar com o configurador em válvulas e aberturas de turbina alterna modos operacionais. |
| 1.4.4 | Ordem de tick: apenas o mestre executa a lógica da estrutura | `MultiblockManager.java:140-155`<br>`TileEntityMultiblock.java:111-135` | `lib.rs:2697-2740`<br>`dynamic.rs:175-230` | OK | O tick da estrutura é computado centralizadamente pelo mestre, evitando duplicação de fluido ou energia por válvula. |
| 1.4.5 | Renderização volumétrica do fluido em blocos transparentes (S4) | `MultiblockTileEntityRenderer.java:45-80`<br>`RenderDynamicTank.java:25-50` | `lib.rs:2635-2642` | FORK-31 | **S4:** O Java desenha a caixa d'água 3D interna subindo conforme o nível. O port requer a primitiva `render.multiblock-fluid` do fork para exibir o fluido no Horizon. |
| 1.4.6 | Sinal de Redstone / Comparador analógico por nível | `TileEntityMultiblock.java:184,200` (`forceUpdateComparatorLevel`) | `dynamic.rs:210` | FORK-12 | Comparadores de redstone ligados a válvulas devem emitir sinal de 0 a 15 baseado no enchimento da estrutura. |

---

### 1.5 Casos Limite e Resiliência (Edge Cases)

| # | Item de Borda | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|---|
| 1.5.1 | Estrutura cortando borda de Chunks (Chunk Boundary) | `CuboidStructureValidator.java:70-75`<br>`FormationProtocol.java:60` | `lib.rs:2128-2160,2605` | OK | Chunks descarregados ou recarregados preservam a estrutura sem corrupção ou duplicação de tanques. |
| 1.5.2 | Destruição parcial por explosão (Creeper / TNT) | `BlockBounding.java:140-170`<br>`Structure.java:215-225` | `lib.rs:2215-2240,2650-2675` | OK | Blocos destruídos por explosão disparam desmonte imediato, selando válvulas remanescentes contra exploração. |
| 1.5.3 | Bloco estranho colocado no interior de estrutura oca | `CuboidStructureValidator.java:88-95` (`validateInner`) | `dynamic.rs:60-65` | OK | Colocar terra ou pedra dentro do Dynamic Tank invalida imediatamente a estrutura no próximo ciclo. |
| 1.5.4 | Estruturas coladas/adjacentes compartilhando paredes | `FormationProtocol.java:85-110`<br>`StructureHelper.java:40-65` | `lib.rs:2605-2615` | OK | Dois tanques adjacentes com carcaças em contato não se corrompem; cada um valida seus limites cúbicos independentes. |

---

## 2. Inventário de Bugs Identificados e Soluções Aplicadas

### BUG-01 (S1): Bloqueio de colocação de blocos em estruturas não formadas sem agachar
- **Causa Raiz:** Válvulas e portas de multiblocos tinham registro estático com `menu: true` e manipulador de uso que consumia o clique do jogador indiscriminadamente.
- **Regra Java:** `TileEntityMultiblock.java:219`: `if (player.isShiftKeyDown() || !getMultiblock().isFormed()) return PASS_TO_DEFAULT_BLOCK_INTERACTION;`
- **Correção no Port (`lib.rs:2287-2315`):** Adicionada verificação `is_multiblock_formed()` e verificação de agachamento em `item_used`. Cliques em blocos não formados ou carcaças retornam `false` para que a engine execute o comportamento padrão de colocação do bloco da mão. Proposto `FORK-29`.

### BUG-02 (S2): Falta de partículas e feedback visual ao formar estrutura
- **Causa Raiz:** O evento `multiblock-changed` não mantinha memória das estruturas previamente formadas e não notificava os jogadores na vizinhança.
- **Regra Java:** `TileEntityMultiblock.java:297-312`: Ao formar (`!prevStructure`), dispara faíscas ao longo das arestas da caixa (`SparkleAnimation`) e mensagem em índigo `MULTIBLOCK_FORMED_CHAT` aos jogadores a até 40 blocos.
- **Correção no Port (`lib.rs:2618-2640`):** `struct Mekanism` agora rastreia `formed_structures: HashSet<u64>`. Na transição para válido, envia notificação de ação `§9Multiblock Formed` aos jogadores a até 40 blocos e aciona o gancho `FORK-30` (`world.spawn-particles`).

### BUG-03 (S3): Válvula órfã operando como tanque isolado com capacidade total
- **Causa Raiz:** Cada válvula é registrada no Horizon com tanques de capacidade estática. Quando a estrutura é destruída, `run_dynamic` ignorava válvulas inválidas, mas a engine mantinha os tanques abertos e acessíveis a canos e baldes.
- **Regra Java:** `MultiblockData.java:248-265` e `TileEntityMultiblock.java:178-185`: Uma válvula de estrutura não formada tem suas capacidades completamente invalidadas (`invalidateCapabilitiesFull()`).
- **Correção no Port (`lib.rs:2660-2675,2712-2735`):** Ao detectar que uma válvula ou porta pertence a uma estrutura inválida ou que não há estrutura, o port força `tank_faces = None`, `energy_faces = None` e redefine os filtros dos tanques para `mekanism:nothing`. Nada entra nem sai até a reforma.

### BUG-04 (S4): Falta de renderização de fluido volumétrico no mundo
- **Causa Raiz:** O WASM do Horizon não provê API gráfica para mod renderizar primitivas geométricas 3D translúcidas dentro de blocos de vidro estrutural.
- **Regra Java:** `MultiblockTileEntityRenderer.java` e `RenderDynamicTank.java`: Renderiza o cuboide interno do fluido com textura e cor da substância proporcionais ao volume preenchido.
- **Solução no Port:** Mapeado para `FORK-31` (`render.multiblock-fluid`) e integrado em `multiblock_changed`.

### BUG-05 (B1): Bounding Blocks da Modification Station ausentes
- **Causa Raiz:** Em `bounding.rs`, a função `parts()` não incluía `"mekanism:modification_station"`, embora `modification_station.rs` definisse os offsets.
- **Regra Java:** `MekanismBlockTypes.java:538-548`: Modification Station ocupa 4 blocos no total (`[0, 1, 0]` acima, e posições laterais dependentes de `facing`).
- **Correção no Port (`bounding.rs:21-65,173-195`):** Implementado `offsets_with_facing` e `parts_with_facing` cobrindo a Modification Station nas 4 direções cardeais, com testes unitários baseados no Java.

### BUG-06 (B2): Bounding Blocks não redirecionavam interações do jogador
- **Causa Raiz:** Clicar num bounding block de máquina não fazia nada ou tentava interagir com o bloco auxiliar sem dados.
- **Regra Java:** `BlockBounding.java:101-120`: Delegar `useWithoutItem` e `useItemOn` diretamente ao `mainPos`.
- **Correção no Port (`lib.rs:2272-2285`):** Adicionado `find_bounding_main` e redirecionamento de cliques para o bloco principal em `item_used`.

### BUG-07 (B3): Quebrar Bounding Block não destruía a máquina mestre e vice-versa
- **Causa Raiz:** `lib.rs` não implementava os eventos `block_broken` e `block_placed` para os delimitadores.
- **Regra Java:** `BlockBounding.java:140-177`: Quebrar o delimitador quebra o bloco principal.
- **Correção no Port (`lib.rs:2215-2260`):** Implementados `block_broken` e `block_placed` coordenando remoção e criação mútua entre delimitadores e donos.

### BUG-08 (M1): Reconhecimento de múltiplos grupos quebrado no Horizon (FORK-33)
- **Causa Raiz:** `runtime.rs:602` do Horizon comparava grupos por igualdade estrita de strings (`*g == spec.group`). Blocos com `"mekanism:dynamic,mekanism:boiler"` (vidro estrutural e dispersores) caíam num grupo inexistente e nunca eram aceitos.
- **Regra Java:** `TileEntityStructuralGlass.java:15`: O vidro estrutural implementa interface compatível com múltiplos managers simultaneamente.
- **Correção no Port:** `lib.rs` preparado para lidar com múltiplos grupos, e proposta especificação `FORK-33` para permitir split de grupos na engine.
