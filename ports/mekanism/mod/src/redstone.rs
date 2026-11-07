//! Redstone control (`IRedstoneControl`, `TileEntityMekanism.isRedstoneActivated`): the mode a machine or a generator is set to
//! by the tab at the right of its screen, and whether it may work this tick for the signal reaching it.
use super::*;

/// `RedstoneControl`: ignore the signal, work with it, work without it, work for one tick on a new signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Control {
    #[default]
    Disabled,
    High,
    Low,
    Pulse,
}

/// The screen's buttons: the tab, left and right click.
pub const NEXT: i32 = 6000;
pub const PREVIOUS: i32 = 6001;

impl Control {
    pub fn from_byte(b: u8) -> Self {
        match b {
            1 => Control::High,
            2 => Control::Low,
            3 => Control::Pulse,
            _ => Control::Disabled,
        }
    }
    pub fn next(self) -> Self {
        Self::from_byte((self as u8 + 1) % 4)
    }
    pub fn previous(self) -> Self {
        Self::from_byte((self as u8 + 3) % 4)
    }
    /// `isRedstoneActivated`: `powered` is the signal now, `last` the one of the tick before.
    pub fn can_function(self, powered: bool, last: bool) -> bool {
        match self {
            Control::Disabled => true,
            Control::High => powered,
            Control::Low => !powered,
            Control::Pulse => powered && !last,
        }
    }
}

/// Where a kind of machine keeps its control in its data bytes (after what it already keeps), and its place in its screen's
/// numbers (the control, then the energy it still needs as two halves).
pub fn offset(kind: Kind) -> usize {
    match kind {
        Kind::Continuous | Kind::Reaction => 39,
        Kind::Generator => 9,
        _ => 37,
    }
}

pub fn menu_slot(kind: Kind) -> usize {
    match kind {
        Kind::Process => 20,
        Kind::Continuous | Kind::Reaction => 24,
        Kind::Generator => 15,
        _ => 16,
    }
}

/// Puts the control and the upgrades into a machine's saved bytes and its screen's numbers: the control, the energy it needs to be
/// full (two halves), then the upgrade window's (the ticks a recipe takes, the install progress, the eight counts, the mask).
pub fn stamp(kind: Kind, control: Control, needed: u64, upgrades: (u32, u32, [u8; 8], u64), saved: &mut Vec<u8>, data: &mut Vec<i32>) {
    let at = offset(kind);
    if saved.len() <= at {
        saved.resize(at + 1, 0);
    }
    saved[at] = control as u8;
    upgrades::put_counts(kind, &upgrades.2, saved);
    data.resize(menu_slot(kind), 0);
    data.extend([control as i32, (needed & 0xFFFF) as i32, ((needed >> 16) & 0xFFFF) as i32]);
    data.extend(upgrades::window_numbers(upgrades.0, upgrades.1, &upgrades.2, upgrades.3));
}

impl Mekanism {
    /// The control and what the signal says for this tick, remembered for the machine's run (`self.can`).
    pub(crate) fn read_control(&mut self, def: &Def, e: &BlockEntity) -> Control {
        let control = Control::from_byte(e.data.get(offset(def.kind)).copied().unwrap_or(0));
        let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let last = self.last_power.insert(key, powered).unwrap_or(false);
        self.can = control.can_function(powered, last);
        control
    }

    /// A click on the tab: the next control (left) or the one before (right), at once.
    pub(crate) fn redstone_button(&mut self, def: &Def, id: i32, e: &BlockEntity) -> Option<BlockEntityUpdate> {
        if def.kind == Kind::Tank {
            return None;
        }
        let now = Control::from_byte(e.data.get(offset(def.kind)).copied().unwrap_or(0));
        let control = if id == NEXT { now.next() } else { now.previous() };
        let mut data = e.data.clone();
        if data.len() <= offset(def.kind) {
            data.resize(offset(def.kind) + 1, 0);
        }
        data[offset(def.kind)] = control as u8;
        let mut menu = e.menu_data.clone();
        if menu.len() > menu_slot(def.kind) {
            menu[menu_slot(def.kind)] = control as i32;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data,
            items: None,
            energy: None,
            energy_faces: None,
            tanks: None,
            tank_faces: None,
            menu_data: Some(menu),
            transmitter_faces: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_control_says_when_a_machine_may_work() {
        // (powered, powered the tick before)
        let cases = [(false, false), (true, false), (true, true), (false, true)];
        let got = |c: Control| cases.map(|(p, l)| c.can_function(p, l));
        assert_eq!(got(Control::Disabled), [true; 4]);
        assert_eq!(got(Control::High), [false, true, true, false]);
        assert_eq!(got(Control::Low), [true, false, false, true]);
        // A pulse works for the one tick the signal arrives.
        assert_eq!(got(Control::Pulse), [false, true, false, false]);
    }

    #[test]
    fn the_modes_go_round_both_ways() {
        assert_eq!(Control::Disabled.next(), Control::High);
        assert_eq!(Control::Pulse.next(), Control::Disabled);
        assert_eq!(Control::Disabled.previous(), Control::Pulse);
        assert_eq!(Control::from_byte(Control::Low as u8), Control::Low);
    }

    #[test]
    fn the_control_is_kept_after_what_a_machine_saves() {
        let mut saved = vec![1, 2, 3];
        let mut data = vec![7; 5];
        let mut counts = [0u8; 8];
        counts[1] = 3;
        stamp(Kind::Electric, Control::Low, 70_000, (200, 4, counts, 26), &mut saved, &mut data);
        assert_eq!(saved.len(), 46);
        assert_eq!((saved[37], saved[39]), (2, 3));
        assert_eq!(data.len(), 31);
        assert_eq!((data[16], data[17], data[18], data[19], data[20]), (2, 4464, 1, 200, 4));
    }
}
