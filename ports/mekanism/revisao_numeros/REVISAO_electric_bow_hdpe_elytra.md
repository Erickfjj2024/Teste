# REVISÃO — Electric Bow e HDPE Reinforced Elytra / Electric Bow and HDPE Elytra review

**Fontes / sources:** `ItemElectricBow`, `ItemHDPEElytra`, `MekanismItems:184`, `GearConfig:145-151` (1.21.1-10.7.19.85) × `electric_bow.rs`, `hdpe_elytra.rs`, `tools/convert.py`.

## Electric Bow
**Conferido:** capacidade 120 000 J (48 000 FE), recarga 600 J/t (240 FE/t), tiro 120 J (48 FE), tiro de fogo 1 200 J (480 FE), componente `mekanism:electric_bow_mode` (booleano, padrão falso); o tiro só sai com energia (`releaseUsing` simula a extração) e só cobra se há munição (`shoot`) e fora do criativo; Flame nível 1 no modo de fogo.
**Corrigido:** (1) o item era um graveto (`minecraft:stick`) no `.hmod`: agora é um arco do jogo (`template minecraft:bow`, pilha 1, rara, sem durabilidade: a barra é a energia). (2) A mensagem de troca de modo é `Fire Mode: on/off` (chave `tooltip.mekanism.fire_mode`, liga/desliga em minúsculas), não `ON/OFF`.
**Aberto (FORK):** a energia **não é cobrada** nem bloqueia o tiro, e o modo de fogo não põe fogo na flecha, porque o jogo não avisa o mod quando a corda é solta (`item-released`, `item-shot`, `set-entity-fire`, em `PROPOSTA-API.wit`). **Por isso a troca de modo não está ligada:** mostraria "Fire Mode: on" sem efeito. O Java troca o modo com a tecla de modo e a rolagem com sneak (`IAttachmentBasedModeItem`); o Android não tem essa tecla (`item-mode-changed`). Até lá o arco funciona como um arco comum, de graça (**diferença conhecida**, pior que o original em custo, igual em dano).

## HDPE Reinforced Elytra
**Conferido:** o Java é um `ElytraItem` com durabilidade 648, raridade RARE, slot do peito e reparo com `mekanism:hdpe_sheet`.
**Corrigido:** o item era um graveto; agora é um élitro do jogo (`template minecraft:elytra`, `max_damage 648`, `repairable hdpe_sheet`, rara). O planar, o desgaste e o reparo na bigorna são do jogo; **a lógica própria de `hdpe_elytra.rs` (dano de voo, reparo por folha) não é usada** e fica só como documentação: o jogo já faz o que o `ElytraItem` faz.
**Aberto:** a textura/camada do élitro em uso (`MekanismElytraLayer`) e o `equippable` com `glider` dependem de o `template minecraft:elytra` herdá-los na versão do fork; só o jogo confirma.
**EN.** Electric Bow: constants confirmed; item now a game bow; energy cost, flame and the mode toggle need release/shot hooks (`PROPOSTA-API.wit`). HDPE Elytra: item now a game elytra with 648 durability, rare, hdpe_sheet repair; the module's own logic is unused.
