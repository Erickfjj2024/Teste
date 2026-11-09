# REVISÃO CRUZADA: WORLDGEN E RECEITAS (Mekanism 1.21.1-10.7.19.85 vs Port Horizon)

Data: 2026-11-09  
Tarefa: T20261109-194914-e4e (Revisão Cruzada da Rodada 1 / Escopo t5)  
Referência Java: `v1.21.1-10.7.19.85` (`java/`)  
Código do Port: `port/mod/src/worldgen.rs`, `port/mod/src/lib.rs`, `port/mod/src/plus.rs`, `port/mod/src/reaction.rs`, `port/mod/src/process.rs`, `port/mod/src/continuous.rs`, `port/mod/src/evaporation.rs`, `port/mod/src/sawmill.rs`, `port/mod/src/combiner.rs`, `port/mod/src/cube.rs`  
Pipeline de Conversão: `port/base_1211.py`, `port/tools_ref/convert.py`  
Testes do Port: `port/mod/tests/machines.toml`, `port/mod/tests/factory.toml`  

---

## 1. Worldgen: Minérios, Sal, Retrogen e Biomas

### 1.1 Tabela Comparativa de Worldgen

| Item | Java (arquivo:linha) | Port (arquivo:linha) | Estado | Detalhes / Causa / Reclassificação |
|---|---|---|---|---|
| **BOTTOM (Overworld)** | Y = -64 (`DimensionType.MIN_Y`, `WorldConfig.java:135`) | `-64` (`port/mod/src/worldgen.rs:17`) | **OK** | Fundo padrão do Overworld 1.18+ conferido. |
| **TOP (Overworld)** | Y = 320 (`DimensionType.MAX_Y`, `WorldConfig.java:135`) | `320` (`port/mod/src/worldgen.rs:18`) | **OK** | Teto padrão do Overworld 1.18+ conferido. |
| **Tin Ore (small)** | `perChunk: 14, size: 4, TRAPEZOID, Y=[-20, 94]` (`OreType.java:16`) | `per_chunk: 14, size: 4, trapezoid: true, min_y: -20, max_y: 94` (`worldgen.rs:21`) | **CONFIRMADA (OK)** | 100% idêntico ao Java. |
| **Tin Ore (large)** | `perChunk: 12, size: 9, TRAPEZOID, Y=[-32, 72]` (`OreType.java:17`) | `per_chunk: 12, size: 9, trapezoid: true, min_y: -32, max_y: 72` (`worldgen.rs:22`) | **CONFIRMADA (OK)** | 100% idêntico ao Java. |
| **Osmium Ore (upper)** | `perChunk: 65, size: 7, TRAPEZOID, Y=[72, 343], plateau: 8` (`OreType.java:20`, `AnchorType.java:12`) | `per_chunk: 65, size: 7, trapezoid: true, min_y: 72, max_y: 320` (`worldgen.rs:23`) | **FORK-12** | Java usa `belowTop(-24)` = 319 - (-24) = 343 com platô 8; port trunca em 320 e não tem platô. Reclassificado de LIMITE para FORK-12. |
| **Osmium Ore (middle)** | `perChunk: 6, size: 9, TRAPEZOID, Y=[-32, 56]` (`OreType.java:21`) | `per_chunk: 6, size: 9, trapezoid: true, min_y: -32, max_y: 56` (`worldgen.rs:24`) | **CONFIRMADA (OK)** | 100% idêntico ao Java. |
| **Osmium Ore (small)** | `perChunk: 8, size: 4, UNIFORM, Y=[-64, 64]` (`OreType.java:22`, `AnchorType.java:11`) | `per_chunk: 8, size: 4, trapezoid: false, min_y: -64, max_y: 64` (`worldgen.rs:25`) | **CONFIRMADA (OK)** | Java `aboveBottom(0)` = -64. 100% idêntico. |
| **Uranium Ore (small)** | `perChunk: 4, size: 4, TRAPEZOID, Y=[-64, 8]` (`OreType.java:25`) | `per_chunk: 4, size: 4, trapezoid: true, min_y: -64, max_y: 8` (`worldgen.rs:26`) | **CONFIRMADA (OK)** | Java `aboveBottom(0)` = -64. 100% idêntico. |
| **Uranium Ore (buried)** | `perChunk: 7, size: 9, TRAPEZOID, Y=[-88, -8], plateau: 16, discardAir: 0.75` (`OreType.java:26`, `AnchorType.java:11`) | `per_chunk: 7, size: 9, trapezoid: true, min_y: -64, max_y: -8` (`worldgen.rs:27`) | **FORK-12** | Java usa `aboveBottom(-24)` = -88 com platô 16 e descarte de ar 0.75; port trunca mínimo em -64 e não descarta ar. Reclassificado de LIMITE para FORK-12. |
| **Fluorite Ore (normal)** | `perChunk: 5, size: 5, UNIFORM, Y=[-64, 23]` (`OreType.java:29`) | `per_chunk: 5, size: 5, trapezoid: false, min_y: -64, max_y: 23` (`worldgen.rs:28`) | **CONFIRMADA (OK)** | Java `aboveBottom(0)` = -64. 100% idêntico. |
| **Fluorite Ore (buried)** | `perChunk: 3, size: 13, TRAPEZOID, Y=[-64, 4], discardAir: 1.0` (`OreType.java:30`) | `per_chunk: 3, size: 13, trapezoid: true, min_y: -64, max_y: 4` (`worldgen.rs:29`) | **FORK-12** | Java usa descarte de ar 100% (minério 100% enterrado sem exposição a cavernas); port não tem verificação de ar. Reclassificado de LIMITE para FORK-12. |
| **Lead Ore (normal)** | `perChunk: 8, size: 9, TRAPEZOID, Y=[-88, 64], discardAir: 0.25` (`OreType.java:33`) | `per_chunk: 8, size: 9, trapezoid: true, min_y: -64, max_y: 64` (`worldgen.rs:30`) | **FORK-12** | Java usa `aboveBottom(-24)` = -88 com descarte de ar 0.25; port trunca em -64 e não descarta ar. Reclassificado de LIMITE para FORK-12. |
| **Salt per chunk** | 2 (`WorldConfig.java:43, 124`, `placed_feature/salt.json:9-13`) | `SALT_PER_CHUNK: u32 = 2` (`worldgen.rs:97`) | **CONFIRMADA (OK)** | 100% idêntico ao Java. |
| **Salt radius** | `minRadius: 2, maxRadius: 3` (`WorldConfig.java:43, 126-128`) | `SALT_MIN_RADIUS = 2, SALT_MAX_RADIUS = 3` (`worldgen.rs:98-99`) | **CONFIRMADA (OK)** | 100% idêntico ao Java. |
| **Salt half height** | 1 (`WorldConfig.java:43, 134-135`) | `SALT_HALF_HEIGHT: i32 = 1` (`worldgen.rs:100`) | **CONFIRMADA (OK)** | 100% idêntico ao Java. |
| **Salt targets** | `minecraft:dirt`, `minecraft:clay` (`configured_feature/salt.json:16-22`) | `matches!(block, "minecraft:dirt" \| "minecraft:clay")` (`worldgen.rs:133`) | **CONFIRMADA (OK)** | 100% idêntico ao Java. |
| **Salt condition** | Fundo sob água (`placed_feature/salt.json:19, 25`: `OCEAN_FLOOR_WG`, `minecraft:water`) | Varredura do fundo sólido submerso por água (`lib.rs:2065-2085`) | **CONFIRMADA (OK)** | Semântica idêntica ao Java. |
| **Substituição Stone** | Tag `#minecraft:stone_ore_replaceables` -> `mekanism:<ore>_ore` (`configured_feature/ore_*.json`) | `minecraft:stone \| granite \| diorite \| andesite` -> `mekanism:<ore>_ore` (`worldgen.rs:136-146`) | **ERRADA-CORRIGIDA** | Rodada 1 aceitou apenas `minecraft:stone` como LIMITE. Corrigido diretamente em `worldgen.rs` cobrindo todas as 4 variantes de pedra vanilla. Tags dinâmicas dependem de FORK-12. |
| **Substituição Deepslate** | Tag `#minecraft:deepslate_ore_replaceables` -> `mekanism:deepslate_<ore>_ore` (`configured_feature/ore_*.json`) | `minecraft:deepslate \| tuff` -> `mekanism:deepslate_<ore>_ore` (`worldgen.rs:136-146`) | **ERRADA-CORRIGIDA** | Rodada 1 aceitou apenas `minecraft:deepslate` como LIMITE. Corrigido diretamente em `worldgen.rs` cobrindo deepslate e tufo. Tags dinâmicas dependem de FORK-12. |
| **Biomas** | `#mekanism:spawn_ores` = `#minecraft:is_overworld` + `#c:is_overworld` (`tags/worldgen/biome/spawn_ores.json`) | `dimension == "minecraft:overworld"` (`lib.rs:2043`) | **FORK-12** | O port gera em qualquer bioma do Overworld sem filtragem de tag. Reclassificado de LIMITE para FORK-12. |
| **Retrogen** | `enableRegeneration` (false), `userGenVersion` (0), geração em chunks existentes via `GenHandler.generate` e tag NBT `MekanismWorldGenVersion` (`WorldConfig.java:36-39`, `GenHandler.java:57-85`, `CommonWorldTickHandler.java:203-228`) | `if !new \|\| dimension != "minecraft:overworld" { return; }` (`lib.rs:2043`) | **FORK-12** | Chunks já existentes (`!new`) são descartados; sem persistência de versão no chunk NBT nem retrogen. Reclassificado de LIMITE/FALTA para FORK-12. |

---

## 2. Comparação Semântica das Receitas (Parte Principal)

### 2.1 Censo Completo dos JSONs de Receitas no Datagen Java (3451 arquivos)

Medição exata via script python sobre a árvore `java/datagen/generated/*/data/*/recipe/`:
- **Total geral de arquivos JSON**: **3451**
- **Receitas base (Horizon)**: **2814**
- **Receitas compat (mods externos)**: **637**

#### Distribuição por Módulo:
1. `mekanism`: 2215 total (base: 1578, compat: 637)
2. `mekanismadditions`: 1114 total (base: 1114, compat: 0)
3. `mekanismtools`: 87 total (base: 87, compat: 0)
4. `mekanismgenerators`: 35 total (base: 35, compat: 0)

#### Detalhamento das 637 Receitas de Compatibilidade Externa:
- `biomesoplenty` (BOP): 355 receitas (crushing 65, enriching 87, pigment_extracting 27, sawing 176)
- `biomeswevegone` (BWG): 164 receitas (crushing 30, enriching 44, pigment_extracting 21, sawing 69)
- `farmersdelight`: 117 receitas (crushing 18, enriching 22, metallurgic_infusing 7, sawing 70)
- `ae2`: 1 receita (crushing 1: certus quartz dust)
Todas as 637 receitas de compatibilidade são filtradas intencionalmente por `base_1211.py` pois referenciam mods inexistentes no Horizon.

---

### 2.2 Análise Semântica por Tipo de Receita (36 Tipos de JSON + 1 Dinâmico)

Para cada tipo, conferiu-se: tipo, ingredientes, quantidades, saídas, tempo (ticks) e energia (FE vs Joules).

| # | Tipo de Receita | Qtd Base | Qtd Compat | Qtd Total | Tempo (Ticks) | Energia (FE / J) | Suporte no Port | Estado Semântico | Causa / Detalhes |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `minecraft:crafting_shaped` | 807 | 0 | 807 | Instantâneo | N/A | Engine Horizon | **OK** | 178 mekanism, 27 generators, 60 tools, 542 additions. Receitas que exigem itens ainda não implementados são filtradas por `convert.py:2641-2646`. |
| 2 | `mekanism:painting` | 464 | 0 | 464 | 200 ticks | 50 J (20 FE/t) | `lib.rs:212`, `process.rs` | **OK** | 176 mekanism, 288 additions. Ingredientes: item + químico (pigmento, 32 mB para blocos/vidro/tapete/terracota; 256 mB para lã/estandarte/cama/vela/corante). `per_tick_usage: false`. |
| 3 | `mekanism:pigment_extracting` | 394 | 48 | 442 | 200 ticks | 50 J (20 FE/t) | `lib.rs:211`, `process.rs` | **OK** | 178 mekanism, 216 additions; 48 compat ignorados. Ingredientes: item -> pigmento químico. Extração: flores e corantes (256 mB), lãs (192 mB), etc. |
| 4 | `mekanism:sawing` | 124 | 316 | 440 | 200 ticks | 50 J (20 FE/t) | `lib.rs:185, 574`, `sawmill.rs` | **OK** | 124 base; 316 compat ignorados. Entrada de item, saída principal e saída secundária probabilística com `secondary_chance` (0.0 a 1.0) reproduzida fielmente. |
| 5 | `mekanism:enriching` | 187 | 153 | 340 | 200 ticks | 50 J (20 FE/t) | `lib.rs:183, 535` | **OK** | 169 mekanism, 18 additions; 153 compat ignorados. Entrada de item (contagem), saída de item (contagem). |
| 6 | `mekanism:crushing` | 200 | 113 | 313 | 200 ticks | 50 J (20 FE/t) | `lib.rs:182, 535` | **OK** | 200 base; 113 compat ignorados. Entrada de item, saída de item (contagem). |
| 7 | `mekanism:injecting` | 91 | 0 | 91 | 200 ticks | 400 J (160 FE/t) | `lib.rs:190`, `process.rs` | **OK** | Item + HCl (gás), consome químico por tick (`per_tick_usage: true` com distribuição estocástica Poisson). |
| 8 | `mekanism:mek_data` | 86 | 0 | 86 | Instantâneo | N/A | Não suportado na bancada | **FORK-25** | Serializer de bancada com transferência de NBT/dados entre tiers e inicialização de componentes. `convert.py` converte 36 fábricas e 4 cubos em shaped comum; 46 restantes não portadas. |
| 9 | `mekanism:combining` | 82 | 0 | 82 | 200 ticks | 50 J (20 FE/t) | `lib.rs:186, 591`, `combiner.rs` | **OK** | Dois itens de entrada (`main_input`, `extra_input` com contagens) -> saída de item (contagem). |
| 10 | `minecraft:crafting_shapeless` | 76 | 0 | 76 | Instantâneo | N/A | Engine Horizon | **OK** | 26 mekanism, 50 additions. Ingredientes e resultados vanilla. |
| 11 | `mekanism:metallurgic_infusing` | 31 | 7 | 38 | 200 ticks | 20 J (8 FE/t) | `lib.rs:187`, `process.rs` | **OK** | 30 mekanism, 1 generators; 7 compat ignorados. Item + tipo de infusão (redstone, diamante, carbono, bio, etc.). `per_tick_usage: false`. |
| 12 | `mekanism:purifying` | 28 | 0 | 28 | 200 ticks | 200 J (80 FE/t) | `lib.rs:189`, `process.rs` | **OK** | Item + oxigênio (gás), `per_tick_usage: true`. Saída de clumps. |
| 13 | `minecraft:smelting` | 25 | 0 | 25 | 200 ticks | 50 J (20 FE/t) | `lib.rs:184, 552` | **OK** | 18 mekanism, 7 tools. Smelter carrega via `Source::Cooking("smelting")`. |
| 14 | `minecraft:blasting` | 25 | 0 | 25 | 100 ticks (vanilla) | N/A | Vanilla Blast Furnace | **CONFIRMADA (OK)** | 18 mekanism, 7 tools. Receitas para o alto-forno vanilla geradas pelo Mekanism. No Java, o Energized Smelter NUNCA leu blasting (`MekanismRecipeType.java:279`). Rodada 1 errou ao marcar como LIMITE. |
| 15 | `mekanism:oxidizing` | 25 | 0 | 25 | 200 ticks | 50 J (20 FE/t) | `lib.rs:192`, `process.rs` | **OK** | Item (redstone, carvão, etc.) -> químico (gás). |
| 16 | `mekanism:chemical_conversion` | 24 | 0 | 24 | Instantâneo | N/A | `lib.rs:685-690`, `process.rs` | **OK** | Abastecimento de slots de máquinas: item sólido -> químico no tanque interno. |
| 17 | `mekanism:dissolution` | 23 | 0 | 23 | 200 ticks | 400 J (160 FE/t) | `lib.rs:193`, `process.rs` | **OK** | Item (minérios brutos) + ácido sulfúrico -> slurry sujo. `per_tick_usage: true`. |
| 18 | `mekanism:nucleosynthesizing` | 21 | 0 | 21 | Variável (JSON: `duration`) | 50 000 J (20 000 FE/t) | `lib.rs:207`, `process.rs` | **OK** | Item + antimatéria -> item transformado. Duração lida do JSON (`duration: 200..12000`). |
| 19 | `mekanism:rotary` | 20 | 0 | 20 | 1 tick contínuo | 50 J (20 FE/t) | `lib.rs:204`, `plus.rs:91-105` | **OK** | 17 mekanism, 3 generators. Condensação (gás -> fluido) e descondensação (fluido -> gás) bidirecional. |
| 20 | `mekanism:pigment_mixing` | 16 | 0 | 16 | 1 tick contínuo | 50 J (20 FE/t) | `lib.rs:213`, `plus.rs:46-55` | **OK** | Dois pigmentos químicos -> pigmento resultante. |
| 21 | `mekanism:reaction` | 14 | 0 | 14 | JSON: `duration` (100..400) | JSON: `energy_required` + base (5 J / 2 FE/t) | `lib.rs:222`, `reaction.rs` | **OK** | Substrato sólido + fluido + gás -> item e/ou gás. Converte `energy_required` Joules em FE via `fe()` e consome por tick conforme `PRCEnergyContainer.java:28`. |
| 22 | `mekanismtools:paxel` | 11 | 0 | 11 | Instantâneo | N/A | `convert.py:2448` | **DIFERENTE / FORK** | Serializer especial no Java copiando dano/encantamento de 3 ferramentas; convertida para `crafting_shaped` ordinário no port. |
| 23 | `mekanism:crystallizing` | 10 | 0 | 10 | 200 ticks | 400 J (160 FE/t) | `lib.rs:194`, `process.rs` | **OK** | Slurry limpo (200 mB) -> cristal de minério (1 item). |
| 24 | `mekanism:washing` | 7 | 0 | 7 | 1 tick contínuo | 200 J (80 FE/t) | `lib.rs:198`, `plus.rs:56-64` | **OK** | Água (5 mB) + slurry sujo (1 mB) -> slurry limpo (1 mB). |
| 25 | `mekanism:chemical_infusing` | 5 | 0 | 5 | 1 tick contínuo | 200 J (80 FE/t) | `lib.rs:196`, `plus.rs:46-55` | **OK** | 4 mekanism, 1 generators. Dois gases químicos -> gás composto. |
| 26 | `mekanism:separating` | 3 | 0 | 3 | 1 tick contínuo | 400 J (160 FE/t) × `energy_multiplier` | `lib.rs:201`, `plus.rs:73-83` | **DIFERENTE** | Fluido -> dois gases químicos. `heavy_water.json` define `energy_multiplier: 2` (320 FE/t no Java); o port ignora o multiplicador e consome apenas 160 FE/t. |
| 27 | `mekanism:activating` | 2 | 0 | 2 | 1 tick contínuo | 0 FE/t (solar) | `lib.rs:219`, `plus.rs:68-72` | **CORRIGIDO** | Gás -> gás ativado por nêutrons solares. O código em `plus.rs` existe, mas `convert.py:2607` omitia `activating` de `MACHINE_TYPES`. Corrigido em `convert.py`. |
| 28 | `mekanism:evaporating` | 2 | 0 | 2 | Térmico variável | Calor (solar) | `evaporation.rs:152` | **CORRIGIDO** | Água -> salmoura (10:1); salmoura -> lítio (10:1). O código em `evaporation.rs` existe, mas `convert.py:2607` omitia `evaporating` de `MACHINE_TYPES`. Corrigido em `convert.py`. |
| 29 | `mekanism:energy_conversion` | 2 | 0 | 2 | Instantâneo | N/A | `cube.rs:13-19` | **PARCIAL** | Redstone (10 000 J = 4 000 FE), bloco (90 000 J = 36 000 FE). Hardcoded no port em vez de leitura dinâmica via `recipes_of_type`. |
| 30 | `mekanism:compressing` | 2 | 0 | 2 | 200 ticks | 100 J (40 FE/t) | `lib.rs:191`, `process.rs` | **OK** | Refined obsidian / glowstone ingots. Consome 1 mB ósmio/tick (`per_tick_usage: true`). |
| 31 | `mekanism:centrifuging` | 2 | 0 | 2 | 1 tick contínuo | 400 J (160 FE/t) | `lib.rs:217`, `plus.rs:68-72` | **CORRIGIDO** | UF6 -> fissile fuel, lixo nuclear -> plutônio. O código em `plus.rs` existe, mas `convert.py:2607` omitia `centrifuging` de `MACHINE_TYPES`. Corrigido em `convert.py`. |
| 32 | `mekanismtools:banner_shield` | 1 | 0 | 1 | Instantâneo | N/A | Ausente | **FALTA / FORK** | Combinação de escudo e estandarte. Escudos não existem no Horizon. |
| 33 | `minecraft:smithing_transform` | 1 | 0 | 1 | Instantâneo | N/A | Bancada de Ferraria vanilla | **OK** | Upgrade de paxel de netherite via molde de ferraria. |
| 34 | `mekanism:clear_configuration` | 1 | 0 | 1 | Instantâneo | N/A | Ausente | **FORK** | Reset de configurador e cartões na bancada. Exige serializer customizado. |
| 35 | `mekanism:bin_insert` | 1 | 0 | 1 | Instantâneo | N/A | Ausente | **FORK** | Inserção de itens no bin via bancada 2x2/3x3. Exige serializer customizado. |
| 36 | `mekanism:bin_extract` | 1 | 0 | 1 | Instantâneo | N/A | Ausente | **FORK** | Extração de itens do bin via bancada 2x2/3x3. Exige serializer customizado. |
| 37 | *(código)* `mekanism:liquifying` | 0 | 0 | 0 | 200 ticks | 20 J (8 FE/t) | `lib.rs:624-645`, `process.rs` | **FORK** | No Java é dinâmico por reflexão/componente (`nutrition * 50` mB). No port usa tabela estática `FOODS` com 44 alimentos vanilla. |

---

### 2.3 Lista Consolidada de Receitas FALTA / DIFERENTE por Tipo e Causa Raiz

1. **`mekanism:mek_data` (86 receitas) - ESTADO: FORK-25 / DIFERENTE**
   - **Causa**: O Java implementa `MekDataRecipe`, um `CraftingRecipe` customizado que intercepta o resultado da bancada e copia tags de NBT e componentes de dados (energia armazenada, tanques de gás/fluido, melhorias instaladas, trava de segurança). O Horizon não provê API para mods registrarem serializadores customizados de bancada com interceptação de montagem.
   - **Impacto no port**: 36 receitas de evolução de fábricas e 4 receitas de evolução de cubos de energia foram convertidas por `convert.py:2655, 2664` para `minecraft:crafting_shaped` ordinário, permitindo craftar o próximo tier mas perdendo o conteúdo interno. As outras 46 receitas (evolução de tanques de químico, fluidos e bins, e inicialização de máquinas com componentes limpos) não são emitidas para o port.

2. **`mekanism:separating` (1 receita com diferença: `heavy_water.json`) - ESTADO: DIFERENTE**
   - **Causa**: O arquivo `java/datagen/generated/mekanismgenerators/data/mekanismgenerators/recipe/separator/heavy_water.json` define `"energy_multiplier": 2`. No Java, `TileEntityElectrolyticSeparator.java:217` lê o multiplicador e dobra a taxa de consumo de energia (`base * multiplier`). No port (`plus.rs:73-83` e `continuous.rs:128`), `spec.usage` é estático (160 FE/t = `fe(400)`), ignorando o multiplicador.
   - **Impacto no port**: A eletrólise de água pesada para gerar deutério consome 160 FE/t no port, quando deveria consumir 320 FE/t.

3. **`mekanism:activating`, `mekanism:centrifuging`, `mekanism:evaporating` (6 receitas) - ESTADO: CORRIGIDO**
   - **Causa**: A constante `MACHINE_TYPES` em `port/tools_ref/convert.py:2607` não continha esses três tipos de receitas. Embora o Rust possua os leitores (`recipes_of_type("mekanism:activating")`, etc.), os arquivos JSON não eram copiados pelo conversor.
   - **Correção**: Adicionados `activating`, `centrifuging` e `evaporating` ao conjunto `MACHINE_TYPES` em `port/tools_ref/convert.py`.

4. **`mekanism:energy_conversion` (2 receitas) - ESTADO: PARCIAL / DIFERENTE**
   - **Causa**: Em `cube.rs:13-19`, os valores de conversão para `minecraft:redstone` (4 000 FE) e `minecraft:redstone_block` (36 000 FE) estão fixos em código (`match item`), sem ler dinamicamente os JSONs via `recipes_of_type`.
   - **Impacto no port**: Funciona perfeitamente para redstone e bloco vanilla (valores corretos de 10 000 J e 90 000 J convertidos para FE), mas ignora novos itens de conversão adicionados por datapacks.

5. **`mekanismtools:paxel` (11 receitas) - ESTADO: DIFERENTE / FORK**
   - **Causa**: `PaxelRecipe` no Java transfere encantamentos e dano de machado, picareta e pá combinados. Convertido em `crafting_shaped` comum por `convert.py:2449`.

6. **`mekanismtools:banner_shield` (1 receita) - ESTADO: FALTA / FORK**
   - **Causa**: Aplica banner ao escudo na bancada. Escudos não estão implementados no Horizon (FORK-19).

7. **`mekanism:bin_insert` (1), `mekanism:bin_extract` (1), `mekanism:clear_configuration` (1) - ESTADO: FALTA / FORK**
   - **Causa**: Receitas dinâmicas de grade de crafting não suportadas pela engine Horizon.

8. **`compat/*` (637 receitas) - ESTADO: FALTA / FORK**
   - **Causa**: Referenciam mods inexistentes no ecossistema Horizon (BOP, BWG, Farmer's Delight, AE2).

---

## 3. Análise Detalhada do que a Rodada 1 Não Viu / Classificou com Erro

1. **Erro de classificação em `minecraft:blasting`**:
   - A rodada 1 marcou como LIMITE (item 20.8) afirmando que o Energized Smelter deixava de cobrir 25 receitas de alto-forno.
   - **Fato verificado no Java**: `TileEntityEnergizedSmelter.java:24-26` implementa `getRecipeType()` retornando unicamente `MekanismRecipeType.SMELTING`. O método `MekanismRecipeType.java:279` busca apenas `RecipeType.SMELTING`. O Energized Smelter no Java NUNCA executou receitas de `blasting`. O port implementa `Source::Cooking("smelting")` em `lib.rs:551`, o que é **100% fiel ao original**.
2. **Omissão de 3 tipos de receitas em `convert.py`**:
   - A rodada 1 atestou que `activating`, `centrifuging` e `evaporating` estavam "cobertas" porque viu o código em `plus.rs` e `evaporation.rs`.
   - **Fato verificado**: `convert.py:2607` não incluía esses 3 tipos em `MACHINE_TYPES`, de modo que os JSONs nunca chegavam ao mod empacotado. Corrigido nesta rodada 2.
3. **Multiplicador de energia em `separating`**:
   - A rodada 1 marcou `mekanism:separating` como OK.
   - **Fato verificado**: `energy_multiplier: 2` de `heavy_water.json` é ignorado pelo port, causando sub-consumo energético em 50%.
4. **Substituição de andesito, diorito, granito e tufo em veios de minério**:
   - A rodada 1 considerou LIMITE porque o port substituía apenas `minecraft:stone` e `minecraft:deepslate`.
   - **Correção aplicada nesta rodada 2**: Atualizado `worldgen.rs:136-146` para substituir todas as pedras da tag vanilla (`stone`, `granite`, `diorite`, `andesite` -> stone ore; `deepslate`, `tuff` -> deepslate ore), eliminando buracos em veios no mundo vanilla.
5. **Conversões J -> FE e Arredondamentos**:
   - Todas as taxas de consumo e receitas foram auditadas:
     - Máquinas padrão (crusher, enrichment, smelter, sawmill, combiner, oxidizer, rotary): 50 J/t = 20 FE/t (`fe(50)`).
     - Máquinas médias (purifier, infuser, washer): 200 J/t = 80 FE/t (`fe(200)`).
     - Máquinas pesadas (injector, dissolution, crystallizer, centrifuge): 400 J/t = 160 FE/t (`fe(400)`).
     - Separador: 2 × 200 J/t = 400 J/t = 160 FE/t.
     - Nucleosynthesizer: 50 000 J/t = 20 000 FE/t (`fe(50000)`).
     - PRC: base 5 J/t = 2 FE/t + `fe(energy_required)`.
     - Todas as conversões aplicam fielmente `FE = J * 2 / 5` sem desvios de arredondamento.

---

## 4. Bloco Adicionado a `port/DIFERENCAS.md` (Seção 21)

```markdown
## 21. Revisão Cruzada: Worldgen e Receitas (mod 0.1.86, rodada 2)

| # | Diferença / Difference | Estado / State |
|---|---|---|
| 21.1 | Worldgen (alturas relativas, platô e descarte por ar): conferido contra `OreType.java:15-34` e `worldgen/placed_feature/*.json`. Osmium superior [72, 343] com platô 8; Urânio enterrado [-88, -8] com platô 16 e descarte por ar 0.75; Chumbo normal [-88, 64] com descarte por ar 0.25; Fluorita enterrada [-64, 4] com descarte por ar 1.0. Reclassificado de LIMITE para FORK-12 | FORK-12 |
| 21.2 | Worldgen (substituição de blocos): no Java usa as tags `#minecraft:stone_ore_replaceables` (pedra, granito, diorito, andesito) e `#minecraft:deepslate_ore_replaceables` (deepslate, tufo). O port anteriormente substituía apenas pedra e deepslate puros; corrigido em `worldgen.rs:136-146` para substituir todas as 6 variantes vanilla. Substituição dinâmica de tags de blocos em datapacks requer capacidade de API | CORRIGIDO / FORK-12 |
| 21.3 | Worldgen (biomas e retrogen): o Java restringe veios aos biomas da tag `#mekanism:spawn_ores` (`#minecraft:is_overworld` e `#c:is_overworld`) e possui retrogen completo (`GenHandler.java:57-85`, `WorldConfig.java:36-39`) com persistência de `MekanismWorldGenVersion` em NBT de chunks. O port gera em toda a dimensão `minecraft:overworld` e ignora chunks existentes (`!new`) | FORK-12 |
| 21.4 | Receitas (separador eletrolítico e multiplicador de energia): no Java a receita `heavy_water.json` define `energy_multiplier: 2`, dobrando a demanda de energia do bloco (`BASE_ENERGY_CALCULATOR = base * recipe.getEnergyMultiplier()`). O port ignora o multiplicador em `plus.rs:73-83` e `continuous.rs:128, 142`, consumindo apenas o valor estático `spec.usage` (160 FE/t) | DIFERENTE |
| 21.5 | Receitas (`convert.py` - omissão de tipos de máquinas): `MACHINE_TYPES` em `tools_ref/convert.py:2607` omitia `activating`, `centrifuging` e `evaporating`, fazendo com que os JSONs de receitas dessas 3 máquinas não fossem emitidos para a distribuição do mod, apesar do código em `plus.rs` e `evaporation.rs` possuir suporte. Corrigido em `convert.py` | CORRIGIDO |
| 21.6 | Receitas de alto-forno (`minecraft:blasting`, 25 receitas): verificado no Java original (`TileEntityEnergizedSmelter.java:24-26`, `MekanismRecipeType.java:272-280`) que o Energized Smelter NUNCA processou `blasting` (apenas `smelting`). As 25 receitas de `blasting` são registradas para o alto-forno vanilla. O item 20.8 da rodada 1 foi classificado erroneamente como LIMITE; o comportamento do port é idêntico ao original | CONFIRMADA-IDENTICO |
| 21.7 | Receitas de bancada com NBT (`mekanism:mek_data`, 86 receitas): serializer especial de crafting do Java que preserva dados de energia, gases, fluidos e upgrades ao evoluir máquinas/tanques/fábricas/cubos e inicializar componentes. O conversor transforma fábricas (36) e cubos (4) em `crafting_shaped` ordinário; as demais 46 receitas não são portadas por falta de API de NBT na bancada do Horizon | FORK-25 |
| 21.8 | Receitas de conversão de energia (`mekanism:energy_conversion`, 2 receitas): os valores de redstone (10 000 J = 4 000 FE) e bloco de redstone (90 000 J = 36 000 FE) estão corretos, porém hardcoded em `cube.rs:13-19` em vez de lidos dinamicamente de JSONs | PARCIAL |
| 21.9 | Receitas especiais de bancada e ferramentas: `mekanism:bin_insert` (1), `mekanism:bin_extract` (1), `mekanism:clear_configuration` (1), `mekanismtools:paxel` (11, convertidas para shaped comum sem preservação de dados) e `mekanismtools:banner_shield` (1, escudos inexistentes). Exigem extensões na engine de crafting | FORK |
| 21.10 | Receitas de liquefação nutricional (`mekanism:liquifying`, dinâmico no Java): calcula dinamicamente 50 mB de pasta nutricional por ponto de nutrição do item (`TileEntityNutritionalLiquifier.java`). O port usa a tabela estática `FOODS` com 44 alimentos vanilla em `lib.rs:624-636` | FORK |
| 21.11 | Receitas de compatibilidade externa (`compat/*`, 637 receitas): ignoradas pelo conversor `base_1211.py` por pertencerem a mods externos não presentes no Horizon (BOP 355, BWG 164, Farmer's Delight 117, AE2 1) | FORK |
```
