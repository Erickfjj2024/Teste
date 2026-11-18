# FORK-20 — widgets de tela que o Mekanism usa e o Horizon ainda não tem / Screen widgets Mekanism uses that Horizon lacks

**PT-BR.** As telas do Horizon (`horizon/menu/*.json`) têm barra, sprite, texto, botão, amostra de cor, medidor e grupos condicionais, e leem **números** (`menu_data`, 16 bits). Várias telas do Mekanism precisam de três coisas que faltam. Sem elas as máquinas abaixo não são utilizáveis em fidelidade.

| Falta / Missing | Onde o Mekanism usa / Where Mekanism uses it | O que o port faz hoje / What the port does today |
|---|---|---|
| **Campo de texto** (`GuiTextField`: digitar e confirmar; o mod recebe a string) | Oredictionificator (a tag do filtro), Digital Miner (id de bloco, tag, nome), Logistical Sorter (tag, nome de item), QIO (busca, tag, nome de frequência), Resistive Heater (consumo, 7 dígitos), Teleporter e Entangloporter (nome de frequência), Security Desk, Modification Station (busca de módulo) | Resistive Heater: 8 botões de passo; os demais: só os valores padrão; filtros novos não podem ser criados |
| **Texto dinâmico** (strings no menu, não só números) | nomes de filtros, de frequências, de jogadores, de itens, listas de módulos | números e textos fixos |
| **Lista rolável** (`GuiScrollList`, `GuiScrollBar`, `FilterButton`) ligada a uma lista de strings/itens | filtros (Oredictionificator, Miner, Sorter, QIO), frequências, módulos instalados, listas do Security Desk | até 3–5 linhas fixas |

## Proposta / Proposal
- `BlockEntityUpdate.menu_text: option<list<string>>` (e o equivalente em `block-entity`): strings que viajam com o menu, até 64 por menu, 256 bytes cada; o widget `{"type":"text","text_data": 3}` mostra a string 3.
- Widget `textfield` `{ "x","y","w","h","max_length","validator": "digit"|"any"|"tag"|"id", "id": 7800 }`: ao confirmar, o cliente manda `menu-text(player, dimension, pos, id, text)` ao mod (evento novo `menu-text-input`).
- Widget `list` `{ "x","y","w","h","row_height","rows": 3, "source": "text"|"items", "first": 8, "count": {"data": 2}, "on_click": 7700 }`: mostra `count` linhas a partir de uma barra de rolagem local e manda `menu-button(id + índice)`.

**Quando existirem:** o Oredictionificator (filtros por tag, `oredictionificator.rs` já guarda tag + saída + estado), o Digital Miner, o Sorter e o QIO passam a ter a interface do Java; o Resistive Heater troca os botões de passo pelo campo.

**EN.** Horizon's screens read 16-bit numbers only. Mekanism needs a text field, dynamic strings in a menu and a scrolling list for the filter machines and several frequency screens. Proposal: `menu_text` strings, a `textfield` widget (event `menu-text-input`) and a `list` widget. Until then: the Oredictionificator cannot get a filter, the Resistive Heater uses step buttons, and filter screens show fixed rows.
