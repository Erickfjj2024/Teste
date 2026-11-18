# RESUMO DA EXECUÇÃO - TAREFA T20261109-194914-e4e

## 1. Escopo e Objetivo
Revisão cruzada (rodada 2) de **WORLDGEN** e **RECEITAS** (escopo da t5) do port Horizon contra o Mekanism Java 1.21.1-10.7.19.85:
- Arquivos auditados no port: `port/mod/src/worldgen.rs`, `port/mod/src/lib.rs`, `port/mod/src/plus.rs`, `port/mod/src/reaction.rs`, `port/mod/src/process.rs`, `port/mod/src/continuous.rs`, `port/mod/src/evaporation.rs`, `port/mod/src/sawmill.rs`, `port/mod/src/combiner.rs`, `port/mod/src/cube.rs`, `port/base_1211.py`, `port/tools_ref/convert.py`, `port/mod/tests/machines.toml`, `port/mod/tests/factory.toml`.
- Fontes de verdade no Java: `java/main/java/mekanism/common/resource/ore/OreType.java`, `java/main/java/mekanism/common/resource/ore/BaseOreConfig.java`, `java/main/java/mekanism/common/resource/ore/OreAnchor.java`, `java/main/java/mekanism/common/world/height/AnchorType.java`, `java/main/java/mekanism/common/config/WorldConfig.java`, `java/main/java/mekanism/common/world/GenHandler.java`, `java/main/java/mekanism/common/CommonWorldTickHandler.java`, `java/main/java/mekanism/common/tile/machine/TileEntityEnergizedSmelter.java`, `java/main/java/mekanism/common/recipe/MekanismRecipeType.java`, `java/main/java/mekanism/common/tile/machine/TileEntityElectrolyticSeparator.java`, `java/main/java/mekanism/common/tile/machine/TileEntityPressurizedReactionChamber.java`, JSONs de worldgen em `java/datagen/generated/mekanism/data/mekanism/worldgen/` e todos os 3451 JSONs de receitas em `java/datagen/generated/*/data/*/recipe/`.

---

## 2. Resultados da Revisão Cruzada de Worldgen

### 2.1 Reclassificação de LIMITE para FORK-12 e Verificação dos 10 Minérios
Conferidos contra `OreType.java:15-34` e JSONs de `worldgen/placed_feature/*.json`:
1. **Tin (small)**: `perChunk: 14`, `size: 4`, TRAPEZOID, Y = `[-20, 94]`. Estado: **CONFIRMADA (OK)**.
2. **Tin (large)**: `perChunk: 12`, `size: 9`, TRAPEZOID, Y = `[-32, 72]`. Estado: **CONFIRMADA (OK)**.
3. **Osmium (upper)**: `perChunk: 65`, `size: 7`, TRAPEZOID, Y = `[72, 343]` (`belowTop(-24)` com `genDepth: 384, minGenY: -64`), platô 8. Port truncava em 320 sem platô. Reclassificado: **FORK-12**.
4. **Osmium (middle)**: `perChunk: 6`, `size: 9`, TRAPEZOID, Y = `[-32, 56]`. Estado: **CONFIRMADA (OK)**.
5. **Osmium (small)**: `perChunk: 8`, `size: 4`, UNIFORM, Y = `[-64, 64]` (`aboveBottom(0)`). Estado: **CONFIRMADA (OK)**.
6. **Uranium (small)**: `perChunk: 4`, `size: 4`, TRAPEZOID, Y = `[-64, 8]` (`aboveBottom(0)`). Estado: **CONFIRMADA (OK)**.
7. **Uranium (buried)**: `perChunk: 7`, `size: 9`, TRAPEZOID, Y = `[-88, -8]` (`aboveBottom(-24)` a `aboveBottom(56)`), platô 16, descarte por ar 0.75. Port truncava em -64 sem platô nem descarte de ar. Reclassificado: **FORK-12**.
8. **Fluorite (normal)**: `perChunk: 5`, `size: 5`, UNIFORM, Y = `[-64, 23]` (`aboveBottom(0)`). Estado: **CONFIRMADA (OK)**.
9. **Fluorite (buried)**: `perChunk: 3`, `size: 13`, TRAPEZOID, Y = `[-64, 4]`, descarte por ar 1.0 (minério 100% enterrado sem exposição a cavernas). Port não checava ar. Reclassificado: **FORK-12**.
10. **Lead (normal)**: `perChunk: 8`, `size: 9`, TRAPEZOID, Y = `[-88, 64]` (`aboveBottom(-24)`), descarte por ar 0.25. Port truncava em -64 sem descarte de ar. Reclassificado: **FORK-12**.

### 2.2 Sal (`salt`)
Conferido contra `WorldConfig.java:43, 119` e `placed_feature/salt.json`:
- `SALT_PER_CHUNK`: 2 (`worldgen.rs:97`, Java: 2).
- `SALT_MIN_RADIUS`: 2, `SALT_MAX_RADIUS`: 3 (`worldgen.rs:98-99`, Java: 2 a 3).
- `SALT_HALF_HEIGHT`: 1 (`worldgen.rs:100`, Java: 1).
- Substituição: `minecraft:dirt` e `minecraft:clay` (`worldgen.rs:133`, Java: idêntico).
- Fundo sob água: `lib.rs:2065-2085` checa colisão sólida sob água (`wet`). Estado: **CONFIRMADA (OK)**.

### 2.3 Substituição de Blocos (Stone vs Deepslate Replaceables)
- No Java, `configured_feature/ore_*.json` usa as tags `#minecraft:stone_ore_replaceables` (pedra, granito, diorito, andesito) e `#minecraft:deepslate_ore_replaceables` (deepslate, tufo).
- Na rodada 1, o port substituía apenas `minecraft:stone` e `minecraft:deepslate`, gerando veios com furos ao cruzar outras rochas vanilla.
- **Correção aplicada diretamente em `port/mod/src/worldgen.rs`**: Atualizada a função `replace()` e o teste `ore_replaces_stone_and_deepslate_only` para substituir todas as 6 variantes vanilla (`stone`, `granite`, `diorite`, `andesite` para minério comum; `deepslate`, `tuff` para deepslate ore). Tags customizadas de outros mods/datapacks dependem de suporte a tags em runtime (FORK-12).

### 2.4 Biomas e Retrogen
- **Biomas**: Java restringe veios aos biomas da tag `#mekanism:spawn_ores` (`#minecraft:is_overworld` e `#c:is_overworld`). Port gera em qualquer bioma do Overworld (`dimension == "minecraft:overworld"`). Reclassificado: **FORK-12**.
- **Retrogen**: Java suporta retrogen via `GenHandler.java:57-85` e `WorldConfig.java:36-39`, salvando `MekanismWorldGenVersion` em NBT de chunks. Port ignora chunks existentes (`!new`). Reclassificado: **FORK-12**.

---

## 3. Resultados da Comparação Semântica de Receitas

### 3.1 Contagem Exata Medida por Script nos JSONs de `java/datagen/generated/`
- **Total geral de arquivos JSON de receita**: **3451**
- **Receitas base (não-compat)**: **2814**
- **Receitas compat (mods externos ignorados por `base_1211.py`)**: **637** (BOP 355, BWG 164, Farmer's Delight 117, AE2 1)

### 3.2 O que a Rodada 1 NÃO Viu / Classificou Erroneamente (Descobertas Críticas)
1. **`minecraft:blasting` (25 receitas) - Rodada 1 ERROU ao classificar como LIMITE**:
   - A rodada 1 registrou item 20.8 afirmando que o Energized Smelter não cobria blasting.
   - **Conferência no Java**: `TileEntityEnergizedSmelter.java:24-26` e `MekanismRecipeType.java:272-280` comprovam que o Energized Smelter no Java NUNCA executou `blasting` (somente `smelting`). As 25 receitas de `blasting` geradas pelo datagen são para o alto-forno vanilla. O conversor `convert.py:2626` emite essas 25 receitas para o Horizon, e o smelter carrega `Source::Cooking("smelting")`, que é **100% idêntico ao original**.
2. **`mekanism:activating`, `mekanism:centrifuging`, `mekanism:evaporating` (6 receitas) - Omissão em `convert.py`**:
   - A rodada 1 afirmou que estavam "cobertas" pelo mod, mas não viu que `convert.py:2607` omitia esses 3 tipos em `MACHINE_TYPES`. Nenhum JSON dessas máquinas era emitido para o mod.
   - **Correção aplicada**: Adicionados `f'{NS}:activating'`, `f'{NS}:centrifuging'` e `f'{NS}:evaporating'` ao conjunto `MACHINE_TYPES` em `port/tools_ref/convert.py`.
3. **`mekanism:separating` (`energy_multiplier`) - Diferença de consumo elétrico**:
   - `heavy_water.json` define `energy_multiplier: 2` (consome o dobro de FE/t no Java: `BASE_ENERGY_CALCULATOR = base * multiplier`). O port ignora o multiplicador e consome 160 FE/t estático em vez de 320 FE/t.
4. **`mekanism:mek_data` (86 receitas) - Comportamento de bancada**:
   - A rodada 1 não detalhou que `convert.py` converte 36 fábricas e 4 cubos em `crafting_shaped` comum (perdendo NBT), enquanto descarta as outras 46 receitas (tanques, bins, inicialização de máquinas). Reclassificado como FORK-25.

### 3.3 Lista Consolidada de Receitas FALTA / DIFERENTE por Tipo e Causa Raiz
- `mekanism:mek_data` (86 receitas): **FORK-25 / DIFERENTE** (falta serializer customizado com preservação de NBT na bancada do Horizon).
- `mekanism:separating` (1 receita afetada: `heavy_water`): **DIFERENTE** (ignora `energy_multiplier: 2`).
- `mekanism:activating` (2 receitas), `mekanism:centrifuging` (2 receitas), `mekanism:evaporating` (2 receitas): **CORRIGIDO** (adicionados a `MACHINE_TYPES` em `convert.py`).
- `mekanism:energy_conversion` (2 receitas): **PARCIAL** (valores corretos de 4 000 FE e 36 000 FE hardcoded em `cube.rs:13-19`).
- `mekanismtools:paxel` (11 receitas): **DIFERENTE / FORK** (convertidas para shaped comum sem transferência de encantamento/dano).
- `mekanismtools:banner_shield` (1 receita): **FALTA / FORK** (escudos não implementados no Horizon).
- `mekanism:bin_insert` (1), `mekanism:bin_extract` (1), `mekanism:clear_configuration` (1): **FALTA / FORK** (grid dinâmico de crafting não suportado).
- `compat/*` (637 receitas): **FORK** (dependência de mods externos ausentes).

---

## 4. Testes e Mutações

- **Execução com `cargo test`**: não medido (toolchain rust/cargo não configurado na VM do cshub).
- **Nome do teste editado**:
  - `ore_replaces_stone_and_deepslate_only` em `port/mod/src/worldgen.rs` (linhas 182-192).
- **Validação com script de parsing e simulação**:
  - Script python simulando `replace()` com todas as 6 variantes vanilla (`stone`, `granite`, `diorite`, `andesite`, `deepslate`, `tuff`, `dirt`, `air`): **0 falhas**.
  - Validação de conversão `base_1211.recipe(j)` sobre todos os 3451 arquivos de receita: **3451 convertidas com sucesso, 0 erros, 0 alterações numéricas**.
  - Simulação de matching de `MACHINE_TYPES` atualizado em `convert.py`: **2414 receitas reconhecidas com sucesso**.
- **Prova de mutação**: A prova de mutação está pendente do juiz (a tarefa não forneceu lista de mutações).

---

## 5. Arquivos Editados e Entregáveis

- `port/mod/src/worldgen.rs`: Atualizada a função `replace()` para suportar `stone`, `granite`, `diorite`, `andesite`, `deepslate`, `tuff` e atualizado o teste `ore_replaces_stone_and_deepslate_only`.
- `port/tools_ref/convert.py`: Adicionados `f'{NS}:activating'`, `f'{NS}:centrifuging'` e `f'{NS}:evaporating'` ao conjunto `MACHINE_TYPES` (linha 2607).
- `port/DIFERENCAS.md`: Adicionada a Seção 21 completa com os itens 21.1 a 21.11 no mesmo padrão da Seção 20.
- `.saida/T20261109-194914-e4e/REVISAO_worldgen_receitas.md`: Relatório detalhado com tabelas linha a linha, justificativas semânticas, análise do que a rodada 1 errou/não viu e bloco de diferenças.
- `.saida/T20261109-194914-e4e/RESUMO.md`: Este relatório final.
