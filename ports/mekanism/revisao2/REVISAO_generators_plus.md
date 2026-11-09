# Relatório de Revisão Cruzada (Rodada 2): Mekanism Generators, Tools e Additions

- **Tarefa:** T20261109-195057-kwj (Revisão Cruzada da t6)
- **Data:** 2026-11-09
- **Escopo:** `generators.rs`, `plus.rs`, `convert.py`, MekanismGenerators, MekanismTools e MekanismAdditions
- **Fontes Java de Referência (Mekanism 1.21.1-10.7.19.85):**
  - `java/generators/java/mekanism/generators/common/config/GeneratorsConfig.java`
  - `java/generators/java/mekanism/generators/common/config/GeneratorsStorageConfig.java`
  - `java/generators/java/mekanism/generators/common/registries/GeneratorsBlockTypes.java`
  - `java/generators/java/mekanism/generators/common/tile/TileEntity*.java`
  - `java/tools/java/mekanism/tools/common/material/impl/*.java`
  - `java/tools/java/mekanism/tools/common/material/BaseMekanismMaterial.java`
  - `java/tools/java/mekanism/tools/common/material/IPaxelMaterial.java`
  - `java/tools/java/mekanism/tools/common/registries/ToolsArmorMaterials.java`
  - `java/tools/java/mekanism/tools/common/registries/ToolsItems.java`
  - `java/additions/java/mekanism/additions/common/registries/AdditionsBlocks.java`
  - `java/additions/java/mekanism/additions/common/registries/AdditionsBlockTypes.java`
  - `java/additions/java/mekanism/additions/common/config/AdditionsConfig.java`
  - `java/api/java/mekanism/api/text/EnumColor.java`
  - `java/api/java/mekanism/api/heat/HeatAPI.java`

---

## 1. Avaliação das Correções da Rodada 1

| # | Item da Rodada 1 | Valor e Lógica Java (arquivo:linha) | Código Port (arquivo:linha) | Veredito | Detalhes da Auditoria |
|---|---|---|---|---|---|
| R1.1 | **Advanced Solar: Faces de Saída (`energy_faces`)** | `TileEntityAdvancedSolarGenerator.java:18`<br>`ENERGY_SIDES = {RelativeSide.FRONT, RelativeSide.BOTTOM}` | `generators.rs:225-228`<br>`faces[DOWN] = Output; faces[FRONT] = Output;` | **CONFIRMADA** | No Java, emite energia pela base e pela frente. A rodada 1 agrupou com Wind emitindo em DOWN e FRONT (rotacionada com `facing`). Válido. |
| R1.2 | **Advanced Solar: Orientação ao Posicionar (`run_generator`)** | `GeneratorsBlockTypes.java:132-136`<br>`Generator.java:15`<br>`AttributeStateFacing(HORIZONTAL_FACING, PLAYER_LOOK)` | `generators.rs:268`<br>`if !placed && kind != GenKind::Solar` | **CONFIRMADA** | No Java, o bloco tem face frontal e vira para o jogador. O Solar simples só emite por baixo, mas o AdvSolar precisa virar pois sua face frontal emite energia. Válido. |
| R1.3 | **Solar & Adv Solar: Penalidade de Chuva (`run_generator`)** | `TileEntitySolarGenerator.java:181-183`<br>`if (needsRainCheck && (isRaining() \|\| isThundering())) return peakMultiplier * 0.2F;` | `generators.rs:401-403`<br>`if env.precipitation && (env.rain > 0.0 \|\| env.thunder > 0.0) { share *= 0.2; }` | **CONFIRMADA** | 80% de penalidade (`* 0.2F`) na produção solar em biomas onde pode chover. Válido. |
| R1.4 | **Solar Neutron Activator: Penalidade de Chuva (`run_continuous`)** | `TileEntitySolarNeutronActivator.java:200-202`<br>`if (needsRainCheck && (world.isRaining() \|\| world.isThundering())) production *= 0.2F;` | `plus.rs:163-165`<br>`if env.precipitation && (env.rain > 0.0 \|\| env.thunder > 0.0) { share *= 0.2; }` | **CONFIRMADA** | 80% de penalidade aplicada na taxa de operação (`rate = 64.0 * share * brightness`). Válido. |

---

## 2. Auditoria Completa dos Geradores (MekanismGenerators)

| Item | Valor Java (arquivo:linha) | Valor Port (arquivo:linha) | Estado | Análise Rodada 2 |
|---|---|---|---|---|
| **Bio Gen: Produção** | 350 J/t (`GeneratorsConfig.java:91`) | `fe(350)` = 140 FE/t (`generators.rs:29`) | OK | `fe(350)` = 140 FE/t exato. |
| **Bio Gen: Capacidade Tanque** | 24 000 mB (`GeneratorsConfig.java:93`) | 24 000 mB (`generators.rs:30`) | OK | Exato. |
| **Bio Gen: Bio Fuel (item)** | 200 mB (`TileEntityBioGenerator.java:62`) | 200 mB (`generators.rs:31`) | OK | Exato. |
| **Bio Gen: Bio Fuel (bloco)** | 1 800 mB (`TileEntityBioGenerator.java:62`) | 1 800 mB (`generators.rs:129`) | OK | Exato. |
| **Bio Gen: Armazenamento Energia** | 160 000 J (`GeneratorsStorageConfig.java:24`) | `fe(160_000)` = 64 000 FE (`generators.rs:52`) | OK | Exato. |
| **Bio Gen: Consumo de Combustível** | 1 mB/t (`TileEntityBioGenerator.java:77`) | 1 mB/t (`generators.rs:322`) | OK | Reduz 1 mB por tick de geração ativa. |
| **Bio Gen: Condição de Disparo** | `room(energy) >= bioGeneration` (`TileEntityBioGenerator.java:75`) | `room(energy) >= BIO_GENERATION` (`generators.rs:320`) | OK | Exige espaço completo para 1 tick de produção antes de queimar. |
| **Bio Gen: Drenagem para Item no Slot 1** | `energySlot.drainContainer()` (`TileEntityBioGenerator.java:72`) | Slot 1 exposto no lado direito (`generators.rs:208`), sem suporte a item de energia | FORK | Exige capacidade de energia em itens (FORK-23/FORK-27). |
| **Heat Gen: Queima de Lava** | 200 J/t (`GeneratorsConfig.java:98`) | 200.0 (`generators.rs:34`) | OK | Exato. |
| **Heat Gen: Taxa Consumo Lava** | 10 mB/t (`GeneratorsConfig.java:106`) | 10 (`generators.rs:35`) | OK | Exato. |
| **Heat Gen: Boost Bloco Lava Vizinho** | 30 J/t (`GeneratorsConfig.java:100`) | 30.0 (`generators.rs:36`) | OK | Exato. |
| **Heat Gen: Boost Nether** | 100 J/t (`GeneratorsConfig.java:102`) | 100.0 (`generators.rs:37`) | OK | Exato. |
| **Heat Gen: Capacidade Tanque** | 24 000 mB (`GeneratorsConfig.java:104`) | 24 000 (`generators.rs:38`) | OK | Exato. |
| **Heat Gen: Produção Máxima** | 510 J/t (`TileEntityHeatGenerator.java:55-59`) | 510 J (`generators.rs:71`) | OK | `fe(510)` = 204 FE/t teto. |
| **Heat Gen: Capacidade Térmica** | 10.0 (`TileEntityHeatGenerator.java:50`) | 10.0 (`generators.rs:58`) | OK | Exato. |
| **Heat Gen: Condução Inversa** | 5.0 (`TileEntityHeatGenerator.java:51`) | 5.0 (`generators.rs:65`) | OK | Exato. |
| **Heat Gen: Isolamento (5 lados)** | 100.0 (`TileEntityHeatGenerator.java:52`) | 100.0 (`generators.rs:66`) | OK | Exato. |
| **Heat Gen: Isolamento (fundo)** | 0.0 (`HeatAPI.java:36`, `TileEntityHeatGenerator.java:173`) | 0.0 (`generators.rs:97`) | OK | Exato. |
| **Heat Gen: Ar Condução Inversa** | 10 000.0 (`HeatAPI.java:24`) | 10 000.0 (`generators.rs:67`) | OK | Exato. |
| **Heat Gen: Eficiência Térmica** | 0.5 (`TileEntityHeatGenerator.java:53`) | 0.5 (`generators.rs:68`) | OK | Exato. |
| **Heat Gen: Eficiência de Carnot** | `1 - min(Ta, T)/max(Ta, T)` (`TileEntityHeatGenerator.java:187`) | `1.0 - temp.min(ambient) / temp.max(ambient)` (`generators.rs:91`) | OK | Exato. |
| **Heat Gen: Temperatura Ambiente** | 300 K base, 25 K/grau em clamp [-5, 5] (`HeatAPI.java:20,81`) | `ambient_of` (`generators.rs:62-64`) | OK | Exato. |
| **Heat Gen: Itens de Combustível** | `burnTime / 20` mB (`TileEntityHeatGenerator.java:98`) | `1_000 * burn / 20_000` (`generators.rs:122`) | OK | Exato. |
| **Solar Gen: Produção Pico** | 50 J/t (`GeneratorsConfig.java:85`) | 50.0 J -> `fe(...)` = 20 FE/t (`generators.rs:41,183`) | OK | Exato. |
| **Solar Gen: Armazenamento Energia** | 96 000 J (`GeneratorsStorageConfig.java:26`) | `fe(96_000)` = 38 400 FE (`generators.rs:53`) | OK | Exato. |
| **Solar Gen: Faces de Energia** | BOTTOM (`TileEntitySolarGenerator.java:33`) | DOWN (`generators.rs:224`) | OK | Exato. |
| **Adv Solar: Produção Pico** | 300 J/t (`GeneratorsConfig.java:86`) | 300.0 J -> `fe(...)` = 120 FE/t (`generators.rs:43,188`) | OK | Exato. |
| **Adv Solar: Armazenamento Energia** | 200 000 J (`GeneratorsStorageConfig.java:28`) | `fe(200_000)` = 80 000 FE (`generators.rs:54`) | OK | Exato. |
| **Adv Solar: Matriz de 9 Posições** | Média de 9 blocos no topo Y+2 (`TileEntityAdvancedSolarGenerator.java:43-52,87-94`) | Checa apenas topo do bloco central (`generators.rs:397`) | FORK | No port checa apenas o bloco do gerador sob o céu; simplificação deliberada do WASM. |
| **Wind Gen: Produção Mín / Máx** | 60 J/t / 480 J/t (`GeneratorsConfig.java:115,117`) | 60.0 / 480.0 J (`generators.rs:44-45`) | OK | `fe(60)` = 24 FE/t; `fe(480)` = 192 FE/t. |
| **Wind Gen: Faixa Y** | Y 24 a 319 (`GeneratorsConfig.java:119,123`) | 24 a 319 (`generators.rs:46-47`) | OK | Exato. |
| **Wind Gen: Armazenamento Energia** | 200 000 J (`GeneratorsStorageConfig.java:30`) | `fe(200_000)` = 80 000 FE (`generators.rs:55`) | OK | Exato. |
| **Wind Gen: Estado Ativo quando Cheio** | `setActive(canFunction() && currentMultiplier != 0L)` (`TileEntityWindGenerator.java:67`) | `active = made > 0;` (antes) -> corrigido para `active = true;` quando `open && can` | **ERRADA-CORRIGIDA** | O port desligava (`lit=false`) e travava a animação do moinho quando o buffer de energia enchia (`made == 0`). No Java o moinho gira continuamente com o vento. Corrigido em `generators.rs:391`. |
| **Gas Gen: Capacidade Tanque** | 18 000 mB (`GeneratorsConfig.java:111`) | 18 000 mB (`generators.rs:49`) | OK | Exato. |
| **Gas Gen: Armazenamento Energia** | 1 000 × densidade H2 = 200 000 J (`GeneratorsBlockTypes.java:123`) | `fe(1_000 * 200)` = 80 000 FE (`generators.rs:56`) | OK | Exato. |
| **Gas Gen: Taxa Queima Máxima** | 256 ticks/t (`TileEntityGasGenerator.java:136`) | 256.0 (`generators.rs:153`) | OK | Exato. |
| **Gas Gen: Hidrogênio** | 1 tick, 200 J/tick (`chemical_attribute_fuel.json:8-9`) | (1 tick, `fe(200)` = 80 FE) (`generators.rs:138`) | OK | Exato. |
| **Gas Gen: Etileno (Ethene)** | 40 ticks, 705 J/tick (`chemical_attribute_fuel.json:4-5`) | (40 ticks, `fe(705)` = 282 FE) (`generators.rs:139`) | OK | Exato. |
| **Gas Gen: Persistência de `burnTicks`** | Não persiste em NBT (`TileEntityGasGenerator.java:52`) | Mantido em memória (`self.burns`, `generators.rs:368-377`) | OK | No Java `burnTicks` é puramente em RAM e reseta ao recarregar chunk. O port emula perfeitamente. |
| **Solar Neutron Activator: Taxa Base** | 64 operações/t (`GeneralConfig.java:141`) | 64.0 (`plus.rs:166`) | OK | Exato. |
| **Solar Neutron Activator: Sub-1.0 Taxa** | Executa 1 op a cada `ceil(1/rate)` ticks (`TileEntitySolarNeutronActivator.java:215`) | `if rate >= 1.0 { ops = rate as u32; }` (`plus.rs:167`) | FORK | No port desliga quando `rate < 1.0` em vez de acumular frações de operação. |

---

## 3. Auditoria MekanismTools

### 3.1 Materiais e Propriedades de Ferramentas e Armaduras

| Material | Uses (Durab.) Java | Velocidade Java | Bônus Dano Java | Encant. Ferramenta Java | Encant. Armadura Java (`ToolsArmorMaterials:39`) | Encant. Armadura Port (`convert.py:2371-2376`) | Toughness Java | Knockback Resist Java | Estado |
|---|---|---|---|---|---|---|---|---|---|
| **Bronze** | 375 (`BronzeMaterialDefaults:34`) | 7.0 (`:39`) | 2.0 (`:44`) | 10 (`:49`) | **10** | 5 (antes) -> **10** (corrigido) | 1.0 (`:54`) | 0.0 (`:104`) | **CORRIGIDO** |
| **Lapis Lazuli** | 128 (`LapisLazuliMaterialDefaults:35`) | 9.0 (`:40`) | 1.0 (`:45`) | 32 (`:50`) | **32** | 35 (antes) -> **32** (corrigido) | 0.0 (`:55`) | 0.0 (`:103`) | **CORRIGIDO** |
| **Osmium** | 1 024 (`OsmiumMaterialDefaults:36`) | 4.0 (`:41`) | 4.0 (`:46`) | 14 (`:51`) | **14** | 9 (antes) -> **14** (corrigido) | 3.0 (`:56`) | 0.1 (`:106`) | **CORRIGIDO** |
| **Refined Glowstone** | 384 (`RefinedGlowstoneMaterialDefaults:34`) | 15.0 (`:39`) | 2.0 (`:44`) | 20 (`:49`) | **20** | 23 (antes) -> **20** (corrigido) | 0.0 (`:54`) | 0.0 (`:103`) | **CORRIGIDO** |
| **Refined Obsidian** | 4 096 (`RefinedObsidianMaterialDefaults:35`) | 12.0 (`:40`) | 8.0 (`:45`) | 18 (`:49`) | **18** | 13 (antes) -> **18** (corrigido) | 5.0 (`:59`) | 0.2 (`:109`) | **CORRIGIDO** |
| **Steel** | 500 (`SteelMaterialDefaults:34`) | 8.0 (`:39`) | 3.0 (`:44`) | 16 (`:49`) | **16** | 11 (antes) -> **16** (corrigido) | 2.0 (`:54`) | 0.0 (`:103`) | **CORRIGIDO** |

*Nota da Rodada 2:* No Java 1.21.1 (`ToolsArmorMaterials.java:39`), a encantabilidade da armadura usa diretamente `material.getEnchantmentValue()`. O `convert.py` continha os valores legados do Mekanism 26.3 (`5, 35, 9, 23, 13, 11`), corrigidos agora para alinhar com o Java 1.21.1.

### 3.2 Armaduras (Durabilidade e Defesa por Peça: B / L / C / H)

| Material | Durabilidade Armadura Java (B / L / C / H) | Defesa Armadura Java (B / L / C / H) | Som de Equipar Java | Resistente ao Fogo Java | Estado |
|---|---|---|---|---|---|
| **Bronze** | 234 / 270 / 288 / 198 (`:60-64`) | 2 / 6 / 7 / 3 (`:70-74`) | `ARMOR_EQUIP_IRON` | Não | OK |
| **Lapis Lazuli** | 130 / 150 / 160 / 110 (`:60-64`) | 1 / 3 / 4 / 1 (`:70-73`) | `ARMOR_EQUIP_DIAMOND` | Não | OK |
| **Osmium** | 390 / 450 / 480 / 330 (`:62-66`) | 3 / 6 / 8 / 4 (`:72-76`) | `ARMOR_EQUIP_IRON` | Não | OK |
| **Refined Glowstone** | 221 / 255 / 272 / 187 (`:60-64`) | 3 / 6 / 8 / 3 (`:70-74`) | `ARMOR_EQUIP_IRON` | Não | OK |
| **Refined Obsidian** | 975 / 1 125 / 1 200 / 825 (`:65-69`) | 5 / 8 / 12 / 6 (`:75-79`) | `ARMOR_EQUIP_IRON` | **Sim** (`burnsInFire=false`, `ToolsItems:168`) | FORK (fogo) |
| **Steel** | 260 / 300 / 320 / 220 (`:60-64`) | 3 / 6 / 8 / 3 (`:70-74`) | `ARMOR_EQUIP_IRON` | Não | OK |

### 3.3 Paxels dos Materiais Mekanism e Vanilla

| Paxel | Durabilidade (2 × Uses) | Dano Ataque Total (Hand=1 + Mod) | Velocidade Ataque Java (`IPaxelMaterial:11`) | Velocidade Ataque Port (`convert.py`) | Estado |
|---|---|---|---|---|---|
| **Bronze Paxel** | 750 (2 × 375) | 11.0 (1 + 2 + 8) | **-2.4F** (1.6 vel) | `axe_s` = -3.0 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Lapis Paxel** | 256 (2 × 128) | 7.0 (1 + 1 + 5) | **-2.4F** (1.6 vel) | `axe_s` = -2.9 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Osmium Paxel** | 2 048 (2 × 1 024) | 14.0 (1 + 4 + 9) | **-2.4F** (1.6 vel) | `axe_s` = -3.3 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Glowstone Paxel** | 768 (2 × 384) | 10.0 (1 + 2 + 7) | **-2.4F** (1.6 vel) | `axe_s` = -2.9 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Obsidian Paxel** | 8 192 (2 × 4 096) | 17.0 (1 + 8 + 8) | **-2.4F** (1.6 vel) | `axe_s` = -2.9 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Steel Paxel** | 1 000 (2 × 500) | 12.0 (1 + 3 + 8) | **-2.4F** (1.6 vel) | `axe_s` = -3.0 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Wood Paxel** | 118 (2 × 59) | 8.0 (1 + 0 + 7) | **-2.4F** (1.6 vel) | `axe_s` = -3.2 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Stone Paxel** | 262 (2 × 131) | 10.0 (1 + 1 + 8) | **-2.4F** (1.6 vel) | `axe_s` = -3.2 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Iron Paxel** | 500 (2 × 250) | 10.0 (1 + 2 + 7) | **-2.4F** (1.6 vel) | `axe_s` = -3.1 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Diamond Paxel** | 3 122 (2 × 1 561) | 10.0 (1 + 3 + 6) | **-2.4F** (1.6 vel) | `axe_s` = -3.0 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Gold Paxel** | 64 (2 × 32) | 8.0 (1 + 0 + 7) | **-2.4F** (1.6 vel) | `axe_s` = -3.0 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |
| **Netherite Paxel** | 4 062 (2 × 2 031) | 11.0 (1 + 4 + 6) | **-2.4F** (1.6 vel) | `axe_s` = -3.0 (antes) -> **-2.4** (corrigido) | **CORRIGIDO** |

*Nota da Rodada 2:* No Java (`IPaxelMaterial.java:11`), todos os paxéis herdam `getPaxelAtkSpeed() = -2.4F`. No `convert.py`, foi inserido por engano `axe_s` (velocidade lenta do machado vanilla: -3.0 a -3.3). Corrigido em `convert.py:2421` e `convert.py:2439`.

---

## 4. Auditoria MekanismAdditions

| Item | Valor Java (arquivo:linha) | Valor Port | Estado | Análise Rodada 2 |
|---|---|---|---|---|
| **18 Cores EnumColor** | 18 cores oficiais (`EnumColor.java:28-46`) | 18 cores em `convert.py:2461-2464` e `transmitter.rs` | OK | Conforme. |
| **Plastic Block** | Dureza 5.0, Resistência 6.0 (`AdditionsBlocks.java:60`) | Dureza 5.0, Resistência 6.0 | OK | Conforme. |
| **Slick Plastic** | Fricção 0.98 (`AdditionsBlocks.java:62`) | Template `minecraft:packed_ice` (`convert.py:2468`) | OK | Conforme. |
| **Plastic Glow Block** | Luz nível 10 (`AdditionsBlocks.java:64`) | `light: 10` (`convert.py:2469`) | OK | Conforme. |
| **Reinforced Plastic** | Dureza 50.0, Resistência 1200.0 (`AdditionsBlocks.java:66`) | Dureza 50.0, Resistência 1200.0 (`convert.py:2470`) | OK | Conforme. |
| **Plastic Road** | Boost 1.6 (`BlockPlasticRoad.java:19`) | Não implementado (sem evento `stepOn` na API) | FORK | Exige evento de movimento de entidade na API Horizon. |
| **Transparent Plastic** | Sem oclusão (`BlockPlasticTransparent.java:24`) | Template `minecraft:glass` (`convert.py:2472`) | OK | Conforme. |
| **Plastic Stairs/Slabs/Fences** | Variantes coloridas (`AdditionsBlocks.java:71-79`) | Geradas via templates vanilla correspondentes | OK | Conforme. |
| **Glow Panel: Nível de Luz** | Luz nível 15 (`AdditionsBlockTypes.java:21`) | `light: 10` (antes) -> `light: 15` (corrigido) | **CORRIGIDO** | No Java o Glow Panel emite luz nível 15. No `convert.py:2481` estava fixado em 10. Corrigido para 15. |
| **Glow Panel: Orientação** | Face clicada (`AdditionsBlockTypes.java:20`) | Fixado em north / sem rotação livre na parede | FORK | Exige suporte a orientação em qualquer face do bloco suporte. |
| **Obsidian TNT** | Delay 100t, raio 12.0 (`AdditionsConfig.java:41,43`) | Não portado | FORK | Exige entidade TNT customizada e lógica de destruição. |
| **Baby Mobs** | 6 tipos de bebês (`AdditionsConfig.java:58-63`) | Não portado | FORK | Exige registro de entidades customizadas e spawn conditions. |
| **Balloons (Balões)** | 18 cores (`AdditionsItems.java:29-35`) | Não portado | FORK | Exige entidade corda/balão e física de elevação. |
| **Walkie-Talkie** | Porta 36123, 9 canais (`ItemWalkieTalkie.java`) | Não portado | FORK | Exige socket de voz / transmissão UDP client-server. |

---

## 5. Resumo das Divergências Detectadas e Correções Aplicadas

1. **`generators.rs:391` (Wind Generator - Estado Ativo quando Cheio):**
   - *Erro encontrado:* `active = made > 0;` fazia com que o gerador eólico entrasse em estado inativo (`lit=false`) e parasse de girar as pás assim que a rede/buffer de energia enchia (`made == 0`).
   - *Correção aplicada:* Alterado para `active = true;` dentro do bloco `if open && can`, garantindo que o moinho continue girando no vento mesmo com armazenamento cheio, exatamente como em `TileEntityWindGenerator.java:67`.
2. **`convert.py:2481` (Glow Panel - Nível de Luz):**
   - *Erro encontrado:* `convert.py` definia `'light': 10` para `_glow_panel`.
   - *Correção aplicada:* Alterado para `'light': 15` conforme especificado em `AdditionsBlockTypes.java:21` (`.withLight(15)`).
3. **`convert.py:2371-2376` (Tools - Encantabilidade de Armadura):**
   - *Erro encontrado:* Usava valores obsoletos do 26.3 (`5, 35, 9, 23, 13, 11`).
   - *Correção aplicada:* Alinhado com o Java 1.21.1 (`ToolsArmorMaterials.java:39`), que adota `material.getEnchantmentValue()` para as armaduras: Bronze 10, Lapis 32, Osmium 14, Refined Glowstone 20, Refined Obsidian 18, Steel 16.
4. **`convert.py:2421, 2439` (Tools - Velocidade de Ataque dos Paxéis):**
   - *Erro encontrado:* Passava `axe_s` (-3.0 a -3.3, velocidade do machado).
   - *Correção aplicada:* Alterado para `-2.4` fixo em todos os paxéis (materiais próprios e vanilla), conforme definido em `IPaxelMaterial.java:11` (`getPaxelAtkSpeed() = -2.4F`).
