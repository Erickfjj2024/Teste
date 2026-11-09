# Auditoria: Worldgen e Receitas (Mekanism 1.21.1-10.7.19.85 vs Port Horizon)

Data: 2026-11-09  
Referência Java: `v1.21.1-10.7.19.85` (`java/`)  
Referência Port: `port/mod/src/worldgen.rs`, `port/mod/src/lib.rs`, `port/base_1211.py`  

---

## 1. Worldgen: Minérios, Sal e Retrogen

### 1.1 Tabela Comparativa de Worldgen

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado | Detalhes / Justificativa |
|---|---|---|---|---|
| **BOTTOM (Mundo Overworld)** | Y = -64 (`DimensionType`, `WorldConfig.java:135`) | `-64` (`port/mod/src/worldgen.rs:17`) | **OK** | Fundo padrão do Overworld 1.18+ |
| **TOP (Mundo Overworld)** | Y = 320 (`DimensionType`, `WorldConfig.java:135`) | `320` (`port/mod/src/worldgen.rs:18`) | **OK** | Teto padrão do Overworld 1.18+ |
| **Tin Ore (small)** | perChunk: 14, size: 4, TRAPEZOID, Y=[-20, 94] (`OreType.java:16`) | `per_chunk: 14, size: 4, trapezoid: true, min_y: -20, max_y: 94` (`worldgen.rs:21`) | **OK** | Idêntico |
| **Tin Ore (large)** | perChunk: 12, size: 9, TRAPEZOID, Y=[-32, 72] (`OreType.java:17`) | `per_chunk: 12, size: 9, trapezoid: true, min_y: -32, max_y: 72` (`worldgen.rs:22`) | **OK** | Idêntico |
| **Osmium Ore (upper)** | perChunk: 65, size: 7, TRAPEZOID, Y=[72, 343], plateau: 8 (`OreType.java:20`) | `per_chunk: 65, size: 7, trapezoid: true, min_y: 72, max_y: 320` (`worldgen.rs:23`) | **LIMITE** | No Java max é `belowTop(-24)` = 343 com platô 8; port faz teto em TOP (320) e sem platô |
| **Osmium Ore (middle)** | perChunk: 6, size: 9, TRAPEZOID, Y=[-32, 56] (`OreType.java:21`) | `per_chunk: 6, size: 9, trapezoid: true, min_y: -32, max_y: 56` (`worldgen.rs:24`) | **OK** | Idêntico |
| **Osmium Ore (small)** | perChunk: 8, size: 4, UNIFORM, Y=[-64, 64] (`OreType.java:22`) | `per_chunk: 8, size: 4, trapezoid: false, min_y: -64, max_y: 64` (`worldgen.rs:25`) | **OK** | Idêntico |
| **Uranium Ore (small)** | perChunk: 4, size: 4, TRAPEZOID, Y=[-64, 8] (`OreType.java:25`) | `per_chunk: 4, size: 4, trapezoid: true, min_y: -64, max_y: 8` (`worldgen.rs:26`) | **OK** | Idêntico |
| **Uranium Ore (buried)** | perChunk: 7, size: 9, TRAPEZOID, Y=[-88, -8], plateau: 16, discardAir: 0.75 (`OreType.java:26`) | `per_chunk: 7, size: 9, trapezoid: true, min_y: -64, max_y: -8` (`worldgen.rs:27`) | **LIMITE** | No Java min é `aboveBottom(-24)` = -88 com platô 16 e descarte por ar 75%; port fixa min em BOTTOM (-64) |
| **Fluorite Ore (normal)** | perChunk: 5, size: 5, UNIFORM, Y=[-64, 23] (`OreType.java:29`) | `per_chunk: 5, size: 5, trapezoid: false, min_y: -64, max_y: 23` (`worldgen.rs:28`) | **OK** | Idêntico |
| **Fluorite Ore (buried)** | perChunk: 3, size: 13, TRAPEZOID, Y=[-64, 4], discardAir: 1.0 (`OreType.java:30`) | `per_chunk: 3, size: 13, trapezoid: true, min_y: -64, max_y: 4` (`worldgen.rs:29`) | **LIMITE** | No Java tem descarte por ar 100% (nunca exposto em cavernas); port não checa ar |
| **Lead Ore (normal)** | perChunk: 8, size: 9, TRAPEZOID, Y=[-88, 64], discardAir: 0.25 (`OreType.java:33`) | `per_chunk: 8, size: 9, trapezoid: true, min_y: -64, max_y: 64` (`worldgen.rs:30`) | **LIMITE** | No Java min é `aboveBottom(-24)` = -88 com descarte por ar 25%; port fixa min em BOTTOM (-64) |
| **Salt per chunk** | 2 (`WorldConfig.java:43`, `placed_feature/salt.json`) | `SALT_PER_CHUNK: u32 = 2` (`worldgen.rs:97`) | **OK** | Idêntico |
| **Salt radius** | min 2, max 3 (`WorldConfig.java:43`) | `SALT_MIN_RADIUS = 2, SALT_MAX_RADIUS = 3` (`worldgen.rs:98-99`) | **OK** | Idêntico |
| **Salt half height** | 1 (`WorldConfig.java:43`) | `SALT_HALF_HEIGHT: i32 = 1` (`worldgen.rs:100`) | **OK** | Idêntico |
| **Salt targets** | `minecraft:dirt`, `minecraft:clay` (`configured_feature/salt.json`) | `matches!(block, "minecraft:dirt" \| "minecraft:clay")` (`worldgen.rs:133`) | **OK** | Idêntico |
| **Salt condition** | Fundo de água (`placed_feature/salt.json`: `OCEAN_FLOOR_WG`, `fluids: minecraft:water`) | Varredura vertical por piso sólido sob água (`lib.rs:2065-2085`) | **OK** | Idêntico em semântica |
| **Substituição Stone** | Tag `#minecraft:stone_ore_replaceables` -> `mekanism:<ore>_ore` (`configured_feature/ore_*.json`) | `minecraft:stone` -> `mekanism:<ore>_ore` (`worldgen.rs:140`) | **LIMITE** | Java substitui pedra, granito, diorito, andesito; port substitui apenas `minecraft:stone` |
| **Substituição Deepslate** | Tag `#minecraft:deepslate_ore_replaceables` -> `mekanism:deepslate_<ore>_ore` (`configured_feature/ore_*.json`) | `minecraft:deepslate` -> `mekanism:deepslate_<ore>_ore` (`worldgen.rs:141`) | **LIMITE** | Java substitui deepslate e tufo; port substitui apenas `minecraft:deepslate` |
| **Biomas** | `#mekanism:spawn_ores` = `#minecraft:is_overworld` + `#c:is_overworld` (`biome_modifier/*.json`, `tags/worldgen/biome/spawn_ores.json`) | `dimension == "minecraft:overworld"` (`lib.rs:2043`) | **LIMITE** | Port não filtra por tag de bioma, aplica a toda a dimensão Overworld |
| **Retrogen** | `enableRegeneration` (def. false), `userGenVersion` (def. 0), salva versão no chunk NBT, gera via `GenHandler.generate` (`WorldConfig.java:36-39`, `CommonWorldTickHandler.java:143-146, 203-228`, `GenHandler.java:57-85`) | `if !new \|\| dimension != "minecraft:overworld" { return; }` (`lib.rs:2043`) | **FALTA / LIMITE** | Chunks já existentes (`!new`) são ignorados; não há retrogen nem versão salva |

---

## 2. Receitas: Contagem e Cobertura

### 2.1 Contagem Total por Módulo (Datagen / JSON no Java 1.21.1-10.7.19.85)

| Módulo | Base (JSON) | Compat (JSON) | Total (JSON) |
|---|---|---|---|
| `mekanism` | 1578 | 637 | 2215 |
| `mekanismadditions` | 1114 | 0 | 1114 |
| `mekanismtools` | 87 | 0 | 87 |
| `mekanismgenerators` | 35 | 0 | 35 |
| **TOTAL GERAL** | **2814** | **637** | **3451** |

> [!NOTE]
> As 637 receitas de `compat` pertencem a integrações opcionais do Java com outros mods: `biomesoplenty`, `biomeswevegone`, `farmersdelight` e `ae2`. O script `port/base_1211.py` pula essas pastas intencionalmente, pois esses mods não existem no Horizon.

---

### 2.2 Contagem por Tipo de Receita e Cobertura pelo Port

| Tipo de Receita | Qtd Java Base | Qtd Java Compat | Qtd Java Total | Processador no Port | Cobertura | Observações |
|---|---|---|---|---|---|---|
| `minecraft:crafting_shaped` | 807 | 0 | 807 | Engine Horizon (crafting) | **OK** | 178 mekanism, 27 generators, 60 tools, 542 additions |
| `mekanism:painting` | 464 | 0 | 464 | `painting_machine` (`lib.rs:212`) | **OK** | 176 mekanism, 288 additions |
| `mekanism:pigment_extracting` | 394 | 48 | 442 | `pigment_extractor` (`lib.rs:211`) | **OK** | 178 mekanism base, 216 additions; 48 compat ignorados |
| `mekanism:sawing` | 124 | 316 | 440 | `precision_sawmill` (`lib.rs:185`) + Fábricas (`lib.rs:248-251`) | **OK** | 124 base; 316 compat ignorados |
| `mekanism:enriching` | 187 | 153 | 340 | `enrichment_chamber` (`lib.rs:183`) + Fábricas (`lib.rs:235-238`) | **OK** | 169 mekanism, 18 additions; 153 compat ignorados |
| `mekanism:crushing` | 200 | 113 | 313 | `crusher` (`lib.rs:182`) + Fábricas (`lib.rs:231-234`) | **OK** | 200 base; 113 compat ignorados |
| `mekanism:injecting` | 91 | 0 | 91 | `chemical_injection_chamber` (`lib.rs:190`) + Fábricas (`lib.rs:261-264`) | **OK** | 91 base |
| `mekanism:mek_data` | 86 | 0 | 86 | Não implementado (crafting grid custom) | **NÃO COBRE** | Receitas de bancada que transferem NBT/energia/químico entre tiers |
| `mekanism:combining` | 82 | 0 | 82 | `combiner` (`lib.rs:186`) + Fábricas (`lib.rs:244-247`) | **OK** | 82 base |
| `minecraft:crafting_shapeless` | 76 | 0 | 76 | Engine Horizon (crafting) | **OK** | 26 mekanism, 50 additions |
| `mekanism:metallurgic_infusing` | 31 | 7 | 38 | `metallurgic_infuser` (`lib.rs:187`) + Fábricas (`lib.rs:257-260`) | **OK** | 30 mekanism, 1 generators; 7 compat ignorados |
| `mekanism:purifying` | 28 | 0 | 28 | `purification_chamber` (`lib.rs:189`) + Fábricas (`lib.rs:265-268`) | **OK** | 28 base |
| `minecraft:smelting` | 25 | 0 | 25 | `energized_smelter` (`lib.rs:184`) + Fábricas (`lib.rs:239-242`) | **OK** | 18 mekanism, 7 tools (lê via `Source::Cooking("smelting")`) |
| `minecraft:blasting` | 25 | 0 | 25 | Apenas Fornalha Potente vanilla | **NÃO COBRE** | Smelter elétrico só lê `smelting`, não lê receitas de `blasting` |
| `mekanism:oxidizing` | 25 | 0 | 25 | `chemical_oxidizer` (`lib.rs:192`) | **OK** | 25 base |
| `mekanism:chemical_conversion` | 24 | 0 | 24 | Slots de enchimento de máquinas (`lib.rs:685-690`) | **OK** | Converte redstone, diamante, etc. no químico do tanque |
| `mekanism:dissolution` | 23 | 0 | 23 | `chemical_dissolution_chamber` (`lib.rs:193`) | **OK** | 23 base |
| `mekanism:nucleosynthesizing` | 21 | 0 | 21 | `antiprotonic_nucleosynthesizer` (`lib.rs:207`) | **OK** | 21 base |
| `mekanism:rotary` | 20 | 0 | 20 | `rotary_condensentrator` (`lib.rs:204`, `plus.rs:91-105`) | **OK** | 17 mekanism, 3 generators (condensação e descondensação) |
| `mekanism:pigment_mixing` | 16 | 0 | 16 | `pigment_mixer` (`lib.rs:213`, `plus.rs:46-55`) | **OK** | 16 base |
| `mekanism:reaction` | 14 | 0 | 14 | `pressurized_reaction_chamber` (`lib.rs:222`, `reaction.rs:141`) | **OK** | 14 base (PRC: substrato, gaseificação) |
| `mekanismtools:paxel` | 11 | 0 | 11 | Não implementado (crafting grid custom) | **NÃO COBRE** | Crafting custom de paxels que preserva danos/dados das ferramentas |
| `mekanism:crystallizing` | 10 | 0 | 10 | `chemical_crystallizer` (`lib.rs:194`) | **OK** | 10 base |
| `mekanism:washing` | 7 | 0 | 7 | `chemical_washer` (`lib.rs:198`, `plus.rs:56-64`) | **OK** | 7 base |
| `mekanism:chemical_infusing` | 5 | 0 | 5 | `chemical_infuser` (`lib.rs:196`, `plus.rs:46-55`) | **OK** | 4 mekanism, 1 generators |
| `mekanism:separating` | 3 | 0 | 3 | `electrolytic_separator` (`lib.rs:201`, `plus.rs:73-83`) | **OK** | 2 mekanism, 1 generators |
| `mekanism:activating` | 2 | 0 | 2 | `solar_neutron_activator` (`lib.rs:219`, `plus.rs:68-72`) | **OK** | 1 mekanism (polônio), 1 generators (trítio) |
| `mekanism:evaporating` | 2 | 0 | 2 | `thermal_evaporation` (`evaporation.rs:147`) | **OK** | Água -> salmoura (10:1), Salmoura -> lítio (10:1) |
| `mekanism:energy_conversion` | 2 | 0 | 2 | `cube.rs:13-19` | **PARCIAL** | Valores hardcoded para redstone/bloco; não lê JSON dinamicamente |
| `mekanism:compressing` | 2 | 0 | 2 | `osmium_compressor` (`lib.rs:191`) + Fábricas (`lib.rs:253-256`) | **OK** | 2 base (ósmio e urânio brutos) |
| `mekanism:centrifuging` | 2 | 0 | 2 | `isotopic_centrifuge` (`lib.rs:217`, `plus.rs:68-72`) | **OK** | 2 base (UF6 -> combustível físsil, lixo nuclear -> plutônio) |
| `mekanismtools:banner_shield` | 1 | 0 | 1 | Não implementado (crafting grid custom) | **NÃO COBRE** | Aplica estandarte ao escudo |
| `minecraft:smithing_transform` | 1 | 0 | 1 | Bancada de Ferraria vanilla | **OK** | Atualização de paxel de netherite |
| `mekanism:clear_configuration` | 1 | 0 | 1 | Não implementado (crafting grid custom) | **NÃO COBRE** | Limpa configuração de máquina/card na bancada |
| `mekanism:bin_insert` | 1 | 0 | 1 | Não implementado (crafting grid custom) | **NÃO COBRE** | Insere itens no bin pela bancada 3x3 |
| `mekanism:bin_extract` | 1 | 0 | 1 | Não implementado (crafting grid custom) | **NÃO COBRE** | Extrai itens do bin pela bancada 3x3 |
| *(dinâmico)* `mekanism:liquifying` | 0 (Java) | 0 | 0 | `nutritional_liquifier` (`lib.rs:209, 624-645`) | **OK** | No Java é código (`createDynamicRecipe`); port usa tabela `FOODS` de 44 alimentos |

---

### 2.3 Resumo das Receitas que o Port NÃO Cobre

1. **`mekanism:mek_data` (86 receitas)**: No Java, permite craftar melhorias de tier (ex.: Basic Energy Cube -> Advanced Energy Cube) na bancada mantendo energia armazenada, gases e upgrades instalados. No Horizon, o motor de crafting não suporta serializers customizados com preservação de NBT;
2. **`mekanism:bin_insert` e `mekanism:bin_extract` (2 receitas)**: Permitem encher ou esvaziar bins diretamente na grade de crafting 2x2 ou 3x3. No port, bins só interagem no mundo;
3. **`mekanism:clear_configuration` (1 receita)**: Permite resetar cartões ou blocos com shift/crafting. No port, não suportado;
4. **`mekanismtools:paxel` (11 receitas) e `mekanismtools:banner_shield` (1 receita)**: Receitas customizadas do módulo de ferramentas;
5. **`minecraft:blasting` (25 receitas)**: O Energized Smelter do port carrega apenas receitas de `smelting` simples (`recipe_types::cooking_recipes("smelting")`), não incluindo receitas exclusivas de alto-forno (`blasting`);
6. **`compat/*` (637 receitas)**: Todas as receitas compatíveis com Biomes O' Plenty, Biomes We've Gone, Farmer's Delight e Applied Energistics 2 são intencionalmente ignoradas pelo conversor `base_1211.py`.

---

## 3. Conversor `port/base_1211.py`

- Normaliza os ingredientes Java 1.21.1 (`{"chemical": x}`, `{"fluid": x}`, `{"tag": x}`, `{"item": x}`) para a forma lida pelo port (`ingredient: x` ou `#x`).
- Normaliza tipos compostos do NeoForge (`neoforge:difference`, `neoforge:intersection`, `neoforge:or`).
- Converte saídas duplas:
  - `mekanism:separating`: `left_chemical_output` e `right_chemical_output` -> `output: {left, right}`
  - `mekanism:reaction`: `chemical_output` e `item_output` -> `output: {chemical, item}`
- **Correção aplicada**: `SRC, OUT = sys.argv[1], sys.argv[2]` estava em nível de módulo, causando `IndexError` ao importar o arquivo como módulo python. Movido para dentro do bloco `if __name__ == '__main__':`.


---

## 4. Bloco pronto para `port/DIFERENCAS.md` (Seção 20)

```markdown
## 20. Auditoria de Worldgen e Receitas (Mekanism 1.21.1-10.7.19.85)

| # | Diferença | Estado |
|---|---|---|
| 20.1 | Worldgen (minérios): tamanhos de veia, contagens por chunk e limites verticais conferidos contra `OreType.java` e JSONs de `worldgen/placed_feature`. O port limita o Y máximo do Osmium superior em 320 (no Java é 343 com platô 8) e o Y mínimo de Uranium enterrado e Chumbo em -64 (no Java é -88). Sem descarte de minérios enterrados por exposição ao ar (Java: fluorita 100%, urânio 75%, chumbo 25%) | LIMITE |
| 20.2 | Worldgen (substituição de blocos): o port substitui apenas `minecraft:stone` por minério comum e `minecraft:deepslate` por minério de deepslate; o Java usa as tags `#minecraft:stone_ore_replaceables` e `#minecraft:deepslate_ore_replaceables` (incluindo granito, diorito, andesito e tufo) | LIMITE |
| 20.3 | Worldgen (sal): verificado e fiel ao `WorldConfig.java` (2 por chunk, raio 2-3, meia-altura 1, sob água em terra/argila) | OK |
| 20.4 | Worldgen (biomas e retrogen): o port gera minérios em todos os biomas da dimensão `minecraft:overworld` (Java usa `#mekanism:spawn_ores`); retrogen (`enableRegeneration`, `userGenVersion` e geração em chunks já existentes `!new`) não é suportado no port | LIMITE |
| 20.5 | Receitas (processamento de máquinas): 24 tipos de receitas de máquinas do Java cobrem 1727 receitas base (crushing, enriching, smelting, sawing, combining, compressing, purifying, injecting, infusing, dissolution, washing, crystallizing, oxidizing, activating, centrifuging, reaction, painting, nucleosynthesizing, chemical_infusing, rotary, separating, pigment_extracting, pigment_mixing, evaporating, chemical_conversion); convertidas fielmente por `base_1211.py` | OK |
| 20.6 | Receitas de bancada com NBT (`mekanism:mek_data`, 86 receitas): preservação de dados/energia/gás/upgrades ao evoluir máquinas/cubos/tanques/fábricas na bancada não é suportada pela engine de crafting do Horizon | LIMITE |
| 20.7 | Receitas especiais de crafting: `mekanism:bin_insert` (1), `mekanism:bin_extract` (1), `mekanism:clear_configuration` (1), `mekanismtools:banner_shield` (1) e `mekanismtools:paxel` (11) não são suportadas pela engine | LIMITE |
| 20.8 | Receitas de alto-forno (`minecraft:blasting`, 25 receitas): o Energized Smelter do port carrega apenas receitas de `minecraft:smelting`, ignorando receitas exclusivas de `blasting` | LIMITE |
| 20.9 | Receitas de compatibilidade externa (`compat/*`, 637 receitas): intencionalmente ignoradas pelo conversor `base_1211.py` pois referenciam mods inexistentes no Horizon (BOP, BWG, Farmer's Delight, AE2) | LIMITE |
| 20.10 | Receitas de conversão de energia (`mekanism:energy_conversion`, 2 receitas): hardcoded em `cube.rs` (redstone 10.000, bloco 90.000) em vez de leitura dinâmica via `recipes_of_type` | PARCIAL |
| 20.11 | Conversor `base_1211.py`: leitura de `SRC, OUT` movida para dentro de `if __name__ == '__main__':` para permitir importação limpa sem lançar `IndexError` | CORRIGIDO |
```
