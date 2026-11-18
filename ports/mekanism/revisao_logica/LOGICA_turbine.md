# Relatório de Revisão de Lógica: Industrial Turbine (Turbina Industrial)

**Identificador da Tarefa:** `T20261109-220423-01q`  
**Escopo:** `port/mod/src/turbine.rs`  
**Referência Java:** Mekanism 1.21.1-10.7.19.85 (`TurbineValidator.java`, `TurbineMultiblockData.java`, `TileEntityTurbineRotor.java`, `TileEntityTurbineValve.java`, `TileEntityTurbineVent.java`, `TileEntityTurbineCasing.java`, `TurbineCache.java`, `CuboidStructureValidator.java`, `StructureHelper.java`, `FormationProtocol.java`, `MultiblockCache.java`, `BlockTurbineRotor.java`)

---

## 1. Tabela Comparativa de Comportamento e Lógica

| Item / Regra | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Efeito Visível para o Jogador |
|---|---|---|---|---|
| **1.1 Dimensões Mínimas/Máximas** | `TurbineValidator.java:37` (`5, 3, 5` a `17, 18, 17`) | `turbine.rs:85` (`5..=17` em X e Z; `3..=18` em Y) | **OK** | Impede turbinas menores que 5x5 de base ou maiores que 17x17, e altura além de 18. |
| **1.2 Paridade das Dimensões Horizontais** | `TurbineValidator.java:64-66` (`length % 2 != 1 \|\| width % 2 != 1`) | `turbine.rs:87` (`size[0] % 2 == 0 \|\| size[2] % 2 == 0`) | **OK** | Comprimento (X) e largura (Z) devem ser ímpares. Turbinas com dimensões pares na base não formam. |
| **1.3 Arestas e Quinas da Carcaça (`WallRelative.EDGE`, `CORNER`)** | `CuboidStructureValidator.java:115` (`needsFrame && !type.isFrame()`), `TurbineValidator.java:43-50` (só `TURBINE_CASING` é `FRAME`) | `turbine.rs:126` (`(2 \| 3, Some(Part::Casing)) => {}`) | **CORRIGIDO** | No Java, as 12 arestas e 8 quinas da moldura exigem estritamente `Turbine Casing`. O port aceitava `structural_glass` nas arestas. Corrigido para rejeitar vidro na moldura. |
| **1.4 Vidro Estrutural nas Faces Planas (`WallRelative.SIDE`)** | `CuboidStructureValidator.java:103-107, 152` (`isFrameCompatible`, `needsFrame == false`) | `turbine.rs:55, 124` (`(1, Some(Part::Casing \| Part::Glass \| Part::Valve)) => {}`) | **CORRIGIDO** | Vidro estrutural agora é aceito exclusivamente nas faces planas da carcaça (paredes, piso e teto), preservando o comportamento original do Java. |
| **1.5 Posição e Contagem de Válvulas** | `TurbineValidator.java:45-46` (`CasingType.VALVE`), `CuboidStructureValidator.java:136` | `turbine.rs:124` (`(1, Some(Part::Valve))`) | **OK** | Válvulas são permitidas em qualquer face plana (paredes, piso, teto); proibidas em arestas, quinas ou interior. |
| **1.6 Posição e Altura Mínima dos Vents (Respiradouros)** | `TurbineValidator.java:186-194` (`coord.getY() < complex.getY()` falha; `ventData.isEmpty()` falha) | `turbine.rs:125, 176` (`v[1] < complex[1]`, `vents.is_empty()`) | **OK** | Pelo menos 1 vent é obrigatório. Vents devem ficar no nível do complexo rotacional ou acima dele. |
| **1.7 Complexo Rotacional Único e Central** | `TurbineValidator.java:80-84, 100` (`pos.getX() != centerX \|\| pos.getZ() != centerZ`) | `turbine.rs:104-109, 132` (`x != cx \|\| z != cz`, `complex.is_some()`) | **OK** | Exatamente um complexo rotacional, estritamente no centro horizontal e no interior da turbina. |
| **1.8 Rotores Contíguos sob o Complexo** | `TurbineValidator.java:85-89, 153-167` (coluna contígua de `minPos.getY() + 1` até `complex.getY() - 1`) | `turbine.rs:110-115, 149-158` (`rotors.remove(&[cx, y, cz])`, `!rotors.is_empty()`) | **OK** | Rotores devem formar uma coluna vertical contígua sob o complexo rotacional até o piso. |
| **1.9 Exigência Mínima de 1 Rotor sob o Complexo** | `TurbineValidator.java:104, 153-170` (`turbineHeight` descende do complexo; 0 rotores implica `blades == 0`, falha com `TURBINE_INVALID_NO_BLADES`) | `turbine.rs:156` (`if height == 0 \|\| !rotors.is_empty() { return bad; }`) | **CORRIGIDO** | O port aceitava turbina com 0 rotores (`height == 0`) se o complexo estivesse encostado no piso. Corrigido: turbina exige pelo menos 1 rotor. |
| **1.10 Proporção Rotor vs Largura da Base** | `TurbineValidator.java:105-108` (`innerRadius < rotors / 4`) | `turbine.rs:134` (`(size[0].min(size[2]) - 3) / 2 < rotor_count / 4`) | **OK** | Turbinas estreitas não aceitam rotores excessivamente altos (ex: base 5x5 aceita até 4 rotores). |
| **1.11 Plano de Dispersores de Pressão** | `TurbineValidator.java:120-139` (preenche todo o plano horizontal em `complex.getY()` exceto o centro) | `turbine.rs:138-145` (`(x, z) != (cx, cz)`, `complex[1]`, sem dispersores em outro local) | **OK** | Exige um plano completo de dispersores no nível do complexo, e proíbe dispersores em qualquer outro nível. |
| **1.12 Bobinas Eletromagnéticas Conectadas** | `TurbineValidator.java:111-113, 176-181` (`FormationProtocol.explore` iniciando em `complex.above()`) | `turbine.rs:134, 159-174` (busca em largura a partir de `[cx, complex[1] + 1, cz]`) | **OK** | Pelo menos 1 bobina necessária; todas as bobinas devem ser contíguas e conectadas à bobina central sobre o complexo. |
| **1.13 Condensadores Saturadores Acima do Complexo** | `TurbineValidator.java:142-146` (`coord.getY() <= complex.getY()` falha) | `turbine.rs:145` (`condensers.iter().any(\|c\| c[1] <= complex[1])`) | **OK** | Condensadores são opcionais; se existirem, devem ficar estritamente acima do plano do complexo. |
| **1.14 Interior Vazio (Air)** | `TurbineValidator.java:54-60`, `CuboidStructureValidator.java:84-90` | `turbine.rs:102, 347` (`inside_empty(p)` via `world::get_block`) | **OK** | Qualquer bloco não pertencente às peças internas permitidas impede a formação da estrutura. |
| **1.15 Adição e Remoção de Lâminas (Pás)** | `TileEntityTurbineRotor.java:80-137`, `BlockTurbineRotor.java:44-70` | `turbine.rs:227-248, 511-568` (`add_blade`, `remove_blade`, `blade_requests`) | **OK** | Até 2 lâminas por rotor. Adição preenche de baixo para cima; remoção retira de cima para baixo. |
| **1.16 Consumo de Vapor e Condensação com Energia Truncada a Zero** | `TurbineMultiblockData.java:151-157` (condicionado unicamente a `clientFlow > 0`) | `turbine.rs:307-315` (removido `if generated == 0`) | **CORRIGIDO** | Quando a energia gerada por tick era menor que 1 FE (arredondava a 0), o port congelava o consumo de vapor e a geração de água. Corrigido para consumir vapor e gerar água fielmente. |
| **1.17 Fórmula de Fluxo Máximo e Multiplicador de Energia** | `TurbineMultiblockData.java:141-147` | `turbine.rs:251-256` (`max_flow`, `energy_multiplier`) | **OK** | `max_flow` limitado por dispersores e vents. Multiplicador de energia limitado pelo teto de 4 lâminas por bobina. |
| **1.18 Capacidades: Vapor, Água e Energia** | `TurbineMultiblockData.java:234-248` (Vapor: `lowerVolume * 64k`; Água: `condensers * 64k`; Energia: `volume * 16MJ = 6.4M FE`) | `turbine.rs:257-269` (`steam_capacity`, `water_capacity`, `energy_capacity`) | **OK** | Idêntico ao Java original (capacidades escalam com volume inferior, condensadores e volume total). |
| **1.19 Persistência de Dados do Multibloco em Rotores** | `TurbineCache.java:21-30`, `MultiblockCache.java:43-56` | `turbine.rs:502-506` (`update.data` para rotor agora usa `keep`) | **CORRIGIDO** | Se os rotores tinham coordenadas menores que válvulas/vents (`group[0]`), a energia salva no mundo ficava congelada no valor da formação. Corrigido. |
| **1.20 Reinicialização de Espelhos ao Desmontar (`t.started`)** | `TurbineCache.java:15-30`, `MultiblockData.java` | `turbine.rs:350-357, 402-407` (`t.started = false; t.steam_given.clear()...`) | **CORRIGIDO** | Quebrar um bloco da turbina não limpava os espelhos de válvulas/vents nem resetava `started`. Corrigido para reinicializar limpo na reforma. |
| **1.21 Nível de Comparador Analógico Redstone** | `TurbineMultiblockData.java:251-253` (`MekanismUtils.redstoneLevelFromContents`) | `turbine.rs:273-281` (`pub fn redstone_level`) | **CORRIGIDO** | Implementada função oficial que calcula o sinal analógico de redstone (0 quando vazio, 15 quando cheio, escala de 1 a 14). |
| **1.22 Modos de Despejo de Vapor (`dumpMode`)** | `TurbineMultiblockData.java:69, 168-179` (`IDLE`, `DUMPING_EXCESS`, `DUMPING`) | Não suportado na interface simplificada do Horizon | **FORK** | Requer botões e alternância de modo na interface gráfica do Horizon (FORK-20). |
| **1.23 Renderizador Animado e Áudio do Rotor** | `RenderTurbineRotor.java`, `TileEntityTurbineRotor.java:29-32` | Não disponível no ambiente do servidor WASM | **FORK** | Puramente cosmético; sem impacto mecânico na simulação física (FORK-10). |

---

## 2. Detalhamento dos Bugs Encontrados e Corrigidos

### Bug 1: Vidro Estrutural nas Arestas e Quinas da Moldura
- **Ocorrência:** `port/mod/src/turbine.rs:52, 122`
- **Causa Raiz:** O port agrupava `CASING` e `"mekanism:structural_glass"` no mesmo enum `Part::Casing`, e aceitava `Part::Casing` em qualquer contagem de arestas (`edges >= 0`). No Java (`CuboidStructureValidator.java:115`), posições de quina e aresta (`relative.isOnEdge()`) exigem `needsFrame == true`, e `TurbineValidator.getCasingType()` retorna `INVALID` para vidro estrutural (apenas `TURBINE_CASING` é `FRAME`).
- **Sintoma em Jogo:** O jogador podia construir a moldura estrutural (quinas e arestas) da turbina com Vidro Estrutural em vez de Turbina Casing, e a estrutura formava incorretamente.
- **Correção Aplicada:** Criado `Part::Glass` no enum `Part`. O validador agora só aceita `Part::Glass` em faces planas (`edges == 1`). Arestas e quinas (`edges == 2 || edges == 3`) aceitam estritamente `Part::Casing`.

### Bug 2: Travamento de Operação quando a Energia Gerada Trunca a Zero
- **Ocorrência:** `port/mod/src/turbine.rs:298-300`
- **Causa Raiz:** O port continha a seguinte cláusula:
  ```rust
  let generated = (mult * rate * 0.4).max(0.0) as u64;
  if generated == 0 {
      return Tick::default();
  }
  ```
  No Java (`TurbineMultiblockData.java:151-157`), a execução do ciclo de vapor/água depende unicamente de `clientFlow > 0`. A inserção de energia insere o que couber/truncar (`energyContainer.insert(...)`), mas o tanque de vapor é contraído (`chemicalTank.shrinkStack(clientFlow)`) e a água é condensada (`ventTank.insert(...)`) independentemente de a energia truncar para 0.
- **Sintoma em Jogo:** Em turbinas com poucas pás, pouco vapor ou vazão fracionária (onde `mult * rate * 0.4 < 1.0` FE), a turbina travava completamente, sem consumir vapor nem gerar água condensada nos respiradouros.
- **Correção Aplicada:** Removida a interrupção prematura `if generated == 0`. Quando `flow > 0`, o vapor é consumido e a água é condensada normalmente, creditando `generated.min(needed)` de energia.

### Bug 3: Formação Inválida com 0 Rotores sob o Complexo Rotacional
- **Ocorrência:** `port/mod/src/turbine.rs:147-156`
- **Causa Raiz:** Quando o complexo rotacional era colocado diretamente sobre o piso (`complex[1] == min[1] + 1`), o laço `for y in (min[1] + 1..complex[1]).rev()` iterava zero vezes, deixando `height = 0`. O código verificava apenas se `rotors` estava vazio, mas não verificava se `height > 0`.
- **Sintoma em Jogo:** Uma turbina sem rotores formava no jogo com `lower_volume = 0` e `steam_capacity = 0`. No Java, isso falha com `TURBINE_INVALID_NO_BLADES` (`TurbineValidator.java:169`).
- **Correção Aplicada:** Adicionada verificação explícita `if height == 0 || !rotors.is_empty() { return bad; }`. Toda turbina industrial válida deve ter pelo menos 1 rotor sob o complexo.

### Bug 4: Persistência Desatualizada de Energia se o Primeiro Membro For Rotor
- **Ocorrência:** `port/mod/src/turbine.rs:478-482`
- **Causa Raiz:** Em `run_turbine`, a cada 20 ticks o buffer de energia era serializado em `data` via `keep = encode(&found, energy)`. No entanto, o código continha uma exceção para rotores: `else { update.data = e.data.clone(); }`. Como a restauração da turbina após reinicialização do servidor lê a energia do primeiro membro da lista ordenada por coordenadas (`group[0].data`), caso os rotores centrais tivessem coordenadas menores que as válvulas/vents (por exemplo, válvulas colocadas nas faces leste ou sul), `group[0]` era um rotor, cujo campo `data` nunca era atualizado.
- **Sintoma em Jogo:** Ao reiniciar o mundo ou descarregar/recarregar chunks, a turbina perdia a energia acumulada e voltava ao valor do momento da formação inicial.
- **Correção Aplicada:** Removida a sobreposição espúria `update.data = e.data.clone();` para rotores, garantindo que todos os blocos ativos mantenham o estado serializado atualizado em `data`.

### Bug 5: Desassociação e Acúmulo de Estado de Espelhos ao Desmontar
- **Ocorrência:** `port/mod/src/turbine.rs:349, 391`
- **Causa Raiz:** `t.started` era definido como `true` no primeiro tick e nunca mais voltava a `false`. Caso a turbina fosse desmontada (quebrando um bloco de carcaça) e reconstruída com modificações nas válvulas ou respiradouros, os mapas diferenciais de espelhos (`steam_given`, `water_given`, `energy_given`) retinham coordenadas antigas de blocos removidos.
- **Sintoma em Jogo:** Alterações na tubulação ou relocalização de válvulas com a turbina desmontada provocavam divergência ou congelamento nos tanques espelhados após reformar.
- **Correção Aplicada:** Em `turbine_structures`, quando `!found.valid`, o estado `t.started` é resetado para `false` e os mapas `steam_given`, `water_given` e `energy_given` são limpos, garantindo ressincronização correta na próxima formação.

---

## 3. Conformidade dos Testes Unitários

O arquivo `port/mod/src/turbine.rs` conta com 9 testes unitários (5 originais preservados + 4 novos testes cobrindo as correções de lógica):

1. `a_complex_with_rotors_coils_and_a_plane_is_a_turbine`: Testa formação padrão 5x8x5, volumes e componentes.
2. `it_needs_everything_in_its_place`: Valida ausência de casing, falta de bobinas, vent incorreto, buraco em rotores, condensador sob complexo, rotor desalinhado e bounds inválidos.
3. `blades_go_where_the_original_puts_them`: Valida algoritmo recursivo de adição e remoção de pás.
4. `steam_turns_the_blades_into_energy_and_water`: Valida conversão de vapor, geração de energia e condensação de água.
5. `a_member_remembers_its_turbine`: Valida codificação e decodificação binária de `Found` e buffer de energia.
6. `structural_glass_allowed_on_faces_but_forbidden_on_edges_and_corners`: Valida que Vidro Estrutural é aceito em faces planas (`edges == 1`), mas rejeitado em arestas (`edges == 2`) e quinas (`edges == 3`).
7. `turbine_requires_at_least_one_rotor_under_the_complex`: Valida que turbinas sem rotores sob o complexo rotacional são rejeitadas.
8. `turbine_consumes_steam_and_makes_water_even_when_energy_rounds_to_zero`: Valida consumo contínuo de vapor e condensação de água mesmo quando a energia gerada trunca a 0 FE.
9. `redstone_comparator_level_matches_java_formula`: Valida o cálculo analógico do nível de sinal de comparador redstone (0 a 15).
