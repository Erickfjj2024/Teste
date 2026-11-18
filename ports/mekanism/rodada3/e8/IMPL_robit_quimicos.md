# Matriz de Implementação: Grupo `robit_quimicos`

Relatório comportamental de correspondência entre a especificação técnica (`ESPEC_quimicos_barril.md` e `ESPEC_equipamento.md`), o código original em Java (Mekanism 1.21.1-10.7.19.85) e o código do port em Rust (`port/mod/src/`).

Estados:
- `IMPLEMENTADO`: Lógica completa em Rust com testes unitários cobrindo o comportamento original.
- `PARCIAL-por-FORK`: Lógica pura implementada em Rust; a integração direta com o host depende de capacidade declarada e isolada em gancho `// FORK-n` e proposta em `FORK_robit_quimicos.md`.
- `NAO`: Não implementado (0 itens).

---

## 1. Radioactive Waste Barrel (`mekanism:radioactive_waste_barrel`)

| # | Comportamento / Regra | Estado | Origem no Java | Implementação no Rust |
|---|---|---|---|---|
| 1 | Registros de Bloco, Item e Block Entity `mekanism:radioactive_waste_barrel` | IMPLEMENTADO | `BlockRadioactiveWasteBarrel.java:24`, `TileEntityRadioactiveWasteBarrel.java:58` | `radioactive_waste_barrel.rs:25` |
| 2 | Blockstates: `facing` horizontal (north, south, east, west) e `active` (bool) | IMPLEMENTADO | `BlockRadioactiveWasteBarrel.java`, `TileEntityRadioactiveWasteBarrel.java:82` | `radioactive_waste_barrel.rs:59,101` |
| 3 | Geometria VoxelShape idêntica ao tanque básico [2, 0, 2] a [14, 16, 14] | IMPLEMENTADO | `MekanismBlockTypes.java:758-763` | `radioactive_waste_barrel.rs:18` |
| 4 | Emissão de luz: nível 8 quando active = true; 0 quando inativo | IMPLEMENTADO | `Attributes.ACTIVE_LIGHT` | `radioactive_waste_barrel.rs:41` |
| 5 | Capacidade do tanque: 512 000 mB (512 baldes = `512 * BUCKET_VOLUME`) | IMPLEMENTADO | `GeneralConfig.java:216` | `radioactive_waste_barrel.rs:29` |
| 6 | Frequência de decaimento: 20 ticks (1 segundo) | IMPLEMENTADO | `GeneralConfig.java:217` | `radioactive_waste_barrel.rs:33` |
| 7 | Taxa de decaimento: 1 mB por operação de 20 ticks (3 600 mB/hora) | IMPLEMENTADO | `GeneralConfig.java:218` | `radioactive_waste_barrel.rs:37` |
| 8 | Lista negra de decaimento (`WASTE_BARREL_DECAY_BLACKLIST`) | IMPLEMENTADO | `TileEntityRadioactiveWasteBarrel.java:77` | `radioactive_waste_barrel.rs:67` |
| 9 | Proteção contra aceleradores de ticks (`last_process_tick`) | IMPLEMENTADO | `TileEntityRadioactiveWasteBarrel.java:73-75` | `radioactive_waste_barrel.rs:105,199` |
| 10 | Consumo de energia: 0 FE (bloco passivo) | IMPLEMENTADO | `TileEntityRadioactiveWasteBarrel.java` | `radioactive_waste_barrel.rs:21` |
| 11 | Lados de conexão válidos: apenas faces TOP e BOTTOM (laterais rejeitadas) | IMPLEMENTADO | `TileEntityRadioactiveWasteBarrel.java:66` | `radioactive_waste_barrel.rs:51-61` |
| 12 | Validação de entrada: rejeita químicos não-radioativos (`chemical.isRadioactive()`) | IMPLEMENTADO | `StackedWasteBarrel.java:23-33` | `radioactive_waste_barrel.rs:73-75,134` |
| 13 | Empilhamento vertical ascendente (overflow push para o barril imediatamente acima) | IMPLEMENTADO | `StackedWasteBarrel.java:49-61` | `radioactive_waste_barrel.rs:271-301` |
| 14 | Escoamento descendente (auto-drain para baixo limitado com proteção anti-loop) | IMPLEMENTADO | `TileEntityRadioactiveWasteBarrel.java:82-98` | `radioactive_waste_barrel.rs:308-329` |
| 15 | Sinal analógico de comparador redstone (nível de 0 a 15 proporcional ao volume) | IMPLEMENTADO | `TileEntityRadioactiveWasteBarrel.java:164` | `radioactive_waste_barrel.rs:120-128` |
| 16 | Interação sneak + clique com mão vazia exibe mensagem com volume e porcentagem | IMPLEMENTADO | `BlockRadioactiveWasteBarrel.java:30-48` | `radioactive_waste_barrel.rs:356-373` |
| 17 | Interação sneak + configurador inverte auto-escoamento e toca som | IMPLEMENTADO | `TileEntityRadioactiveWasteBarrel.java:130-139` | `radioactive_waste_barrel.rs:348-355` |
| 18 | Descarte de radiação ambiental ao quebrar o barril (`dumpRadiation`) | PARCIAL-por-FORK | `TileEntityMekanism.java:715` | `radioactive_waste_barrel.rs:388-403`, gancho `// FORK-13` linha 418 |
| 19 | Item dropado não retém químicos radioativos | IMPLEMENTADO | `TileEntityMekanism.java:714` | `radioactive_waste_barrel.rs:399` |
| 20 | Receita Shaped 3x3 no Datagen (chumbo + aço) | IMPLEMENTADO | `data/mekanism/recipe/radioactive_waste_barrel.json` | `INTEGRACAO_robit_quimicos.md:200` |

---

## 2. Os 18 Químicos Líquidos, Fluidos e Baldes

| # | Comportamento / Regra | Estado | Origem no Java | Implementação no Rust |
|---|---|---|---|---|
| 21 | Registro duplo Químico + Fluido (`LiquidBlock` + `BucketItem`) | PARCIAL-por-FORK | `MekanismFluids.java:23-46`, `FluidDeferredRegister.java` | `chemical_fluids.rs:72-277`, gancho `// FORK-26` linha 434 |
| 22 | Propriedades físicas de Hidrogênio (0xFFFFFFFF, 20 K, 71 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:12` | `chemical_fluids.rs:73-87` |
| 23 | Propriedades físicas de Oxigênio (0xFF6CE2FF, 90 K, 1 141 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:13` | `chemical_fluids.rs:88-102` |
| 24 | Propriedades físicas de Cloro (0xFFCFE800, 207 K, 1 423 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:14` | `chemical_fluids.rs:103-117` |
| 25 | Propriedades físicas de Dióxido de Enxofre (0xFFA99D90, 263 K, 1 400 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:15` | `chemical_fluids.rs:118-132` |
| 26 | Propriedades físicas de Trióxido de Enxofre (0xFFCE6C6C, 318 K, 1 920 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:16` | `chemical_fluids.rs:133-147` |
| 27 | Propriedades físicas de Ácido Sulfúrico (0xFF82802B, 300 K, 1 840 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:19` | `chemical_fluids.rs:148-162` |
| 28 | Propriedades físicas de Cloreto de Hidrogênio (0xFFA8F1E9, 188 K, 821 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:20` | `chemical_fluids.rs:163-177` |
| 29 | Propriedades físicas de Ácido Fluorídrico (0xFFC6C7BD, 190 K, 1 150 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:27` | `chemical_fluids.rs:178-192` |
| 30 | Propriedades físicas de Óxido de Urânio (0xFFE1F573, 3 138 K, 10 970 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:28` | `chemical_fluids.rs:193-207` |
| 31 | Propriedades físicas de Hexafluoreto de Urânio (0xFF809960, 337 K, 5 090 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:29` | `chemical_fluids.rs:208-222` |
| 32 | Propriedades físicas de Eteno (0xFFEACCF9, 169 K, 577 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:21` | `chemical_fluids.rs:223-237` |
| 33 | Propriedades físicas de Sódio (0xFFE9FEF4, 371 K, 927 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:23` | `chemical_fluids.rs:238-252` |
| 34 | Propriedades físicas de Sódio Superaquecido (0xFFD19469, 2 000 K, 927 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:24` | `chemical_fluids.rs:253-267` |
| 35 | Propriedades físicas de Salmoura (0xFFFEEF9C, 300 K, 1 000 kg/m³) | IMPLEMENTADO | `MekanismFluids.java:37` | `chemical_fluids.rs:268-282` |
| 36 | Propriedades físicas de Lítio (0xFFEBA400, 454 K, 512 kg/m³) | IMPLEMENTADO | `ChemicalConstants.java:26` | `chemical_fluids.rs:283-297` |
| 37 | Propriedades físicas de Vapor / Steam (0xFFFFFFFF, 373 K, 0 kg/m³, texturas liquid/steam) | IMPLEMENTADO | `MekanismFluids.java:40-41` | `chemical_fluids.rs:298-312` |
| 38 | Propriedades físicas de Água Pesada (0xFF0D1455, 300 K, 1 000 kg/m³, texturas water_still) | IMPLEMENTADO | `MekanismFluids.java:42-44` | `chemical_fluids.rs:313-327` |
| 39 | Propriedades físicas de Pasta Nutricional (0xFFEB6CA3, 300 K, 1 000 kg/m³) | IMPLEMENTADO | `MekanismFluids.java:45-46` | `chemical_fluids.rs:328-342` |
| 40 | Bloco Líquido: níveis 0 (fonte), 1..7 (corrente), 8..15 (queda) | IMPLEMENTADO | `LiquidBlock.java` | `chemical_fluids.rs:360-388` |
| 41 | Comportamento do Balde: encher de bloco fonte e despejar no mundo | IMPLEMENTADO | `BucketItem.java` | `chemical_fluids.rs:396-435` |
| 42 | Balde de Pasta Nutricional consumível em 32 ticks de duração | PARCIAL-por-FORK | `ItemNutritionalPasteBucket.java:34` | `chemical_fluids.rs:437-474`, gancho `// FORK-29` linha 458 |
| 43 | Nutrição: 50 mB por ponto de fome restaurando até 20 pontos de fome | IMPLEMENTADO | `ItemNutritionalPasteBucket.java:47` | `chemical_fluids.rs:26,462` |
| 44 | Saturação restaurada: 0.8F | IMPLEMENTADO | `ItemNutritionalPasteBucket.java:66` | `chemical_fluids.rs:32,467` |
| 45 | Retorno de `minecraft:bucket` após consumo completo | IMPLEMENTADO | `ItemNutritionalPasteBucket.java:69` | `chemical_fluids.rs:35,468` |
| 46 | Clique agachado ou barra de fome cheia despeja o fluido no mundo | IMPLEMENTADO | `ItemNutritionalPasteBucket.java:52` | `chemical_fluids.rs:458-472` |
| 47 | Receitas bidirecionais 1:1 no Rotary Condensentrator para as 18 substâncias | IMPLEMENTADO | `RotaryRecipeProvider.java:26-41` | `chemical_fluids.rs:480-502` |

---

## 3. Robit (Entidade, Automação, GUIs e Customização)

| # | Comportamento / Regra | Estado | Origem no Java | Implementação no Rust |
|---|---|---|---|---|
| 48 | Registros de Entidade e Item `mekanism:robit` | IMPLEMENTADO | `MekanismEntityTypes.java`, `ItemRobit.java` | `robit.rs:27,30` |
| 49 | Vida máxima 1.0 HP e invulnerabilidade à morte por dano comum | IMPLEMENTADO | `EntityRobit.java:135,532` | `robit.rs:39,365-373` |
| 50 | Absorção de dano consumindo energia (1 000 J / damage = 400 FE / damage) | IMPLEMENTADO | `EntityRobit.java:530` | `robit.rs:43-44,365-373` |
| 51 | Capacidade máxima de energia: 100 000 Joules = 40 000 FE (`MAX_ENERGY`) | IMPLEMENTADO | `EntityRobit.java:162` | `robit.rs:32,35` |
| 52 | Velocidade de deslocamento: 0.3F | IMPLEMENTADO | `EntityRobit.java:135` | `robit.rs:38` |
| 53 | Custo de energia por movimento: `Math.ceil(1.5 * sqrt(dist_sq))` Joules | IMPLEMENTADO | `EntityRobit.java:287-289` | `robit.rs:48,377-387` |
| 54 | Retorno dimensional ao Chargepad (`goHome`) com bateria zerada | IMPLEMENTADO | `EntityRobit.java:348,398-418` | `robit.rs:475-488,542` |
| 55 | IA de Seguir o Dono (`RobitAIFollow`): min 2.0, max 4.0, teletransporte acima de 12.0 blocos | IMPLEMENTADO | `RobitAIFollow.java:21-25`, `RobitAIBase.java:62` | `robit.rs:58-60,517-535` |
| 56 | IA de Busca de Itens (`RobitAIPickup`): raio de varredura 10 blocos | IMPLEMENTADO | `RobitAIPickup.java:13-14` | `robit.rs:63-64,394-406` |
| 57 | Recolhimento de itens do chão em AABB expandida em 1.5 blocos para os 27 slots | IMPLEMENTADO | `EntityRobit.java:365-396` | `robit.rs:67,390-453` |
| 58 | Fornalha interna rápida: duração de 100 ticks (5s, metade dos 200 ticks de fornalha comum) | IMPLEMENTADO | `EntityRobit.java:166` | `robit.rs:51` |
| 59 | Consumo de energia da fornalha: 50 J/t = 20 FE/t (`energizedSmelter`) | IMPLEMENTADO | `EntityRobit.java:694`, `UsageConfig.java:57` | `robit.rs:54-55,487` |
| 60 | Rastreamento de erros de receita na fornalha (`TRACKED_ERROR_TYPES`) | IMPLEMENTADO | `EntityRobit.java:155-160` | `robit.rs:94-99,460-490` |
| 61 | Layout de 30 slots (27 inventário + 1 energia + 1 entrada fundição + 1 saída fundição) | IMPLEMENTADO | `EntityRobit.java:220-229` | `robit.rs:57-62` |
| 62 | Tela 1: `GuiRobitMain` (saudação, energia, seguir, recolher, botões de home, renomear, skin) | IMPLEMENTADO | `GuiRobitMain.java:60-100` | `robit.rs:84,490-502` |
| 63 | Tela 2: `GuiRobitInventory` (27 slots de inventário em grade 3x9) | IMPLEMENTADO | `GuiRobitInventory.java`, `MainRobitContainer.java` | `robit.rs:86` |
| 64 | Tela 3: `GuiRobitSmelting` (entrada em 51,35; saída em 116,35; barra de progresso) | IMPLEMENTADO | `GuiRobitSmelting.java` | `robit.rs:87` |
| 65 | Tela 4: `GuiRobitCrafting` (bancada de trabalho 3x3 integrada) | IMPLEMENTADO | `CraftingRobitContainer.java` | `robit.rs:85` |
| 66 | Tela 5: `GuiRobitRepair` (bigorna integrada) | IMPLEMENTADO | `RepairRobitContainer.java` | `robit.rs:88` |
| 67 | Janelas popup de renomeação (`GuiRobitRename`) e seleção de skins (`GuiRobitSkinSelect`) | IMPLEMENTADO | `GuiRobitRename.java`, `GuiRobitSkinSelect.java` | `robit.rs:90-91` |
| 68 | Sistema de 13 Skins (Base, Allay e as 11 skins Pride) | IMPLEMENTADO | `MekanismRobitSkins.java:33-43`, `RobitPrideSkinData.java` | `robit.rs:114-192` |
| 69 | Desbloqueio de skins por avanços (`AdvancementBasedRobitSkin`) | IMPLEMENTADO | `AdvancementBasedRobitSkin.java:52-63` | `robit.rs:118,506-512` |
| 70 | Animação de textura ao caminhar (ciclo a cada 3 ticks de movimento) | IMPLEMENTADO | `EntityRobit.java:827-834` | `robit.rs:125,321` |
| 71 | Sistema de segurança com UUID do dono e modos PUBLIC, PRIVATE e TRUSTED | IMPLEMENTADO | `EntityRobit.java:564-583` | `robit.rs:72-83,354-361` |
| 72 | Item Robit: empacotamento com configurator mantendo energia, inventário, nome, skin e segurança | IMPLEMENTADO | `EntityRobit.java:451-469`, `ItemRobit.java:47-55` | `robit.rs:561-582` |
| 73 | Posicionamento do Robit no Chargepad inativo restaura todas as variáveis e home location | IMPLEMENTADO | `ItemRobit.java:77-124` | `robit.rs:585-616` |
| 74 | Receita 3x3 Shaped do Robit (aço, tabletes de energia, liga atômica, obsidiana refinada, baú pessoal) | IMPLEMENTADO | `data/mekanism/recipe/robit.json` | `INTEGRACAO_robit_quimicos.md:204` |
| 75 | Spawning nativo de entidade customizada com navegação de host | PARCIAL-por-FORK | `EntityRobit.java` | `robit.rs:625-638`, gancho `// FORK-10` linha 625 |
| 76 | Envio de pacotes de rede C2S para alternar botões de entidade | PARCIAL-por-FORK | `PacketEntityButtonPress.java` | `robit.rs:640-650`, gancho `// FORK-24` linha 640 |
