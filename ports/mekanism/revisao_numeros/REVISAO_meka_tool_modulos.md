# Revisão dos módulos do Meka-Tool (teletransporte, explosão, fazenda, tosquia) contra o Java / Meka-Tool modules reviewed against Java

> Código: `meka_tool.rs`. Verificado com `tools/verificador/check.sh` (429 testes passam) e contra `ItemMekaTool:128-146,373-417`, `ModuleTeleportationUnit`, `ModuleBlastingUnit`, `IBlastingItem`, `MekanismUtils.veinMineArea`, `ModuleFarmingUnit`, `ModuleShearingUnit`, `MekanismModules:74-125`.

## Conferido e igual ao Java / Checked and equal
Geometria da explosão (UP/DOWN `(−r..r, 0, −r..r)`, EAST/WEST `(0, −1..2r−1, −r..r)`, NORTH/SOUTH `(−r..r, −1..2r−1, 0)`), tabelas de raio (explosão 0–4, fazenda 0/1/3/5/7), custos 10 J (enxada, pá, machado, tosquia de entidade, poda), 1 000 J de teletransporte, alcance máximo 100, destino com 2 blocos livres acima (ar ou líquido), distância mínima `distanceToSqr ≥ 5`, ponto de chegada `(x+0,5, y+1,5, z+0,5)`, exigir bloco-alvo (padrão ligado), flags de exclusividade (fazenda `INTERACT_BLOCK`, tosquia `INTERACT_ENTITY|BLOCK`, silk e fortuna `OVERRIDE_DROPS`, teletransporte `INTERACT_ANY`).

## Erros encontrados e corrigidos / Bugs found and fixed
| # | Módulo | Problema | Java |
|---|---|---|---|
| 1 | **Teletransporte** | O Java calcula `ceil(1000 J × (distance / 10))` onde `distance` é **`distanceToSqr`** (a distância **ao quadrado**). O port tirava a raiz: 50 blocos custavam 5 000 J em vez de **250 000 J** (50× menos); 20 blocos, 800 FE em vez de **16 000 FE**. O port ainda recusava acima de 100 blocos, o que o Java não faz (o raio é cortado pelo host) | `ItemMekaTool:388-389` |
| 2 | **Explosão** | Faltava a regra de energia: `available = guardada − necessária(alvo)`; um bloco é **pulado quando `usada + custo ≥ disponível`** (igual não passa) mas os seguintes ainda são tentados; ar e indestrutíveis (−1) são ignorados; dureza 0 custa metade; o custo da explosão usa a eficiência e o do veio não. Também faltava `canBlastBlock` (não ar, não líquido, não bounding, dureza > 0). A ordem de saída é a do `betweenClosed` | `MekanismUtils.veinMineArea`, `IBlastingItem.canBlastBlock` |
| 3 | **Fazenda** | O port usava o plano "perpendicular à face" para **todas** as ferramentas. No Java só o **machado** faz isso; **pá e enxada** usam sempre o plano horizontal. O raio `Off` (diâmetro 0) é "não faz nada" e o port o confundia com `Low` (só o bloco). Faltava a regra de energia (o primeiro bloco sempre; os outros pagam `custo` cada, parando quando `usada + custo > energia`; igual passa; bloco que não muda não paga) e a validade da pá (ar em cima, ou planta substituível sem fluido e não sólida) | `ModuleFarmingUnit:108-310` |
| 4 | **Tosquia** | Só havia as constantes. Agora: tosquiar entidade e podar bloco custam 10 J cada (grátis se o custo é 0, recusado sem energia), o `SHEARS_DISARM` exige a energia de um bloco de dureza 0 com a eficiência (20 J; com silk 200 J) e `hasEnergyForDigAction` aceita qualquer energia acima da razão 1e-5 | `ModuleShearingUnit`, `ItemMekaTool:135-146` |

Testes novos citando o Java: `java_blast_energy_rule_skips_when_used_plus_cost_reaches_the_available`, `java_blast_block_filter_and_order`, `java_farming_planes_per_tool_and_the_off_setting`, `java_farming_energy_stops_when_the_next_use_would_exceed_it`, `java_shearing_and_dig_energy_rules`, e o do teletransporte reescrito (20 blocos = 16 000 FE; 50 blocos = 100 000 FE; a borda de `distanceToSqr` 5).

## Ainda diferente (pedidos de fork) / Still different
Tudo o que age no mundo é do host: o traçado de raio do teletransporte (`MekanismUtils.rayTrace`), descer do veículo ao teletransportar, o evento `MekanismTeleportEvent`, o efeito de portal e o som, `getToolModifiedState` (o que cada ferramenta faz a cada bloco), o `onSheared` das entidades e os drops, o `CommonHooks.fireBlockBreak`/`playerDestroy` da explosão, a seta de luz (`PacketLightningRender`). As conversões em FE arredondam para cima, para que um uso nunca seja mais barato que o original.

## Ainda não conferido / Not yet checked
Mudança de modo por roda radial do Meka-Tool, `adjustAttributes` (dano e velocidade de ataque), a tosquia por dispensador, o comportamento da pá `douse`, o eixo do machado (`AxeToolAOEData.getAxis`) e a fortuna e o silk touch como encantamentos. Falta também o Electric Bow, o HDPE Elytra e o Robit.
