# REVISÃO — Chargepad / Chargepad review

**PT-BR.** Fonte: `TileEntityChargepad` (tag `v1.21.1-10.7.19.85`) × `mod/src/chargepad.rs`.
- Confere: capacidade 2 048 000 J (`fe()`), taxa 1 024 000 J por tick, caixa de varredura `[0,0,0,1,0.4,1]`, sons `stone_pressure_plate.click_on/off` (tom 0,8/0,7, volume 0,3), entradas de energia por trás e por baixo.
- **Corrigido:** o Java entrega ao item `min(taxa, energia do bloco, falta do item)` **e** respeita o limite de inserção do item; o port ignorava o limite. Campo `max_insert` em `RechargeableSlot`; teste `java_chargepad_gives_one_item_per_player_within_the_item_insert_limit`.
- **Aberto (FORK):** `scan_and_charge_entities` é esqueleto: precisa de consulta de entidades por caixa e dos manipuladores de energia do inventário do jogador (ver `PROPOSTA-API.wit`: `entities-in-box`, `player-energy-slots`).

**EN.** Source `TileEntityChargepad` × `chargepad.rs`. Matches: capacity, rate, scan box, sounds, input faces. **Fixed:** the item's insert limit is now honoured (`max_insert`) with a new test. **Open (FORK):** the entity scan is a stub; it needs an entity-by-box query and player inventory energy handlers.
