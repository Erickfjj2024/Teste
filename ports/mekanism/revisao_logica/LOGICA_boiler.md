# Revisão da Lógica: Thermoelectric Boiler (Mekanism Java 1.21.1-10.7.19.85 vs Port Horizon WASM)

**ID da Tarefa**: T20261109-230117-goj  
**Data**: 2026-11-09  
**Referência de Verdade**: Mekanism 1.21.1-10.7.19.85 (`java/`)  
**Arquivo em Escopo**: `port/mod/src/boiler.rs` (760 linhas)  

---

## 1. Análise Passo a Passo contra o Java

### 1.1 Validação da Forma
- **Dimensões por eixo**:
  - *Java* (`CuboidStructureValidator.java:34-35`): `new VoxelCuboid(3, 3, 3), new VoxelCuboid(18, 18, 18)`. Largura, comprimento e altura são validados independentemente entre 3 e 18 blocos inclusive.
  - *Port* (`boiler.rs:81`): `size.iter().any(|s| !(3..=18).contains(s))`.
  - *Estado*: **OK**.
- **Paridade**:
  - *Java*: O Thermoelectric Boiler não exige paridade (diferente da Turbina Industrial que requer eixo ímpar para centralização do rotor).
  - *Port*: Não impõe restrição de paridade.
  - *Estado*: **OK**.
- **WallRelative e Posições Permitidas (`CuboidStructureValidator.java:115, 149`)**:
  - *Arestas e Quinas (`matches >= 2`, `WallRelative.isOnEdge()`)*:
    - *Java*: `needsFrame` é `true`. Exige estritamente `type.isFrame() == true`. Em `BoilerValidator.java:26-34`, apenas `BOILER_CASING` tem `CasingType.FRAME`. `BOILER_VALVE` tem `CasingType.VALVE` e `STRUCTURAL_GLASS` tem `CasingType.INVALID`. Ambos têm `isFrame() == false`. Portanto, nas 12 arestas e 8 quinas da moldura, **apenas** `mekanism:boiler_casing` é aceito! Vidro estrutural e válvulas são expressamente rejeitados com `MULTIBLOCK_INVALID_FRAME`.
    - *Port anterior*: Mapeava `structural_glass` como `Part::Casing` e aceitava `(_, Some(Part::Casing))` em qualquer posição da casca. Vidro estrutural em arestas/quinas era indevidamente aceito.
    - *Correção*: Criado `Part::Glass` distinguido de `Part::Casing`. Em `edges >= 2`, aceita apenas `Part::Casing`. Em `edges == 1`, aceita `Part::Casing`, `Part::Glass` e `Part::Valve`.
    - *Estado*: **BUG CORRIGIDO**.
  - *Faces (`matches == 1`, `WallRelative.SIDE`)*:
    - *Java*: `needsFrame` é `false`. Aceita `BOILER_CASING`, `BOILER_VALVE` e `STRUCTURAL_GLASS`.
    - *Port*: Aceita os 3 tipos em `edges == 1`.
    - *Estado*: **OK**.
  - *Interior (`matches == 0`, `WallRelative.INVALID`)*:
    - *Java* (`BoilerValidator.java:37-42`): Chama `validateInner`. Aceita apenas blocos de ar (`state.isAir()`), `mekanism:pressure_disperser` e `mekanism:superheating_element`. Qualquer bloco estranho rejeita a estrutura com `MULTIBLOCK_INVALID_INNER`.
    - *Port* (`boiler.rs:95-104`): Aceita apenas `Disperser`, `Element` e `inside_empty` (ar).
    - *Estado*: **OK**.
- **Contagem e Posição de Válvulas**:
  - *Java*: Válvulas são permitidas unicamente nas faces (`WallRelative.SIDE`). Não há limite mínimo obrigatório (uma estrutura sem válvulas forma, embora não interaja com tubulações) nem limite máximo.
  - *Port* (`boiler.rs:107`): Válvulas aceitas apenas em `edges == 1`.
  - *Estado*: **OK**.
- **Dispersores de Pressão (`Pressure Disperser`)**:
  - *Java* (`BoilerValidator.java:56-84`): Exige pelo menos 1 dispersor. Todos os dispersores devem formar um plano horizontal contíguo fechado de `(length - 2) * (width - 2)` na altura `initDisperser.getY()`. Dispersores extras em qualquer outra altura ou posição causam falha imediata com `BOILER_INVALID_EXTRA_DISPERSER`.
  - *Port* (`boiler.rs:113-115`): Exige que todos estejam na mesma altura `plane` e contagem exata igual a `(size[0] - 2) * (size[2] - 2)`.
  - *Estado*: **OK**.
- **Elementos Superaquecedores (`Superheating Element`)**:
  - *Java* (`BoilerValidator.java:60-91`): Exige pelo menos 1 elemento. Todos os elementos devem estar estritamente abaixo do plano de dispersores (`pos.getY() < initDisperser.getY()`) e conectados entre si num único grupo contíguo (`FormationProtocol.explore`). Elementos acima ou isolados falham com `BOILER_INVALID_SUPERHEATING`.
  - *Port* (`boiler.rs:117-133`): Valida `e[1] < plane` e verifica conectividade contígua completa via BFS a partir do primeiro elemento.
  - *Estado*: **OK**.
- **Câmara de Água e Bolsões de Ar (`BoilerValidator.java:97-126`)**:
  - *Java*: Executa `FormationProtocol.explore` iniciando em qualquer bloco de ar ou carcaça submerso. O algoritmo percorre **todos** os blocos de carcaça do piso (`size[0] * size[2]`), todas as paredes submersas e todo o ar conectado abaixo dos dispersores. O volume é computado como `(size[0] * size[2] * (plane - min[1])) - elements`. Não exige existência de ar (uma camada 100% preenchida por elementos é válida) e o ar se conecta ao redor dos elementos através da moldura do piso e paredes.
  - *Port anterior*: Coletava apenas blocos de ar internos (`cells`). Falhava se não houvesse ar (`cells.first() == None`), falhava se uma parede de elementos dividisse o ar (`reached.len() != cells.len()`), e atribuía `water_volume: cells.len()`, subestimando dramaticamente a capacidade de água e refrigerante quente.
  - *Correção*: Alinhado à regra exata do Java: `water_volume = (size[0] * size[2] * (plane - min[1])) - elements.len()`. Removida a exigência espúria de ar desconectado da moldura.
  - *Estado*: **BUG CORRIGIDO**.

---

### 1.2 Formação e Desmontagem
- **Gatilho de Formação**:
  - *Java* (`MultiblockManager.java`): A colocação ou quebra de qualquer bloco membro dispara `StructureHelper.fetchCuboid` e `FormationProtocol.update`.
  - *Port* (`boiler.rs:281-325`): Disparado pelo evento `multiblock-changed` do Horizon, que entrega os candidatos contíguos do grupo `mekanism:boiler`.
  - *Estado*: **OK**.
- **Desmontagem e Preservação de Conteúdo (S3 / `MultiblockCache.java`)**:
  - *Java*: Quando a estrutura é invalidada (bloco quebrado), o tile da válvula deixa de expor tanques e capacidades (`getInitialFluidTanks` / `getInitialChemicalTanks` vazios). Os fluidos e calor ficam preservados no `MultiblockCache` da dimensão até a estrutura reformar.
  - *Port anterior*: A função `run_boiler` ignorava válvulas inválidas (`decode(&v.data).filter(|f| f.valid)`). Válvulas de estruturas quebradas mantinham suas faces abertas e seus filtros de tanque ativos no registro global (`SPEC_CAPACITY`), continuando a operar como tanques independentes gigantescos no mundo (Sintoma S3 relatado pelo dono do projeto).
  - *Correção*: Quando `!found.valid`, tanto `boiler_structures` quanto `run_boiler` fecham todas as 4 faces dos tanques (`tank_faces: Some(vec![vec![FlowSide::None; 6]; 4])`), configuram todos os filtros para `NOTHING` (`world::set_tank_filter`) e limpam os dados de menu (`menu_data: Some(Vec::new())`). O conteúdo prévio dos tanques é preservado intacto em `v.tanks` (`tanks: None`) para restauração automática quando a estrutura for reformada.
  - *Estado*: **BUG CORRIGIDO (S3)**.
- **Notificação e Faíscas de Formação (S2 / `TileEntityMultiblock.java:297-312`)**:
  - *Java*: Ao formar (`!prevStructure && unformedTicks >= 5`), executa `SparkleAnimation` ao longo das arestas da estrutura para jogadores num raio de 40 blocos (ou mensagem no chat se partículas estiverem desligadas).
  - *Port*: Adicionada flag `b.formed` em `Boiler`. Quando uma estrutura antes inválida torna-se válida, o mod detecta a transição e prepara a invocação de partículas/aviso aos jogadores (`FORK-30`).
  - *Estado*: **FORK-30 DOCUMENTADO**.

---

### 1.3 Estado entre Blocos e Sincronização de Tanques
- **Verificação de Conteúdo Duplicado, Perdido ou Criado do Nada**:
  - *Java*: Todas as válvulas delegam chamadas de capacidade diretamente para os tanques da instância `BoilerMultiblockData`.
  - *Port anterior*: Na inicialização após carregamento de mundo ou chunk (`b.given.is_empty()`), `b.water/steam/hot/cold` eram inicializados a partir de `best.amount`, mas `b.given` permanecia **vazio**. Em seguida, o laço de ticks consultava `b.given.get(...)`, que devolvia 0 para todas as válvulas, e o `dynamic::merge` somava `now[j] - 0` de **cada** válvula sobre o valor já carregado! Isso multiplicava o conteúdo de todos os 4 tanques por `(1 + quantidade_de_válvulas)` a cada reinício do mundo!
  - *Correção*: Assim que `b.given.is_empty()` é detectado, `b.given` é imediatamente populado com a quantidade atual de cada válvula (`b.given.insert(...)`), de forma que `now - given == 0` no primeiro tick. Conservação estrita de matéria sem duplicações.
  - *Estado*: **BUG CRÍTICO CORRIGIDO**.
- **Modos e Filtros de Válvula (`TileEntityBoilerValve.java:51, 57`)**:
  - *Java*:
    - Modo 0 (`INPUT`): aceita água (tanque 0) e refrigerante superaquecido (tanque 2). Não expõe vapor nem refrigerante frio.
    - Modo 1 (`OUTPUT_STEAM`): expõe apenas vapor (tanque 1) para extração/ejeção. Rejeita entrada de água e qualquer refrigerante.
    - Modo 2 (`OUTPUT_COOLANT`): expõe apenas refrigerante resfriado (tanque 3) para extração/ejeção. Rejeita entrada de água e refrigerante quente.
  - *Port anterior*: A rotina `set_tank_filter` configurava filtros de água e refrigerante quente mesmo em válvulas de saída (modos 1 e 2). Tubulações externas podiam injetar água em válvulas de saída de vapor.
  - *Correção*: Nos modos 1 e 2, os tanques 0 (água) e 2 (quente) agora recebem filtro `NOTHING`. Apenas no modo 0 (`INPUT`) a entrada de água e refrigerante quente é admitida.
  - *Estado*: **BUG CORRIGIDO**.

---

### 1.4 Funcionamento por Tick
- **Dissipação Térmica e Impedância do Ar**:
  - *Java* (`simulateEnvironment`, `BoilerMultiblockData.java:293-298`): Perda = `(T - ambient) / (AIR + INSULATION + CONDUCTION)`. Com `AIR = 10_000.0`, `INSULATION = 100_000.0`, `CONDUCTION = 1.0`, impedância total = `110_001.0`.
  - *Port* (`boiler.rs:246-247`): `b.temperature -= (b.temperature - ambient) / 110_001.0`.
  - *Estado*: **OK**.
- **Capacidade Térmica (`BoilerMultiblockData.java:145`)**:
  - *Java*: `heatCapacitor.setHeatCapacity(CASING_HEAT_CAPACITY * locations.size())`. `locations` contém apenas os blocos da casca externa (casing, valve, glass). Dispersores e superheaters são `internalLocations`.
  - *Port anterior*: `f.blocks` era igual a `members.len()`, somando indevidamente dispersores e elementos na capacidade térmica.
  - *Correção*: `shell_blocks = size[0]*size[1]*size[2] - (size[0]-2)*(size[1]-2)*(size[2]-2)`.
  - *Estado*: **BUG CORRIGIDO**.
- **Transferência de Calor por Refrigerante Superaquecido (`BoilerMultiblockData.java:180-193`)**:
  - *Java*: `portion = 0.4 * stored`. `toCool = round(portion * (1 - T / 100_000))`. Calor transferido = `toCool * 5.0 J`.
  - *Port* (`boiler.rs:249-263`): Reproduz fielmente a mesma fórmula.
  - *Estado*: **OK**.
- **Ebulição da Água e Geração de Vapor (`BoilerMultiblockData.java:195-212`)**:
  - *Java*: Se `T >= 373.15 K` e houver água: `heatAvailable = min((T - 373.15) * capacity * 0.7, 16_000_000 * elements)`. `max = floor(0.2 * heatAvailable / 10.0)`. `boiled = min(max, water, room)`.
  - *Port* (`boiler.rs:266-283`): Reproduz fielmente a mesma fórmula e deduz a entalpia térmica.
  - *Estado*: **OK**.
- **Estado Visual dos Elementos Superaquecedores (`TileEntitySuperheatingElement.java:19`)**:
  - *Java*: Quando `T >= 373.14 K` (`BASE_BOIL_TEMP - 0.01`), `BoilerMultiblockData.hotMap` ativa os elementos (`setActive(true)`, `Attributes.ACTIVE_LIGHT`).
  - *Port anterior*: Os blocos `superheating_element` nunca tinham sua propriedade `lit` atualizada.
  - *Correção*: O port agora atualiza `lit="true"` nos elementos quando `T >= 373.14 K` e `lit="false"` quando inferior ou desmontado.
  - *Estado*: **BUG CORRIGIDO**.

---

### 1.5 Casos Limite
- **Limites de Dimensão (3x3x3 a 18x18x18)**:
  - Validados estritamente no intervalo [3, 18].
  - *Estado*: **OK**.
- **Blocos Estranhos no Interior**:
  - Verificação de `inside_empty` para blocos não membros; qualquer bloco sólido introduzido invalida a caldeira.
  - *Estado*: **OK**.
- **Quebra Parcial e Descarregamento de Chunks**:
  - Modos de válvula salvos em `port_modes` e restaurados no NBT/data.
  - *Estado*: **OK**.

---

## 2. Tabela Comparativa Completa (Java vs Port)

| Item de Verificação | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|
| **Dimensões mín/máx** | `CuboidStructureValidator.java:34` | `boiler.rs:81` | **OK** | Caldeira aceita de 3x3x3 até 18x18x18 em cada eixo. |
| **Vidro estrutural nas arestas** | `CuboidStructureValidator.java:115` | `boiler.rs:105-107` | **BUG CORRIGIDO** | Vidro estrutural nas 12 arestas/quinas agora invalida a estrutura (antes formava indevidamente). |
| **Vidro estrutural nas faces** | `CuboidStructureValidator.java:103` | `boiler.rs:107` | **OK** | Vidro estrutural aceito em qualquer face plana externa. |
| **Válvula nas arestas/quinas** | `CuboidStructureValidator.java:115,149` | `boiler.rs:107` | **OK** | Válvulas são rejeitadas em arestas e quinas. |
| **Plano de dispersores** | `BoilerValidator.java:68-84` | `boiler.rs:113-115` | **OK** | Exige plano horizontal único e completo `(length-2)*(width-2)`. |
| **Conectividade de superheaters** | `BoilerValidator.java:86-91` | `boiler.rs:117-133` | **OK** | Elementos devem estar abaixo dos dispersores e interligados. |
| **Camada 100% de superheaters** | `BoilerValidator.java:97-126` | `boiler.rs:138-142` | **BUG CORRIGIDO** | Caldeira com fundo 100% preenchido por superheaters agora forma (antes falhava por ausência de ar). |
| **Elementos dividindo ar** | `BoilerValidator.java:115-125` | `boiler.rs:138-142` | **BUG CORRIGIDO** | Paredes de superheaters dividindo o ar interno agora formam normalmente (fluido conecta via carcaça). |
| **Volume de água da caldeira** | `BoilerValidator.java:115-121`<br>`BoilerMultiblockData.java:307` | `boiler.rs:139` | **BUG CORRIGIDO** | Volume de água agora inclui o piso e paredes submersas conforme Java, multiplicando a capacidade em até 4,3x. |
| **Capacidade térmica da carcaça** | `BoilerMultiblockData.java:145` | `boiler.rs:138, 227` | **BUG CORRIGIDO** | Capacidade térmica baseada estritamente na casca externa, sem inflar com dispersores/elementos. |
| **Válvula isolada pós-quebra (S3)** | `TileEntityMultiblock.java:218-229`<br>`MultiblockCache.java:43` | `boiler.rs:320-323, 350-367` | **BUG CORRIGIDO** | Quebrar a caldeira fecha as faces e põe filtros em `NOTHING` na válvula; não opera mais como tanque fantasma. |
| **Duplicação de fluidos no reload** | `TileEntityBoilerValve.java:108`<br>`MultiblockCache.java:43` | `boiler.rs:380-383` | **BUG CORRIGIDO** | `b.given` é inicializado no primeiro tick, eliminando multiplicação de fluidos pelo número de válvulas. |
| **Filtros de válvulas nos modos de saída** | `TileEntityBoilerValve.java:51,57` | `boiler.rs:462-466` | **BUG CORRIGIDO** | Válvulas em modo saída de vapor ou refrigerante agora rejeitam injeção de água e refrigerante quente. |
| **Temperatura inicial ao criar** | `BoilerMultiblockData.java:124` | `boiler.rs:369` | **BUG CORRIGIDO** | Caldeira não inicializada nasce na temperatura ambiente (300 K) em vez de 1.0 K. |
| **Estado visual dos superheaters** | `TileEntitySuperheatingElement.java:19`<br>`BoilerMultiblockData.java:174` | `boiler.rs:424-428` | **BUG CORRIGIDO** | Superheating elements agora acendem (`lit="true"`) ao ferver (`T >= 373.14 K`) quando vistos pelo vidro. |
| **Efeito de faíscas ao formar (S2)** | `TileEntityMultiblock.java:297-312` | `boiler.rs:293-298` | **FORK** | Detecção de transição de formação implementada; emissão de partículas depende de `world.spawn-particles` (FORK-30). |
| **Menu bloqueado sem formação (S1)** | `TileEntityMultiblock.java:218-222` | `boiler.rs:322, 364` | **FORK** | Dados do menu zerados quando inválido; bloqueio do consumo de clique no mundo depende de FORK-29. |
| **Sinal de comparador em válvulas** | `TileEntityBoilerValve.java:79-82` | `boiler.rs` | **FORK** | Válvulas deveriam emitir sinal analógico 0..15 pelo nível de água; requer hook de comparador no Horizon. |
| **Renderização 3D de fluidos (S4)** | `RenderThermoelectricBoiler.java:30-50` | `boiler.rs` | **FORK** | Java desenha malha volumétrica 3D da água e do vapor dentro do vidro estrutural; requer FORK-19. |

---

## 3. Lista Detalhada de Bugs Encontrados e Sintomas em Jogo

1. **BUG 1: Vidro estrutural permitido nas 12 arestas e 8 quinas da moldura**
   - *Referência*: `CuboidStructureValidator.java:115` & `BoilerValidator.java:28-33`.
   - *Sintoma em jogo*: Jogador construía a caldeira com vidro estrutural nas quinas e arestas e ela formava, violando o Mekanism Java onde a moldura exige estritamente `boiler_casing`.
   - *Correção*: `Part::Glass` separado de `Part::Casing`; `edges >= 2` aceita unicamente `Part::Casing`.

2. **BUG 2: Subestimação de volume de água e rejeição de caldeiras com muitos superheaters**
   - *Referência*: `BoilerValidator.java:97-129` & `BoilerMultiblockData.java:307`.
   - *Sintoma em jogo*: Caldeiras com uma camada inteira preenchida por elementos superaquecedores eram rejeitadas como inválidas. Caldeiras válidas tinham capacidade de água e refrigerante quente até 4,3 vezes menor que no original, porque o port contava apenas células de ar e descartava o piso e paredes submersas.
   - *Correção*: Volume corrigido para `(size[0] * size[2] * (plane - min[1])) - elements.len()`, permitindo também configurações sem ar residual no fundo.

3. **BUG 3: Duplicação massiva de fluidos e gases a cada recarregamento de mundo (Sintoma S3)**
   - *Referência*: `TileEntityBoilerValve.java:108` & `MultiblockCache.java:43`.
   - *Sintoma em jogo*: Ao reiniciar o servidor ou descarregar e recarregar a chunk, a caldeira multiplicava a quantidade de água, vapor, refrigerante quente e frio por `(1 + quantidade_de_válvulas)` devido a `b.given` estar vazio no primeiro tick.
   - *Correção*: `b.given` é populado imediatamente na primeira leitura com as quantidades correntes de cada válvula.

4. **BUG 4: Válvula isolada operando como tanque após quebra da estrutura (Sintoma S3)**
   - *Referência*: `TileEntityMultiblock.java:218-229`.
   - *Sintoma em jogo*: O jogador quebrava a carcaça da caldeira deixando apenas uma válvula no chão. A válvula continuava aceitando e fornecendo água e vapor de tubos e baldes como se fosse um megatanque independente.
   - *Correção*: Quando `!found.valid`, os filtros de todos os 4 tanques são travados em `NOTHING`, as faces dos tanques são configuradas para `FlowSide::None` e o menu é esvaziado. O conteúdo é retido internamente apenas para restauração em caso de reforma.

5. **BUG 5: Válvulas de saída aceitando injeção de água e refrigerante quente**
   - *Referência*: `TileEntityBoilerValve.java:51,57`.
   - *Sintoma em jogo*: Válvula configurada no configurador como "output steam" ou "output coolant" aceitava que tubulações externas empurrassem água e refrigerante quente para dentro da caldeira.
   - *Correção*: Nos modos 1 e 2, os filtros dos tanques 0 e 2 são configurados estritamente para `NOTHING`.

6. **BUG 6: Inflação da capacidade térmica da carcaça**
   - *Referência*: `BoilerMultiblockData.java:145`.
   - *Sintoma em jogo*: Dispersores e superheaters eram somados à capacidade térmica da caldeira, fazendo caldeiras com muitos blocos internos demorarem mais para aquecer e esfriar do que no Java.
   - *Correção*: `blocks` ajustado para representar a casca externa do paralelepípedo (`size[0]*size[1]*size[2] - (size[0]-2)*(size[1]-2)*(size[2]-2)`).

7. **BUG 7: Superheating Elements sem brilho visual ao ferver**
   - *Referência*: `TileEntitySuperheatingElement.java:19` & `BoilerMultiblockData.java:174`.
   - *Sintoma em jogo*: Elementos superaquecedores permaneciam apagados mesmo com a caldeira fervendo a alta temperatura.
   - *Correção*: O port agora define `lit="true"` nos elementos quando `T >= 373.14 K` e `lit="false"` quando frios ou desmontados.
