# Ordem do dano: `player-damaged` e `item-hit-damage` / Order of damage hooks

## PT-BR
**Onde o Mekanism intercepta (Java 1.21.1-10.7.19.85, `CommonPlayerTickHandler`):**
1. `LivingIncomingDamageEvent` (`onEntityAttacked`, prioridade normal): roda em `LivingEntity.hurtServer` **depois** das guardas (invulnerável/creative, morto, imune a fogo) e **antes** de qualquer redução. Ordem dentro dele: (a) máscara de gás: se o dano está na tag `mekanism:is_preventable_magic` e o jogador usa máscara + tanque de scuba com gás, **cancela**; (b) `ItemMekaSuitArmor.getDamageAbsorbed(player, source, damage)` devolve a fração absorvida (gasta energia das peças) e o dano restante vai para `DamageContainer.setNewDamage` (0 cancela).
2. Só depois o vanilla aplica, na ordem do `DamageContainer`: armadura/resistência de armadura → encantamentos (proteção) → efeito de resistência → absorção.
3. `LivingFallEvent` (`livingFall`) é outro ponto: roda **antes** do dano de queda ser calculado e muda a **distância** de queda (os pés do MekaSuit/Free Runners absorvem); o dano de queda restante passa depois pelo item 1.
4. `EntityInvulnerabilityCheckEvent` (radiação) e `LivingDeathEvent` (lança-chamas) são outros pontos; não cobertos aqui.

**Onde o gancho está no Horizon:** `player-damaged` roda antes de `PlayerMotion::hurt_caused_by` (a armadura e os encantamentos do vanilla), depois das guardas de creative/espectador, em `hurt_player_from` (golpes de mobs/projéteis/ataques), `hurt_player_by` (dano de mods), explosões, bloco caindo e efeitos de poção (veneno/wither/dano instantâneo). Devolve o dano a seguir (0 ou menos cancela o golpe: nada de som, empurrão ou tempo de invulnerabilidade). Recebe o tipo, a quantidade, a entidade causadora, `is-magic` (`minecraft:magic`/`indirect_magic`), `is-fire`/`is-fall`/`is-explosion` (tags do tipo) e todas as tags.
`item-hit-damage` roda em `melee_damage`, sobre o atributo de dano de ataque (com os modificadores da arma), **antes** da escala de força de ataque, do acerto crítico e dos encantamentos — como o Meka-Tool do Java, que dá o dano pelo atributo.

**Diferenças conhecidas:** (1) o gancho do Horizon roda antes das guardas internas de `hurt` (tempo de invulnerabilidade, imunidade a fogo, escudo), então um golpe que o vanilla recusaria depois pode gastar energia do traje; (2) `player-items.get-inventory` é o do começo do tick: dois golpes no mesmo tick enxergam a mesma energia; (3) o `LivingFallEvent` não tem gancho ainda.

## English
**Where Mekanism intercepts (Java 1.21.1-10.7.19.85, `CommonPlayerTickHandler`):** `LivingIncomingDamageEvent` runs in `LivingEntity.hurtServer` after its guards and **before** any reduction: first the gas mask cancel for `mekanism:is_preventable_magic`, then `ItemMekaSuitArmor.getDamageAbsorbed` (spends the pieces' energy; the remainder goes to `DamageContainer.setNewDamage`, 0 cancels). Vanilla's armour, enchantment, resistance and absorption reductions follow. `LivingFallEvent` runs earlier and changes the fall **distance**.

**Where the hook sits in Horizon:** `player-damaged` runs before `PlayerMotion::hurt_caused_by` (vanilla's armour and enchantments), after the creative/spectator guards, in every player-damage path of the game (hits, mod damage, explosions, falling blocks, potion effects). It returns the damage to go on with (0 or less cancels). It gets the type, the amount, the causing entity, `is-magic`, `is-fire`, `is-fall`, `is-explosion` and all the type's tags. `item-hit-damage` runs in `melee_damage` on the attack damage attribute, before the attack-strength scale, the critical hit and the enchantments.

**Known differences:** the hook runs before `hurt`'s inner guards (invulnerable time, fire immunity, shield); inventories are as of the tick's start; `LivingFallEvent` has no hook yet.
