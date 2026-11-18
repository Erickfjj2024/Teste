# IMPL_assemblicator_cartao.md — Matriz de Implementação

Mapeamento completo dos comportamentos especificados em `port/especificacoes/ESPEC_utilidades.md` (seções 7, 8 e 9) para o código Rust implementado e sua fonte de verdade no Java (Mekanism 1.21.1-10.7.19.85).

---

## 1. Formulaic Assemblicator

| Comportamento / Requisito | Origem no Java | Localização no Rust | Estado | Observações |
|---|---|---|---|---|
| **Registro do bloco e item** (`mekanism:formulaic_assemblicator`) | `MekanismBlocks.java:375`, `MekanismBlocks.FORMULAIC_ASSEMBLICATOR` | `assemblicator.rs:28` (`BLOCK`) | IMPLEMENTADO | Registrado com propriedades horizontal facing e active |
| **Tempo base de fabricação** (40 ticks = 2,0s) | `TileEntityFormulaicAssemblicator.java:80` (`BASE_TICKS_REQUIRED = 2 * TICKS_PER_SECOND`) | `assemblicator.rs:61` (`BASE_TICKS_REQUIRED`) | IMPLEMENTADO | Reduzido com upgrades de velocidade |
| **Consumo base de energia** (100 J/t = 40 FE/t) | `UsageConfig.formulaicAssemblicator` (100 J/t) | `assemblicator.rs:65` (`BASE_ENERGY_USAGE`) | IMPLEMENTADO | Calculado como `fe(100)` = 40 FE/t |
| **Buffer de energia** (40.000 J = 16.000 FE) | `StorageConfig.formulaicAssemblicator` (40.000 J) | `assemblicator.rs:69` (`BASE_STORAGE`) | IMPLEMENTADO | Calculado como `fe(40_000)` = 16.000 FE |
| **Estrutura de 35 Slots** (1 fórmula, 18 entrada, 9 grid, 6 saída, 1 energia) | `TileEntityFormulaicAssemblicator.java:154-190` | `assemblicator.rs:34-57` (`TOTAL_SLOTS`, `SLOT_FORMULA`, `SLOT_INPUT_START`, `SLOT_CRAFT_START`, `SLOT_OUTPUT_START`, `SLOT_ENERGY`) | PARCIAL-por-FORK | Implementação lógica completa em Rust puro; contêiner com 35 slots catalogado em `FORK-33` |
| **Slot de Fórmula** (aceita apenas `mekanism:crafting_formula`) | `TileEntityFormulaicAssemblicator.java:77, 154` (`FORMULA_SLOT_VALIDATOR`) | `assemblicator.rs:38`, `assemblicator.rs:984` | IMPLEMENTADO | Restrito a `CRAFTING_FORMULA` |
| **Configuração de lados (Side Config)** | `TileEntityFormulaicAssemblicator.java:115-121` (`setupItemIOConfig`, `setupInputConfig`) | `assemblicator.rs:615-633` (`assemblicator_item_faces`) | IMPLEMENTADO | Input: 1..18, Output: 28..33, In/Out: 1..18 + 28..33, Energy: 34, Extra: 0 |
| **Codificação de Fórmula** (`encodeFormula`) | `TileEntityFormulaicAssemblicator.java:673-686` | `assemblicator.rs:983-1006` (`encode_formula_slot`) | IMPLEMENTADO | Grava 9 stacks da grade 3x3 no item do slot 0 se houver receita válida |
| **Controle de Estoque** (`stockControl`) | `TileEntityFormulaicAssemblicator.java:523-541, 642-658` | `assemblicator.rs:642-657` (`build_stock_control_map`), `assemblicator.rs:661-681` (`is_item_valid_for_input`) | IMPLEMENTADO | Mapeia cada posição de receita para 2 slots de entrada (`j = i*2, j+1 = i*2+1`) |
| **Organização do Estoque** (`organizeStock`) | `TileEntityFormulaicAssemblicator.java:543-616` | `assemblicator.rs:926-976` (`organize_stock`) | IMPLEMENTADO | Distribui itens nos slots controlados e os excedentes nos não controlados |
| **Modo Automático** (`autoMode`) | `TileEntityFormulaicAssemblicator.java:238-264, 490-500` | `assemblicator.rs:1027-1049` (`tick_assemblicator`) | IMPLEMENTADO | Desliga automaticamente se a fórmula estiver vazia |
| **Abastecimento Automático da Grade** (`moveItemsToGrid`) | `TileEntityFormulaicAssemblicator.java:406-459` | `assemblicator.rs:685-745` (`move_items_to_grid`) | IMPLEMENTADO | Busca ingredientes da entrada de trás para frente (17 até 0) |
| **Execução de Crafting** (`doSingleCraft`) | `TileEntityFormulaicAssemblicator.java:368-396` | `assemblicator.rs:825-866` (`do_single_craft`) | IMPLEMENTADO | Consome 1 de cada slot da grade, entrega produtos na saída e cuida de remanescentes |
| **Craft Single e Craft All** | `TileEntityFormulaicAssemblicator.java:398-404, 461-465` | `assemblicator.rs:873-909` (`craft_single`, `craft_all`) | IMPLEMENTADO | Acionados pelos botões GUI 7903 e 7904 |
| **Esvaziar Grade para Entrada** (`emptyGrid`) | `TileEntityFormulaicAssemblicator.java:473-477` | `assemblicator.rs:912-922` (`empty_grid`) | IMPLEMENTADO | Acionado pelo botão GUI 7902 quando a fórmula está vazia |
| **Resolução Dinâmica de Receitas de Crafting** | `RecipeFormula.java:21-36`, `CraftingInput.Positioned` | `assemblicator.rs:255-406` (`CraftingRecipeDef`, `RecipeResolver`) | PARCIAL-por-FORK | Motor completo com receitas nativas em Rust; consulta em tempo de execução ao motor do jogo catalogada em `FORK-32` |
| **Upgrades Suportados (Speed e Energy)** | `TileEntityFormulaicAssemblicator.java:712-723` | `assemblicator.rs:975-978` (`tick_assemblicator`), `upgrades.rs:81-93` | IMPLEMENTADO | Speed reduz ticks de 40 para 4; Energy amplia armazenamento e eficiência |
| **Controle de Redstone** (Disabled, High, Low, Pulse) | `TileEntityFormulaicAssemblicator.java:706-709` | `assemblicator.rs:980-985` | IMPLEMENTADO | Suporta modo Pulse (`pulseOperations`) |
| **Serialização de Estado** (`loadAdditional` / `saveAdditional`) | `TileEntityFormulaicAssemblicator.java:689-704` | `assemblicator.rs:518-602` (`AssemblicatorState::encode`, `decode`) | IMPLEMENTADO | Serializa `autoMode`, `stockControl`, `operatingTicks`, `ticksRequired`, `pulseOperations`, `config`, upgrades |
| **Botões da GUI** (7900 a 7905) | `GuiFormulaicAssemblicator.java:64-85` | `assemblicator.rs:72-77`, `assemblicator.rs:1110-1175` (`assemblicator_button`) | IMPLEMENTADO | Encode Formula, Stock Control, Fill/Empty, Craft Single, Craft All, Auto Mode |

---

## 2. Crafting Formula (Item)

| Comportamento / Requisito | Origem no Java | Localização no Rust | Estado | Observações |
|---|---|---|---|---|
| **Registro do item** (`mekanism:crafting_formula`) | `MekanismItems.java`, `ItemCraftingFormula.java:24` | `assemblicator.rs:29` (`CRAFTING_FORMULA`) | IMPLEMENTADO | Item base sem limite de empilhamento desnecessário |
| **Componente de Dados** (`mekanism:formula_holder`) | `FormulaAttachment.java:20-37`, `MekanismDataComponents.FORMULA_HOLDER` | `assemblicator.rs:30` (`FORMULA_COMPONENT`), `assemblicator.rs:86-173` (`FormulaAttachment`) | PARCIAL-por-FORK | Serializador/deserializador SNBT e validação completa em Rust; persistência rica em `ItemStack.components` catalogada em `FORK-23` |
| **Nome Dinâmico** | `ItemCraftingFormula.java:57-65` (`getName`) | `assemblicator.rs:238-249` (`display_name`) | IMPLEMENTADO | "Crafting Formula", "Crafting Formula (Encoded)", "Crafting Formula (Invalid)" |
| **Tooltip com Ingredientes Agrupados** | `ItemCraftingFormula.java:31-39` (`appendHoverText`) | `assemblicator.rs:253-264` (`tooltip_ingredients`) | IMPLEMENTADO | Agrupa os itens da matriz 3x3 por ID e soma contagens |
| **Limpeza com Shift + Clique no ar** | `ItemCraftingFormula.java:44-53` (`use`) | `assemblicator.rs:267-269` (`clear`), `lib.rs` (`on_player_interact`) | IMPLEMENTADO | Redefine `FORMULA_COMPONENT` para `EMPTY` e notifica o jogador |
| **Receita Shapeless** (1 papel + 1 basic circuit) | `RecipeProvider`, `ESPEC_utilidades.md:441-445` | `assemblicator.rs:380-394`, `INTEGRACAO_assemblicator_cartao.md` | IMPLEMENTADO | Declarada para o `convert.py` e registrada no `RecipeResolver` nativo |

---

## 3. Configuration Card (Item)

| Comportamento / Requisito | Origem no Java | Localização no Rust | Estado | Observações |
|---|---|---|---|---|
| **Registro do item** (`mekanism:configuration_card`) | `ItemConfigurationCard.java:38-42` | `configuration_card.rs:32` (`ITEM`) | IMPLEMENTADO | Stacks to 1, Uncommon rarity |
| **Componente de Dados** (`mekanism:configuration_data`) | `ItemConfigurationCard.java:68-71, 113-119` | `configuration_card.rs:33` (`CONFIG_DATA_COMPONENT`), `configuration_card.rs:59-224` (`ConfigurationData`) | PARCIAL-por-FORK | Serializador SNBT completo (chaves `data_name`, `data_type`, `control_type`, `config0..4`, `eject0..4`, `strict_input`, `color`, `input_color`); catalogado em `FORK-23` |
| **Copiar Configuração (Shift + Clique Direito na Máquina)** | `ItemConfigurationCard.java:65-74` | `configuration_card.rs:602-628` (`use_on` com `sneaking = true`), `configuration_card.rs:415-464` (`extract_configuration`) | IMPLEMENTADO | Extrai dados da máquina, salva no cartão, envia mensagem "Retrieved configuration data from %s" |
| **Gatilho de Avanço ao Copiar** (`copied = true`) | `ItemConfigurationCard.java:73` (`CONFIGURATION_CARD.trigger(player, true)`) | `configuration_card.rs:624-625` | PARCIAL-por-FORK | Ponto de extensão isolado catalogado em `FORK-34` |
| **Colar Configuração (Clique Direito sem agachar na Máquina)** | `ItemConfigurationCard.java:75-92` | `configuration_card.rs:629-652` (`use_on` com `sneaking = false`), `configuration_card.rs:468-540` (`apply_configuration`) | IMPLEMENTADO | Valida compatibilidade e injeta configuração na máquina de destino |
| **Mensagem de Sucesso ao Colar** | `ItemConfigurationCard.java:85-86` | `configuration_card.rs:642-646` | IMPLEMENTADO | Envia "Injected configuration data of type %s" |
| **Mensagem de Erro por Incompatibilidade** | `ItemConfigurationCard.java:89` | `configuration_card.rs:648-651` | IMPLEMENTADO | Envia "Unequal configuration data formats." |
| **Gatilho de Avanço ao Colar** (`copied = false`) | `ItemConfigurationCard.java:87` (`CONFIGURATION_CARD.trigger(player, false)`) | `configuration_card.rs:644-645` | PARCIAL-por-FORK | Ponto de extensão isolado catalogado em `FORK-34` |
| **Limpeza do Cartão (Shift + Clique Direito no Ar)** | `ItemConfigurationCard.java:100-110` (`use`) | `configuration_card.rs:584-599` (`clear_data`), `configuration_card.rs:655-663` (`use_air`) | IMPLEMENTADO | Remove `CONFIG_DATA_COMPONENT` e envia mensagem "Cleared Configuration Card" |
| **Compatibilidade de Fábricas e Tiers** (`isConfigurationDataCompatible`) | `TileEntityElectricMachine.java:132-134`, `TileEntityFactory.java:444-446`, `MekanismUtils.java:650-657` | `configuration_card.rs:248-349` (`is_compatible`) | IMPLEMENTADO | Compatível entre máquinas e seus tiers de fábrica (Smelting, Enriching, Crushing, Compressing, Combining, Purifying, Injecting, Infusing, Sawing), cubos de energia, tanques e bins |
| **Tooltip Informativo** | `ItemConfigurationCard.java:45-47` (`appendHoverText`) | `configuration_card.rs:227-238` (`card_tooltip`) | IMPLEMENTADO | Exibe "Data: <nome>" ou "Data: None" |
| **Receita em Cruz** (4 ligas infundidas + 1 basic circuit) | `ESPEC_utilidades.md:488-497` | `assemblicator.rs:395-412`, `INTEGRACAO_assemblicator_cartao.md` | IMPLEMENTADO | Declarada para o `convert.py` e registrada no `RecipeResolver` nativo |
