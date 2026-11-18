# Revisão dos módulos de movimento do MekaSuit contra o Java / MekaSuit movement modules reviewed against Java

> Código: `mekasuit.rs` (`MoveInput`, `tick_gravitational_modulation`, `tick_locomotive_boost`, `calculate_hydraulic_jump`, `tick_hydrostatic_repulsor`, `tick_magnetic_attraction`, `can_elytra_fly`, `tick_elytra`, `step_boost`, `is_on_ground_or_sleeping`) e `modules.rs`. Verificado com `tools/verificador/check.sh` (424 testes passam) e contra `ModuleGravitationalModulatingUnit`, `ModuleLocomotiveBoostingUnit`, `ModuleHydraulicPropulsionUnit`, `ModuleHydrostaticRepulsorUnit`, `ModuleMagneticAttractionUnit`, `ModuleElytraUnit`, `ItemMekaSuitArmor:296-331`, `CommonPlayerTickHandler:62-90,272-292`, `MekanismModules:148-232`.

## Conferido e igual ao Java / Checked and equal
Tabelas de valores: sprint 0/0,05/0,1/0,25/0,5, salto 0/0,5/1/3/5, passo 0/0,5/1/1,5/2, alcance magnético 0/1/3/5/10; padrão LOW em todos; peça de cada módulo (botas: hidráulico, magnético, frost walker, soul surfer; calça: locomotive, giroscópio, repulsor, servo; peitoral: dosímetro, Geiger, jetpack, distribuição, gravitacional, elytra); custo do elytra 32 000 J a cada 20 ticks; custo do repulsor 500 J/tick.

## Erros encontrados e corrigidos / Bugs found and fixed
| # | Módulo | Problema | Java |
|---|---|---|---|
| 1 | Gravitacional | Com a tecla de boost e boost **desligado (0)** o port cobrava **4×** a energia; o Java só cobra 4× se o boost é maior que 0, senão a energia base. Também processava jogador criativo/espectador (o Java só processa quem **voa de fato** em modo de jogo) | `tickServer:80-96`, `shouldProcess` |
| 2 | Locomotive | A redução de 1/5 no ar usava "está voando" (voo criativo); o Java usa **`!onGround`**. Não bloqueava durante o planeio de elytra (`!isFallFlying`). O custo era `ceil` em float de FE e podia dar 1 FE a mais; agora é `long × float / 0,1` em Joules (50/100/250/500 J = 20/40/100/200 FE). Criativo não paga (mas ainda precisa de energia para funcionar) | `tick:47-71`, `canFunction` |
| 3 | Salto hidráulico | Não exigia a **tecla de boost** e debitava das botas. O Java confere as botas mas **debita da calça** (`useEnergy(player, legs, usage, true)`): quirk do original, replicado e documentado. O custo é `ceil(1000 J × boost / 0,1F)` em float (2 000/4 000/12 000/20 000 FE). A raiz quadrada vale quando a locomotive **funciona** (não só quando existe) | `onLivingJump:272-292` |
| 4 | Repulsor hidrostático | Criativo pagava energia; faltava o limite `BOOST_STACKS = 4` como constante e a eficiência de movimento na água `min(1, 0,33333334 × instalados)` | `ModuleHydrostaticRepulsorUnit` |
| 5 | Atração magnética | Criativo pagava; o Java começa só se as botas têm o custo de **um** puxão, para quando não consegue pagar ou quando o resto fica abaixo do custo; custo `ceil(250 J × alcance)` em float | `tickServer:25-55` |
| 6 | Elytra | Não olhava **sneak** (que deixa o jogador sair do planeio), nem a regra do jetpack (hover com hidrogênio **bloqueia** o elytra; sem hidrogênio ou em outro modo, libera), e cobrava mesmo em criativo | `canElytraFly:296-313`, `elytraFlightTick` |
| 7 | Escolhas limitadas | Sprint, salto, passo e alcance magnético só oferecem os primeiros **instalados + 1** valores (como o jetpack); ao tirar uma unidade a escolha é rebaixada. (O gravitacional **não** é limitado: aceita todos os valores) | `MekanismModules:155-226` |
| 8 | Passo assistido | Não existia. `step_boost`: sneak = 0; o módulo hidráulico ligado decide **mesmo em 0** (sem olhar os free runners); sem módulo, os free runners dão 0,5 | `getStepBoost:72-90` |
| 9 | Hover | `isOnGroundOrSleeping` inclui **voar** (`abilities.flying`): quem voa com o módulo gravitacional conta como "no chão" para o hover do jetpack | `CommonPlayerTickHandler:62-64` |

Testes novos citando o Java: `java_gravitational_unit_only_works_while_flying_in_a_playing_mode`, `java_locomotive_boost_costs_and_throttles`, `java_hydraulic_jump_needs_the_boost_key_and_takes_energy_from_the_legs`, `java_hydrostatic_repulsor_swim_boost_needs_four_units`, `java_magnetic_attraction_pays_per_item_and_stops_when_it_cannot`, `java_elytra_conditions_and_energy`, `java_bounded_choices_of_the_movement_modules`, `java_step_boost_and_on_ground_rules`.

## Ainda diferente (pedidos de fork) / Still different
Os efeitos no mundo dependem do host: mover o jogador (`moveRelative`), o atributo de voo criativo do módulo gravitacional (`CREATIVE_FLIGHT` com 1 000 J), os atributos de movimento na água e velocidade de nado, o `STEP_HEIGHT`, o puxão e o raio de luz dos itens, os eventos de jogo (`ELYTRA_GLIDE`, `GRAVITY_MODULATE`) e as teclas (boost, ascender). Estão como `host_*` / FORK-29.

## Ainda não conferido / Not yet checked
Giroscópio, servo motorizado, frost walker e soul surfer (módulos sem custo numérico no que li), visão noturna, e o HUD de cada módulo. O comportamento de "mudar o modo" (tecla) de cada módulo e o gravitacional/magnético como alternância (`canChangeModeWhenDisabled`).
