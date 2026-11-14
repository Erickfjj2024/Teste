# Resumo da Revisão de Lógica: Industrial Turbine (Turbina Industrial)

**Identificador da Tarefa:** `T20261109-220423-01q`  
**Escopo:** `port/mod/src/turbine.rs`  
**Referência Java:** Mekanism 1.21.1-10.7.19.85 (`TurbineValidator.java`, `TurbineMultiblockData.java`, `TileEntityTurbineRotor.java`, `TileEntityTurbineValve.java`, `TileEntityTurbineVent.java`, `TileEntityTurbineCasing.java`, `TurbineCache.java`, `CuboidStructureValidator.java`, `StructureHelper.java`, `FormationProtocol.java`, `MultiblockCache.java`, `BlockTurbineRotor.java`, `MekanismUtils.java`)

---

## 1. Visão Geral da Auditoria de Lógica

Nesta auditoria do multibloco **Industrial Turbine** (`port/mod/src/turbine.rs`), foi realizada a reconstrução do zero das regras do Mekanism Java 1.21.1 e sua comparação detalhada contra a implementação no port WASM/Horizon.

Foram identificados e corrigidos **5 bugs conceituais e comportamentais de formação, execução e persistência**, além da adição da função oficial de cálculo de sinal de **comparador Redstone**.

---

## 2. Bugs Encontrados e Corrigidos

1. **Vidro Estrutural Aceito nas Arestas e Quinas da Moldura (`Part::Glass` vs `Part::Casing`):**
   - **Java:** `CuboidStructureValidator.java:115` e `TurbineValidator.java:41-50`. As 12 arestas e 8 quinas da casca externa (`WallRelative.EDGE`, `CORNER`) exigem `needsFrame == true`, permitindo unicamente blocos cujo tipo seja `CasingType.FRAME` (`Turbine Casing`). `Structural Glass` retorna `INVALID` e é aceito estritamente nas faces planas (`WallRelative.SIDE`).
   - **Port:** Mapeava `structural_glass` para `Part::Casing` e aceitava `Part::Casing` em qualquer contagem de arestas (`edges >= 0`), permitindo que toda a moldura da turbina fosse feita de vidro.
   - **Correção:** Criada a variante `Part::Glass` no enum `Part`. `Part::Glass` agora é aceito estritamente em faces planas (`edges == 1`). Arestas e quinas (`edges == 2 \|\| edges == 3`) aceitam apenas `Part::Casing`.
   - **Sintoma corrigido:** A turbina não mais se forma se houver vidro estrutural nas quinas ou arestas.

2. **Travamento de Fluxo de Vapor e Condensação com Energia Fracionária/Truncada a Zero:**
   - **Java:** `TurbineMultiblockData.java:151-157`. O ciclo de consumo de vapor e condensação de água ocorre sempre que `clientFlow > 0`. A energia inserida é o que couber/truncar (`energyContainer.insert(...)`), não interrompendo a contração do tanque nem a geração de água.
   - **Port:** Continha `if generated == 0 { return Tick::default(); }`, interrompendo o ciclo e congelando a turbina sem consumir vapor nem produzir água caso a energia gerada em um tick fosse menor que 1 FE (0.4 FE/J).
   - **Correção:** Removida a verificação indevida `if generated == 0`. O fluxo de vapor é consumido e a água condensada é gerada normalmente, atribuindo a energia calculada `generated.min(needed)`.
   - **Sintoma corrigido:** Turbinas com baixa vazão ou poucas pás não mais congelam quando a geração por tick é inferior a 1 FE.

3. **Formação Indevida de Turbina com 0 Rotores:**
   - **Java:** `TurbineValidator.java:153-170`. A contagem de rotores descende do complexo até a base. Se o complexo estiver encostado no piso, 0 rotores são contabilizados, resultando em 0 pás e falha de formação (`TURBINE_INVALID_NO_BLADES`).
   - **Port:** O laço de rotores executava 0 vezes e não havia verificação de que `height > 0`, formando uma turbina sem rotores com volume inferior zerado.
   - **Correção:** Adicionado requisito `if height == 0 \|\| !rotors.is_empty() { return bad; }`.
   - **Sintoma corrigido:** Impede que turbinas sem rotores sob o complexo se formem.

4. **Persistência de Energia Desatualizada quando o Primeiro Membro é um Rotor:**
   - **Java:** `TurbineCache.java:21-30`, `MultiblockCache.java:43-56`. O estado do multibloco é preservado no cache e restaurado fielmente.
   - **Port:** Em `run_turbine`, para blocos de rotor, o código sobrescrevia `update.data = e.data.clone();`, impedindo que os rotores recebessem a serialização atualizada da energia (`keep = encode(&found, energy)`). Quando as válvulas eram construídas em coordenadas maiores que a coluna de rotores (faces leste/sul), o primeiro membro do grupo (`group[0]`) era um rotor, cujo campo `data` nunca era atualizado. Ao reiniciar o mundo, o buffer de energia era restaurado com o valor desatualizado da formação inicial.
   - **Correção:** Removida a sobreposição em rotores, garantindo que `update.data` receba `keep`.
   - **Sintoma corrigido:** Energia armazenada não é mais perdida ou revertida para valores antigos ao reiniciar o servidor ou recarregar chunks.

5. **Falta de Reinicialização de Espelhos de Válvulas/Vents ao Desmontar (`t.started`):**
   - **Java:** `TurbineCache.java`, `MultiblockData.java`.
   - **Port:** Ao quebrar um bloco da turbina, `t.started` permanecia `true` e os mapas `steam_given`, `water_given` e `energy_given` não eram limpos. Caso o jogador adicionasse, movesse ou removesse válvulas com a turbina desmontada, a estrutura mantinha registros obsoletos, provocando cálculos diferenciais corrompidos na remontagem.
   - **Correção:** Em `turbine_structures`, quando `!found.valid`, define `t.started = false` e limpa `t.steam_given`, `t.water_given` e `t.energy_given`. Na próxima formação, o estado é reinicializado a partir do tanque real.
   - **Sintoma corrigido:** Válvulas e respiradouros manipulados durante desmontagens agora sincronizam perfeitamente na reforma.

6. **Implementação do Sinal Analógico de Comparador Redstone:**
   - **Java:** `TurbineMultiblockData.java:251-253` (`MekanismUtils.redstoneLevelFromContents`).
   - **Port:** Ausente. Implementada a função pública `pub fn redstone_level(steam: u64, steam_cap: u64) -> u8`.

---

## 3. Testes Unitários

Lista completa dos testes em `port/mod/src/turbine.rs` (5 existentes preservados + 4 novos adicionados):
1. `a_complex_with_rotors_coils_and_a_plane_is_a_turbine`
2. `it_needs_everything_in_its_place`
3. `blades_go_where_the_original_puts_them`
4. `steam_turns_the_blades_into_energy_and_water`
5. `a_member_remembers_its_turbine`
6. `structural_glass_allowed_on_faces_but_forbidden_on_edges_and_corners` (NOVO)
7. `turbine_requires_at_least_one_rotor_under_the_complex` (NOVO)
8. `turbine_consumes_steam_and_makes_water_even_when_energy_rounds_to_zero` (NOVO)
9. `redstone_comparator_level_matches_java_formula` (NOVO)

- **Total de testes no arquivo:** 9
- **Execução na VM:** não medido (a VM não possui toolchain Rust configurada, em conformidade com as regras da tarefa).
- **Prova de mutação:** a prova de mutação está pendente do juiz.

---

## 4. Análise de Riscos de Compilação

Todos os trechos editados em `port/mod/src/turbine.rs` foram inspecionados cuidadosamente para conformidade de tipos, sintaxe Rust e sistema de empréstimos:
1. `Part::Glass` deriva `Clone, Copy, Debug, PartialEq, Eq`, idêntico às demais variantes de `Part`.
2. O padrão `(1, Some(Part::Casing | Part::Glass | Part::Valve))` e `(2 | 3, Some(Part::Casing))` utiliza sintaxe válida de subpadrões or em tuplas do Rust 2021.
3. A verificação `height == 0 || !rotors.is_empty()` opera sobre variável `height: i32` e `HashSet<[i32; 3]>`.
4. `redstone_level` realiza coerções explícitas seguras (`as f64`, `as u8`) e não gera divisão por zero (`steam_cap == 0` tratado).
5. `turbine_structures` e `run_turbine` utilizam métodos existentes de `HashMap` (`get_mut`, `.clear()`) e preservam a assinatura de `BlockEntityUpdate`.

---

## 5. Arquivos Gerados e Modificados

- Modificado:
  - `port/mod/src/turbine.rs`
- Gerados:
  - `.saida/T20261109-220423-01q/LOGICA_turbine.md`
  - `.saida/T20261109-220423-01q/FORK_turbine.md`
  - `.saida/T20261109-220423-01q/RESUMO.md`
