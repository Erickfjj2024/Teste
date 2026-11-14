# Auditoria: Mekanism Generators, MekanismTools e MekanismAdditions

- **Data da auditoria:** 2026-11-09
- **Escopo:** Módulo `generators` (`generators.rs`, `turbine.rs`, `fission.rs`, `fusion.rs`, `plus.rs`), `mekanismtools` (`java/tools`) e `mekanismadditions` (`java/additions`).
- **Fontes Java (1.21.1-10.7.19.85):**
  - `java/generators/java/mekanism/generators/common/config/GeneratorsConfig.java`
  - `java/generators/java/mekanism/generators/common/config/GeneratorsStorageConfig.java`
  - `java/generators/java/mekanism/generators/common/registries/GeneratorsBlockTypes.java`
  - `java/generators/java/mekanism/generators/common/tile/*.java`
  - `java/tools/java/mekanism/tools/common/material/impl/*.java`
  - `java/tools/java/mekanism/tools/common/material/BaseMekanismMaterial.java`
  - `java/tools/java/mekanism/tools/common/material/IPaxelMaterial.java`
  - `java/tools/java/mekanism/tools/common/registries/ToolsArmorMaterials.java`
  - `java/additions/java/mekanism/additions/common/registries/AdditionsBlocks.java`
  - `java/additions/java/mekanism/additions/common/registries/AdditionsBlockTypes.java`
  - `java/additions/java/mekanism/additions/common/config/AdditionsConfig.java`
  - `java/api/java/mekanism/api/text/EnumColor.java`
  - `java/api/java/mekanism/api/heat/HeatAPI.java`
- **Código Port:** `port/mod/src/generators.rs`, `port/mod/src/plus.rs`, `port/mod/src/turbine.rs`, `port/mod/src/fission.rs`, `port/mod/src/fusion.rs`, `port/mod/src/lib.rs`.

---

## 1. Mekanism Generators (Geradores & Multiblocos)

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado |
|---|---|---|---|
| **Bio Generator: Geração** | 350 J/t (`GeneratorsConfig.java:91`) | `fe(350)` = 140 FE/t (`generators.rs:29`) | OK |
| **Bio Generator: Capacidade do Tanque** | 24 000 mB (`GeneratorsConfig.java:93`) | 24 000 mB (`generators.rs:30`) | OK |
| **Bio Generator: Conversão Bio Fuel (item)** | 200 mB (`TileEntityBioGenerator.java:62`) | 200 mB (`generators.rs:31`) | OK |
| **Bio Generator: Conversão Bio Fuel (bloco)** | 1 800 mB (9 × 200) (`TileEntityBioGenerator.java:62`) | 1 800 mB (`generators.rs:129`) | OK |
| **Bio Generator: Armazenamento de Energia** | 160 000 J (`GeneratorsStorageConfig.java:24`) | `fe(160_000)` = 64 000 FE (`generators.rs:52`) | OK |
| **Bio Generator: Fluido Aceito** | `mekanismgenerators:bioethanol` (`TileEntityBioGenerator.java:53`) | `mekanismgenerators:bioethanol` (`generators.rs:27`) | OK |
| **Bio Generator: Face de Energia** | Frente (`TileEntityGenerator.java:30,48`) | Frente (`generators.rs:229`) | OK |
| **Bio Generator: Faces do Tanque** | Todas exceto Frente (`TileEntityBioGenerator.java:53-54`) | Todas exceto Frente (`generators.rs:216`) | OK |
| **Bio Generator: Slots de Itens** | Slot 0 combustível (exceto direita), Slot 1 energia (direita) (`TileEntityBioGenerator.java:63-65`) | Slot 0 exceto direita, slot 1 direita (`generators.rs:208`) | OK |
| **Heat Generator: Geração por Queima de Lava** | 200 J/t (`GeneratorsConfig.java:98`) | 200.0 (`generators.rs:34`) | OK |
| **Heat Generator: Consumo de Lava** | 10 mB/t (`GeneratorsConfig.java:106`) | 10 (`generators.rs:35`) | OK |
| **Heat Generator: Boost por Bloco de Lava Adjacente** | 30 J/t (`GeneratorsConfig.java:100`) | 30.0 (`generators.rs:36`) | OK |
| **Heat Generator: Boost Dimensão Nether** | 100 J/t (`GeneratorsConfig.java:102`) | 100.0 (`generators.rs:37`) | OK |
| **Heat Generator: Capacidade do Tanque de Lava** | 24 000 mB (`GeneratorsConfig.java:104`) | 24 000 (`generators.rs:38`) | OK |
| **Heat Generator: Produção Máxima Passiva+Ativa** | 510 J/t (30×7 + 100 + 200) (`TileEntityHeatGenerator.java:56-58`) | 510 (`generators.rs:71`) | OK |
| **Heat Generator: Armazenamento de Energia** | 160 000 J (`GeneratorsStorageConfig.java:22`) | `fe(160_000)` = 64 000 FE (`generators.rs:51`) | OK |
| **Heat Generator: Capacidade Térmica (Heat Capacity)** | 10.0 (`TileEntityHeatGenerator.java:50`) | 10.0 (`generators.rs:58`) | OK |
| **Heat Generator: Condução Inversa** | 5.0 (`TileEntityHeatGenerator.java:51`) | 5.0 (`generators.rs:65`) | OK |
| **Heat Generator: Isolamento Inverso (lados)** | 100.0 (`TileEntityHeatGenerator.java:52`) | 100.0 (`generators.rs:66`) | OK |
| **Heat Generator: Isolamento Inverso (fundo)** | 0.0 (`TileEntityHeatGenerator.java:173,178`) | 0.0 (`generators.rs:97`) | OK |
| **Heat Generator: Transferência para o Ar** | 10 000.0 (`HeatAPI.java:24`) | 10 000.0 (`generators.rs:67`) | OK |
| **Heat Generator: Eficiência Térmica** | 0.5 (`TileEntityHeatGenerator.java:53`) | 0.5 (`generators.rs:68`) | OK |
| **Heat Generator: Temperatura Ambiente Baseline** | 300 K em bioma 0.8, inclinação 25 K/grau (`HeatAPI.java:20,81`) | 300 K em 0.8, 25 K/grau clamp [-5, 5] (`generators.rs:59,63`) | OK |
| **Heat Generator: Conversão de Combustível em Lava** | burnTime / 20 mB (`TileEntityHeatGenerator.java:98`) | 1000 * burn / 20 000 = burn / 20 (`generators.rs:122`) | OK |
| **Heat Generator: Faces de Energia e Tanque** | Energia: Frente; Tanque: todas exceto Frente (`TileEntityHeatGenerator.java:87, TileEntityGenerator.java:30`) | Energia: Frente; Tanque: todas exceto Frente (`generators.rs:216,229`) | OK |
| **Solar Generator: Geração Base** | 50 J/t (`GeneratorsConfig.java:85`) | 50.0 (`generators.rs:41`) | OK |
| **Solar Generator: Armazenamento de Energia** | 96 000 J (`GeneratorsStorageConfig.java:26`) | `fe(96_000)` = 38 400 FE (`generators.rs:53`) | OK |
| **Solar Generator: Face de Saída de Energia** | Fundo (BOTTOM / DOWN) (`TileEntitySolarGenerator.java:33`) | DOWN (`generators.rs:224`) | OK |
| **Solar Generator: Limiar de Escuridão do Céu** | `getSkyDarken() < 4` (`WorldUtils.java:904`) | `darken < 4` (`generators.rs:397`) | OK |
| **Solar Generator: Multiplicador de Pico (Temp/Chuva)** | `tempEff = 0.3*(0.8-T)`, `humidityEff = -0.3*downfall` (`TileEntitySolarGenerator.java:155-161`) | `peak()` (`generators.rs:174-178`) | OK |
| **Solar Generator: Penalidade de Chuva** | Penalidade de 80% (`* 0.2F`) em biomas com chuva (`TileEntitySolarGenerator.java:183`) | `share *= 0.2` em bioma com chuva e tempo chuvoso/trovoada (`generators.rs:401`) | CORRIGIDO |
| **Advanced Solar Generator: Geração Base** | 300 J/t (`GeneratorsConfig.java:86`) | 300.0 (`generators.rs:43`) | OK |
| **Advanced Solar Generator: Armazenamento de Energia** | 200 000 J (`GeneratorsStorageConfig.java:28`) | `fe(200_000)` = 80 000 FE (`generators.rs:54`) | OK |
| **Advanced Solar Generator: Faces de Saída de Energia** | Frente e Fundo (`TileEntityAdvancedSolarGenerator.java:18`) | Frente e Fundo (`generators.rs:226`) | CORRIGIDO (estava só DOWN) |
| **Advanced Solar Generator: Orientação ao Posicionar** | Vira para o jogador por ter face frontal (`Generator.java:15`) | Vira para o jogador mais próximo (`generators.rs:268`) | CORRIGIDO (era ignorado) |
| **Advanced Solar Generator: Penalidade de Chuva** | Penalidade de 80% (`* 0.2F`) (`TileEntitySolarGenerator.java:183`) | `share *= 0.2` em chuva/trovoada (`generators.rs:401`) | CORRIGIDO |
| **Wind Generator: Geração Mínima** | 60 J/t (`GeneratorsConfig.java:115`) | 60.0 (`generators.rs:44`) | OK |
| **Wind Generator: Geração Máxima** | 480 J/t (`GeneratorsConfig.java:117`) | 480.0 (`generators.rs:45`) | OK |
| **Wind Generator: Altura Mínima (minY)** | Y = 24 (`GeneratorsConfig.java:119`) | Y = 24 (`generators.rs:46`) | OK |
| **Wind Generator: Altura Máxima (maxY)** | Y = 319 (320 - 1) (`GeneratorsConfig.java:123`, `TileEntityWindGenerator.java:111`) | Y = 319 (`generators.rs:47`) | OK |
| **Wind Generator: Armazenamento de Energia** | 200 000 J (`GeneratorsStorageConfig.java:30`) | `fe(200_000)` = 80 000 FE (`generators.rs:55`) | OK |
| **Wind Generator: Curva de Produção por Altura** | Interpolação linear entre minY e maxY (`TileEntityWindGenerator.java:117-118`) | Interpolação linear idêntica (`generators.rs:194-198`) | OK |
| **Wind Generator: Faces de Saída de Energia** | Frente e Fundo (`TileEntityWindGenerator.java:31`) | Frente e Fundo (`generators.rs:226-227`) | OK |
| **Wind Generator: Altura do Topo da Turbina** | Y + 4 (`TileEntityWindGenerator.java:104`) | Y + 4 (`generators.rs:384`) | OK |
| **Gas-Burning Generator: Capacidade do Tanque** | 18 000 mB (`GeneratorsConfig.java:111`) | 18 000 (`generators.rs:49`) | OK |
| **Gas-Burning Generator: Armazenamento de Energia** | 1 000 × densidade do H2 = 200 000 J (`GeneratorsBlockTypes.java:123`) | `fe(1_000 * 200)` = 80 000 FE (`generators.rs:56`) | OK |
| **Gas-Burning Generator: Taxa Máxima de Ticks de Queima** | Até 256 ticks por tick (`TileEntityGasGenerator.java:136`) | 256.0 (`generators.rs:153`) | OK |
| **Gas-Burning Generator: Combustível Hidrogênio** | 1 tick de queima, 200 J/tick (`MekanismDataMapsProvider.java:73`) | (1 tick, `fe(200)` = 80 FE) (`generators.rs:138`) | OK |
| **Gas-Burning Generator: Combustível Etileno (Ethene)** | 40 ticks de queima, 705 J/tick (`MekanismDataMapsProvider.java:74,92-99`) | (40 ticks, `fe(705)` = 282 FE) (`generators.rs:139`) | OK |
| **Gas-Burning Generator: Faces de Energia e Tanque** | Energia: Frente; Tanque: todas exceto Frente (`TileEntityGasGenerator.java:70, TileEntityGenerator.java:30`) | Energia: Frente; Tanque: todas exceto Frente (`generators.rs:216,229`) | OK |
| **Turbina Industrial: Pás por Bobina (bladesPerCoil)** | 4 (`GeneratorsConfig.java:128`) | 4 (`turbine.rs:29`) | OK |
| **Turbina Industrial: Vazão do Vent (ventChemicalFlow)** | 32 000 mB/t (32 × balde) (`GeneratorsConfig.java:130`) | 32 000.0 (`turbine.rs:30`) | OK |
| **Turbina Industrial: Vazão do Disperser (disperserChemicalFlow)** | 1 280 mB/t (`GeneratorsConfig.java:132`) | 1 280.0 (`turbine.rs:31`) | OK |
| **Turbina Industrial: Taxa do Condensador (condenserRate)** | 64 000 mB/t (64 × balde) (`GeneratorsConfig.java:134`) | 64 000 (`turbine.rs:32`) | OK |
| **Turbina Industrial: Armazenamento de Energia por Volume** | 16 000 000 J (`GeneratorsConfig.java:136`) | `fe(16_000_000)` = 6 400 000 FE (`turbine.rs:34`) | OK |
| **Turbina Industrial: Capacidade de Vapor por Tanque** | 64 000 mB (64 × balde) (`GeneratorsConfig.java:140`) | 64 000 (`turbine.rs:35`) | OK |
| **Turbina Industrial: Energia Máxima por Vapor** | 10 J/mB (`GeneralConfig.java:202`) | 10.0 (`turbine.rs:37`) | OK |
| **Turbina Industrial: Máximo de Pás (MAX_BLADES)** | 28 (`TurbineValidator.java:21`) | 28.0 (`turbine.rs:28`) | OK |
| **Reator de Fissão: Energia por Combustível Queimado** | 1 000 000 J (`GeneratorsConfig.java:144`) | 1 000 000.0 (`fission.rs:16`) | OK |
| **Reator de Fissão: Capacidade Térmica do Casing** | 1 000.0 (`GeneratorsConfig.java:146`) | 1 000.0 (`fission.rs:22`) | OK |
| **Reator de Fissão: Meta de Área de Superfície** | 4.0 (`GeneratorsConfig.java:148`) | 4.0 (`fission.rs:23`) | OK |
| **Reator de Fissão: Taxa Padrão de Queima (defaultBurnRate)** | 0.1 (`GeneratorsConfig.java:150`) | 0.1 (`fission.rs:24`) | OK |
| **Reator de Fissão: Queima por Assembly (burnPerAssembly)** | 1 (`GeneratorsConfig.java:152`) | 1 (`fission.rs:25`) | OK |
| **Reator de Fissão: Combustível Máximo por Assembly** | 8 000 mB (8 × balde) (`GeneratorsConfig.java:154`) | 8 000 (`fission.rs:26`) | OK |
| **Reator de Fissão: Refrigerante Frio por Tanque** | 100 000 mB (100 × balde) (`GeneratorsConfig.java:157`) | 100 000 (`fission.rs:27`) | OK |
| **Reator de Fissão: Refrigerante Aquecido por Tanque** | 1 000 000 mB (1 000 × balde) (`GeneratorsConfig.java:159`) | 1 000 000 (`fission.rs:28`) | OK |
| **Reator de Fissão: Razão de Rejeito Excessivo** | 0.9 (`GeneratorsConfig.java:161`) | 0.9 (`fission.rs:29`) | OK |
| **Reator de Fusão: Energia por Combustível (fuelEnergy)** | 10 000 000 J (`GeneratorsConfig.java:179`) | 10 000 000.0 (`fusion.rs:16`) | OK |
| **Reator de Fusão: Eficiência do Termopar** | 0.05 (`GeneratorsConfig.java:181`) | 0.05 (`fusion.rs:21`) | OK |
| **Reator de Fusão: Condutividade Térmica do Casing** | 0.1 (`GeneratorsConfig.java:183`) | 0.1 (`fusion.rs:22`) | OK |
| **Reator de Fusão: Razão de Aquecimento de Água** | 0.3 (`GeneratorsConfig.java:185`) | 0.3 (`fusion.rs:23`) | OK |
| **Reator de Fusão: Capacidade de Combustível D-T** | 1 000 mB (1 × balde) (`GeneratorsConfig.java:187`) | 1 000 (`fusion.rs:24`) | OK |
| **Reator de Fusão: Capacidade de Armazenamento de Energia** | 1 000 000 000 J (`GeneratorsConfig.java:189`) | `fe(1_000_000_000)` = 400 000 000 FE (`fusion.rs:25`) | OK |
| **Reator de Fusão: Água por Injeção** | 1 000 000 mB (1 000 × balde) (`GeneratorsConfig.java:192`) | 1 000 000 (`fusion.rs:26`) | OK |
| **Reator de Fusão: Vapor por Injeção** | 100 000 000 mB (100 × água base) (`GeneratorsConfig.java:194`) | 100 000 000 (`fusion.rs:27`) | OK |
| **Hohlraum: Capacidade / Taxa de Enchimento** | Capacidade 10, fillRate 1 (`GeneratorsConfig.java:79,81`) | N/A (item ativado no reator) | LIMITE |
| **Solar Neutron Activator: Taxa e Penalidade de Chuva** | Taxa base 64, penalidade de 80% na chuva (`TileEntitySolarNeutronActivator.java:143,201`) | Taxa base 64.0, penalidade de 80% na chuva aplicada (`plus.rs:162-164`) | CORRIGIDO |

---

## 2. MekanismTools (Materiais e Ferramentas)

### 2.1 Propriedades Básicas dos Materiais

| Material | Durabilidade Ferramentas (`getUses`) | Velocidade (`getSpeed`) | Bônus Dano Ataque (`getAttackDamageBonus`) | Encantabilidade (`getEnchantmentValue`) | Toughness Armadura | Knockback Resistance | Durabilidade Escudo | Estado |
|---|---|---|---|---|---|---|---|---|
| **Bronze** | 375 (`BronzeMaterialDefaults.java:34`) | 7.0 (`:39`) | 2.0 (`:44`) | 10 (`:49`) | 1.0 (`:54`) | 0.0 (`:104`) | 403 (`:19`) | OK |
| **Lapis Lazuli** | 128 (`LapisLazuliMaterialDefaults.java:35`) | 9.0 (`:40`) | 1.0 (`:45`) | 32 (`:50`) | 0.0 (`:55`) | 0.0 (`:103`) | 224 (`:19`) | OK |
| **Osmium** | 1 024 (`OsmiumMaterialDefaults.java:36`) | 4.0 (`:41`) | 4.0 (`:46`) | 14 (`:51`) | 3.0 (`:56`) | 0.1 (`:106`) | 672 (`:21`) | OK |
| **Refined Glowstone** | 384 (`RefinedGlowstoneMaterialDefaults.java:34`) | 15.0 (`:39`) | 2.0 (`:44`) | 20 (`:49`) | 0.0 (`:54`) | 0.0 (`:103`) | 381 (`:19`) | OK |
| **Refined Obsidian** | 4 096 (`RefinedObsidianMaterialDefaults.java:35`) | 12.0 (`:40`) | 8.0 (`:45`) | 18 (`:49`) | 5.0 (`:59`) | 0.2 (`:109`) | 1 680 (`:19`) | OK |
| **Steel** | 500 (`SteelMaterialDefaults.java:34`) | 8.0 (`:39`) | 3.0 (`:44`) | 16 (`:49`) | 2.0 (`:54`) | 0.0 (`:103`) | 448 (`:19`) | OK |

### 2.2 Armaduras (Durabilidade e Defesa por Peça)

| Material | Durabilidade (H / C / L / B) | Defesa / Pontos de Armadura (H / C / L / B) | Resistente ao Fogo | Som de Equipar | Estado |
|---|---|---|---|---|---|
| **Bronze** | 198 / 288 / 270 / 234 (`BronzeMaterialDefaults.java:60-64`) | 3 / 7 / 6 / 2 (`BronzeMaterialDefaults.java:70-74`) | Não (`burnsInFire=true`) | `ARMOR_EQUIP_IRON` | OK |
| **Lapis Lazuli** | 110 / 160 / 150 / 130 (`LapisLazuliMaterialDefaults.java:60-64`) | 1 / 4 / 3 / 1 (`LapisLazuliMaterialDefaults.java:70-73`) | Não (`burnsInFire=true`) | `ARMOR_EQUIP_DIAMOND` | OK |
| **Osmium** | 330 / 480 / 450 / 390 (`OsmiumMaterialDefaults.java:62-66`) | 4 / 8 / 6 / 3 (`OsmiumMaterialDefaults.java:72-76`) | Não (`burnsInFire=true`) | `ARMOR_EQUIP_IRON` | OK |
| **Refined Glowstone** | 187 / 272 / 255 / 221 (`RefinedGlowstoneMaterialDefaults.java:60-64`) | 3 / 8 / 6 / 3 (`RefinedGlowstoneMaterialDefaults.java:70-74`) | Não (`burnsInFire=true`) | `ARMOR_EQUIP_IRON` | OK |
| **Refined Obsidian** | 825 / 1 200 / 1 125 / 975 (`RefinedObsidianMaterialDefaults.java:65-69`) | 6 / 12 / 8 / 5 (`RefinedObsidianMaterialDefaults.java:75-79`) | Sim (`burnsInFire=false`, `ToolsItems.java:168`) | `ARMOR_EQUIP_IRON` | LIMITE (`fire_resistant` ausente no port) |
| **Steel** | 220 / 320 / 300 / 260 (`SteelMaterialDefaults.java:60-64`) | 3 / 8 / 6 / 3 (`SteelMaterialDefaults.java:70-74`) | Não (`burnsInFire=true`) | `ARMOR_EQUIP_IRON` | OK |

### 2.3 Ferramentas e Paxels (Dano, Velocidade de Ataque e Eficiência)

*Regra Vanilla:* Dano total = 1 (mão) + `attackDamageBonus` + `toolDamageModifier`. Velocidade total = 4.0 (mão) + `toolAtkSpeed`.
*Paxel:* Durabilidade = 2 × durabilidade base (`BaseMekanismMaterial.java:64`); Dano = Machado + 1 (`:59`); Velocidade de ataque = -2.4 (`IPaxelMaterial.java:11`).

| Item / Material | Dano Modificador | Dano Total | Velocidade de Ataque Modificador | Durabilidade | Eficiência de Mineração | Estado |
|---|---|---|---|---|---|---|
| **Espada Padrão** | +3.0 (`BaseMekanismMaterial.java:21`) | 4.0 + Bônus | -2.4 (`:25`) | `getUses` | N/A | OK |
| **Pá Padrão** | +1.5 (`BaseMekanismMaterial.java:29`) | 2.5 + Bônus | -3.0 (`:33`) | `getUses` | `getSpeed` | OK |
| **Picareta Padrão** | +1.0 (`BaseMekanismMaterial.java:41`) | 2.0 + Bônus | -2.8 (`:45`) | `getUses` | `getSpeed` | OK |
| **Enxada Padrão** | `-attackDamageBonus` (`:50`) | 1.0 fixo | `bonus - 3.0` (`:54`) | `getUses` | `getSpeed` | OK |
| **Bronze Machado** | +7.0 (`BronzeMaterialDefaults.java:24`) | 10.0 (1+2+7) | -3.0 (`:29`) | 375 | 7.0 | OK |
| **Bronze Paxel** | +8.0 (7+1) | 11.0 (1+2+8) | -2.4 (`IPaxelMaterial.java:11`) | 750 (2×375) | 7.0 | OK |
| **Lapis Machado** | +4.0 (`LapisLazuliMaterialDefaults.java:24`) | 6.0 (1+1+4) | -2.9 (`:29`) | 128 | 9.0 | OK |
| **Lapis Paxel** | +5.0 (4+1) | 7.0 (1+1+5) | -2.4 (`IPaxelMaterial.java:11`) | 256 (2×128) | 9.0 | OK |
| **Osmium Machado** | +8.0 (`OsmiumMaterialDefaults.java:26`) | 13.0 (1+4+8) | -3.3 (`:31`) | 1 024 | 4.0 | OK |
| **Osmium Paxel** | +9.0 (8+1) | 14.0 (1+4+9) | -2.4 (`IPaxelMaterial.java:11`) | 2 048 (2×1024) | 4.0 | OK |
| **Glowstone Machado** | +6.0 (`RefinedGlowstoneMaterialDefaults.java:24`) | 9.0 (1+2+6) | -2.9 (`:29`) | 384 | 15.0 | OK |
| **Glowstone Paxel** | +7.0 (6+1) | 10.0 (1+2+7) | -2.4 (`IPaxelMaterial.java:11`) | 768 (2×384) | 15.0 | OK |
| **Obsidian Machado** | +7.0 (`RefinedObsidianMaterialDefaults.java:24`) | 16.0 (1+8+7) | -2.9 (`:29`) | 4 096 | 12.0 | OK |
| **Obsidian Paxel** | +8.0 (7+1) | 17.0 (1+8+8) | -2.4 (`IPaxelMaterial.java:11`) | 8 192 (2×4096) | 12.0 | OK |
| **Steel Machado** | +7.0 (`SteelMaterialDefaults.java:24`) | 11.0 (1+3+7) | -3.0 (`:29`) | 500 | 8.0 | OK |
| **Steel Paxel** | +8.0 (7+1) | 12.0 (1+3+8) | -2.4 (`IPaxelMaterial.java:11`) | 1 000 (2×500) | 8.0 | OK |

### 2.4 Paxels Vanilla

| Paxel | Tier Vanilla | Durabilidade (2 × Uses) | Dano | Eficiência | Encantabilidade | Resistente ao Fogo | Estado |
|---|---|---|---|---|---|---|---|
| **Wood Paxel** | WOOD (`WoodPaxelMaterialDefaults.java:14`) | 118 (2 × 59) | 7.0 (`:19`) | 2.0 | 15 | Não | OK |
| **Stone Paxel** | STONE (`StonePaxelMaterialDefaults.java:14`) | 262 (2 × 131) | 8.0 (`:19`) | 4.0 | 5 | Não | OK |
| **Iron Paxel** | IRON (`IronPaxelMaterialDefaults.java:14`) | 500 (2 × 250) | 7.0 (`:19`) | 6.0 | 14 | Não | OK |
| **Diamond Paxel** | DIAMOND (`DiamondPaxelMaterialDefaults.java:14`) | 3 122 (2 × 1561) | 6.0 (`:19`) | 8.0 | 10 | Não | OK |
| **Gold Paxel** | GOLD (`GoldPaxelMaterialDefaults.java:14`) | 64 (2 × 32) | 7.0 (`:19`) | 12.0 | 22 | Não | OK |
| **Netherite Paxel** | NETHERITE (`NetheritePaxelMaterialDefaults.java:14`) | 4 062 (2 × 2031) | 6.0 (`:19`) | 9.0 | 15 | Sim (`ToolsItems.java:145`) | OK |

---

## 3. MekanismAdditions (Plásticos, Glow Panels e Cores)

### 3.1 Cores e Variações de Blocos

- **18 Cores Suportadas (`EnumColor.java:28-46`):**
  1. Black (`black`)
  2. Dark Blue (`blue`)
  3. Dark Green (`green`)
  4. Dark Aqua (`cyan`)
  5. Dark Red (`dark_red`)
  6. Purple (`purple`)
  7. Orange (`orange`)
  8. Gray (`light_gray`)
  9. Dark Gray (`gray`)
  10. Indigo (`light_blue`)
  11. Bright Green (`lime`)
  12. Aqua (`aqua`)
  13. Red (`red`)
  14. Pink (`magenta`)
  15. Yellow (`yellow`)
  16. White (`white`)
  17. Brown (`brown`)
  18. Bright Pink (`pink`)

### 3.2 Tipos de Blocos de Plástico e Parâmetros

| Bloco | Dureza / Resistência Java (`AdditionsBlocks.java`) | Propriedades Especiais Java | Estado no Port |
|---|---|---|---|
| **Plastic Block** | Dureza 5.0, Resistência 6.0 (`:60`) | Bloco sólido opaco em 18 cores | OK |
| **Slick Plastic** | Dureza 5.0, Resistência 6.0 (`:62`) | Fricção 0.98 (`friction(0.98F)`) | OK |
| **Plastic Glow Block** | Dureza 5.0, Resistência 6.0 (`:64`) | Emissão de luz nível 10 (`lightLevel=10`), render emissivo | OK (nível 10; sem render emissivo especial) |
| **Reinforced Plastic** | Dureza 50.0, Resistência 1 200.0 (`:66`) | Alta resistência a explosões (1200) | OK |
| **Plastic Road** | Dureza 5.0, Resistência 6.0 (`:67`) | Aceleração ao pisar (`boost = 1.6`, `BlockPlasticRoad.java:19`) | LIMITE (aceleração não implementada na API) |
| **Transparent Plastic** | Dureza 5.0, Resistência 6.0 (`:69`) | Sem oclusão, brilho de sombra 0.8 (`BlockPlasticTransparent.java:24`) | OK |
| **Plastic Stairs** (comum / glow / transparente) | Base do plástico (`:71,75,77`) | Glow emite nível 10; transparent sem oclusão | OK |
| **Plastic Slabs** (comum / glow / transparente) | Base do plástico (`:72,76,79`) | Glow emite nível 10; transparent sem oclusão | OK |
| **Plastic Fence & Fence Gate** | Dureza 5.0, Resistência 6.0 (`:73,74`) | Conecta com tag `#mekanismadditions:plastic_fences` | OK |
| **Glow Panel** | Dureza 1.0, Resistência 6.0 (`BlockGlowPanel.java:35`) | Nível de luz 15 (`AdditionsBlockTypes.java:21`), orientação na face clicada | LIMITE (luz 15 OK; nasce virado para norte) |
| **Obsidian TNT** | Delay 100 ticks (5s), Raio 12.0 (`AdditionsConfig.java:41,43`) | TNT resistente a explosão | LIMITE (não portado) |
| **Baby Mobs** | Multiplicador de dano de flecha 0.25 (`AdditionsConfig.java:56`) | 6 tipos de mobs bebês | LIMITE (não portado) |
