# Resumo da Revisão Cruzada: Transmissores e Armazenamento (Tarefa T20261109-195239-ljc)

## 1. Escopo da Revisão Cruzada (Rodada 2)
Foi realizada a conferência independente do zero contra o código Java oficial (**Mekanism 1.21.1-10.7.19.85**) para todos os 7 arquivos do escopo da t7:
- `port/mod/src/transmitter.rs`
- `port/mod/src/induction.rs`
- `port/mod/src/bin.rs`
- `port/mod/src/cube.rs`
- `port/mod/src/continuous.rs`
- `port/mod/src/sides.rs`
- `port/mod/src/bounding.rs`

Componentes conferidos contra o Java:
- **Universal Cables**: capacidade por tier (Basic, Advanced, Elite, Ultimate) em Joules → FE; sem pull rate (rede contínua).
- **Pressurized Tubes**: capacidade interna por tier e taxas de extração/pull em mB.
- **Mechanical Pipes**: capacidade interna por tier e taxas de extração/pull em mB.
- **Logistical Transporters**: taxas de pull (throughput), velocidades (speed/delay), cores do configurador e variantes (Restrictive, Diversion).
- **Thermodynamic Conductors**: condução inversa, capacidade calorífica e isolamento térmico por tier.
- **Bins**: capacidades por tier (Basic, Advanced, Elite, Ultimate, Creative), cadência e face de auto-ejeção (10 ticks, face DOWN) e comportamento de slots (output 0, input 1).
- **Energy Cubes**: capacidades e taxas de saída por tier em Joules → FE; slot de descarga e conversão energética de redstone; face frontal padrão (FRONT_OUT_EJECT).
- **Induction Matrix**: capacidades das células por tier, taxas dos provedores por tier, validação cubóide (3 a 18 por lado), casca com Casing/Port/Structural Glass e modo da porta.
- **Máquinas Contínuas**: tanques e capacidades do Separador Eletrolítico, Lavadora Química, Infusor Químico, Condensentrador Rotativo, Misturador de Pigmentos, Centrífuga Isotópica e Ativador de Nêutrons Solares.
- **Bounding Blocks**: offsets de blocos subordinados do Advanced Solar Generator (10 blocos), Wind Generator (4 blocos), Pigment Mixer (1 bloco), Isotopic Centrifuge (1 bloco), Solar Neutron Activator (1 bloco), Digital Miner (17 blocos), Seismic Vibrator (1 bloco) e Security Desk (1 bloco).

---

## 2. Resultados da Conferência das Correções da Rodada 1

1. **Conversão de Redstone no Energy Cube (`port/mod/src/cube.rs`)**:
   - **Resultado**: **CONFIRMADA**.
   - **Origem Java**: `java/datagen/generated/mekanism/data/mekanism/recipe/energy_conversion/redstone.json:7` concede 10.000 J; `redstone_block.json:7` concede 90.000 J.
   - **Port**: Com a taxa oficial de conversão energética `FE = J * 2 / 5` (`feConversionRate = 2.5`), os valores convertidos são exatamente 4.000 FE para `minecraft:redstone` e 36.000 FE para `minecraft:redstone_block`. A correção da rodada 1 foi validada e confirmada.

2. **Paleta de 18 Cores do Logistical Transporter (`port/mod/src/transmitter.rs`)**:
   - **Resultado**: **ERRADA-CORRIGIDA**.
   - **Origem Java**: `java/api/java/mekanism/api/text/EnumColor.java:28-46` e `java/datagen/generated/mekanism/assets/mekanism/lang/en_us.json:666-683`. Os ordinais 1, 2 e 3 de `EnumColor` possuem `englishName` e tradução oficiais `"Blue"`, `"Green"` e `"Cyan"`.
   - **Divergência da Rodada 1**: A rodada 1 expandiu o array para 19 estados (None + 18), mas registrou `"Dark Blue"`, `"Dark Green"` e `"Dark Aqua"` no código em `transmitter.rs:80-83`, embora no texto do relatório anterior tivesse citado `"Blue"`, `"Green"` e `"Cyan"`.
   - **Correção aplicada na Rodada 2**: Array `COLORS` em `port/mod/src/transmitter.rs:80-83` atualizado para conter os nomes corretos do Java: `"None"`, `"Black"`, `"Blue"`, `"Green"`, `"Cyan"`, `"Dark Red"`, `"Purple"`, `"Orange"`, `"Light Gray"`, `"Gray"`, `"Light Blue"`, `"Lime"`, `"Aqua"`, `"Red"`, `"Magenta"`, `"Yellow"`, `"White"`, `"Brown"`, `"Pink"`.
   - **Teste atualizado**: Teste `the_connections_cycle_and_the_state_round_trips` em `port/mod/src/transmitter.rs:178` atualizado com asserções para o tamanho de `COLORS` e as entradas corrigidas.

---

## 3. Novas Divergências Encontradas e Correções da Rodada 2

1. **Offsets de Bounding Blocks de Seismic Vibrator e Security Desk (`port/mod/src/bounding.rs`)**:
   - **Origem Java**: `MekanismBlockTypes.java:464` (`SEISMIC_VIBRATOR`) e `MekanismBlockTypes.java:527` (`SECURITY_DESK`) utilizam `AttributeHasBounding.ABOVE_ONLY`, exigindo o bloco imediatamente acima (`[0, 1, 0]`).
   - **Port**: `bounding.rs:19` listava apenas `pigment_mixer`, `isotopic_centrifuge` e `solar_neutron_activator`.
   - **Correção**: Adicionados `"mekanism:seismic_vibrator"` e `"mekanism:security_desk"` ao padrão `ABOVE_ONLY` em `port/mod/src/bounding.rs:18-23`.
   - **Teste atualizado**: Teste `big_machines_take_the_blocks_their_models_fill` em `port/mod/src/bounding.rs:137` atualizado com verificação de `seismic_vibrator`.

2. **Restrição de Arestas da Induction Matrix e Vidro Estrutural (`port/mod/src/induction.rs`)**:
   - **Origem Java**: `CuboidStructureValidator.java:115, 149` e `MatrixValidator.java:26-34`. No Java, blocos nas 12 arestas/moldura da carcaça exigem `needsFrame == true`, permitindo apenas `INDUCTION_CASING` (`CasingType.FRAME`). Vidro estrutural (`Structural Glass`) não é aceito nas arestas, apenas nas paredes/faces planas (`edges == 1`).
   - **Port**: `induction.rs:52` mapeia `CASING` e `"mekanism:structural_glass"` indistintamente para `Part::Casing`, aceitando vidro estrutural também nas arestas da casca. Catalogado como FORK/SUPOSIÇÃO (item 21.7.6).

3. **Reatividade a Redstone em Transmissores**:
   - **Origem Java**: `Transmitter.java:421`. Quando `redstoneReactive` está ativo e o bloco recebe sinal de redstone, `canConnect` retorna `false`, desconectando a rede.
   - **Port**: O estado é alternado e serializado em bytes, mas não afeta a conectividade nativa da rede do Horizon por ausência de hook na API. Catalogado como FORK (item 21.7.5).

4. **Bins - Extração por Ataque (Punch) e Lock Frontal**:
   - **Origem Java**: `BlockBin.java:38-71, 88`. Ataque simples retira 1 item, sneak-ataque retira 1 stack; shift-clique com mão vazia na frente bloqueia/desbloqueia o item.
   - **Port**: A API do Horizon WASM não possui evento de ataque a blocos (extração ocorre via GUI/automação), e o bloqueio é operado por botão na interface gráfica. Catalogado como FORK (item 21.7.7).

---

## 4. Testes do Escopo e Contagens Reais

Nomes e contagens de testes extraídos diretamente dos arquivos do escopo:
- `port/mod/src/transmitter.rs`: 1 teste
  - `the_connections_cycle_and_the_state_round_trips` (linha 178) — **editado** (asserções de `COLORS`)
- `port/mod/src/induction.rs`: 3 testes
  - `a_matrix_adds_up_its_cells_and_providers` (linha 289)
  - `it_needs_a_cell_and_a_closed_shell` (linha 295)
  - `the_energy_follows_the_ports_and_is_kept_in_their_bytes` (linha 311)
- `port/mod/src/bin.rs`: 4 testes
  - `items_put_in_are_counted_and_the_window_shows_a_stack` (linha 234)
  - `what_leaves_the_window_leaves_the_bin` (linha 248)
  - `tools_show_one_at_a_time` (linha 264)
  - `a_bin_remembers_what_it_holds` (linha 272)
- `port/mod/src/cube.rs`: 6 testes
  - `redstone_gives_the_energy_the_recipes_say` (linha 123)
  - `the_front_is_kept_after_the_modes_and_found_in_an_old_cube` (linha 130)
  - `the_window_reads_the_sides_from_the_front` (linha 146)
  - `a_press_changes_the_face_the_side_is` (linha 158)
  - `clear_sets_every_side_alike` (linha 172)
  - `the_screen_numbers_are_seventeen` (linha 181)
- `port/mod/src/continuous.rs`: 6 testes
  - `a_converter_turns_one_chemical_into_another` (linha 186)
  - `two_chemicals_make_a_third_on_either_side` (linha 200)
  - `a_washer_uses_five_of_fluid_for_one_of_slurry` (linha 216)
  - `a_separator_splits_water_into_two_gases` (linha 226)
  - `a_rotary_condensentrator_turns_gas_to_fluid_and_back_by_its_mode` (linha 237)
  - `it_needs_a_recipe_energy_and_room` (linha 260)
- `port/mod/src/sides.rs`: 7 testes
  - `modes_cycle_both_ways` (linha 180)
  - `a_new_cube_outputs_in_front_and_takes_elsewhere` (linha 193)
  - `the_front_looks_at_the_player` (linha 202)
  - `faces_of_the_game_get_their_modes` (linha 211)
  - `a_state_string_gives_its_properties` (linha 219)
  - `without_a_look_the_face_is_the_one_toward_the_player` (linha 228)
  - `a_look_enters_the_face_it_meets` (linha 235)
- `port/mod/src/bounding.rs`: 2 testes
  - `big_machines_take_the_blocks_their_models_fill` (linha 137) — **editado** (asserção de `seismic_vibrator`)
  - `a_bounding_block_remembers_its_machine` (linha 147)

**Total de testes nos 7 arquivos do escopo**: 29 testes.

---

## 5. Medições e Entregáveis

- **Execução de testes na VM**: não medido (a VM não possui toolchain Rust/Cargo instalada, conforme as diretrizes do ambiente).
- **Prova de mutação**: não medido (a tarefa não trouxe lista de mutações; pendente do juiz / orquestrador).
- **Arquivos modificados no port (`port/mod/src/`)**:
  - `port/mod/src/transmitter.rs` (correção da paleta de cores e teste `the_connections_cycle_and_the_state_round_trips`)
  - `port/mod/src/bounding.rs` (adicionados offsets de Seismic Vibrator e Security Desk e teste `big_machines_take_the_blocks_their_models_fill`)
  - `port/DIFERENCAS.md` (adicionado bloco 21.7 com as diferenças que ainda persistem)
- **Arquivos gerados em `.saida/T20261109-195239-ljc/`**:
  - `REVISAO_transmissores_armazenamento.md` (tabela completa de revisão cruzada item a item)
  - `BLOCO_DIFERENCAS_t7_rodada2.md` (bloco da seção 21.7 pronto para documentação)
  - `RESUMO.md` (este documento de entrega)
