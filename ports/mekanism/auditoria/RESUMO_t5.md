# RESUMO DA EXECUÇÃO - TAREFA T20261109-184326-z78

## 1. Escopo e Objetivo
Auditoria de **WORLDGEN** e **RECEITAS** do port Horizon contra o Mekanism Java 1.21.1-10.7.19.85:
- Arquivos auditados no port: `port/mod/src/worldgen.rs`, `port/mod/src/lib.rs`, `port/base_1211.py`.
- Fontes de verdade no Java: `java/datagen/generated/mekanism/data/mekanism/worldgen/` (`configured_feature`, `placed_feature`, `neoforge/biome_modifier`), `java/main/java/mekanism/common/resource/ore/OreType.java`, `java/main/java/mekanism/common/config/WorldConfig.java`, `java/main/java/mekanism/common/world/GenHandler.java`, `java/main/java/mekanism/common/CommonWorldTickHandler.java`, e os JSONs de receitas em `java/datagen/generated/*/data/*/recipe/`.

---

## 2. Resultados da Auditoria de Worldgen

### Minérios (10 Veios)
Conferidos contra `OreType.java:15-34` e `worldgen/placed_feature/*.json`:
1. **Tin (small)**: `per_chunk: 14`, `size: 4`, trapezoidal, Y = `[-20, 94]`. Estado: **OK**.
2. **Tin (large)**: `per_chunk: 12`, `size: 9`, trapezoidal, Y = `[-32, 72]`. Estado: **OK**.
3. **Osmium (upper)**: `per_chunk: 65`, `size: 7`, trapezoidal. Java: Y = `[72, 343]` (`belowTop(-24)`), platô 8. Port (`worldgen.rs:23`): Y = `[72, 320]` (`TOP`), sem platô. Estado: **LIMITE**.
4. **Osmium (middle)**: `per_chunk: 6`, `size: 9`, trapezoidal, Y = `[-32, 56]`. Estado: **OK**.
5. **Osmium (small)**: `per_chunk: 8`, `size: 4`, uniforme, Y = `[-64, 64]`. Estado: **OK**.
6. **Uranium (small)**: `per_chunk: 4`, `size: 4`, trapezoidal, Y = `[-64, 8]`. Estado: **OK**.
7. **Uranium (buried)**: `per_chunk: 7`, `size: 9`, trapezoidal. Java: Y = `[-88, -8]` (`aboveBottom(-24)` a `aboveBottom(56)`), platô 16, descarte por ar 0.75. Port (`worldgen.rs:27`): Y = `[-64, -8]`, sem platô nem descarte de ar. Estado: **LIMITE**.
8. **Fluorite (normal)**: `per_chunk: 5`, `size: 5`, uniforme, Y = `[-64, 23]`. Estado: **OK**.
9. **Fluorite (buried)**: `per_chunk: 3`, `size: 13`, trapezoidal, Y = `[-64, 4]`. Java tem descarte por ar 1.0 (não visível em cavernas). Port: sem descarte por ar. Estado: **LIMITE**.
10. **Lead (normal)**: `per_chunk: 8`, `size: 9`, trapezoidal. Java: Y = `[-88, 64]`, descarte por ar 0.25. Port (`worldgen.rs:30`): Y = `[-64, 64]`, sem descarte por ar. Estado: **LIMITE**.

### Sal (`salt`)
Conferido contra `WorldConfig.java:43` e `worldgen/placed_feature/salt.json`:
- `SALT_PER_CHUNK`: 2 (`worldgen.rs:97`, Java: 2).
- `SALT_MIN_RADIUS`: 2 (`worldgen.rs:98`, Java: 2).
- `SALT_MAX_RADIUS`: 3 (`worldgen.rs:99`, Java: 3).
- `SALT_HALF_HEIGHT`: 1 (`worldgen.rs:100`, Java: 1).
- Substituição: `minecraft:dirt` e `minecraft:clay` (`worldgen.rs:133`, Java: idêntico).
- Condição: Fundo oceânico coberto por água (`lib.rs:2065-2085`). Estado: **OK**.

### Substituição de Minérios (Deepslate vs Stone)
- Port (`worldgen.rs:138-144`): Substitui apenas `minecraft:stone` por `mekanism:<ore>_ore` e `minecraft:deepslate` por `mekanism:deepslate_<ore>_ore`.
- Java: Substitui tags completas `#minecraft:stone_ore_replaceables` (incluindo granito, diorito, andesito) e `#minecraft:deepslate_ore_replaceables` (incluindo tufo). Estado: **LIMITE**.

### Biomas e Retrogen
- **Biomas**: Java vincula os recursos a `#mekanism:spawn_ores` (`#minecraft:is_overworld` e `#c:is_overworld`) via NeoForge biome modifiers. O port checa apenas `dimension == "minecraft:overworld"` (`lib.rs:2043`). Estado: **LIMITE**.
- **Retrogen**: Java suporta retrogen via `WorldConfig.java` (`enableRegeneration`, `userGenVersion`) e `GenHandler.java`. O port descarta chunks existentes (`if !new { return; }` em `lib.rs:2043`). Estado: **LIMITE / FALTA**.

---

## 3. Resultados da Auditoria de Receitas

### Contagem Total Medida por Script nos JSONs de `java/datagen/generated/`
- **Total de arquivos JSON de receita**: **3451**
- **Receitas base (não-compat)**: **2814**
- **Receitas compat (mods externos)**: **637**

#### Distribuição por Módulo:
- `mekanism`: 2215 (base: 1578, compat: 637)
- `mekanismadditions`: 1114 (base: 1114, compat: 0)
- `mekanismtools`: 87 (base: 87, compat: 0)
- `mekanismgenerators`: 35 (base: 35, compat: 0)

#### Contagem Exata por Tipo de Receita:
1. `minecraft:crafting_shaped`: 807 (base: 807, compat: 0) - Engine Horizon
2. `mekanism:painting`: 464 (base: 464, compat: 0) - Coberto (`painting_machine`)
3. `mekanism:pigment_extracting`: 442 (base: 394, compat: 48) - Coberto (`pigment_extractor`)
4. `mekanism:sawing`: 440 (base: 124, compat: 316) - Coberto (`precision_sawmill`, fábricas)
5. `mekanism:enriching`: 340 (base: 187, compat: 153) - Coberto (`enrichment_chamber`, fábricas)
6. `mekanism:crushing`: 313 (base: 200, compat: 113) - Coberto (`crusher`, fábricas)
7. `mekanism:injecting`: 91 (base: 91, compat: 0) - Coberto (`chemical_injection_chamber`, fábricas)
8. `mekanism:mek_data`: 86 (base: 86, compat: 0) - **NÃO COBRE** (crafting com NBT transfer)
9. `mekanism:combining`: 82 (base: 82, compat: 0) - Coberto (`combiner`, fábricas)
10. `minecraft:crafting_shapeless`: 76 (base: 76, compat: 0) - Engine Horizon
11. `mekanism:metallurgic_infusing`: 38 (base: 31, compat: 7) - Coberto (`metallurgic_infuser`, fábricas)
12. `mekanism:purifying`: 28 (base: 28, compat: 0) - Coberto (`purification_chamber`, fábricas)
13. `minecraft:smelting`: 25 (base: 25, compat: 0) - Coberto (`energized_smelter`, fábricas)
14. `minecraft:blasting`: 25 (base: 25, compat: 0) - **NÃO COBRE** no Smelter (só smelting)
15. `mekanism:oxidizing`: 25 (base: 25, compat: 0) - Coberto (`chemical_oxidizer`)
16. `mekanism:chemical_conversion`: 24 (base: 24, compat: 0) - Coberto (slots de carga química)
17. `mekanism:dissolution`: 23 (base: 23, compat: 0) - Coberto (`chemical_dissolution_chamber`)
18. `mekanism:nucleosynthesizing`: 21 (base: 21, compat: 0) - Coberto (`antiprotonic_nucleosynthesizer`)
19. `mekanism:rotary`: 20 (base: 20, compat: 0) - Coberto (`rotary_condensentrator`)
20. `mekanism:pigment_mixing`: 16 (base: 16, compat: 0) - Coberto (`pigment_mixer`)
21. `mekanism:reaction`: 14 (base: 14, compat: 0) - Coberto (`pressurized_reaction_chamber`)
22. `mekanismtools:paxel`: 11 (base: 11, compat: 0) - **NÃO COBRE** (serializer custom paxel)
23. `mekanism:crystallizing`: 10 (base: 10, compat: 0) - Coberto (`chemical_crystallizer`)
24. `mekanism:washing`: 7 (base: 7, compat: 0) - Coberto (`chemical_washer`)
25. `mekanism:chemical_infusing`: 5 (base: 5, compat: 0) - Coberto (`chemical_infuser`)
26. `mekanism:separating`: 3 (base: 3, compat: 0) - Coberto (`electrolytic_separator`)
27. `mekanism:activating`: 2 (base: 2, compat: 0) - Coberto (`solar_neutron_activator`)
28. `mekanism:evaporating`: 2 (base: 2, compat: 0) - Coberto (`thermal_evaporation`)
29. `mekanism:energy_conversion`: 2 (base: 2, compat: 0) - **PARCIAL** (hardcoded em `cube.rs`)
30. `mekanism:compressing`: 2 (base: 2, compat: 0) - Coberto (`osmium_compressor`, fábricas)
31. `mekanism:centrifuging`: 2 (base: 2, compat: 0) - Coberto (`isotopic_centrifuge`)
32. `mekanismtools:banner_shield`: 1 (base: 1, compat: 0) - **NÃO COBRE** (serializer custom escudo)
33. `minecraft:smithing_transform`: 1 (base: 1, compat: 0) - Engine Horizon (ferraria)
34. `mekanism:clear_configuration`: 1 (base: 1, compat: 0) - **NÃO COBRE** (reset na bancada)
35. `mekanism:bin_insert`: 1 (base: 1, compat: 0) - **NÃO COBRE** (inserção na bancada)
36. `mekanism:bin_extract`: 1 (base: 1, compat: 0) - **NÃO COBRE** (extração na bancada)
37. *(dinâmico no Java)* `mekanism:liquifying`: 0 JSONs - Coberto (44 comidas em `lib.rs:624-636`)

### Lista de Receitas que o Port NÃO Cobre
1. `mekanism:mek_data` (86 receitas) - Serializer especial de crafting table que preserva energia, químicos e upgrades; Horizon não possui suporte na engine.
2. `mekanism:bin_insert` (1 receita) e `mekanism:bin_extract` (1 receita) - Manipulação de bins na grade de crafting.
3. `mekanism:clear_configuration` (1 receita) - Reset de cartões e configurações na grade de crafting.
4. `mekanismtools:banner_shield` (1 receita) e `mekanismtools:paxel` (11 receitas) - Serializers customizados do MekanismTools.
5. `minecraft:blasting` (25 receitas) - O Energized Smelter do port carrega apenas `cooking_recipes("smelting")`, não incluindo `blasting`.
6. `compat/*` (637 receitas) - Receitas de compatibilidade com Biomes O' Plenty, Biomes We've Gone, Farmer's Delight e AE2 são filtradas pelo `base_1211.py`.

---

## 4. Testes e Mutações

- **Execução com `cargo test`**: não medido (toolchain rust/cargo não configurado para compilar nesta VM).
- **Validação com script de parsing**: 2814 receitas base convertidas por `base_1211.py` foram simuladas contra a lógica de parsing dos livros do Rust (`load_book`, `load_proc_book`, `load_cont_book`, etc.), com **0 falhas**.
- **Prova de mutação**: A prova de mutação está pendente do juiz (a tarefa não forneceu lista de mutações).

---

## 5. Arquivos Editados e Entregáveis

- `port/base_1211.py`: Correção de importação (linhas 10-14 e 102-106) movendo `SRC, OUT` para dentro do bloco `if __name__ == '__main__':`.
- `.saida/T20261109-184326-z78/AUDITORIA_worldgen_receitas.md`: Relatório completo com tabelas detalhadas, linha a linha, justificativas de limites e bloco pronto para a Seção 20 de `port/DIFERENCAS.md`.
- `.saida/T20261109-184326-z78/RESUMO.md`: Este relatório final.
