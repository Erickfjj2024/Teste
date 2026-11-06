//! The infusion types the metallurgic infuser's screen draws, and the numbers it reads about its tank.
use crate::infuser::Tank;

/// The button id of the dump button (`GuiDumpButton`), past the side configuration window's.
pub const DUMP: i32 = 4000;

/// The infusion types, in the order the screen knows them (the number 0 is an empty tank): the same list as the converter's
/// `INFUSE_TYPES`, which draws the tank with each one's colour.
pub const TYPES: [&str; 8] = [
    "mekanism:bio",
    "mekanism:fungi",
    "mekanism:gold",
    "mekanism:refined_obsidian",
    "mekanism:redstone",
    "mekanism:diamond",
    "mekanism:tin",
    "mekanism:carbon",
];

/// The screen's two numbers about the tank: how much it holds, and which infusion type (1 to 8; 0 when empty).
pub fn screen(tank: &Tank) -> [i32; 2] {
    let which = TYPES.iter().position(|t| *t == tank.chemical).map_or(0, |i| i as i32 + 1);
    [tank.amount.min(i32::MAX as u64) as i32, if tank.amount == 0 { 0 } else { which }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screen_reads_the_amount_and_the_type() {
        let t = Tank { chemical: "mekanism:redstone".into(), amount: 250 };
        assert_eq!(screen(&t), [250, 5]);
        assert_eq!(screen(&Tank::default()), [0, 0]);
        // A chemical the screen does not know draws as none, with its amount.
        assert_eq!(screen(&Tank { chemical: "mekanism:steam".into(), amount: 3 }), [3, 0]);
    }
}
