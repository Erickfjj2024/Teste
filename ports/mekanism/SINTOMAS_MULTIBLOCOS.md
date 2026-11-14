# SINTOMAS_MULTIBLOCOS — defeitos vistos em jogo nos multiblocos / Multiblock defects seen in game

> Relatados pelo dono do projeto (teste do Dynamic Tank, antes da rodada 3). / Reported by the project owner (Dynamic Tank test, before round 3).
> Cada sintoma tem a regra do Java que o explica e a correção esperada. / Each symptom has the Java rule that explains it and the expected fix.
> Regra do projeto / Project rule: sem "limites do Horizon" — o que falta na API vira pedido de fork (`API-REQUISITOS.md`). / No "Horizon limits": what the API lacks becomes a fork request.

## S1. Só dá para colocar outro bloco do tanque em cima de um bloco do tanque com shift / Placing another tank block on a tank block needs sneak

**PT.** Com o tanque ainda **não formado**, clicar num bloco dele (carcaça ou válvula) deveria *colocar* o bloco da mão. O port consome o clique (a válvula abre menu, e a carcaça também consome o uso), então só com shift funciona.
**Java:** `TileEntityMultiblock.onActivate` (`common/tile/prefab/TileEntityMultiblock.java:218-229`): `if (player.isShiftKeyDown() || !getMultiblock().isFormed()) return PASS_TO_DEFAULT_BLOCK_INTERACTION;` — só abre a GUI se **formado e sem shift**; caso contrário o clique passa para o comportamento normal do bloco (colocar o bloco da mão).
**Correção esperada:** o menu da válvula (e de qualquer porta/controlador de multibloco) só abre quando a estrutura é válida e o jogador não está agachado; fora disso o clique passa adiante. A carcaça nunca abre menu e nunca consome o clique.
**FORK-29:** uso de bloco condicional pelo mod (o mod decide `consumir` / `passar` por clique, com o estado do bloco).

**EN.** While the tank is **unformed**, clicking one of its blocks should *place* the held block. The port consumes the click, so only sneaking works. Java: the GUI opens only when formed and not sneaking, otherwise the click falls through to default block use. Fix: open the menu only if valid and not sneaking; the casing never consumes. **FORK-29:** mod-decided conditional block use.

## S2. Sem partículas ao formar / No particles when it forms

**PT.** Ao formar, o Mekanism toca uma animação de faíscas no contorno da estrutura (e, com a opção desligada, uma mensagem na barra: "multiblock formado").
**Java:** `TileEntityMultiblock.doMultiblockSparkle` (`:297-312`): no cliente, quando a estrutura acabou de formar (`!prevStructure && unformedTicks >= 5`) e o jogador está a até 40 blocos: `new SparkleAnimation(this, renderLocation, length-1, width-1, height-1).run()` se `enableMultiblockFormationParticles` (padrão **true**, `ClientConfig.java:90-91`), senão `MULTIBLOCK_FORMED_CHAT` em índigo.
**Correção esperada:** ao validar uma estrutura que antes não era válida, jogadores a até 40 blocos veem as faíscas ao longo das arestas da caixa (mesmos parâmetros do `SparkleAnimation`). Enquanto o fork não tiver partículas por mod, mostrar a mensagem de barra.
**FORK-30:** `world.spawn-particles` (tipo, posição, velocidade, quantidade, alcance de jogadores) para mods.

**EN.** On formation Mekanism plays a sparkle animation along the structure's edges (or an action-bar message when particles are off). Fix: on a newly valid structure, players within 40 blocks see the sparkles. Until the fork has mod particles, show the action-bar message. **FORK-30:** `world.spawn-particles`.

## S3. Quebrou o tanque inteiro e deixou só a válvula: continuou funcionando / Broke the whole tank leaving one valve: it kept working

**PT.** Uma válvula sozinha **não é** um tanque. No Java, sem estrutura formada o tile não expõe inventário nem capacidades; o conteúdo fica no cache (`MultiblockCache`) até a estrutura reformar. No port, a válvula tem tanques de capacidade enorme declarados no registro (`dynamic::SPEC_CAPACITY`), e `run_dynamic` ignora válvulas inválidas, então elas continuam aceitando fluido de baldes e canos como um tanque próprio.
**Correção esperada:** válvula/porta de uma estrutura inválida: filtros dos tanques em `NOTHING` e faces dos tanques/energia/itens fechadas (nada entra nem sai), menu fechado (ver S1), conteúdo preservado para a reforma. Vale para **todos** os multiblocos (portas do Matrix, da turbina, do reator, etc.).
**EN.** A lone valve is not a tank. In Java an unformed tile exposes no capabilities; contents stay in the cache until it re-forms. The port's valve declares huge tanks and `run_dynamic` skips invalid valves, so they keep acting as a standalone tank. Fix: for an invalid structure, filter tanks to `NOTHING`, close faces and menu, keep the content for the re-formation. Applies to all multiblocks.

## S4. A água não aparece fisicamente dentro do tanque / The water is not visible inside the tank

**PT.** O Mekanism desenha o fluido *dentro* da estrutura (vidro estrutural, ou qualquer tanque de paredes transparentes) com um renderizador de tile entity que preenche o interior até a altura proporcional ao conteúdo, com a textura e a cor do fluido.
**Java:** `client/render/tileentity/MultiblockTileEntityRenderer.java` e os renderizadores de cada multibloco (`RenderDynamicTank`, `RenderThermoelectricBoiler`, `RenderThermalEvaporationPlant`, `RenderInductionMatrix`, `RenderTurbine`, `RenderFissionReactor`, `RenderFusionReactor`, `RenderSPS`).
**Correção esperada:** primitiva do fork para desenhar fluido/químico num volume (caixa, textura, cor, fração de altura) ligada ao estado da estrutura, chamada pelo mod a cada mudança do conteúdo; o port usa-a em todos os multiblocos que o Java renderiza.
**FORK-19 (detalhado):** `render.multiblock-fluid(structure-id, min, max, fluid, fraction)` + remoção ao desmontar.

**EN.** Mekanism draws the fluid inside the structure with a tile-entity renderer filling the inside to a height proportional to the content, with the fluid's texture and colour. Fix: a fork primitive to draw fluid/chemical in a volume tied to the structure, used by every multiblock that Java renders. **FORK-19 (detailed):** `render.multiblock-fluid(...)`.

## S5. A Thermal Evaporation Plant não produzia salmoura / The Thermal Evaporation Plant produced no brine

**PT.** Causa: o `.hmod` testado **não continha nenhuma receita `mekanism:evaporating`**. O conversor (`tools/convert.py`, `MACHINE_TYPES`) não incluía os tipos `evaporating`, `activating` e `centrifuging`, então `evaporation::load_book()` (`recipe_types::recipes_of_type("mekanism:evaporating")`) devolvia um livro vazio: a água entrava no tanque de entrada e nada era produzido. O mesmo valia para o **Solar Neutron Activator** (`activating`) e a **Isotopic Centrifuge** (`centrifuging`). O inventário mecânico já mostrava `evaporating/brine.json` e `evaporating/lithium.json` como FALTA.
**Java:** `data/mekanism/recipe/evaporating/brine.json` (`input` 10 mB de `minecraft:water` → `output` 1 mB de `mekanism:brine`) e `lithium.json`.
**Correção (feita, ainda sem build):** os três tipos entraram em `MACHINE_TYPES` (commit `831bff0`, patch da r5). Conferência mecânica: todo tipo de receita de máquina que o Rust lê agora é convertido (`bin_*`, `clear_configuration`, `energy_conversion` e `mek_data` são tratados à parte).
**EN.** Cause: the tested `.hmod` held no `mekanism:evaporating` recipe: the converter's `MACHINE_TYPES` lacked `evaporating`, `activating` and `centrifuging`, so the plant's recipe book was empty (same for the Solar Neutron Activator and the Isotopic Centrifuge). Fixed by adding the three types; every machine recipe type the Rust reads is now converted. Not built yet.

## Estado atual dos sintomas / Current state of the symptoms (2026-11-10)

| # | Estado / State | O que foi feito e o que falta / What was done and what is missing |
|---|---|---|
| S1 | **FORK-29 implementado no fork (patch 0018, teste `examples/block-use`); mod ligado em `block_used`; falta compilar o mod real e testar em jogo** | Nenhuma correção no mod resolve: com `menu: true` o jogo abre o menu da válvula antes de qualquer outra coisa, e `item_used` devolvendo `false` só "deixa para o jogo". O código do m7 devolve `false` nesses casos (documentado, inerte). Só um clique condicional decidido pelo mod (FORK-29) permite colocar o bloco da mão em estrutura não formada. / No mod-side fix works. Needs FORK-29. |
| S2 | **Parcial** | Ao formar, jogadores a até 40 blocos recebem a mensagem índigo `Multiblock Formed` (texto exato de `multiblock.mekanism.formed.chat`), para todos os multiblocos (`multiblock_changed` em `lib.rs`). As faíscas dependem do FORK-30. / Message done; sparkles need FORK-30. |
| S3 | **Corrigido no mod (sem build)** | Válvulas e portas de estrutura inválida ficam seladas (filtros `NOTHING`, faces fechadas), com o conteúdo **preservado** para reformar (não é zerado). Dynamic Tank (m1), Boiler, Evaporation, SPS, Fission, Fusion, Matrix. Vedação por tick com o número real de tanques de cada bloco. |
| S4 | **ABERTO — FORK-31** | Falta a primitiva de renderização de fluido num volume. / Needs FORK-31. |
| S5 | **Corrigido (sem build)** | Receitas `evaporating`/`activating`/`centrifuging` convertidas. |
