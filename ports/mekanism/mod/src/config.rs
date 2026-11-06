//! The side configuration of a machine, as Mekanism's `TileComponentConfig` keeps it: each of the six sides, relative to
//! the way the machine faces, has a data type for each transmission (items, energy) that says which slots a hopper reaches
//! there and whether energy comes in; items may also be ejected (pushed out) from the sides that output.
//! Pure rules, tested on the host.

/// What a side does (Mekanism's `DataType`; the numbers are its ordinals).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Data {
    None = 0,
    Input = 1,
    Input1 = 2,
    Input2 = 3,
    Output = 4,
    Output1 = 5,
    Output2 = 6,
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
            Data::Input1 => "Input (1)",
            Data::Input2 => "Input (2)",
            Data::Output => "Output",
            Data::Output1 => "Output (1)",
            Data::Output2 => "Output (2)",
            Data::InputOutput => "Input/Output",
            Data::Energy => "Energy",
            Data::Extra => "Extra",
        }
    }
    /// The name of its colour (`EnumColor`).
    pub fn color_name(self) -> &'static str {
        match self {
            Data::None => "Light Gray",
            Data::Input | Data::Input1 => "Dark Red",
            Data::Input2 => "Orange",
            Data::Output | Data::Output1 => "Blue",
            Data::Output2 => "Cyan",
            Data::InputOutput => "Purple",
            Data::Energy => "Green",
            Data::Extra => "Yellow",
        }
    }
    pub fn from_byte(b: u8) -> Data {
        match b {
            1 => Data::Input,
            2 => Data::Input1,
            3 => Data::Input2,
            4 => Data::Output,
            5 => Data::Output1,
            6 => Data::Output2,
            7 => Data::InputOutput,
            8 => Data::Energy,
            9 => Data::Extra,
            _ => Data::None,
        }
    }
}

/// The transmissions a machine can configure (Mekanism's `TransmissionType`; the numbers are this port's and fix where
/// each one's words are in the screen's data: 6 + 2 * number).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transmission {
    Items = 0,
    Energy = 1,
    Fluids = 2,
    Chemicals = 3,
    Heat = 4,
}

/// Every transmission, in number order.
pub const TRANSMISSIONS: [Transmission; 5] = [
    Transmission::Items,
    Transmission::Energy,
    Transmission::Fluids,
    Transmission::Chemicals,
    Transmission::Heat,
];

impl Transmission {
    pub fn label(self) -> &'static str {
        match self {
            Transmission::Items => "Items",
            Transmission::Energy => "Energy",
            Transmission::Fluids => "Fluids",
            Transmission::Chemicals => "Chemicals",
            Transmission::Heat => "Heat",
        }
    }
    /// The mode a configurator goes to next when sneaking in the air: the original's order (`ConfiguratorMode`), items,
    /// fluids, chemicals, energy, heat.
    pub fn next_tool(self) -> Transmission {
        match self {
            Transmission::Items => Transmission::Fluids,
            Transmission::Fluids => Transmission::Chemicals,
            Transmission::Chemicals => Transmission::Energy,
            Transmission::Energy => Transmission::Heat,
            Transmission::Heat => Transmission::Items,
        }
    }
    pub fn from_number(n: i32) -> Option<Transmission> {
        TRANSMISSIONS.get(usize::try_from(n).ok()?).copied()
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
    /// The metallurgic infuser: the input (0), the output (1) and the infusion item slot (2), which the extra data type
    /// reaches; its infusion tank is configured as the chemical transmission.
    Infuser,
    /// The purification and injection chambers: slots as the infuser's, a chemical tank that only takes in.
    Adv,
    /// The osmium compressor: slots as the infuser's, a chemical tank that can also give.
    Compressor,
    /// The chemical oxidizer: the input (0) and the chemical item slot (1); its tank only gives.
    Oxidizer,
    /// The chemical dissolution chamber: slots as the infuser's, a tank that takes and a tank that gives.
    Dissolution,
    /// The chemical crystallizer: the chemical item slot (0) and the output (1); its tank only takes in.
    Crystallizer,
}

impl Layout {
    /// The transmissions a machine of this layout configures, in the order of its tabs (`getTransmissions`): the first is
    /// the one the window opens on.
    pub fn transmissions(self) -> &'static [Transmission] {
        match self {
            Layout::Electric | Layout::Sawmill | Layout::Combiner => &[Transmission::Items, Transmission::Energy],
            _ => &[Transmission::Items, Transmission::Chemicals, Transmission::Energy],
        }
    }
    /// Whether it has a third item slot the extra data type reaches (the chemical item slot of the infuser's family).
    pub fn extra_slot(self) -> bool {
        matches!(self, Layout::Infuser | Layout::Adv | Layout::Compressor | Layout::Dissolution)
    }
    /// Whether the chemical tanks push into the blocks beside them by themselves where a side gives (what `eject` says):
    /// the other machines are emptied by pipes, which the port has not.
    pub fn pushes_chemicals(self) -> bool {
        matches!(self, Layout::Oxidizer | Layout::Dissolution)
    }
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
        // `setupIOConfig` of the chemical tanks: none, input, output and both.
        (Layout::Infuser | Layout::Compressor | Layout::Dissolution, Transmission::Chemicals) => {
            &[Data::None, Data::Input, Data::Output, Data::InputOutput]
        }
        // `setupInputConfig`: a tank that only takes in, and `setupOutputConfig`: one that only gives.
        (Layout::Adv | Layout::Crystallizer, Transmission::Chemicals) => &[Data::None, Data::Input],
        (Layout::Oxidizer, Transmission::Chemicals) => &[Data::None, Data::Output],
        // No other machine has fluid, chemical or heat tanks yet: the machines that do will list their data types here.
        (_, Transmission::Fluids | Transmission::Chemicals | Transmission::Heat) => &[],
        (_, Transmission::Energy) => &[Data::None, Data::Input],
        (l, Transmission::Items) if l == Layout::Combiner || l.extra_slot() => {
            &[Data::None, Data::Input, Data::Output, Data::InputOutput, Data::Energy, Data::Extra]
        }
        (_, Transmission::Items) => &[Data::None, Data::Input, Data::Output, Data::InputOutput, Data::Energy],
    }
}

/// The next data type after `current` (or the one before): the next supported one in order, around.
pub fn cycle(layout: Layout, t: Transmission, current: Data, back: bool) -> Data {
    let list = supported(layout, t);
    if list.is_empty() {
        return current;
    }
    let at = list.iter().position(|d| *d == current).unwrap_or(0);
    let n = list.len();
    list[(at + if back { n - 1 } else { 1 }) % n]
}

/// A machine's whole configuration: the data type of each side for each transmission, and whether each ejects
/// (`ConfigInfo.isEjecting`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub sides: [[Data; 6]; 5],
    pub eject: [bool; 5],
}

/// What a new machine starts with: items in on the sides, out on the right, its charge slot at the back (a combiner's extra
/// input at the bottom) and eject off (`LightConfigInfo.MACHINE`, `EXTRA_MACHINE`); energy in everywhere.
pub fn default_config(layout: Layout) -> Config {
    let mut items = [Data::Input; 6];
    items[RIGHT] = Data::Output;
    items[BACK] = Data::Energy;
    if layout == Layout::Combiner || layout.extra_slot() {
        items[BOTTOM] = Data::Extra;
    }
    let mut sides = [[Data::None; 6]; 5];
    sides[Transmission::Items as usize] = items;
    sides[Transmission::Energy as usize] = [Data::Input; 6];
    let mut eject = [false; 5];
    let chem = &mut sides[Transmission::Chemicals as usize];
    match layout {
        // `LightConfigInfo.INPUT_ONLY`: the tank takes in on every side.
        Layout::Infuser | Layout::Adv | Layout::Crystallizer => *chem = [Data::Input; 6],
        // `OUT_NO_EJECT`: in everywhere, out on the right.
        Layout::Compressor => {
            *chem = [Data::Input; 6];
            chem[RIGHT] = Data::Output;
        }
        // `OUT_EJECT`: in everywhere, out on the right, and it pushes out.
        Layout::Dissolution => {
            *chem = [Data::Input; 6];
            chem[RIGHT] = Data::Output;
            eject[Transmission::Chemicals as usize] = true;
        }
        // `RIGHT_OUTPUT`: only the right gives, and it pushes out.
        Layout::Oxidizer => {
            *chem = [Data::None; 6];
            chem[RIGHT] = Data::Output;
            eject[Transmission::Chemicals as usize] = true;
        }
        Layout::Electric | Layout::Sawmill | Layout::Combiner => {}
    }
    Config { sides, eject }
}

impl Config {
    pub fn side(&self, t: Transmission) -> &[Data; 6] {
        &self.sides[t as usize]
    }
    pub fn side_mut(&mut self, t: Transmission) -> &mut [Data; 6] {
        &mut self.sides[t as usize]
    }
    pub fn items(&self) -> &[Data; 6] {
        self.side(Transmission::Items)
    }
    pub fn energy(&self) -> &[Data; 6] {
        self.side(Transmission::Energy)
    }
    pub fn ejects(&self, t: Transmission) -> bool {
        self.eject[t as usize]
    }

    /// The bytes it is kept in: six sides for each of the five transmissions, then five eject flags.
    pub fn encode(&self) -> Vec<u8> {
        let mut out: Vec<u8> = self.sides.iter().flatten().map(|d| *d as u8).collect();
        out.extend(self.eject.iter().map(|e| u8::from(*e)));
        out
    }

    /// A configuration read back: the current format, or the first one (items and energy sides and the items' eject flag,
    /// 13 bytes), which machines placed with it keep.
    pub fn decode(bytes: &[u8], layout: Layout) -> Option<Config> {
        let mut c = default_config(layout);
        if bytes.len() >= 35 {
            for (i, d) in c.sides.iter_mut().flatten().enumerate() {
                *d = Data::from_byte(bytes[i]);
            }
            for (i, e) in c.eject.iter_mut().enumerate() {
                *e = bytes[30 + i] == 1;
            }
            return Some(c);
        }
        if bytes.len() >= 13 {
            for i in 0..6 {
                c.sides[Transmission::Items as usize][i] = Data::from_byte(bytes[i]);
                c.sides[Transmission::Energy as usize][i] = Data::from_byte(bytes[6 + i]);
            }
            c.eject[Transmission::Items as usize] = bytes[12] == 1;
            return Some(c);
        }
        None
    }

    /// The numbers the screen reads, four nibbles to a data slot, two data slots for each transmission (its sides from the
    /// front to the back, then its top and bottom with its eject flag in the third nibble).
    pub fn screen_words(&self) -> [i32; 10] {
        let pack = |d: &[Data]| d.iter().enumerate().fold(0i32, |a, (i, d)| a | ((*d as i32) << (4 * i)));
        let mut out = [0i32; 10];
        for (t, sides) in self.sides.iter().enumerate() {
            out[2 * t] = pack(&sides[..4]);
            out[2 * t + 1] = pack(&sides[4..]) | (i32::from(self.eject[t]) << 8);
        }
        out
    }
}

/// The slots of a machine a data type reaches.
pub fn slots_of(layout: Layout, data: Data) -> &'static [u32] {
    match (layout, data) {
        (_, Data::None | Data::Energy | Data::Input1 | Data::Input2 | Data::Output1 | Data::Output2) => &[],
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
        (l, Data::Input) if l.extra_slot() => &[0],
        (l, Data::Output) if l.extra_slot() => &[1],
        (l, Data::InputOutput) if l.extra_slot() => &[0, 1],
        (l, Data::Extra) if l.extra_slot() => &[2],
        // The oxidizer's and the crystallizer's two slots.
        (Layout::Oxidizer | Layout::Crystallizer, Data::Input) => &[0],
        (Layout::Oxidizer | Layout::Crystallizer, Data::Output) => &[1],
        (Layout::Oxidizer | Layout::Crystallizer, Data::InputOutput) => &[0, 1],
        (_, Data::Input | Data::Output | Data::InputOutput | Data::Extra) => &[],
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

/// What each face of the world does with the machine's tanks, in the order of its tanks (the one the chemical goes into
/// first, then the one it comes out in): none, in, out or both (`FlowSide` numbers 0 to 3). A machine with one tank gives
/// and takes by that tank; one with two takes by the first and gives by the second. A tank gives by itself only where the
/// machine pushes (`pushes_chemicals`) and its eject is on.
pub fn tank_flow(layout: Layout, facing: &str, cfg: &Config) -> Vec<[u8; 6]> {
    let sides = cfg.side(Transmission::Chemicals);
    let pushes = layout.pushes_chemicals() && cfg.ejects(Transmission::Chemicals);
    let two = layout == Layout::Dissolution;
    let mut first = [0u8; 6];
    let mut second = [0u8; 6];
    for (side, data) in sides.iter().enumerate() {
        let face = face_of(facing, side);
        let takes = matches!(data, Data::Input | Data::InputOutput);
        let gives = matches!(data, Data::Output | Data::InputOutput) && pushes;
        if two {
            first[face] = u8::from(takes);
            second[face] = if gives { 2 } else { 0 };
        } else {
            first[face] = u8::from(takes) + if gives { 2 } else { 0 };
        }
    }
    if two { vec![first, second] } else { vec![first] }
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
    /// The eject toggle of a transmission.
    Eject(Transmission),
    /// Every side of a transmission (`back` for the right click).
    Clear { t: Transmission, back: bool },
}

/// The ids the screen's buttons send: 1000 + transmission * 100 + side * 10 + back, 2000 + transmission for the eject toggle
/// and 3000 + transmission * 10 + back for the clear button.
pub fn click_of(id: i32) -> Option<Click> {
    match id {
        1000..=1499 => {
            let n = id - 1000;
            let side = ((n % 100) / 10) as usize;
            (side < 6 && n % 10 < 2).then_some(Click::Side { t: Transmission::from_number(n / 100)?, side, back: n % 10 == 1 })
        }
        2000..=2004 => Some(Click::Eject(Transmission::from_number(id - 2000)?)),
        3000..=3049 => {
            let n = id - 3000;
            Some(Click::Clear { t: Transmission::from_number(n / 10)?, back: n % 10 == 1 })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_infuser_has_items_chemicals_and_energy() {
        assert_eq!(
            Layout::Infuser.transmissions(),
            &[Transmission::Items, Transmission::Chemicals, Transmission::Energy]
        );
        let c = default_config(Layout::Infuser);
        assert_eq!(c.items()[BOTTOM], Data::Extra);
        assert_eq!(c.items()[RIGHT], Data::Output);
        assert_eq!(*c.side(Transmission::Chemicals), [Data::Input; 6]);
        assert_eq!(slots_of(Layout::Infuser, Data::Extra), &[2]);
        assert_eq!(cycle(Layout::Infuser, Transmission::Chemicals, Data::InputOutput, false), Data::None);
        // The tank takes in everywhere, by the faces of the world.
        assert_eq!(tank_flow(Layout::Infuser, "north", &c), vec![[1; 6]]);
        // The oxidizer gives on its right (the west face facing north) and pushes there; its eject off, it does not.
        let o = default_config(Layout::Oxidizer);
        assert_eq!(tank_flow(Layout::Oxidizer, "north", &o)[0][4], 2);
        let mut quiet = o;
        quiet.eject[Transmission::Chemicals as usize] = false;
        assert_eq!(tank_flow(Layout::Oxidizer, "north", &quiet)[0], [0; 6]);
        // The dissolution chamber takes by one tank and gives by the other.
        let d = tank_flow(Layout::Dissolution, "north", &default_config(Layout::Dissolution));
        assert_eq!((d[0][4], d[1][4], d[0][2], d[1][2]), (0, 2, 1, 0));
    }

    #[test]
    fn the_defaults_are_the_originals() {
        let c = default_config(Layout::Electric);
        assert_eq!(c.items()[RIGHT], Data::Output);
        assert_eq!(c.items()[BACK], Data::Energy);
        assert_eq!(c.items()[FRONT], Data::Input);
        assert_eq!(c.items()[TOP], Data::Input);
        assert!(c.energy().iter().all(|d| *d == Data::Input));
        assert!(!c.ejects(Transmission::Items));
        // The transmissions no machine has yet start with nothing.
        assert!(c.side(Transmission::Fluids).iter().all(|d| *d == Data::None));
        // A combiner has its extra input at the bottom.
        assert_eq!(default_config(Layout::Combiner).items()[BOTTOM], Data::Extra);
        assert_eq!(default_config(Layout::Electric).items()[BOTTOM], Data::Input);
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
        let faces = item_faces(Layout::Electric, "north", c.items());
        assert_eq!(faces[WEST], vec![1]);
        assert_eq!(faces[SOUTH], Vec::<u32>::new());
        assert_eq!(faces[UP], vec![0]);
        assert_eq!(faces[NORTH], vec![0]);
        // A sawmill's output reaches both output slots; a combiner's extra side reaches the extra slot.
        assert_eq!(slots_of(Layout::Sawmill, Data::Output), &[1, 2]);
        let comb = item_faces(Layout::Combiner, "north", default_config(Layout::Combiner).items());
        assert_eq!(comb[DOWN], vec![1]);
        assert_eq!(comb[WEST], vec![2]);
    }

    #[test]
    fn energy_comes_in_where_a_side_takes_it() {
        let mut c = default_config(Layout::Electric);
        assert_eq!(energy_in("north", c.energy()), [true; 6]);
        c.side_mut(Transmission::Energy)[RIGHT] = Data::None;
        let got = energy_in("north", c.energy());
        assert!(!got[WEST]);
        assert!(got[EAST]);
    }

    #[test]
    fn it_is_kept_as_bytes_and_shown_as_nibbles() {
        let mut c = default_config(Layout::Sawmill);
        c.eject[Transmission::Items as usize] = true;
        c.side_mut(Transmission::Energy)[TOP] = Data::None;
        c.side_mut(Transmission::Fluids)[FRONT] = Data::Output2;
        assert_eq!(Config::decode(&c.encode(), Layout::Sawmill), Some(c));
        assert_eq!(c.encode().len(), 35);
        assert_eq!(Config::decode(&[1, 2], Layout::Sawmill), None);
        let w = c.screen_words();
        // The items' front (1: input), left (1), right (4: output) and back (8: energy), a nibble each.
        assert_eq!(w[0], 0x8_4_1_1);
        // Top and bottom (input, input), then the eject flag in the third nibble.
        assert_eq!(w[1], 0x1_1 | (1 << 8));
        assert_eq!(w[2], 0x1_1_1_1);
        assert_eq!(w[3], 0x1 << 4);
        // The fluids' words follow the energy's: the front is output (2).
        assert_eq!(w[4], 6);
    }

    #[test]
    fn a_configuration_of_the_first_format_is_still_read() {
        // 0.1.17 to 0.1.19 kept 13 bytes: items' sides, energy's sides, the items' eject flag.
        let old = [1, 1, 4, 8, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let c = Config::decode(&old, Layout::Electric).unwrap();
        assert_eq!(c.items()[RIGHT], Data::Output);
        assert_eq!(c.items()[BACK], Data::Energy);
        assert!(c.ejects(Transmission::Items));
        assert!(c.side(Transmission::Heat).iter().all(|d| *d == Data::None));
    }

    #[test]
    fn the_clear_button_uniforms_then_cycles() {
        let c = default_config(Layout::Electric);
        // Mixed sides clear to none.
        assert_eq!(clear_target(Layout::Electric, Transmission::Items, c.items(), false), Data::None);
        // All the same: the next type.
        assert_eq!(clear_target(Layout::Electric, Transmission::Energy, c.energy(), false), Data::None);
        assert_eq!(clear_target(Layout::Electric, Transmission::Items, &[Data::None; 6], false), Data::Input);
    }

    #[test]
    fn button_ids_decode() {
        assert_eq!(click_of(1000), Some(Click::Side { t: Transmission::Items, side: 0, back: false }));
        assert_eq!(click_of(1151), Some(Click::Side { t: Transmission::Energy, side: 5, back: true }));
        assert_eq!(click_of(1170), None);
        assert_eq!(click_of(2000), Some(Click::Eject(Transmission::Items)));
        assert_eq!(click_of(2002), Some(Click::Eject(Transmission::Fluids)));
        assert_eq!(click_of(2005), None);
        assert_eq!(click_of(3011), Some(Click::Clear { t: Transmission::Energy, back: true }));
        assert_eq!(click_of(5), None);
    }
}
