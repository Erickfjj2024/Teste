//! Bins (`TileEntityBin`): a block that holds a great many of one item (4096 in a basic one, up to 262144 in an ultimate one; a creative one never
//! runs out). The items sit in the bin's own count; two slots show them: an output (what a hopper, pipe or hand takes, a stack at most) and
//! an input (what is put in is absorbed into the count). Using an item on the bin puts the whole stack in; an empty hand opens its screen.
//!
//! A bin that is "active" (sneak-click in the original; the switch in the screen here) pushes a stack out of its bottom every half second.
//! Not here: taking by punching (a stack, or one when sneaking), putting everything of the same item in by clicking twice, items with data
//! (enchantments, names...: they lose it), the lock by the configurator.
use super::*;

pub const BINS: [(&str, u64); 5] = [
    ("mekanism:basic_bin", 4_096),
    ("mekanism:advanced_bin", 8_192),
    ("mekanism:elite_bin", 32_768),
    ("mekanism:ultimate_bin", 262_144),
    ("mekanism:creative_bin", u64::MAX / 2),
];
pub const OUTPUT: usize = 0;
pub const INPUT: usize = 1;
pub const TOGGLE_ACTIVE: i32 = 7700;
pub const TOGGLE_LOCK: i32 = 7701;
pub const EJECT_EVERY: u64 = 10;

pub fn capacity_of(block: &str) -> Option<u64> {
    BINS.iter().find(|(b, _)| *b == block).map(|(_, c)| *c)
}

/// What a bin holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bin {
    pub item: String,
    pub count: u64,
    pub locked: bool,
    pub active: bool,
}

pub fn encode(b: &Bin) -> Vec<u8> {
    let mut out = vec![1, u8::from(b.locked), u8::from(b.active), b.item.len().min(200) as u8];
    out.extend(b.item.bytes().take(200));
    out.extend(b.count.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<Bin> {
    if data.len() < 4 || data[0] != 1 {
        return None;
    }
    let n = usize::from(data[3]);
    let item = std::str::from_utf8(data.get(4..4 + n)?).ok()?.to_owned();
    let mut c = [0u8; 8];
    c.copy_from_slice(data.get(4 + n..12 + n)?);
    Some(Bin { item, count: u64::from_le_bytes(c), locked: data[1] == 1, active: data[2] == 1 })
}

/// How many of an item a stack holds at most (a guess from its name: the port cannot ask the game): 64 for the materials and blocks, 1 for
/// tools, armor and the like, so that no stack the bin shows is cut short by the game.
pub fn stack_size(item: &str) -> u64 {
    let name = item.split(':').nth(1).unwrap_or(item);
    const ONE: [&str; 17] = ["_sword", "_pickaxe", "_axe", "_shovel", "_hoe", "_helmet", "_chestplate", "_leggings", "_boots", "bow", "shield", "elytra", "_boat", "minecart", "potion", "saddle", "_bucket"];
    if ONE.iter().any(|s| name.contains(s)) || name == "bucket" || name.contains("enchanted_book") {
        return 1;
    }
    const SIXTY_FOUR: [&str; 40] = [
        "_ingot", "_nugget", "_dust", "_block", "_ore", "_planks", "_log", "cobblestone", "stone", "dirt", "sand", "gravel", "coal", "redstone", "diamond",
        "emerald", "_gem", "clay", "glass", "brick", "wool", "carpet", "concrete", "_slab", "_stairs", "_wall", "stick", "string", "feather", "bone", "flint",
        "leather", "paper", "torch", "ladder", "rail", "chest", "cactus", "netherrack", "obsidian",
    ];
    if item.starts_with("mekanism:") || SIXTY_FOUR.iter().any(|s| name.contains(s)) {
        return 64;
    }
    1
}

/// A tick of the contents: the input is absorbed, the output is refilled. `window` is what the output slot holds now (its item and count),
/// `intake` what the input slot holds; returns the new output and the part of the intake left over.
pub fn step(bin: &mut Bin, cap: u64, window: Option<(String, u64)>, shown_before: u64, intake: Option<(String, u64)>) -> (Option<(String, u64)>, Option<(String, u64)>) {
    // What the window lost since last time was taken out of the bin.
    let now = window.as_ref().map_or(0, |w| w.1);
    let taken = shown_before.saturating_sub(now);
    bin.count = bin.count.saturating_sub(taken);
    let mut left = None;
    if let Some((item, n)) = intake {
        if bin.item.is_empty() || (bin.count == 0 && !bin.locked) {
            bin.item = item.clone();
            bin.count = 0;
        }
        if bin.item == item {
            let room = cap.saturating_sub(bin.count);
            let add = n.min(room);
            bin.count += add;
            if add < n {
                left = Some((item, n - add));
            }
        } else {
            left = Some((item, n));
        }
    }
    if bin.count == 0 && !bin.locked {
        bin.item.clear();
    }
    // The window shows a stack of what is stored.
    let show = bin.count.min(stack_size(&bin.item));
    (if bin.item.is_empty() || show == 0 { None } else { Some((bin.item.clone(), show)) }, left)
}

impl Mekanism {
    pub(crate) fn run_bins(&mut self, bins: &[&BlockEntity], tick: u64) -> Vec<BlockEntityUpdate> {
        let presses = std::mem::take(&mut self.bin_buttons);
        let mut out = Vec::new();
        for e in bins {
            let block = e.state.split('[').next().unwrap_or("");
            let Some(cap) = capacity_of(block) else { continue };
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            let known = decode(&e.data);
            let mut bin = known.clone().unwrap_or_default();
            let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
            if known.is_none() {
                // A new bin looks toward the player that placed it.
                let front = cube::facing_name(front_for(e));
                if front != facing {
                    let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                    facing = front.to_owned();
                }
            }
            for (_, p, id) in presses.iter().filter(|(d, p, _)| *d == e.dimension && (p.x, p.y, p.z) == (e.pos.x, e.pos.y, e.pos.z)) {
                let _ = p;
                match *id {
                    TOGGLE_ACTIVE => bin.active = !bin.active,
                    TOGGLE_LOCK => bin.locked = !bin.locked && !bin.item.is_empty(),
                    _ => {}
                }
            }
            // The stacks players used on the bin go in.
            let mut intake = e.items.get(INPUT).cloned().flatten().map(|s| (s.item, u64::from(s.count)));
            if let Some(requests) = self.bin_inserts.remove(&key) {
                for (who, slot) in requests {
                    let Some(inv) = player_items::get_inventory(&who) else { continue };
                    let Some(Some(have)) = inv.get(slot as usize) else { continue };
                    if have.components != "{}" && !have.components.is_empty() {
                        continue;
                    }
                    let same = bin.item.is_empty() && intake.is_none() || bin.item == have.item;
                    if !same || bin.locked && bin.item != have.item {
                        continue;
                    }
                    let room = cap.saturating_sub(bin.count);
                    let add = u64::from(have.count).min(room);
                    if add == 0 {
                        continue;
                    }
                    if bin.item.is_empty() {
                        bin.item = have.item.clone();
                    }
                    bin.count += add;
                    let rest = (u64::from(have.count) > add).then(|| ItemStack { item: have.item.clone(), count: have.count - add as u32, components: have.components.clone() });
                    let _ = player_items::set_slot(&who, slot, rest.as_ref());
                }
            }
            let window = e.items.get(OUTPUT).cloned().flatten().map(|s| (s.item, u64::from(s.count)));
            let before = self.bin_shown.get(&key).copied().unwrap_or(0);
            let (show, left) = step(&mut bin, cap, window.clone(), before, intake.take());
            self.bin_shown.insert(key.clone(), show.as_ref().map_or(0, |s| s.1));
            let data = encode(&bin);
            let first = !self.bin_set.contains(&key) || known.is_none();
            let faces_changed = first || known.as_ref().is_some_and(|k| k.active != bin.active);
            if faces_changed {
                self.bin_set.insert(key.clone());
                let faces = if bin.active {
                    let mut f = vec![vec![1u32]; 6];
                    f[laser::DOWN] = vec![0];
                    f
                } else {
                    vec![vec![0u32, 1]; 6]
                };
                let _ = world::set_item_faces(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &faces);
                let _ = world::set_item_eject(&e.dimension, e.pos.x, e.pos.y, e.pos.z, bin.active);
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if bin.active { "true" } else { "false" });
            }
            let _ = tick;
            let slots = vec![
                show.as_ref().map(|(i, n)| ItemStack { item: i.clone(), count: *n as u32, components: String::new() }),
                left.as_ref().map(|(i, n)| ItemStack { item: i.clone(), count: (*n).min(64) as u32, components: String::new() }),
            ];
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let shown_cap = cap.min(i32::MAX as u64 * 2);
            let menu = vec![
                half(bin.count.min(u64::from(u32::MAX)), 0),
                half(bin.count.min(u64::from(u32::MAX)), 16),
                half(shown_cap, 0),
                half(shown_cap, 16),
                i32::from(bin.active),
                i32::from(bin.locked),
            ];
            let same_slots = e.items.get(OUTPUT).cloned().flatten().map(|s| (s.item, u64::from(s.count))) == show
                && e.items.get(INPUT).cloned().flatten().is_none()
                && left.is_none();
            if same_slots && data == e.data && menu == e.menu_data && !faces_changed {
                continue;
            }
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data,
                items: (!same_slots).then_some(slots),
                energy: None,
                energy_faces: None,
                tanks: None,
                tank_faces: None,
                menu_data: Some(menu),
                transmitter_faces: None,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bin(item: &str, count: u64) -> Bin {
        Bin { item: item.into(), count, locked: false, active: false }
    }

    #[test]
    fn items_put_in_are_counted_and_the_window_shows_a_stack() {
        let mut b = Bin::default();
        let (show, left) = step(&mut b, 4096, None, 0, Some(("minecraft:iron_ingot".into(), 64)));
        assert_eq!((b.count, left), (64, None));
        assert_eq!(show, Some(("minecraft:iron_ingot".to_owned(), 64)));
        // Another item is refused; so is what does not fit.
        let (_, left) = step(&mut b, 4096, show.clone(), 64, Some(("minecraft:gold_ingot".into(), 5)));
        assert_eq!(left, Some(("minecraft:gold_ingot".to_owned(), 5)));
        let mut full = bin("minecraft:iron_ingot", 4090);
        let (_, left) = step(&mut full, 4096, None, 0, Some(("minecraft:iron_ingot".into(), 10)));
        assert_eq!((full.count, left), (4096, Some(("minecraft:iron_ingot".to_owned(), 4))));
    }

    #[test]
    fn what_leaves_the_window_leaves_the_bin() {
        let mut b = bin("minecraft:cobblestone", 100);
        // The window showed 64 and now 20 are left: 44 were taken.
        let (show, _) = step(&mut b, 4096, Some(("minecraft:cobblestone".into(), 20)), 64, None);
        assert_eq!(b.count, 56);
        assert_eq!(show, Some(("minecraft:cobblestone".to_owned(), 56)));
        // Emptied, an unlocked bin forgets its item; a locked one keeps it.
        let mut e = bin("minecraft:cobblestone", 10);
        let (show, _) = step(&mut e, 4096, None, 10, None);
        assert_eq!((e.count, e.item.as_str(), show), (0, "", None));
        let mut l = Bin { locked: true, ..bin("minecraft:cobblestone", 10) };
        step(&mut l, 4096, None, 10, None);
        assert_eq!(l.item, "minecraft:cobblestone");
    }

    #[test]
    fn tools_show_one_at_a_time() {
        assert_eq!(stack_size("minecraft:diamond_pickaxe"), 1);
        assert_eq!(stack_size("minecraft:iron_ingot"), 64);
        assert_eq!(stack_size("mekanism:enriched_iron"), 64);
        assert_eq!(stack_size("minecraft:ender_pearl"), 1);
    }

    #[test]
    fn a_bin_remembers_what_it_holds() {
        let b = Bin { item: "mekanism:steel_ingot".into(), count: 123_456, locked: true, active: true };
        assert_eq!(decode(&encode(&b)), Some(b));
        assert_eq!(decode(&[]), None);
    }
}
