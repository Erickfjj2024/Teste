# Revisão de Lógica: Fission Reactor (Reator de Fissão)

**Escopo:** `port/mod/src/fission.rs`  
**Referência Java:** Mekanism 1.21.1-10.7.19.85 (`FissionReactorValidator.java`, `FissionReactorMultiblockData.java`, `FissionReactorCache.java`, `TileEntityFissionReactorPort.java`, `TileEntityFissionReactorLogicAdapter.java`, `TileEntityReactorGlass.java`, `TileEntityFissionReactorCasing.java`, `TileEntityFissionAssembly.java`, `CuboidStructureValidator.java`, `StructureHelper.java`, `FormationProtocol.java`, `Meltdown.java`, `GeneratorsConfig.java`)  
**Tarefa:** T20261109-230337-0z4 (continuação de T20261109-220615-jd9 após reinício da VM)

---

## 1. Passo a Passo da Lógica contra o Java

### 1.1 Validação da Forma (Estrutura, Paredes, Interior e Assemblies)
- **Dimensões e limites:** Java define bounds de 3x3x3 até 18x18x18 (`CuboidStructureValidator.java:34`, `GeneratorsConfig.java:155` volume máx 5832). O port valida `(3..=18).contains(s)` em `fission.rs:92`. Não há restrição de paridade (ao contrário da Turbine). A menor dimensão viável em jogo é **3x4x3**, pois o interior requer ao menos 1 Fuel Assembly e 1 Control Rod Assembly empilhados verticalmente (altura interna mínima 2). Um reator 3x3x3 possui altura interna 1 e falha na validação em ambos.
- **Arestas e quinas (Edges/Corners):** Em Java, `CuboidStructureValidator.java:115, 150` exige `needsFrame()`, o que restringe as 12 arestas e 8 quinas estritamente a `CasingType.FRAME` (`FissionReactorValidator.java:42` -> `GeneratorsBlockTypes.FISSION_REACTOR_CASING`). No port, `(_, Some(Part::Casing))` permite casing nas arestas e quinas; qualquer outro bloco em arestas (`edges > 1`) é rejeitado.
- **Paredes / Faces (Walls):** Em Java, posições de parede (`edges == 1`) aceitam:
  1. `FISSION_REACTOR_CASING` (`CasingType.FRAME`)
  2. `FISSION_REACTOR_PORT` (`CasingType.VALVE`)
  3. `FISSION_REACTOR_LOGIC_ADAPTER` (`CasingType.OTHER`)
  4. `REACTOR_GLASS` (`TileEntityReactorGlass.java:16`, `CuboidStructureValidator.java:103-118`: implementa `IStructuralMultiblock` com `canInterface() == true`).
  *Discrepância corrigida (BUG 1):* O port não aceitava `mekanism:reactor_glass` no reator de fissão. Corrigido adicionando `Part::Glass` e aceitando-o em `(1, Some(Part::Port | Part::Adapter | Part::Glass))`.
- **Interior:** Em Java, posições internas (`edges == 0`) aceitam apenas ar (`state.isAir()`), `FISSION_FUEL_ASSEMBLY` e `CONTROL_ROD_ASSEMBLY` (`FissionReactorValidator.java:56`). No port, posições com `edges == 0` aceitam apenas `Part::Fuel`, `Part::Rod` e ar via `inside_empty` (`fission.rs:107-114`).
- **Regras das Colunas de Combustível:**
  - Em Java (`FissionReactorValidator.java:124-149`), cada coluna (x, z) com combustível deve ter:
    - Pelo menos 1 `FISSION_FUEL_ASSEMBLY` contíguo em Y.
    - Exatamente 1 `CONTROL_ROD_ASSEMBLY` no topo (`controlRodAssembly.getY() == prevY + 1`).
    - Nenhuma coluna pode ter barra de controle sem combustível, combustível sem barra, ou barra no meio/embaixo da coluna.
    - É necessária pelo menos 1 coluna formada (`map.isEmpty() => FAIL`).
    - Não é obrigatório tocar o piso nem o teto da carcaça.
  - O port reproduz integralmente essa validação em `fission.rs:122-135`.
- **Área de Superfície do Combustível:**
  - Cada bloco de combustível tem 6 faces expostas; a cada par de blocos de combustível adjacentes (em qualquer uma das 6 direções), 2 faces deixam de ser expostas (`FissionReactorValidator.java:79-85`).
  - Eficiência de fervura: `boilEfficiency = min(1.0, (surfaceArea / fuelAssemblies) / 4.0)` (`FissionReactorMultiblockData.java:565-566`).
  - O port computa exatamente a mesma fórmula (`fission.rs:137-144, 244-249`).

### 1.2 Formação e Desmontagem
- **Disparo de Formação:** Em Java, a formação ocorre via `FormationProtocol.doUpdate()` acionado por colocação/quebra de bloco ou atualização de vizinho. No Horizon, o motor chama `multiblock-changed` com os blocos contíguos do grupo.
- **Desmontagem e Quebra de Bloco:**
  - Em Java, ao quebrar qualquer bloco, a estrutura se desfaz (`multiblockUnformed`), transferindo todo o estado para o cache `MultiblockCache` indexado pelo UUID do inventário. Os blocos no mundo deixam de expor inventários/fluidos/capacidades.
  - No port, `fission_structures` detecta invalidação. O estado fica preservado em memória em `self.reactors` indexado por `(dimension, min)`.
  - *Sintoma S3 corrigido:* Portas pertencentes a estruturas inválidas (ou desfeitas) antes continuavam funcionando como tanques autônomos com capacidade enorme (`SPEC_CAPACITY`). Corrigido: quando `!found.valid` ou `!f.valid`, todas as portas têm suas faces fechadas (`FlowSide::None`), filtros travados em `NOTHING`, inventário zerado e menus desativados, enquanto o conteúdo permanece seguro em `self.reactors` para a reforma.
- **Animação/Mensagem de Formação (Sintoma S2):**
  - Em Java (`TileEntityMultiblock.java:297-312`), ao formar, faíscas são desenhadas nas arestas (`SparkleAnimation`) ou a mensagem `Multiblock formed` é exibida.
  - No port, detectamos a transição `found.valid && !was_valid` e enviamos notificação para jogadores a até 40 blocos de distância, com solicitação de API em `FORK-30` para geração de partículas.
- **Preservação de Conteúdo:** Água, sódio refrigerante, combustível fissil, vapor, sódio superaquecido, resíduo, calor acumulado e taxa de queima persistem entre desmontagens e remontagens. Se reconstruído com tamanho menor, os estoques são reduzidos às capacidades da nova estrutura (`fission.rs:448, 469`).

### 1.3 Estado entre Blocos
- **Sincronização entre Portas:** Em Java, o `FissionReactorMultiblockData` centraliza os tanques; as portas são meros proxies. No port, o struct `Shared` em `self.reactors` centraliza o estado. Cada porta recebe um espelho das quantidades (`tanks_now`). No início de cada tick, `dynamic::merge` computa o delta `now - given` de todas as portas, impedindo duplicação ou sumiço de substâncias.
- **Modos de Porta:** Suporta 3 modos (`AttributeStateFissionPortMode.java:49-51`):
  - Modo 0: `INPUT` (aceita água, refrigerante sódio e combustível fissil).
  - Modo 1: `OUTPUT_WASTE` (ejeta resíduo nuclear).
  - Modo 2: `OUTPUT_COOLANT` (ejeta vapor ou sódio superaquecido).
  Configurados no jogo via Shift+RightClick com o Configurador (`fission.rs:568-588`).
- **Persistência em Salvamento/Reload:**
  - *Bug 3 corrigido:* Ao salvar o reator com `rate_limit = 0.0`, no carregamento o port forçava `rate_limit` de volta para `DEFAULT_RATE` (0.1) devido ao check `saved.rate_limit > 0.0`. Corrigido para `saved.rate_limit >= 0.0` e inicialização padrão unificada em `fission_structures`.

### 1.4 Funcionamento por Tick
- **Ordem de Operações:** A ordem de operações do Java (`FissionReactorMultiblockData.java:181-210`) é estritamente seguida:
  1. Queima de combustível (`burnFuel`) -> aquece o capacitor térmico e gera resíduo nuclear.
  2. Fervura do refrigerante (`handleCoolant`) -> resfria o reator e produz vapor / sódio superaquecido.
  3. Dissipação ambiental (`simulateEnvironment`) -> troca calor com o ambiente (`divisor = 20_010.0`).
  4. Cálculo de dano e meltdown (`handleDamage`).
- **Queima Residual de Combustível (Bug 2 corrigido):**
  - Em Java (`FissionReactorMultiblockData.java:474-478`), o reator queima `storedFuel = fuelTank.getStored() + burnRemaining`. Se `fuelTank.getStored() == 0`, a fração restante em `burnRemaining` continua queimando até zerar.
  - No port, a condição anterior `if r.fuel.amount > 0` congelava a queima assim que o tanque atingia 0 mB inteiros, deixando a fração presa indefinidamente no reator. Corrigido para `(r.fuel.amount > 0 || r.burn_remaining > 0.0)`.
- **Fervura de Refrigerante (Bug 4 corrigido):**
  - O port agora valida explicitamente se a substância do tanque de água é de fato água (`r.water.chemical == WATER || r.water.chemical.is_empty()`) antes de ferver em vapor.

### 1.5 Casos Limite
- **Meltdown:** Quando `damage >= 100.0` e `temp >= 1200 K`, o reator tem chance de 0.1% a cada tick de explodir. Ao explodir, remove blocos em raio 8 (exceto blocos indestrutíveis), reseta a estrutura para dano pós-meltdown de 75.0%, limpa os tanques de combustível, resíduo e refrigerante aquecido, e emite 50× a radioatividade dos estoques na atmosfera (`fission.rs:500-512, 607-629`, Java: `FissionReactorMultiblockData.java:343-372`).
- **Interação e Menus (S1):** Clique direito na carcaça nunca consome o uso; portas abrem menu quando formadas e sem agachar. Com estrutura inválida, o clique não abre menu funcional.

---

## 2. Tabela de Correspondência Java vs Port

| Item | Regra / Lógica no Java (arquivo:linha) | Implementação no Port (arquivo:linha) | Estado | Efeito visível para o jogador |
|---|---|---|---|---|
| **Dimensões mín/máx** | Min 3x3x3, Max 18x18x18 (`CuboidStructureValidator.java:34`) | `(3..=18).contains(s)` (`fission.rs:92`) | **OK** | Reator forma entre 3x4x3 e 18x18x18. |
| **Arestas e moldura** | Apenas Casing em quinas e arestas (`CuboidStructureValidator.java:115, 150`) | Arestas com `edges > 1` exigem Casing (`fission.rs:115, 117`) | **OK** | Portas e adaptadores em quinas impedem formação. |
| **Paredes e Vidro** | Casing, Port, Adapter e Reactor Glass aceitos (`TileEntityReactorGlass.java:16`) | `Part::Glass` aceito em `(1, Some(Part::Port \| Part::Adapter \| Part::Glass))` (`fission.rs:116`) | **CORRIGIDO** | Janelas de Reactor Glass agora formam o reator normalmente. |
| **Assemblies internas** | Colunas contíguas de combustível encimadas por 1 barra de controle (`FissionReactorValidator.java:124-149`) | Validação estrita de colunas e barras de controle (`fission.rs:126-135`) | **OK** | Barra sem combustível ou desalinhada impede formação. |
| **Cálculo de superfície** | 6 faces por assembly menos 2 por faces compartilhadas (`FissionReactorValidator.java:79-85`) | 6 * fuels - 2 * adjacentes (`fission.rs:137-144`) | **OK** | Eficiência de fervura idêntica ao original. |
| **Portas em estrutura inválida** | Sem estrutura formada, portas não operam nem expõem tanques (`SINTOMAS S3`) | `FlowSide::None`, filtros em `NOTHING`, tanques e menus zerados (`fission.rs:442-458, 486-502`) | **CORRIGIDO** | Porta quebrada/isolada não funciona como tanque infinito. |
| **Notificação de formação** | Partículas ou chat ao formar estrutura (`TileEntityMultiblock.java:297-312`, `SINTOMAS S2`) | Mensagem aos jogadores a até 40 blocos (`fission.rs:409-421`) | **CORRIGIDO** | Jogador recebe feedback de que o reator foi montado. |
| **Queima de combustível residual** | Continua queimando `burnRemaining` com tanque zerado (`FissionReactorMultiblockData.java:474-478`) | Permite queima enquanto `has_fuel` ou `burn_remaining > 0` (`fission.rs:273-286`) | **CORRIGIDO** | Frações de combustível não congelam ao acabar o suprimento. |
| **Persistência de taxa zero** | `rateLimit = 0.0` é mantido após recarregar chunk/mundo (`FissionReactorCache.java:21-26`) | `saved.rate_limit >= 0.0` mantido no carregamento (`fission.rs:511`) | **CORRIGIDO** | Reator desligado ou em taxa 0 não reacende em 0.1 após reload. |
| **Validação química da água** | Tanque de água só ferve `minecraft:water` (`FissionReactorMultiblockData.java:153`) | Checa `r.water.chemical == WATER` (`fission.rs:306`) | **CORRIGIDO** | Fluidos espúrios não fervem gerando vapor indevido. |
| **Meltdown e radiação** | Meltdown gera explosão em raio 8, 75% dano pós-meltdown e irradia 50× estoques (`FissionReactorMultiblockData.java:343-372`) | Raio 8, 75% dano, limpeza de combustível/lixo e radiação 50× (`fission.rs:500-512, 607-629`) | **OK** | Reator superaquecido explode e irradia a atmosfera. |
| **Modos de porta** | 3 modos: Input, Output Waste, Output Coolant (`AttributeStateFissionPortMode.java:49-51`) | 3 modos com configurador, nomes e faces correspondentes (`fission.rs:568-588`) | **OK** | Automação e separação de vapor/resíduo funcionam fielmente. |
| **Logic Adapter** | 6 modos com emissão/recepção de sinal redstone (`TileEntityFissionReactorLogicAdapter.java`) | Bloco estrutural passivo por ausência de API de redstone | **FORK** | Requer `FORK-21` para leitura e emissão analógica/digital. |
| **Renderização de fluidos** | Renderização volumétrica 3D do fluido e vapor no interior com vidro (`RenderFissionReactor.java`) | Sem renderização volumétrica no WASM | **FORK** | Requer `FORK-19` / `FORK-31` (`render.multiblock-fluid`). |
| **Uso condicional de bloco** | Clicar com bloco na mão em estrutura inválida coloca o bloco em vez de abrir menu (`SINTOMAS S1`) | Menu estático no registro do bloco | **FORK** | Requer `FORK-29` para controle de clique pelo mod. |

---

## 3. Lista Detalhada de Bugs Corrigidos

### BUG 1: Rejeição de Reactor Glass nas paredes do Reator de Fissão
- **Arquivo / Linha:** `port/mod/src/fission.rs:16, 56-76, 116`
- **Regra Java:** `TileEntityReactorGlass.java:16`, `CuboidStructureValidator.java:115-118`.
- **Sintoma:** Jogador construía o reator de fissão com blocos de `mekanism:reactor_glass` nas paredes externas e a estrutura recusava-se a formar.
- **Correção:** Adicionada a constante `GLASS = "mekanism:reactor_glass"`, a variante `Part::Glass` no enum `Part`, o reconhecimento em `part_of`, e aceitação em `(1, Some(Part::Port | Part::Adapter | Part::Glass))`.

### BUG 2: Congelamento da queima fracionária quando o tanque de combustível zera
- **Arquivo / Linha:** `port/mod/src/fission.rs:273-286`
- **Regra Java:** `FissionReactorMultiblockData.java:474-478`.
- **Sintoma:** Quando o combustível de entrada se esgotava e o tanque atingia 0 mB inteiros, mas ainda restava fração decimal em `burn_remaining` (ex: 0.7 mB), a queima cessava abruptamente. A fração de combustível ficava presa e inerte no reator sem produzir calor nem resíduo até que um novo mB inteiro fosse injetado.
- **Correção:** A condição de queima foi estendida para `(r.fuel.chemical == FISSILE && r.fuel.amount > 0) || r.burn_remaining > 0.0`, consumindo o combustível fracionário residual até a combustão completa.

### BUG 3: Redefinição forçada de Burn Rate = 0.0 para 0.1 no reload do mundo/chunk
- **Arquivo / Linha:** `port/mod/src/fission.rs:427-430, 511`
- **Regra Java:** `FissionReactorCache.java:20-27`.
- **Sintoma:** Jogador configurava a taxa de queima para 0.0 (para suspender a geração sem desligar a estrutura). Ao descarregar a chunk ou reiniciar o servidor, o reator reaparecia queimando a 0.1 mB/t.
- **Correção:** Inicialização unificada de novas estruturas em `DEFAULT_RATE` (0.1), permitindo que `saved.rate_limit >= 0.0` preserve taxas ajustadas para 0.0.

### BUG 4: Fervura de fluido incorreto no tanque de refrigerante líquido
- **Arquivo / Linha:** `port/mod/src/fission.rs:306`
- **Regra Java:** `FissionReactorMultiblockData.java:153-154`.
- **Sintoma:** O tanque de água apenas verificava `r.water.amount > 0` sem checar se a substância química era `minecraft:water`.
- **Correção:** Condição ajustada para `if r.water.amount > 0 && (r.water.chemical == WATER || r.water.chemical.is_empty())`.

### BUG 5 (Sintoma S3): Portas de reatores inválidos/quebrados continuavam funcionando como tanques autônomos
- **Arquivo / Linha:** `port/mod/src/fission.rs:442-458, 486-502`
- **Regra Java:** `common/tile/prefab/TileEntityMultiblock.java:218-229`, `SINTOMAS_MULTIBLOCOS.md S3`.
- **Sintoma:** Ao quebrar o reator de fissão deixando apenas uma porta, a porta continuava aceitando e entregando fluidos e gases com capacidade gigante (`SPEC_CAPACITY`), funcionando como um supertanque isolado.
- **Correção:** Ao detectar estrutura inválida em `fission_structures` ou porta inválida em `run_fission`, todas as faces são fechadas (`FlowSide::None`), filtros de tanques são fixados em `NOTHING`, capacidade aparente é zerada e menu é desativado.

### BUG 6 (Sintoma S2): Ausência de notificação visual/sonora ao formar o reator
- **Arquivo / Linha:** `port/mod/src/fission.rs:408-421`
- **Regra Java:** `TileEntityMultiblock.java:297-312`, `SINTOMAS_MULTIBLOCOS.md S2`.
- **Sintoma:** Ao completar a montagem do reator de fissão, não havia nenhum feedback no cliente avisando que a estrutura foi validada com sucesso.
- **Correção:** Ao detectar a transição de inválido para válido, os jogadores em um raio de 40 blocos na dimensão recebem notificação de formação de multibloco.
