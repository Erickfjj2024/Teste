# REVISÃO — Formulaic Assemblicator / Formulaic Assemblicator review

**Fontes / sources:** `TileEntityFormulaicAssemblicator`, `RecipeFormula`, `GuiFormulaicAssemblicator`, `PacketGuiInteract` (1.21.1-10.7.19.85) × `assemblicator.rs`.

## Corrigido / Fixed
| # | Java | Antes (port) | Agora |
|---|---|---|---|
| 1 | `RecipeFormula.valid()` = há receita | uma fórmula sem receita era válida | só com receita; sem ela a fórmula é marcada *inválida* no item e conta como nenhuma (`loadFormula`) |
| 2 | `isIngredientInPos`, `isValidIngredient`: mesmo item e componentes **ou** qualquer item que ainda faça a receita naquele lugar | só o mesmo id | recipe-aware (`recipe.matches` na grade com o item trocado) |
| 3 | `moveItemsToGrid` usa `isIngredientInPos` nos slots de entrada | id igual | idem 2 |
| 4 | `organizeStock`: mapa ordenado, chave = item + componentes, pilha = limite do item, sobras vão aos slots sem item controlado e depois a qualquer slot vazio | `HashMap` (ordem aleatória), **perdia os componentes** (espada encantada virava espada comum) e **perdia itens** quando faltava slot | reescrito; teste de que nada se perde |
| 5 | o estoque é reorganizado quando a entrada muda (`needsOrganize`) | nunca no tick | todo tick com controle de estoque ligado (idempotente) |
| 6 | validador do slot de entrada (java:159-172): sem item próprio cai em `isValidIngredient` | rejeitava | cai no ingrediente da fórmula |
| 7 | `operatingTicks = 0` em todo caso que não opera (manual, sem fórmula, redstone nega) | só no modo manual | idem Java |
| 8 | `nextMode`: ligar move antes ao input o que não é da fórmula (`moveItemsToInput(false)`); desligar funciona sem fórmula | alternava só com fórmula, sem mover | idem Java |
| 9 | `doSingleCraft` só com a grade fazendo a receita (`lastOutputStack`) | cozinhava com qualquer grade | confere `recipe.matches(grade)` |
| 10 | pilhas pelo limite de cada item | 64 fixo | `bin::stack_size` (palpite pelo nome) |
| 11 | `tryMoveToOutput(SIMULATE)` | só olhava se havia um slot livre | simula o mesmo insert numa cópia |
| 12 | `FILL/EMPTY`, `ENCODE` | encode sem checar modo automático | encode só fora do modo automático (botão inativo no cliente) |

## Aberto / Open (FORK — ver `PROPOSTA-API.wit`)
1. **Receitas de crafting do jogo todo** (`crafting-find`, `crafting-matches`): sem elas o Assemblicator só conhece 3 receitas da própria família; qualquer outra fórmula é marcada inválida. É o ponto que impede o uso real. `fork_utilidades::crafting_find/matches` já são o ponto de ligação.
2. **Limite de pilha real** (`item-max-stack-size`): hoje é palpite pelo nome.
3. **Validador por slot**: o controle de estoque não impede um funil de pôr o item errado no slot; o Java impede.
4. **Upgrades** de velocidade/energia: o Assemblicator não está na tabela de máquinas (`MACHINES`), então o jogador não instala upgrades nele.
5. `canTryToMove` (otimização do Java) não replicado: o resultado é o mesmo, só se tenta mais vezes.
6. Slots de *crafting* aceitam item só em modo manual (`FormulaicCraftingSlot`): o contêiner do jogo não distingue.
7. Chamadas de computador (CC) e anexos do cartão para as outras máquinas: fora deste passo.

**EN.** Fixed 12 deviations (formula validity, recipe-aware ingredient checks, `organizeStock` that lost components and items, per-tick stock organizing, ticks reset, auto-mode switch, craft only on a matching grid, per-item stack limits, output simulation). Open (needs fork API): the whole game's crafting recipes (the blocking one), real max stack size, per-slot validators, upgrades.
