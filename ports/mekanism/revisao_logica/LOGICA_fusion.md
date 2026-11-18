# Relatório de Auditoria de Lógica: Fusion Reactor

**Tarefa:** T20261109-230438-3c3  
**Escopo de código:** `port/mod/src/fusion.rs`  
**Referência Java:** Mekanism 1.21.1-10.7.19.85 (`FusionReactorValidator.java`, `FusionReactorMultiblockData.java`, `FusionReactorCache.java`, `TileEntityFusionReactorBlock.java`, `TileEntityFusionReactorController.java`, `TileEntityFusionReactorPort.java`, `TileEntityFusionReactorLogicAdapter.java`, `TileEntityLaserFocusMatrix.java`, `RenderFusionReactor.java`, `CuboidStructureValidator.java`, `FormationProtocol.java`, `StructureHelper.java`, `MultiblockManager.java`, `MultiblockCache.java`)

---

## 1. Tabela Comparativa Passo a Passo: Java vs Port

| Seção | Aspecto Auditado | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|---|
| **1. Forma** | Dimensões do reator (5×5×5 fixo) | `FusionReactorValidator.java:24` | `fusion.rs:87-89` | **OK** | Cubóide delimitador fixo de 5×5×5 em X, Y, Z. |
| **1. Forma** | Formato arredondado (12 blocos vazios por face) | `FusionReactorValidator.java:25-31, 76-78` | `fusion.rs:47-48, 78-83` | **OK** | 72 posições de quinas e sub-quinas externas vazadas. |
| **1. Forma** | Interior (câmara oca 3×3×3 de ar) | `CuboidStructureValidator.java:96-98` | `fusion.rs:101-105` | **OK** | Qualquer bloco não-ar no interior de 3×3×3 impede a formação. |
| **1. Forma** | Posição única do Controlador | `FusionReactorValidator.java:48-56` | `fusion.rs:113-121` | **OK** | Controlador obrigatório no topo central `(2, 4, 2)`. Qualquer outra posição falha. |
| **1. Forma** | Vidro de Reator nas molduras (aresta / ring) | `FusionReactorValidator.java:61-72`, `CuboidStructureValidator.java:115` | `fusion.rs:122-132` | **BUG -> CORRIGIDO** | No port anterior, `Reactor Glass` era aceito na moldura (`need == 1`). No Java, vidro só pode ir nas faces (`need == 2`). Corrigido para restringir vidro a faces. |
| **1. Forma** | Vidro Estrutural comum | `TileEntityStructuralGlass.java:17` | `fusion.rs:59-69` | **OK** | `Structural Glass` rejeita qualquer multiblock "reactor"; apenas `Reactor Glass` é aceito. |
| **1. Forma** | Portas, Logic Adapters e Laser Focus Matrix | `FusionReactorValidator.java:65-70` | `fusion.rs:127-132` | **OK** | Permitidos em qualquer posição de face (`need == 2`), exceto no slot do controlador. |
| **2. Formação** | Sparkles e Notificação ao Formar (S2) | `TileEntityMultiblock.java:297-312` | `fusion.rs:346-364` | **BUG -> CORRIGIDO** | Ao formar, jogadores a até 40 blocos recebem a mensagem "Multiblock formed" e faíscas ao longo das arestas (FORK-30). |
| **2. Desmontagem** | Lockdown de Válvulas/Portas Inválidas (S3) | `TileEntityMultiblock.java:218-229`, `SINTOMAS_MULTIBLOCOS.md:25-29` | `fusion.rs:388-425, 439-470` | **BUG -> CORRIGIDO** | Porta de reator desmontado ou colocada avulsa fechava faces mas continuava com filtros abertos ou vice-versa. Agora fecha todas as faces (`FlowSide::None`), zera energia, esvazia tanques espelho e tranca todos os 5 filtros com `NOTHING`. |
| **2. Desmontagem** | Preservação de Conteúdo na Quebra e Reforma | `FusionReactorCache.java:23-48`, `MultiblockCache.java:43-71` | `fusion.rs:388-400, 485-520` | **BUG -> CORRIGIDO** | No port anterior, re-formar subtraía tanques antigos de 0 via `dynamic::merge` e zerava tudo. Corrigido para manter `Shared.reactor` persistente no cache `self.fusions` ao desmontar e restaurar integralmente ao reformar. |
| **3. Estado** | Sincronização e Distribuição de Energia nas Portas | `FusionReactorMultiblockData.java:157, 249-251` | `fusion.rs:522-540, 595-620` | **BUG -> CORRIGIDO** | No port anterior, a energia total era dividida por todos os blocos de porta (inclusive portas de entrada com faces fechadas). Apenas portas de saída recebem cotas de saída, disponibilizando 100% da energia do reator para cabos externos. |
| **3. Estado** | Filtros de Tanque por Modo de Porta | `TileEntityFusionReactorPort.java:47-56` | `fusion.rs:595-635` | **BUG -> CORRIGIDO** | Portas em modo saída travam tanques de entrada (0..3) com `NOTHING` e abrem vapor (4); portas de entrada abrem água/D/T/Fuel e travam vapor com `NOTHING`. |
| **4. Tick** | Ignição Térmica e Consumo de Hohlraum | `FusionReactorMultiblockData.java:222-235, 300-309` | `fusion.rs:222-235` | **OK** | Hohlraum com combustível D-T no slot do controlador é vaporizado e inicia a queima quando plasma >= 100M K. |
| **4. Tick** | Injeção em Números Pares de D-T | `FusionReactorMultiblockData.java:311-320` | `fusion.rs:237-254` | **OK** | Proporção 1:1 rigorosa (`inject -= inject % 2; half = inject / 2`). |
| **4. Tick** | Teto de 32 bits na Geração Passiva | `FusionReactorMultiblockData.java:358` | `fusion.rs:296-307` | **BUG -> CORRIGIDO** | Removido clamp artificial em `i32::MAX`, restaurando integridade aritmética de 64 bits (`clampToLong` do Java). |
| **4. Tick** | Aquecimento por Laser na Focus Matrix | `FusionReactorMultiblockData.java:195-201`, `TileEntityLaserFocusMatrix.java:18-23` | `fusion.rs:209-216` | **OK** | 10× de eficiência térmica quando desligado, 1× quando em queima. |
| **4. Tick** | Latência de Atualização Visual da Queima | `FusionReactorMultiblockData.java:261` | `fusion.rs:580, 638` | **BUG -> CORRIGIDO** | No port anterior, `data` só atualizava a cada 20 ticks. Mudanças em `burning` (ignição ou extinção) agora forçam atualização imediata no tick para iluminação e som sem lag. |
| **4. Tick** | Redstone Comparator do Reator | `FusionReactorMultiblockData.java:447-449` | `fusion.rs:340-348` | **FALTA -> IMPLEMENTADO** | Nível proporcional ao tanque de combustível D-T (0 se vazio, 1..15 conforme enchimento de 1 000 mB). |
| **4. Tick** | Modos do Logic Adapter | `TileEntityFusionReactorLogicAdapter.java:85-97` | `fusion.rs:350-368` | **FALTA -> IMPLEMENTADO** | Modos READY, CAPACITY, DEPLETED e DISABLED implementados com os limiares exatos do Java. |
| **4. Interação** | Clique com Bloco sem Agachar (S1) | `TileEntityMultiblock.java:218-229`, `SINTOMAS_MULTIBLOCOS.md:7-15` | `fusion.rs:500-515` | **FORK (FORK-29)** | Menu só abre se formado e sem shift. Bloco unformed ou com shift deve passar clique adiante para posicionar bloco da mão. |
| **4. Visual** | Núcleo de Plasma Tridimensional (S4) | `RenderFusionReactor.java:34-52` | `fusion.rs:378-385` | **FORK (FORK-19)** | Esferas concêntricas rotativas Red, Pink e Orange no interior 3×3×3 quando em queima. |
| **5. Limites** | Kill Zone Instantânea no Interior 3×3×3 | `FusionReactorMultiblockData.java:287-298` | `fusion.rs:387-393` | **FORK (FORK-7/11)** | 50 000 de dano por segundo a entidades na câmara interna durante a queima. |

---

## 2. Detalhamento dos Defeitos Identificados e Correções Aplicadas

### Defeito 1: Vidro de Reator (`Reactor Glass`) aceito nas molduras estruturais
- **Java:** `FusionReactorValidator.java:61-72` define `GeneratorsBlockTypes.FUSION_REACTOR_FRAME` como `CasingType.FRAME`. No validador (`CuboidStructureValidator.java:115`), posições com `needsFrame == true` (onde `ALLOWED_GRID == 1`) exigem estritamente `type.isFrame()`. Como `ReactorGlass` é `CasingType.OTHER` (via `IStructuralMultiblock`), ele **falha** se colocado na moldura.
- **Port anterior:** `fusion.rs:122` agrupava `Some(Part::Frame | Part::Glass) => { if is_controller_spot { return bad; } }`, permitindo que o anel estrutural fosse inteiramente de vidro.
- **Sintoma em jogo:** Jogador conseguia formar um reator substituindo a carcaça de metal da moldura por vidro.
- **Correção:** `fusion.rs` agora restringe `Part::Glass` estritamente a posições de face (`need == 2`), rejeitando vidro em posições `need == 1`.

### Defeito 2: Sintoma S3 — Portas e Válvulas inválidas mantinham filtros e tanques abertos
- **Java:** `TileEntityMultiblock.java:218-229` e `TileEntityFusionReactorPort.java:47-68`. Sem estrutura formada, nenhuma capacidade de fluido, químico ou energia é exposta para o mundo externo.
- **Port anterior:** Em `fusion.rs:378`, o loop `run_fusion` filtrava apenas blocos com `f.valid == true`. Blocos de porta com `!f.valid` eram ignorados e permaneciam com as capacidades padrão enormes do registro e faces abertas, aceitando canos e baldes como um tanque isolado.
- **Sintoma em jogo:** Uma porta colocada sozinha no chão ou deixada após quebrar o reator funcionava como um super-tanque de 5 substâncias.
- **Correção:** Portas e controladores inválidos são agora coletados em `invalid_parts`. A cada tick, portas inválidas têm todas as faces trancadas com `FlowSide::None`, energia zerada, tanques esvaziados e os 5 filtros de tanque cravados em `mekanism:nothing`. Controladores inválidos têm `menu_data` forçado para `None`.

### Defeito 3: Perda total de fluidos e energia ao quebrar e reformar o reator
- **Java:** `FusionReactorCache.java` e `MultiblockCache.java`. Quando a estrutura é desmontada, os conteúdos de todos os tanques e capacitores são preservados no cache sob o `inventoryID`. Quando a estrutura reforma, `cache.apply` restaura 100% dos fluidos, energia, calor e taxa de injeção.
- **Port anterior:** `Shared.started` não era resetado para `false` ao desmontar. Quando a estrutura era reformada, `dynamic::merge` comparava `now` (que vinha zerado das portas recém-colocadas) com `given` (que retinha o valor antes de quebrar) e subtraía o total, zerando todos os tanques de água, combustível e vapor.
- **Sintoma em jogo:** Qualquer manutenção que retirasse e recolocasse 1 bloco de carcaça deletava instantaneamente todos os fluidos e todo o D-T acumulado.
- **Correção:** Em `fusion_structures`, quando a estrutura é invalidada, `s_entry.started = false` e `s_entry.given.clear()`, mantendo os dados preservados em `s_entry.reactor`. Ao reformar, `!s.started` reutiliza o reator em cache sem perdas.

### Defeito 4: Sintoma S2 — Ausência de feedback visual/mensagens ao formar o reator
- **Java:** `TileEntityMultiblock.java:297-312`. Ao completar a montagem, o cliente dispara partículas ao longo do contorno (`SparkleAnimation`) e, caso partículas estejam desligadas, uma mensagem no chat em índigo: "Multiblock formed".
- **Port anterior:** Nenhuma mensagem ou efeito era emitido.
- **Sintoma em jogo:** Jogador não sabia se o reator tinha sido montado corretamente sem abrir o menu do controlador.
- **Correção:** Em `fusion_structures`, a transição de não-formado para formado detecta a validação e envia "Multiblock formed" para todos os jogadores a até 40 blocos de distância (`players::online()`), acionando também o hook isolado de partículas FORK-30.

### Defeito 5: Divisão indevida de energia entre portas de entrada e saída
- **Java:** `FusionReactorMultiblockData.java:157, 249-251` e `TileEntityFusionReactorPort.java:87-94`. As portas de entrada têm faces de energia desabilitadas (`FlowSide::None`). A energia do reator só pode ser ejetada por portas em modo saída (`mode == true`).
- **Port anterior:** O port dividia a energia por `ports.len()`. Se o reator tivesse 3 portas de entrada e 1 de saída, 75% da energia ficava presa nas portas de entrada onde nenhum cabo conseguia extrair.
- **Sintoma em jogo:** Reator produzia energia, mas a taxa máxima de saída por cabos ficava severamente estrangulada se houvesse mais de uma porta.
- **Correção:** A cota de energia compartilhada é calculada estritamente sobre as portas em modo saída (`output_ports_count`). Portas de entrada recebem cota 0 e faces fechadas.

### Defeito 6: Teto artificial de 32 bits (`i32::MAX`) na geração passiva
- **Java:** `FusionReactorMultiblockData.java:358`. Usa aritmética `MathUtils.clampToLong` de 64 bits.
- **Port anterior:** `(case_air * THERMOCOUPLE * 0.4).clamp(0.0, f64::from(i32::MAX)) as u64` impunha teto de 2,14 bilhões de FE.
- **Sintoma em jogo:** Truncamento em taxas astronômicas de queima.
- **Correção:** Removido o clamp de 32 bits, preservando aritmética inteira de 64 bits (`(case_air * THERMOCOUPLE * 0.4).max(0.0) as u64`).

### Defeito 7: Latência de até 20 ticks na sincronização de estado de queima
- **Java:** `FusionReactorMultiblockData.java:261` força `needsPacket = true` no mesmo tick em que `isBurning()` altera seu estado.
- **Port anterior:** Os dados `data` dos blocos eram gravados apenas a cada 20 ticks (`tick_no % 20 == 0`).
- **Sintoma em jogo:** Atraso de até 1 segundo para o reator registrar que apagou ou acendeu.
- **Correção:** O port detecta `burning_changed` e re-codifica o bloco imediatamente no tick em que a ignição ou extinção ocorre.
