# Revisão do Robit contra o Java / Robit reviewed against Java

> Código: `robit.rs`. Verificado com `tools/verificador/check.sh` (436 testes passam) e contra `EntityRobit`, `ItemRobit`, `RobitAIFollow`, `RobitAIBase`, `RobitAIPickup`, `MekanismEntityTypes`, `MekanismRobitSkins`, `CachedRecipe.process`, `MekanismTeleportEvent.Robit` e os JSON de `data/mekanism/mekanism/robit_skin/` do jar.

## Conferido e igual ao Java / Checked and equal
Saúde 1, velocidade 0,3, energia máxima 100 000 J, multiplicador de viagem 1,5, 100 ticks (5 s) de fundição a 50 J/tick, raio de busca de itens 10 (r² = 100), e a ordem das 30 posições do inventário (27 de armazenamento, energia, entrada e saída da fundição). O dano consome `1 000 J × dano` e a saúde volta ao máximo. O item colocado no mundo nasce em `y + 0,1`.

## Erros encontrados e corrigidos / Bugs found and fixed
| # | Problema | Java |
|---|---|---|
| 1 | **Dados das skins errados:** a skin Allay exigia a conquista `adventure/allay_drop_item_on_block`; a certa é **`husbandry/allay_deliver_item_to_player`**. Os nomes ("Robit", "Allay", "Ace"...) são outros ("Default", "Allay Costume", "Ace Pride"...). As texturas eram `entity/robit`, `entity/robit_roll`...; o jar lista `mekanism:robit`, `mekanism:robit2`... e **quantidades diferentes** por skin (2, 2, 6, 8, 4, 5, 4, 5, 4, 6, 7, 6, 5). Faltava o modelo de item da Allay (`mekanism:item/robit_allay`). A tabela agora é **gerada do jar** | `robit_skin/*.json`, lang |
| 2 | **Seguir o dono:** os nomes dos limites estavam trocados e não havia histerese. `RobitAIFollow(this, 1, 4, 2)`: começa com o dono a **4 blocos ou mais** e só para a **2 blocos**; o port parava em 4. O teleporte (a partir de 12 blocos) é decidido a cada 10 ticks e leva a um ponto **2 ou 3 blocos ao redor** do dono (até 10 tentativas, onde o host diz que dá para ficar de pé), **não em cima dele** como o port fazia | `RobitAIFollow`, `RobitAIBase.updateTask` |
| 3 | **Energia de viagem:** só paga quando segue, com o dono a mais de 2 blocos e o caminho ainda em curso, e paga o que andou no tick **anterior**; o port pagava sempre e sempre arredondava para baixo (um passo curto saía de graça). Agora: `ceil(1,5 × distância)` J, arredondado para cima em FE | `baseTick:330-337`, `getRoundedTravelEnergy` |
| 4 | **Dano:** `clampToLong(1000 × dano)` **corta** a casa decimal; o port arredondava para cima | `onDamageTaken` |
| 5 | **Coleta de itens:** o Java percorre os slots **em ordem** e o **primeiro** vazio ou com o mesmo item (mesmos componentes) e espaço recebe o item e **encerra**; o resto espera o próximo tick. O port juntava em todos os slots iguais antes e usava 64 fixo, ignorando componentes (uma espada com dano diferente se fundia). A caixa é a do Robit (0,6 × 0,65) +1,5 em cada lado | `collectItems`, `isItemValid` |
| 6 | **Fundição:** sem entrada (ou com uma que não dá nada) o progresso é **perdido**; sem energia ou sem espaço só espera. O port nunca zerava | `CachedRecipe.process` |
| 7 | **Voltar para casa:** o Java só volta com a bateria vazia **e fora de um chargepad**, e chega em `y + 0,3` (o port usava 0,1 e voltava mesmo em cima do chargepad). Faltava a regra de **sumir e dropar** a cada 20 ticks se o chargepad de casa foi destruído (e sumir sem casa) | `baseTick:347`, `tick`, `MekanismTeleportEvent.Robit` |
| 8 | **Item do Robit:** a skin do item era copiada **sem checar o desbloqueio** (um jogador usava a Allay sem a conquista); o Java chama `setSkin(skin, player)`, que recusa e deixa o padrão, e só depois copia a flag "escolhida à mão". `setSkin` não faz nada se a skin já é a mesma (o port marcava "à mão" mesmo assim). O nome do dono não pode ser o de quem colocou se o dono é outro. Só se coloca em chargepad que não está em uso | `ItemRobit.useOn`, `setSkin` |

Testes novos citando o Java: `java_robit_collection_respects_components_pickup_delay_and_robit_items`, `java_robit_damage_costs_the_cut_joules`, `java_robit_follow_starts_at_four_blocks_and_stops_at_two`, `java_robit_teleports_around_the_owner_from_twelve_blocks`, `java_robit_home_rules`, `java_robit_smelting_loses_the_progress_without_input`, `java_robit_item_skin_needs_to_be_unlocked_and_the_owner_keeps_the_name`, e os de skins, coleta e viagem reescritos.

## Ainda diferente (pedidos de fork) / Still different
O Robit do port é uma **simulação**: anda em linha reta um passo por tick (0,3 bloco), enquanto o original usa o pathfinder (`getNavigation`), flutua na água, olha para o jogador (`LookAtPlayerGoal`) e tem a IA de pegar itens (`RobitAIPickup`, que anda até o item mais próximo em até 10 blocos). Dependem do host: entidade própria com IA e navegação, `EntityRobit.create`, `MekanismTeleportEvent`, mudar de dimensão ao voltar para casa, `PacketSyncSecurity`, `GameEvent.ENTITY_PLACE`, efeitos de luz e sons, e o sorteio de skin de feriado (`HolidayManager`).

## Ainda não conferido / Not yet checked
A tela do Robit (5 abas: principal, inventário, fundição, aparência, configurações), `interactAt` (configurador e shift), `IEntitySecurityUtils`, a renderização e os modelos por skin, `RobitAIPickup` (a busca do item mais próximo) e o drop ao morrer.


## Ligação ao jogo / Wiring into the game (FORK-10 a a f, d2)

**PT-BR.** `robit_world.rs` liga `robit.rs` ao tipo de entidade `mekanism:robit` (conteúdo gerado em `entity_kind/robit.json`, 13 skins, 30 slots, 0,6 x 0,65, vida 1, velocidade 0,3):
o item usado numa Chargepad livre coloca o Robit com a energia, o nome, a skin, o dono, a segurança e os 30 slots do item; o clique abre o armazenamento (27 slots, como baú); sneak + clique o devolve como item; um golpe vira gasto de energia no próximo tick (e a vida volta ao máximo); a morte solta o conteúdo; sem casa ele some, e sem Chargepad na casa ele cai como item. Cada tick roda `RobitEntity::tick` (seguir o dono, recolher itens, fornalha interna, custo de energia de andar, voltar para casa sem energia).
**Diferenças:** sem tela com abas (inventário, fornalha, crafting, bigorna e botões ficam de fora), a fornalha não aparece no menu; a skin usa só a primeira textura de cada grupo; o modelo é de item (sem animação); outros danos além do golpe do jogador não chegam; o andar é em linha reta; o item guarda os dados em `minecraft:custom_data` (sem os componentes próprios do original). Nada foi visto no jogo.

**EN.** `robit_world.rs` wires `robit.rs` to the `mekanism:robit` entity kind (generated content): placing the item on a free Chargepad, the chest-like menu, sneak-click packing, hits paid in energy, drops on death, the home rules and the per-tick follow, pick-up, furnace and energy costs. Differences as above; nothing seen in game.
