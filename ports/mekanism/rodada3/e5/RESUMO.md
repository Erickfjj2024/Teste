# RESUMO — Tarefa T20261109-204238-bzb
Grupo: `assemblicator_cartao`
Alvo: FORMULAIC ASSEMBLICATOR, CRAFTING FORMULA (item) e CONFIGURATION CARD (item).
Fonte de verdade: Mekanism Java 1.21.1-10.7.19.85.

---

## 1. Módulos Criados

1. [assemblicator.rs](file:///home/erickfjj2016/espelho/mekanism_audit/port/mod/src/assemblicator.rs) (1429 linhas):
   - Bloco `mekanism:formulaic_assemblicator` e item `mekanism:crafting_formula`.
   - Layout completo de 35 slots (`TOTAL_SLOTS = 35`):
     - Slot 0: Fórmula (`SLOT_FORMULA = 0`)
     - Slots 1..18: 18 slots de entrada (`SLOT_INPUT_START = 1`, `INPUT_SLOTS = 18`)
     - Slots 19..27: 9 slots da grade 3x3 (`SLOT_CRAFT_START = 19`, `CRAFT_SLOTS = 9`)
     - Slots 28..33: 6 slots de saída (`SLOT_OUTPUT_START = 28`, `OUTPUT_SLOTS = 6`)
     - Slot 34: 1 slot de energia (`SLOT_ENERGY = 34`)
   - Constantes exatas do Java:
     - `BASE_TICKS_REQUIRED = 40` ticks (2,0s, `TileEntityFormulaicAssemblicator.java:80`)
     - `BASE_ENERGY_USAGE = 40` FE/t (`fe(100)`, `UsageConfig.formulaicAssemblicator`)
     - `BASE_STORAGE = 16_000` FE (`fe(40_000)`, `StorageConfig.formulaicAssemblicator`)
   - Suporte a upgrades:
     - `Upgrade.SPEED`: reduz de 40 para 4 ticks em 8 upgrades (`upgrades::ticks`)
     - `Upgrade.ENERGY`: amplia capacidade e reduz consumo (`upgrades::max_energy`, `upgrades::energy_per_tick`)
   - `FormulaAttachment`: serializador/deserializador SNBT para `mekanism:formula_holder`, verificação de validade e cópia como inválido.
   - `CraftingFormulaItem`: nomes dinâmicos ("Crafting Formula", "Crafting Formula (Encoded)", "Crafting Formula (Invalid)"), tooltip agrupado por ingrediente e limpeza com Shift + clique direito no ar.
   - `RecipeFormula`: casamento posicional 3x3, ingredientes válidos por posição e suporte a receitas com sobra de recipientes.
   - `RecipeResolver`: motor nativo puro em Rust com receitas embutidas do Mekanism + gancho fino isolado para o motor do jogo (`// FORK-32`).
   - Lógica operacional completa:
     - `move_items_to_grid`: abastecimento automático de trás para frente (17 até 0)
     - `do_single_craft`: validação prévia de espaço de saída e sobras, consumo de 1 item e reposição imediata
     - `craft_single` e `craft_all`: fabricação manual
     - `empty_grid`: devolução da grade para a entrada
     - `encode_formula_slot`: gravação da receita da grade no cartão de fórmula
     - `organize_stock`: organização do estoque pelos slots controlados (`build_stock_control_map`, 2 slots por posição de receita)
     - `tick_assemblicator`: tick server com controle de redstone (Disabled, High, Low, Pulse com `pulse_operations`)
     - `assemblicator_item_faces`: mapeamento das faces por transmissão de itens (Input, Output, In/Out, Energy, Extra)

2. [configuration_card.rs](file:///home/erickfjj2016/espelho/mekanism_audit/port/mod/src/configuration_card.rs) (845 linhas):
   - Item `mekanism:configuration_card`.
   - Propriedades: `stacksTo(1)`, raridade `UNCOMMON`.
   - `ConfigurationData`: estrutura e serialização/deserialização SNBT para o componente `mekanism:configuration_data`, preservando todas as chaves do Java:
     - `data_name`, `data_type`, `control_type`
     - `config0` .. `config4` (5 transmissões x 6 lados relativos)
     - `eject0` .. `eject4` (flags de auto-eject)
     - `strict_input`, `color`, `input_color`
     - Chaves extras por máquina (`auto`, `stock_control`, `energy_usage`, etc.)
   - Compatibilidade de máquinas (`is_compatible`):
     - Igualdade exata de bloco
     - 9 famílias de fábricas (`MekanismUtils.isSameTypeFactory`): Smelter, Enricher, Crusher, Compressor, Combiner, Purifier, Injector, Infuser, Sawmill
     - Tiers de cubos de energia, tanques químicos, tanques de fluidos e bins
     - Transmissores do mesmo meio
   - Extração e injeção de configuração (`extract_configuration`, `apply_configuration`)
   - Interações:
     - Shift + clique direito na máquina: COPIA configuração, grava no cartão, envia mensagem ("Retrieved configuration data from %s") e aciona gancho de avanço (`copied = true`)
     - Clique direito sem agachar na máquina: COLA configuração (se compatível: injeta, envia "Injected configuration data of type %s", avanço com `copied = false`; se incompatível: erro "Unequal configuration data formats.")
     - Shift + clique direito no ar: LIMPA cartão, remove componente, envia "Cleared Configuration Card"
     - Tooltip dinâmico: "Data: %s" ou "Data: None"

---

## 2. Entregáveis de Documentação

- [.saida/T20261109-204238-bzb/INTEGRACAO_assemblicator_cartao.md](file:///home/erickfjj2016/espelho/mekanism_audit/.saida/T20261109-204238-bzb/INTEGRACAO_assemblicator_cartao.md) (251 linhas): trechos exatos de código com contexto vizinho para colar em `lib.rs`, adições completas para `tools/convert.py` (menu de 35 slots, botões 7900..7905, receitas e tags) e especificações para `horizon-mod.toml`.
- [.saida/T20261109-204238-bzb/FORK_assemblicator_cartao.md](file:///home/erickfjj2016/espelho/mekanism_audit/.saida/T20261109-204238-bzb/FORK_assemblicator_cartao.md) (108 linhas): tabela de requisitos e assinaturas WIT detalhadas para `FORK-23` (componentes ricos), `FORK-32` (resolução dinâmica de receitas 3x3), `FORK-33` (contêiner de 35 slots sem limite de 32) e `FORK-34` (critério de avanço do servidor).
- [.saida/T20261109-204238-bzb/IMPL_assemblicator_cartao.md](file:///home/erickfjj2016/espelho/mekanism_audit/.saida/T20261109-204238-bzb/IMPL_assemblicator_cartao.md) (62 linhas): matriz de conformidade detalhada com cada requisito da especificação, estado (IMPLEMENTADO / PARCIAL-por-FORK), linhas exatas do Rust e origem no Java.

---

## 3. Contagem de Testes Unitários

Total de testes unitários implementados: **16 testes** (todos com citações a arquivos e linhas do Java 1.21.1).

### Em `assemblicator.rs` (10 testes):
1. `test_inventory_slot_layout` (linha 1229) — validação dos 35 slots (`TOTAL_SLOTS`, `SLOT_FORMULA`, `SLOT_INPUT_START`, `SLOT_CRAFT_START`, `SLOT_OUTPUT_START`, `SLOT_ENERGY`).
2. `test_base_constants` (linha 1244) — comparação com `TileEntityFormulaicAssemblicator.java:80` (40 ticks) e configs de energia (40 FE/t e 16.000 FE).
3. `test_formula_attachment_snbt_roundtrip` (linha 1253) — serialização e deserialização SNBT de `FormulaAttachment`.
4. `test_crafting_formula_display` (linha 1275) — nomes dinâmicos e tooltips de ingredientes (`ItemCraftingFormula.java:31-65`).
5. `test_stock_control_map` (linha 1310) — mapeamento de 2 slots de entrada por ingrediente (`TileEntityFormulaicAssemblicator.java:642-658`).
6. `test_assemblicator_state_encode_decode` (linha 1328) — persistência binária do estado e upgrades do bloco.
7. `test_crafting_execution` (linha 1355) — execução de receita, consumo de itens da grade e depósito no buffer de saída (`TileEntityFormulaicAssemblicator.java:368-396`).
8. `test_speed_upgrades_scaling` (linha 1386) — redução de 40 para 4 ticks em 8 upgrades de velocidade (`TileEntityFormulaicAssemblicator.java:714-717`).
9. `test_stock_control_input_validation` (linha 1394) — validação de entrada de itens com controle de estoque ativo e inativo (`TileEntityFormulaicAssemblicator.java:159-172`).
10. `test_empty_grid_moves_to_inputs` (linha 1419) — devolução de itens da grade de criação para os slots de entrada (`TileEntityFormulaicAssemblicator.java:473-477`).

### Em `configuration_card.rs` (6 testes):
1. `test_configuration_data_snbt_roundtrip` (linha 677) — serialização e deserialização de todas as chaves de configuração (`TileComponentConfig.java:340-353`).
2. `test_factory_compatibility` (linha 724) — compatibilidade cruzada entre máquinas elétricas e suas respectivas fábricas (`TileEntityElectricMachine.java:132-134`).
3. `test_cube_and_tank_compatibility` (linha 740) — compatibilidade entre tiers de cubos, tanques e bins.
4. `test_card_tooltip` (linha 753) — apresentação do tooltip "Data: %s" e "Data: None" (`ItemConfigurationCard.java:45-47`).
5. `test_card_clear` (linha 768) — remoção dos dados do cartão ao limpar (`ItemConfigurationCard.java:100-110`).
6. `test_card_use_on_copy_and_paste` (linha 790) — cópia de configuração de uma máquina e colagem em máquina compatível vs incompatível (`ItemConfigurationCard.java:51-96`).

---

## 4. Prova de Mutação

A prova de mutação está pendente do juiz (a tarefa não forneceu lista de mutações predefinida).

---

## 5. Revisão de Risco e Compilação

Como não há rustc/cargo funcional no ambiente para compilação WASM (erro de toolchain rustup sem release instalada), os módulos foram auditados duplamente contra o borrow checker e regras de tipos:
1. **Empréstimos de fatias mutáveis (`slots: &mut [Option<ItemStack>]`)**:
   - `move_items_to_grid`: as consultas de checagem e cópia foram isoladas em variáveis locais antes de qualquer mutação de `slots[craft_idx]` ou `slots[in_idx]`, prevenindo conflito de empréstimos mútuos.
   - `do_single_craft`: o decréscimo dos slots da grade calcula `should_clear` antes de atribuir `slots[idx] = None`, liberando o empréstimo mutável `st` antes da alteração estrutural.
   - `encode_formula_slot`: a leitura da grade é concluída antes do empréstimo de `slots[SLOT_FORMULA]`.
2. **Parser SNBT**:
   - Usa o split estrutural de `config::split_components` e faz parsing tolerante a sufixos de tipo (`1b`, etc.).
3. **Trechos de maior atenção para o orquestrador**:
   - Resolução de receitas dinâmicas de crafting (`FORK-32`): em tempo de execução no host, o motor do Horizon fornecerá a tabela global de receitas via WASM import; enquanto o fork não implementa essa chamada, o motor interno `RecipeResolver::built_in_recipes` garante funcionamento standalone com os itens do mod.
   - Contêiner de 35 slots (`FORK-33`): certificar que o fork suporte o valor 35 em `ContainerSpec.slots` na camada de sincronização de inventários.
