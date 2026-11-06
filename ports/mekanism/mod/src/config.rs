//! The side configuration of a machine, as Mekanism's `TileComponentConfig` keeps it: each of the six sides, relative to
//! the way the machine faces, has a data type for each transmission (items, energy) that says which slots a hopper reaches
//! there and whether energy comes in; items may also be ejected (pushed out) from the sides that output.
//! Pure rules, tested on the host.

/// What a side does (Mekanism's `DataType`; the numbers are its ordinals).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Data {
    None = 0,
    Input = 1,
    Output = 4,
    InputOutput = 7,
    Energy = 8,
    Extra = 9,
}

impl Data {
    /// Its name in the screen and the messages (`side_data.mekanism.*`).
    pub fn label(self) -> &'static str {
        match self {
            Data::None => "None",
            Data::Input => "Input",
            Data::Output => "Output",
            Data::InputOutput => "Input/Output",
            Data::Energy => "Energy",
            Data::Extra => "Extra",
        }
    }
    /// The name of its colour (`EnumColor`): input dark red, output blue, input/output purple, energy green...
    pub fn color_name(self) -> &'static str {
        match self {
            Data::None => "Light Gray",
            Data::Input => "Dark Red",
            Data::Output => "Blue",
            Data::InputOutput => "Purple",
            Data::Energy => "Green",
            Data::Extra => "Yellow",
        }
    }
    pub fn from_byte(b: u8) -> Data {
        match b {
            1 => Data::Input,
            4 => Data::Output,
            7 => Data::InputOutput,
            8 => Data::Energy,
            9 => Data::Extra,
            _ => Data::None,
        }
    }
}

/// The transmissions a machine configures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transmission {
    Items = 0,
    Energy = 1,
}

impl Transmission {
    pub fn label(self) -> &'static str {
        match self {
            Transmission::Items => "Items",
            Transmission::Energy => "Energy",
        }
    }
    /// The next transmission a configurator is set to (`ConfiguratorMode`'s order).
    pub fn next(self) -> Transmission {
        match self {
            Transmission::Items => Transmission::Energy,
            Transmission::Energy => Transmission::Items,
        }
    }
}

/// How a machine's slots are laid out, which says what its data types reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// An input slot (0) and an output slot (1).
    Electric,
    /// The input (0), the main output (1) and the secondary output (2).
    Sawmill,
    /// The main input (0), the extra input (1) and the output (2).
    Combiner,
}

/// The sides in the order a configuration keeps them (`RelativeSide.VALUES`).
pub const FRONT: usize = 0;
pub const LEFT: usize = 1;
pub const RIGHT: usize = 2;
pub const BACK: usize = 3;
pub const TOP: usize = 4;
pub const BOTTOM: usize = 5;

/// The data types a transmission supports, in the order a click goes through them (their ordinals').
pub fn supported(layout: Layout, t: Transmission) -> &'static [Data] {
    match (layout, t) {
        (_, Transmission::Energy) => &[Data::None, Data::Input],
        (Layout::Combiner, Transmission::Items) => {
            &[Data::None, Data::Input, Data::Output, Data::InputOutput, Data::Energy, Data::Extra]
        }
        (_, Transmission::Items) => &[Data::None, Data::Input, Data::Output, Data::InputOutput, Data::Energy],
    }
}

/// The next data type after `current` (or the one before): the next supported one in order, around.
pub fn cycle(layout: Layout, t: Transmission, current: Data, back: bool) -> Data {
    let list = supported(layout, t);
    let at = list.iter().position(|d| *d == current).unwrap_or(0);
    let n = list.len();
    list[(at + if back { n - 1 } else { 1 }) % n]
}

/// A machine's whole configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub items: [Data; 6],
    pub energy: [Data; 6],
    /// Whether it pushes its output items out (`ConfigInfo.isEjecting`).
    pub eject_items: bool,
}

/// What a new machine starts with: items in on the sides, out on the right, its charge slot at the back (a combiner's extra
/// input at the bottom) and eject off (`LightConfigInfo.MACHINE`, `EXTRA_MACHINE`); energy in everywhere.
pub fn default_config(layout: Layout) -> Config {
    let mut items = [Data::Input; 6];
    items[RIGHT] = Data::Output;
    items[BACK] = Data::Energy;
    if layout == Layout::Combiner {
        items[BOTTOM] = Data::Extra;
    }
    Config { items, energy: [Data::Input; 6], eject_items: false }
}

impl Config {
    pub fn side_mut(&mut self, t: Transmission) -> &mut [Data; 6] {
        match t {
            Transmission::Items => &mut self.items,
            Transmission::Energy => &mut self.energy,
        }
    }
    pub fn side(&self, t: Transmission) -> &[Data; 6] {
        match t {
            Transmission::Items => &self.items,
            Transmission::Energy => &self.energy,
        }
    }

    /// The bytes it is kept in: six item sides, six energy sides and the eject flag.
    pub fn encode(&self) -> Vec<u8> {
        let mut out: Vec<u8> = self.items.iter().chain(&self.energy).map(|d| *d as u8).collect();
        out.push(u8::from(self.eject_items));
        out
    }

    pub fn decode(bytes: &[u8], layout: Layout) -> Option<Config> {
        if bytes.len() < 13 {
            return None;
        }
        let mut c = default_config(layout);
        for i in 0..6 {
            c.items[i] = Data::from_byte(bytes[i]);
            c.energy[i] = Data::from_byte(bytes[6 + i]);
        }
        c.eject_items = bytes[12] == 1;
        Some(c)
    }

    /// The numbers the screen reads, four nibbles to a data slot: items' front to back, items' top and bottom with the
    /// eject flag, energy's front to back, energy's top and bottom.
    pub fn screen_words(&self) -> [i32; 4] {
        let pack = |d: &[Data]| d.iter().enumerate().fold(0i32, |a, (i, d)| a | ((*d as i32) << (4 * i)));
        [
            pack(&self.items[..4]),
            pack(&self.items[4..]) | (i32::from(self.eject_items) << 8),
            pack(&self.energy[..4]),
            pack(&self.energy[4..]),
        ]
    }
}

/// The slots of a machine a data type reaches.
pub fn slots_of(layout: Layout, data: Data) -> &'static [u32] {
    match (layout, data) {
        (_, Data::None | Data::Energy) => &[],
        (Layout::Electric, Data::Input) => &[0],
        (Layout::Electric, Data::Output) => &[1],
        (Layout::Electric, Data::InputOutput) => &[0, 1],
        (Layout::Sawmill, Data::Input) => &[0],
        (Layout::Sawmill, Data::Output) => &[1, 2],
        (Layout::Sawmill, Data::InputOutput) => &[0, 1, 2],
        (Layout::Combiner, Data::Input) => &[0],
        (Layout::Combiner, Data::Output) => &[2],
        (Layout::Combiner, Data::InputOutput) => &[0, 2],
        (Layout::Combiner, Data::Extra) => &[1],
        (_, Data::Extra) => &[],
    }
}

/// The faces of the world, in the game's order: down, up, north, south, west, east.
pub const DOWN: usize = 0;
pub const UP: usize = 1;
pub const NORTH: usize = 2;
pub const SOUTH: usize = 3;
pub const WEST: usize = 4;
pub const EAST: usize = 5;

fn clockwise(face: usize) -> usize {
    match face {
        NORTH => EAST,
        EAST => SOUTH,
        SOUTH => WEST,
        _ => NORTH,
    }
}

/// The face of the world a relative side is for a machine facing `facing` (`RelativeSide.getDirection`): the front is the
/// way it faces, its right the counterclockwise one seen from above.
pub fn face_of(facing: &str, side: usize) -> usize {
    let front = match facing {
        "east" => EAST,
        "south" => SOUTH,
        "west" => WEST,
        _ => NORTH,
    };
    let right = clockwise(clockwise(clockwise(front)));
    match side {
        FRONT => front,
        BACK => clockwise(clockwise(front)),
        LEFT => clockwise(front),
        RIGHT => right,
        TOP => UP,
        _ => DOWN,
    }
}

/// The relative side a face of the world is.
pub fn side_of(facing: &str, face: usize) -> usize {
    (0..6).find(|s| face_of(facing, *s) == face).unwrap_or(FRONT)
}

/// The slots each face reaches (down, up, north, south, west, east).
pub fn item_faces(layout: Layout, facing: &str, items: &[Data; 6]) -> Vec<Vec<u32>> {
    let mut faces = vec![Vec::new(); 6];
    for (side, data) in items.iter().enumerate() {
        faces[face_of(facing, side)] = slots_of(layout, *data).to_vec();
    }
    faces
}

/// Whether energy comes in through each face (down, up, north, south, west, east).
pub fn energy_in(facing: &str, energy: &[Data; 6]) -> [bool; 6] {
    let mut out = [false; 6];
    for (side, data) in energy.iter().enumerate() {
        out[face_of(facing, side)] = *data == Data::Input;
    }
    out
}

/// What a click on the "clear sides" button does (`GuiSideConfiguration.getTargetType`): when every side has the same data
/// type, the next (or the one before) of it, otherwise none.
pub fn clear_target(layout: Layout, t: Transmission, sides: &[Data; 6], back: bool) -> Data {
    if sides.iter().all(|d| *d == sides[0]) {
        cycle(layout, t, sides[0], back)
    } else {
        Data::None
    }
}

/// A menu button of the side configuration window, decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// A side of a transmission, next or (`back`) previous.
    Side { t: Transmission, side: usize, back: bool },
    Eject,
    /// Every side of a transmission (`back` for the right click).
    Clear { t: Transmission, back: bool },
}

/// The ids the screen's buttons send: 1000 + transmission * 100 + side * 10 + back, 2000 for the eject toggle and
/// 3000 + transmission * 10 + back for the clear button.
pub fn click_of(id: i32) -> Option<Click> {
    let t = |n: i32| match n {
        0 => Some(Transmission::Items),
        1 => Some(Transmission::Energy),
        _ => None,
    };
    match id {
        1000..=1199 => {
            let n = id - 1000;
            let side = ((n % 100) / 10) as usize;
            (side < 6 && n % 10 < 2).then_some(Click::Side { t: t(n / 100)?, side, back: n % 10 == 1 })
        }
        2000 => Some(Click::Eject),
        3000..=3019 => {
            let n = id - 3000;
            Some(Click::Clear { t: t(n / 10)?, back: n % 10 == 1 })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_the_originals() {
        let c = default_config(Layout::Electric);
        assert_eq!(c.items[RIGHT], Data::Output);
        assert_eq!(c.items[BACK], Data::Energy);
        assert_eq!(c.items[FRONT], Data::Input);
        assert_eq!(c.items[TOP], Data::Input);
        assert!(c.energy.iter().all(|d| *d == Data::Input));
        assert!(!c.eject_items);
        // A combiner has its extra input at the bottom.
        assert_eq!(default_config(Layout::Combiner).items[BOTTOM], Data::Extra);
        assert_eq!(default_config(Layout::Electric).items[BOTTOM], Data::Input);
    }

    #[test]
    fn a_click_goes_through_the_supported_types_in_order() {
        let mut d = Data::None;
        let mut seen = vec![d];
        for _ in 0..5 {
            d = cycle(Layout::Electric, Transmission::Items, d, false);
            seen.push(d);
        }
        assert_eq!(seen, [Data::None, Data::Input, Data::Output, Data::InputOutput, Data::Energy, Data::None]);
        assert_eq!(cycle(Layout::Combiner, Transmission::Items, Data::Energy, false), Data::Extra);
        assert_eq!(cycle(Layout::Combiner, Transmission::Items, Data::None, true), Data::Extra);
        assert_eq!(cycle(Layout::Electric, Transmission::Items, Data::None, true), Data::Energy);
        // Energy only takes or does not.
        assert_eq!(cycle(Layout::Sawmill, Transmission::Energy, Data::Input, false), Data::None);
        assert_eq!(cycle(Layout::Sawmill, Transmission::Energy, Data::None, true), Data::Input);
    }

    #[test]
    fn sides_turn_with_the_facing() {
        assert_eq!(face_of("north", FRONT), NORTH);
        assert_eq!(face_of("north", BACK), SOUTH);
        assert_eq!(face_of("north", RIGHT), WEST);
        assert_eq!(face_of("north", LEFT), EAST);
        assert_eq!(face_of("east", FRONT), EAST);
        assert_eq!(face_of("east", RIGHT), NORTH);
        assert_eq!(face_of("south", RIGHT), EAST);
        assert_eq!(face_of("west", RIGHT), SOUTH);
        assert_eq!(face_of("west", TOP), UP);
        for f in ["north", "east", "south", "west"] {
            let mut seen: Vec<usize> = (0..6).map(|s| face_of(f, s)).collect();
            seen.sort();
            assert_eq!(seen, vec![0, 1, 2, 3, 4, 5]);
            for s in 0..6 {
                assert_eq!(side_of(f, face_of(f, s)), s);
            }
        }
    }

    #[test]
    fn data_types_reach_slots_and_faces() {
        // A crusher facing north: the right (west) outputs, the back (south) is the charge slot, the rest take.
        let c = default_config(Layout::Electric);
        let faces = item_faces(Layout::Electric, "north", &c.items);
        assert_eq!(faces[WEST], vec![1]);
        assert_eq!(faces[SOUTH], Vec::<u32>::new());
        assert_eq!(faces[UP], vec![0]);
        assert_eq!(faces[NORTH], vec![0]);
        // A sawmill's output reaches both output slots; a combiner's extra side reaches the extra slot.
        assert_eq!(slots_of(Layout::Sawmill, Data::Output), &[1, 2]);
        let comb = item_faces(Layout::Combiner, "north", &default_config(Layout::Combiner).items);
        assert_eq!(comb[DOWN], vec![1]);
        assert_eq!(comb[WEST], vec![2]);
    }

    #[test]
    fn energy_comes_in_where_a_side_takes_it() {
        let mut c = default_config(Layout::Electric);
        assert_eq!(energy_in("north", &c.energy), [true; 6]);
        c.energy[RIGHT] = Data::None;
        let got = energy_in("north", &c.energy);
        assert!(!got[WEST]);
        assert!(got[EAST]);
    }

    #[test]
    fn it_is_kept_as_bytes_and_shown_as_nibbles() {
        let mut c = default_config(Layout::Sawmill);
        c.eject_items = true;
        c.energy[TOP] = Data::None;
        assert_eq!(Config::decode(&c.encode(), Layout::Sawmill), Some(c));
        assert_eq!(Config::decode(&[1, 2], Layout::Sawmill), None);
        let w = c.screen_words();
        // The items' front (1: input), left (1), right (4: output) and back (8: energy), a nibble each.
        assert_eq!(w[0], 0x8_4_1_1);
        // Top and bottom (input, input), then the eject flag in the third nibble.
        assert_eq!(w[1], 0x1_1 | (1 << 8));
        assert_eq!(w[2], 0x1_1_1_1);
        assert_eq!(w[3], 0x1 << 4);
    }

    #[test]
    fn the_clear_button_uniforms_then_cycles() {
        let c = default_config(Layout::Electric);
        // Mixed sides clear to none.
        assert_eq!(clear_target(Layout::Electric, Transmission::Items, &c.items, false), Data::None);
        // All the same: the next type.
        assert_eq!(clear_target(Layout::Electric, Transmission::Energy, &c.energy, false), Data::None);
        assert_eq!(clear_target(Layout::Electric, Transmission::Items, &[Data::None; 6], false), Data::Input);
    }

    #[test]
    fn button_ids_decode() {
        assert_eq!(click_of(1000), Some(Click::Side { t: Transmission::Items, side: 0, back: false }));
        assert_eq!(click_of(1151), Some(Click::Side { t: Transmission::Energy, side: 5, back: true }));
        assert_eq!(click_of(1170), None);
        assert_eq!(click_of(2000), Some(Click::Eject));
        assert_eq!(click_of(3011), Some(Click::Clear { t: Transmission::Energy, back: true }));
        assert_eq!(click_of(5), None);
    }
}
