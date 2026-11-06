//! The chemicals the machines' screens draw (infusion types, gases and slurries) and the numbers they read about a tank.
use crate::process::Tank;

/// The button id of the infuser's dump button (`GuiDumpButton`), past the side configuration window's.
pub const DUMP: i32 = 4000;

pub use crate::chem_table::CHEMICALS;

/// A chemical's number for the screens (1 up to the table's length; 0 for none or one the table lacks): the same list as
/// the converter's gauges, written by it into `chem_table.rs`.
pub fn index(chemical: &str) -> i32 {
    CHEMICALS.iter().position(|c| *c == chemical).map_or(0, |i| i as i32 + 1)
}

/// The screen's two numbers about a tank: how much it holds, and which chemical (0 when empty).
pub fn screen(tank: &Tank) -> [i32; 2] {
    [tank.amount.min(i32::MAX as u64) as i32, if tank.amount == 0 { 0 } else { index(&tank.chemical) }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screen_reads_the_amount_and_the_chemical() {
        let first = CHEMICALS[0];
        let t = Tank { chemical: first.into(), amount: 250 };
        assert_eq!(screen(&t), [250, 1]);
        assert_eq!(screen(&Tank::default()), [0, 0]);
        // A chemical the screen does not know draws as none, with its amount.
        assert_eq!(screen(&Tank { chemical: "mekanism:nothing".into(), amount: 3 }), [3, 0]);
        assert!(index("mekanism:oxygen") > 0 && index("mekanism:redstone") > 0);
    }
}
