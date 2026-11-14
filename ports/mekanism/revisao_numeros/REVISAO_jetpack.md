# Revisão do jetpack do MekaSuit contra o Java / MekaSuit jetpack reviewed against Java

> Código: `mekasuit.rs` (`tick_jetpack` e funções novas), `modules.rs` (`JetpackMode`, `ThrustMultiplier`). Verificado com `tools/verificador/check.sh` (416 testes passam) e contra `ModuleJetpackUnit`, `IJetpackItem`, `ItemMekaSuitArmor:334-384`, `CommonPlayerTickHandler.tickEnd:104-124`, `MekanismModules:148-160`.

## Estado antes / Before
O jetpack do port tinha 3 modos, empuxo e consumo escalares e **o resultado de `tick_jetpack` era descartado** pelo chamador (nada chegava ao jogador). O hover usava `−0,08 × mult` ao agachar, que não existe no Java.

## Erros encontrados e corrigidos / Bugs found and fixed
| # | Problema | Java |
|---|---|---|
| 1 | Faltava o modo **Vector**; a ordem dos modos era Disabled, Normal, Hover. A do Java é **Normal, Hover, Vector, Disabled** (é a ordem em que a tecla de modo avança) | `IJetpackItem.JetpackMode` |
| 2 | **O hover gastava hidrogênio parado no chão**, o tempo todo. No Java o combustível só é gasto quando o modo do jogador não é `Disabled`: no hover, só no ar ou subindo sem descer | `getPlayerJetpackMode`, `tickEnd:111-120` |
| 3 | Movimento inventado. Agora é o do Java: **Normal** soma `(0,08·vx, empuxo·min(1, e^(−vy)), 0,08·vz)`; **Vector** soma o vetor "para cima" do jogador × empuxo (y com o mesmo fator); sneak transforma Vector em Normal; **Hover** prende a velocidade vertical a `[−2·empuxo, 2·empuxo]` e amortece para 0; **planando com elytra** usa a direção do olhar mais um arrasto até `1,5·olhar` e não zera a queda | `handleJetpackMotion` |
| 4 | Empuxo: `0,15 × mult`, mas quando o tanque tem menos mB que `ceil(mult)` o multiplicador **vira o que sobrou** (o port não fazia a redução) | `getJetpackThrust:354-370` |
| 5 | Combustível: `ceil(mult)` mB por tick, ou o que houver (o port pulava o tick inteiro se faltasse) | `useJetpackFuel:375-383` |
| 6 | `canUseJetpack`: módulo ligado + hidrogênio, **ou** qualquer módulo ligado que sobrescreva o pulo | `ItemMekaSuitArmor:334-342` |
| 7 | O multiplicador de empuxo ignorava o limite: só os primeiros **instalados + 1** valores (HALF, NORMAL, FAST, FASTER, FASTEST) podem ser escolhidos, para o normal e para o hover; ao tirar uma unidade a escolha é rebaixada | `MekanismModules:155-160` |
| 8 | O contexto do jogador não tinha o que o jetpack precisa (movimento, olhar, vetor "para cima", no chão, cooldown, passageiro, espectador) e o resultado não era entregue ao host | — |

Testes novos citando o Java: `java_jetpack_modes_step_normal_hover_vector_disabled`, `java_jetpack_thrust_drops_when_the_tank_holds_less_than_the_multiplier_needs`, `java_player_jetpack_mode`, `java_hover_on_the_ground_does_not_burn_hydrogen`, `java_jetpack_motion_per_mode`, `java_jetpack_thrust_choices_grow_with_the_installed_units`.

## Ainda diferente / Still different
- O movimento é entregue ao host como diferença de velocidade por `host_apply_vector_impulse` (**FORK-29**: física customizada). O Java no servidor soma/define o `deltaMovement`; sem essa chamada o jetpack calcula certo mas não move o jogador.
- Falta a leitura das teclas (ascender, descer) e o `getUpVector`/`getLookAngle` reais: vêm de `JetpackInput`, que o host precisa preencher.
- `resetFallDistance` e `aboveGroundTickCount` (anti-fly do servidor) e o `gameEvent JETPACK_BURN` a cada meio segundo: pedidos de fork.
- **O limite "instalados + 1" só foi aplicado ao jetpack.** Os outros módulos com `addInstalledCountConfig` (dano de ataque, escavação, explosão, vein mining, fazenda, sprint, salto, passo, alcance magnético, repulsor hidrostático…) têm o mesmo tipo de limite no Java e ainda não o têm no port: ficam para a revisão dos módulos de movimento.
- Não conferi o **Jetpack** e o **Armored Jetpack** avulsos (`ItemJetpack`, empuxo 0,15 também) nem o HUD do jetpack.
