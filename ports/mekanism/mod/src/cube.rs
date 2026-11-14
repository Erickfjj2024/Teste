//! The energy cube's screen (`GuiEnergyCube`): its side configuration window shows the cube's energy modes by the sides
//! relative to its front, the discharge slot converts redstone to energy (`EnergyInventorySlot.fillOrConvert`, with the
//! `mekanism:energy_conversion` recipes) and the screen reads the numbers the cube puts in its data. Pure rules.

use crate::config::{self, Click, Data, Layout, Transmission};
use crate::sides::{self, Mode};

/// The discharge slot and the charge slot of the cube's container.
pub const DISCHARGE: usize = 0;
pub const CHARGE: usize = 1;

/// The energy in FE an item gives in the discharge slot (the recipes of `energy_conversion`: redstone 10_000 J = 4_000 FE, and a block of it 90_000 J = 36_000 FE).
pub fn conversion(item: &str) -> Option<u64> {
    match item {
        "minecraft:redstone" => Some(4_000),
        "minecraft:redstone_block" => Some(36_000),
        _ => None,
    }
}

/// `EnergyInventorySlot.fillOrConvert` for a machine's charge slot: the item in it that turns into energy (`conversion`) gives its energy when the whole of it fits
/// under `capacity`, and one is taken from the stack. Returns the new energy and the slot's new contents, or none when nothing was converted.
pub fn fill_from_slot(slot: &Option<horizon_mod_sdk::server::ItemStack>, energy: u64, capacity: u64) -> Option<(u64, Option<horizon_mod_sdk::server::ItemStack>)> {
    let stack = slot.as_ref()?;
    let gain = conversion(&stack.item)?;
    if energy + gain > capacity {
        return None;
    }
    let left = stack.count.saturating_sub(1);
    Some((energy + gain, if left == 0 { None } else { Some(horizon_mod_sdk::server::ItemStack { count: left, ..stack.clone() }) }))
}

/// The name of the way a horizontal face looks, as a machine's `facing` says it.
pub fn facing_name(front: usize) -> &'static str {
    match front {
        sides::EAST => "east",
        sides::SOUTH => "south",
        sides::WEST => "west",
        _ => "north",
    }
}

/// The data type a mode shows as in the window (the four an energy cube supports).
pub fn data_of(mode: Mode) -> Data {
    match mode {
        Mode::None => Data::None,
        Mode::Input => Data::Input,
        Mode::Output => Data::Output,
        Mode::Both => Data::InputOutput,
    }
}

/// The front a cube's data holds (the byte after its six modes), or, for a cube placed before it was kept, the side that
/// outputs (the default's front), north when none does.
pub fn front_of(data: &[u8], modes: &[Mode; 6]) -> usize {
    if let Some(b) = data.get(6) {
        let f = usize::from(*b);
        if (sides::NORTH..=sides::EAST).contains(&f) {
            return f;
        }
    }
    (sides::NORTH..=sides::EAST).find(|f| modes[*f] == Mode::Output).unwrap_or(sides::NORTH)
}

/// The item modes a cube's data holds after its front (bytes 7..12); none for a cube saved before they existed.
pub fn item_modes(data: &[u8]) -> [Mode; 6] {
    let mut m = [Mode::None; 6];
    for (i, b) in data.iter().skip(7).take(6).enumerate() {
        m[i] = Mode::from_byte(*b);
    }
    m
}

/// The bytes a cube keeps: the mode of each face of the world, its front, then the item mode of each face.
pub fn encode(modes: &[Mode; 6], front: usize, item_modes: &[Mode; 6]) -> Vec<u8> {
    let mut out = sides::encode(modes);
    out.push(front as u8);
    out.extend(item_modes.iter().map(|m| *m as u8));
    out
}

/// The four words of the energy tab's sides for the screen (`Config::screen_words`), by the sides relative to the front.
pub fn words(modes: &[Mode; 6], front: usize) -> [i32; 10] {
    let facing = facing_name(front);
    let mut cfg = config::default_config(Layout::Electric);
    *cfg.side_mut(Transmission::Items) = [Data::None; 6];
    for side in 0..6 {
        cfg.side_mut(Transmission::Energy)[side] = data_of(modes[config::face_of(facing, side)]);
    }
    cfg.screen_words()
}

/// A press of the window's buttons on the cube's modes; true when they changed.
pub fn press(modes: &mut [Mode; 6], front: usize, click: Click) -> bool {
    let before = *modes;
    match click {
        Click::Side { t: Transmission::Energy, side, back } => {
            let face = config::face_of(facing_name(front), side);
            modes[face] = modes[face].step(back);
        }
        Click::Clear { t: Transmission::Energy, back } => {
            *modes = if modes.iter().all(|m| *m == modes[0]) { [modes[0].step(back); 6] } else { [Mode::None; 6] };
        }
        _ => {}
    }
    *modes != before
}

/// The screen's numbers: the input rate, the energy and capacity as low and high 16 bits (kept to what 32 bits hold, the
/// creative cube's too, so the gauge reads full), the status (unused), then `words`.
pub fn screen(rate: u64, energy: u64, capacity: u64, modes: &[Mode; 6], front: usize) -> Vec<i32> {
    let cap = capacity.min(u64::from(u32::MAX));
    let held = if energy >= capacity { cap } else { energy.min(cap) };
    let half = |v: u64, shift: u32| ((v >> shift) & 0xFFFF) as i32;
    let mut data = vec![
        rate.min(i32::MAX as u64) as i32,
        half(held, 0),
        half(held, 16),
        half(cap, 0),
        half(cap, 16),
        0,
    ];
    data.extend(words(modes, front));
    // Where its front looks, for the window's icons of the blocks beside each side.
    data.push(crate::redstone::facing_number(facing_name(front)));
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use sides::{DOWN, EAST, NORTH, SOUTH, UP, WEST};

    #[test]
    fn redstone_gives_the_energy_the_recipes_say() {
        assert_eq!(conversion("minecraft:redstone"), Some(4_000));
        assert_eq!(conversion("minecraft:redstone_block"), Some(36_000));
        assert_eq!(conversion("minecraft:stick"), None);
    }

    #[test]
    fn the_front_is_kept_after_the_modes_and_found_in_an_old_cube() {
        let modes = sides::default_modes(EAST);
        assert_eq!(encode(&modes, EAST, &[Mode::None; 6]).len(), 13);
        assert_eq!(front_of(&encode(&modes, WEST, &[Mode::None; 6]), &modes), WEST);
        // A cube kept with six bytes: its front is the side that outputs.
        assert_eq!(front_of(&sides::encode(&modes), &modes), EAST);
        assert_eq!(front_of(&[], &[Mode::None; 6]), NORTH);
        // The item modes ride after the front, and an old cube has none.
        let mut items = [Mode::None; 6];
        items[DOWN] = Mode::Input;
        items[EAST] = Mode::Both;
        assert_eq!(item_modes(&encode(&modes, EAST, &items)), items);
        assert_eq!(item_modes(&encode(&modes, EAST, &[Mode::None; 6])), [Mode::None; 6]);
    }

    #[test]
    fn the_window_reads_the_sides_from_the_front() {
        // A cube facing south outputs on its front and takes on the rest.
        let modes = sides::default_modes(SOUTH);
        let w = words(&modes, SOUTH);
        // Energy words are the third and fourth: front (output 4), left, right, back (input 1) in the low nibbles.
        assert_eq!(w[2] & 0xF, 4);
        assert_eq!((w[2] >> 4) & 0xF, 1);
        assert_eq!(w[3] & 0xF, 1);
        assert_eq!((w[3] >> 4) & 0xF, 1);
    }

    #[test]
    fn a_press_changes_the_face_the_side_is() {
        let mut modes = [Mode::None; 6];
        // Facing north, the left side is the east face (as for machines) and the top is up.
        assert!(press(&mut modes, NORTH, Click::Side { t: Transmission::Energy, side: config::LEFT, back: false }));
        assert_eq!(modes[EAST], Mode::Input);
        assert!(press(&mut modes, NORTH, Click::Side { t: Transmission::Energy, side: config::TOP, back: false }));
        assert_eq!(modes[UP], Mode::Input);
        assert!(press(&mut modes, NORTH, Click::Side { t: Transmission::Energy, side: config::BOTTOM, back: true }));
        assert_eq!(modes[DOWN], Mode::Both);
        // Items are not the cube's to configure here.
        assert!(!press(&mut modes, NORTH, Click::Eject(Transmission::Items)));
    }

    #[test]
    fn clear_sets_every_side_alike() {
        let mut modes = sides::default_modes(NORTH);
        assert!(press(&mut modes, NORTH, Click::Clear { t: Transmission::Energy, back: false }));
        assert_eq!(modes, [Mode::None; 6]);
        assert!(press(&mut modes, NORTH, Click::Clear { t: Transmission::Energy, back: false }));
        assert_eq!(modes, [Mode::Input; 6]);
    }

    #[test]
    fn the_screen_numbers_are_seventeen() {
        let d = screen(5, 4_000_000, 4_000_000, &sides::default_modes(NORTH), NORTH);
        assert_eq!(d.len(), 17);
        assert_eq!((d[0], d[16]), (5, 0));
        // The front it looks to is the last number (north 0, east 1, south 2, west 3).
        assert_eq!(screen(0, 0, 4, &sides::default_modes(sides::SOUTH), sides::SOUTH)[16], 2);
        // A creative cube's trillions read as full in 32 bits.
        let c = screen(0, 1_000_000_000_000, 1_000_000_000_000, &[Mode::None; 6], NORTH);
        assert_eq!((c[1], c[2]), (0xFFFF, 0xFFFF));
        assert_eq!((c[3], c[4]), (0xFFFF, 0xFFFF));
    }

    #[test]
    fn a_charge_slot_converts_one_item_when_it_fits() {
        let stack = |n| Some(horizon_mod_sdk::server::ItemStack { item: "minecraft:redstone".into(), count: n, components: String::new() });
        assert_eq!(fill_from_slot(&stack(3), 0, 10_000), Some((4_000, stack(2))));
        assert_eq!(fill_from_slot(&stack(1), 0, 10_000), Some((4_000, None)));
        assert_eq!(fill_from_slot(&stack(1), 8_000, 10_000), None, "the whole conversion must fit");
        assert_eq!(fill_from_slot(&None, 0, 10_000), None);
    }
}
