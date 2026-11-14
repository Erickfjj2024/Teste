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
    pub fn from_number(n: i32) -> Option<Transmission> {
        TRANSMISSIONS.get(usize::try_from(n).ok()?).copied()
    }
}

/// The configurator's mode (Mekanism's `ConfiguratorMode`, in its order): the five transmissions it configures, then
/// emptying, rotating and the wrench. The numbers are the mode's ordinals, kept in the item's components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolMode {
    Items = 0,
    Fluids = 1,
    Chemicals = 2,
    Energy = 3,
    Heat = 4,
    Empty = 5,
    Rotate = 6,
    Wrench = 7,
}

impl ToolMode {
    /// The transmission a configurating mode configures; none for empty, rotate and the wrench.
    pub fn transmission(self) -> Option<Transmission> {
        match self {
            ToolMode::Items => Some(Transmission::Items),
            ToolMode::Fluids => Some(Transmission::Fluids),
            ToolMode::Chemicals => Some(Transmission::Chemicals),
            ToolMode::Energy => Some(Transmission::Energy),
            ToolMode::Heat => Some(Transmission::Heat),
            _ => None,
        }
    }
    /// Whether the mode configures a transmission (the original's `isConfigurating`).
    pub fn is_configurating(self) -> bool {
        self.transmission().is_some()
    }
    /// The next mode, around, in the original's order.
    pub fn next(self) -> ToolMode {
        ToolMode::from_number(self as u8 + 1)
    }
    pub fn from_number(n: u8) -> ToolMode {
        match n % 8 {
            0 => ToolMode::Items,
            1 => ToolMode::Fluids,
            2 => ToolMode::Chemicals,
            3 => ToolMode::Energy,
            4 => ToolMode::Heat,
            5 => ToolMode::Empty,
            6 => ToolMode::Rotate,
            _ => ToolMode::Wrench,
        }
    }
    /// How the mode reads in messages (`Configure State: ...`): `Configurate (Items)` for the five, the plain name else.
    pub fn label(self) -> &'static str {
        match self {
            ToolMode::Items => "Configurate (Items)",
            ToolMode::Fluids => "Configurate (Fluids)",
            ToolMode::Chemicals => "Configurate (Chemicals)",
            ToolMode::Energy => "Configurate (Energy)",
            ToolMode::Heat => "Configurate (Heat)",
            ToolMode::Empty => "Empty",
            ToolMode::Rotate => "Rotate",
            ToolMode::Wrench => "Wrench",
        }
    }
}

/// The data component the mode is kept in, as SNBT (`{"mekanism:configurator_mode":3}`).
pub const MODE_COMPONENT: &str = "mekanism:configurator_mode";

/// Skips whitespace in `text` from `i`.
fn skip_ws(text: &str, i: &mut usize) {
    while text[*i..].starts_with(|c: char| c.is_whitespace()) {
        *i += text[*i..].chars().next().map_or(0, |c| c.len_utf8());
    }
}

/// Skips a string starting at `i` (either quote), honouring backslash escapes; whether it closed. The string's
/// content is not decoded here: skipping only needs its end.
fn skip_string(text: &str, i: &mut usize) -> bool {
    let Some(quote) = text[*i..].chars().next() else { return false };
    if quote != '"' && quote != '\'' {
        return false;
    }
    *i += 1;
    let mut chars = text[*i..].char_indices();
    while let Some((at, c)) = chars.next() {
        if c == '\\' {
            chars.next();
            continue;
        }
        if c == quote {
            *i += at + 1;
            return true;
        }
    }
    false
}

/// The unescaped content of a string's raw inner text (no quotes).
fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    Some(c) => out.push(c),
                    None => out.push_str(&hex),
                }
            }
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// Parses a key at `i` (quoted or bare); `i` ends past it.
fn parse_key(text: &str, i: &mut usize) -> Option<String> {
    skip_ws(text, i);
    match text[*i..].chars().next()? {
        '"' | '\'' => {
            let start = *i + 1;
            if !skip_string(text, i) {
                return None;
            }
            Some(unescape(&text[start..*i - 1]))
        }
        _ => {
            let start = *i;
            while let Some(c) = text[*i..].chars().next() {
                if c.is_alphanumeric() || matches!(c, '_' | '.' | '-' | ':' | '+') {
                    *i += c.len_utf8();
                } else {
                    break;
                }
            }
            if *i == start { None } else { Some(text[start..*i].to_owned()) }
        }
    }
}

/// Skips one SNBT value at `i`: a string, a compound or list (nesting honoured, typed arrays like `[I;1,2]` included)
/// or a bare value up to the next comma or closing brace.
fn skip_value(text: &str, i: &mut usize) -> bool {
    skip_ws(text, i);
    match text[*i..].chars().next() {
        Some('"') | Some('\'') => skip_string(text, i),
        Some('{') | Some('[') => {
            let open = text[*i..].chars().next().unwrap_or('{');
            let close = if open == '{' { '}' } else { ']' };
            let mut depth = 0;
            while let Some(c) = text[*i..].chars().next() {
                if c == '"' || c == '\'' {
                    if !skip_string(text, i) {
                        return false;
                    }
                } else {
                    *i += c.len_utf8();
                    if c == open {
                        depth += 1;
                    } else if c == close {
                        depth -= 1;
                        if depth == 0 {
                            return true;
                        }
                    }
                }
            }
            false
        }
        Some(_) => {
            while let Some(c) = text[*i..].chars().next() {
                if c == ',' || c == '}' || c == ']' {
                    break;
                }
                *i += c.len_utf8();
            }
            true
        }
        None => false,
    }
}

/// A component map split into its top-level entries: each key (unquoted) and the byte range of its value, so values
/// the mode does not touch round-trip byte-for-byte. The split is structural — strings, escapes and nesting are
/// honoured — so a key named inside another component's string value is never mistaken for the component itself, as
/// a substring search could. `None` when the text is not a compound.
pub(crate) fn split_components(components: &str) -> Option<Vec<(String, std::ops::Range<usize>)>> {
    let mut i = 0;
    let mut entries = Vec::new();
    skip_ws(components, &mut i);
    if components[i..].chars().next() != Some('{') {
        return None;
    }
    i += 1;
    loop {
        skip_ws(components, &mut i);
        match components[i..].chars().next() {
            Some('}') => {
                i += 1;
                skip_ws(components, &mut i);
                return if i == components.len() { Some(entries) } else { None };
            }
            None => return None,
            _ => {}
        }
        let key = parse_key(components, &mut i)?;
        skip_ws(components, &mut i);
        if components[i..].chars().next() != Some(':') {
            return None;
        }
        i += 1;
        skip_ws(components, &mut i);
        let start = i;
        if !skip_value(components, &mut i) {
            return None;
        }
        let mut end = i;
        while end > start && components.as_bytes()[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        entries.push((key, start..end));
        skip_ws(components, &mut i);
        match components[i..].chars().next() {
            Some(',') => i += 1,
            Some('}') => {}
            _ => return None,
        }
    }
}

/// Whether two component strings are the same stack's: equal once the mode component is set aside (the game may
/// already have written a mode into the copy in the inventory). "" and "{}" both mean no components.
pub fn same_components(a: &str, b: &str) -> bool {
    let bare = |c: &str| c.trim().is_empty() || c.trim() == "{}";
    match (split_components(a), split_components(b)) {
        (Some(ea), Some(eb)) => {
            let mut ka: Vec<(&str, &str)> = ea.iter().filter(|(k, _)| k != MODE_COMPONENT).map(|(k, r)| (k.as_str(), a[r.clone()].trim())).collect();
            let mut kb: Vec<(&str, &str)> = eb.iter().filter(|(k, _)| k != MODE_COMPONENT).map(|(k, r)| (k.as_str(), b[r.clone()].trim())).collect();
            ka.sort_unstable();
            kb.sort_unstable();
            ka == kb
        }
        _ => bare(a) && bare(b) || a.trim() == b.trim(),
    }
}

/// The mode a stack's components hold; items when none is written (a fresh configurator).
pub fn mode_of(components: &str) -> ToolMode {
    let Some(entries) = split_components(components) else {
        return mode_of_unparsed(components);
    };
    for (key, range) in entries {
        if key == MODE_COMPONENT {
            return components[range].trim().parse().map_or(ToolMode::Items, ToolMode::from_number);
        }
    }
    ToolMode::Items
}

/// The old substring read, kept only for components the structural parser rejects (malformed text the game should not
/// produce): the mode is a bare integer after the key.
fn mode_of_unparsed(components: &str) -> ToolMode {
    let Some(at) = components.find(MODE_COMPONENT) else { return ToolMode::Items };
    let rest = &components[at + MODE_COMPONENT.len()..];
    let digits: String = rest.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().map_or(ToolMode::Items, ToolMode::from_number)
}

/// The components of a stack whose mode is `mode`, keeping the other components it has, byte-for-byte.
pub fn with_mode(components: &str, mode: ToolMode) -> String {
    let pair = format!("\"{MODE_COMPONENT}\":{}", mode as u8);
    if components.trim().is_empty() || components.trim() == "{}" {
        return format!("{{{pair}}}");
    }
    if let Some(entries) = split_components(components) {
        for (key, range) in &entries {
            if key == MODE_COMPONENT {
                let mut out = components.to_owned();
                out.replace_range(range.clone(), &(mode as u8).to_string());
                return out;
            }
        }
        // No such component: add it before the closing brace.
        let at = components.rfind('}').unwrap_or(components.len());
        let mut out = components.to_owned();
        if entries.is_empty() {
            out.insert_str(at, &pair);
        } else {
            out.insert_str(at, &format!(",{pair}"));
        }
        return out;
    }
    with_mode_unparsed(components, mode)
}

/// The old substring write, kept only for components the structural parser rejects: replace the mode's digits when
/// the key is there, else add the pair before the closing brace.
fn with_mode_unparsed(components: &str, mode: ToolMode) -> String {
    let pair = format!("\"{MODE_COMPONENT}\":{}", mode as u8);
    if let Some(key) = components.find(MODE_COMPONENT) {
        let rest = &components[key + MODE_COMPONENT.len()..];
        if let Some(colon) = rest.find(':') {
            let after = &rest[colon + 1..];
            let skip = after.len() - after.trim_start_matches(|c: char| !c.is_ascii_digit()).len();
            let digits = after[skip..].chars().take_while(|c| c.is_ascii_digit()).count();
            if digits > 0 {
                let start = key + MODE_COMPONENT.len() + colon + 1 + skip;
                let mut out = components.to_owned();
                out.replace_range(start..start + digits, &(mode as u8).to_string());
                return out;
            }
        }
    }
    match components.rfind('}') {
        Some(at) => {
            let mut out = components.to_owned();
            out.insert_str(at, &format!(",{pair}"));
            out
        }
        None => format!("{{{pair}}}"),
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
    /// A chemical tank: the drain slot (0) and the fill slot (1); its tank takes in and gives out.
    ChemTank,
    /// The chemical infuser: two chemical item slots in (0, 1) and one out (2); tanks: left, right, centre.
    ChemInfuser,
    /// A factory of that many lines (`FactoryTier.processes`): the input and output slots of each line one after the other
    /// (line `i` has input slot `2i` and output slot `2i + 1`), then the charge slot.
    Factory(u8, u8),
    /// The electric pump: a bucket slot in (top) and out (bottom), its charge slot (back); its tank gives on top.
    Pump(u8),
    /// The pigment mixer: the infuser's shape, but its top side is disabled (`addDisabledSides(TOP)`).
    Mixer,
    /// The chemical washer: the fluid slot (0), the slurry output slot (1) and the fluid output slot (2); tanks: fluid,
    /// slurry in, slurry out.
    Washer,
    /// The electrolytic separator: the fluid slot (0) and the two gas slots (1, 2); tanks: fluid, left gas, right gas.
    Separator,
    /// The rotary condensentrator: the gas input (0) and output (1) slots, the fluid input (2) and output (3) slots and the
    /// charge slot (4); tanks: fluid, gas.
    Rotary,
    /// The pressurized reaction chamber: the item in (0), out (1) and the charge slot (2); tanks: fluid, gas in, gas out.
    Reaction,
}

impl Layout {
    /// The transmissions a machine of this layout configures, in the order of its tabs (`getTransmissions`): the first is
    /// the one the window opens on.
    pub fn transmissions(self) -> &'static [Transmission] {
        use Transmission::{Chemicals as C, Energy as E, Fluids as F, Items as I};
        match self {
            Layout::Pump(_) => &[F, I, E],
            Layout::Factory(_, form) if form >= 3 => &[I, C, E],
            Layout::Electric | Layout::Sawmill | Layout::Combiner | Layout::Factory(..) => &[I, E],
            Layout::ChemTank => &[C, I],
            Layout::ChemInfuser | Layout::Mixer => &[C, I, E],
            Layout::Washer => &[C, F, I, E],
            Layout::Separator => &[F, C, I, E],
            Layout::Rotary => &[C, F, I, E],
            Layout::Reaction => &[I, C, F, E],
            _ => &[I, C, E],
        }
    }
    /// Whether it has a third item slot the extra data type reaches (the chemical item slot of the infuser's family).
    pub fn extra_slot(self) -> bool {
        matches!(self, Layout::Infuser | Layout::Adv | Layout::Compressor | Layout::Dissolution)
    }
    /// Whether the chemical tanks push into the blocks beside them by themselves where a side gives (what `eject` says):
    /// the other machines are emptied by pipes, which the port has not.
    pub fn pushes_chemicals(self) -> bool {
        matches!(
            self,
            Layout::Oxidizer | Layout::Dissolution | Layout::ChemTank | Layout::ChemInfuser | Layout::Mixer | Layout::Washer | Layout::Separator | Layout::Rotary | Layout::Reaction
        )
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
        (Layout::Pump(0), Transmission::Fluids) => &[Data::None, Data::Output],
        (Layout::Pump(_), Transmission::Fluids) => &[Data::None, Data::Input],
        (Layout::Pump(_), Transmission::Items) => &[Data::None, Data::Input, Data::Output, Data::InputOutput, Data::Energy],
        (Layout::Factory(_, 3), Transmission::Chemicals) => &[Data::None, Data::Input],
        (Layout::Factory(_, 4), Transmission::Chemicals) => &[Data::None, Data::Input, Data::Output, Data::InputOutput],
        (Layout::Oxidizer, Transmission::Chemicals) => &[Data::None, Data::Output],
        (Layout::ChemTank | Layout::Washer | Layout::Rotary | Layout::Reaction, Transmission::Chemicals) => {
            &[Data::None, Data::Input, Data::Output, Data::InputOutput]
        }
        (Layout::ChemInfuser | Layout::Mixer, Transmission::Chemicals) => {
            &[Data::None, Data::Input1, Data::Input2, Data::Output, Data::InputOutput]
        }
        (Layout::Separator, Transmission::Chemicals) => &[Data::None, Data::Output1, Data::Output2, Data::InputOutput],
        // The fluid tanks only take in.
        (Layout::Washer | Layout::Separator | Layout::Reaction, Transmission::Fluids) => &[Data::None, Data::Input],
        (Layout::Rotary, Transmission::Fluids) => &[Data::None, Data::Input, Data::Output, Data::InputOutput],
        // Their item slots: a tank's two, the infuser's two inputs and output, the washer's and the separator's.
        (Layout::ChemTank, Transmission::Items) => &[Data::None, Data::Input, Data::Output, Data::InputOutput],
        (Layout::ChemInfuser | Layout::Mixer, Transmission::Items) => {
            &[Data::None, Data::Input1, Data::Input2, Data::Output, Data::InputOutput, Data::Energy]
        }
        (Layout::Separator, Transmission::Items) => {
            &[Data::None, Data::Input, Data::Output1, Data::Output2, Data::InputOutput, Data::Energy]
        }
        // No other machine has fluid, chemical or heat tanks yet: the machines that do will list their data types here.
        (_, Transmission::Fluids | Transmission::Chemicals | Transmission::Heat) => &[],
        (_, Transmission::Energy) => &[Data::None, Data::Input],
        (l, Transmission::Items) if l == Layout::Combiner || l.extra_slot() || matches!(l, Layout::Factory(_, f) if factory_has_extra(f)) => {
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
    if layout == Layout::Combiner || layout.extra_slot() || matches!(layout, Layout::Factory(_, f) if factory_has_extra(f)) {
        items[BOTTOM] = Data::Extra;
    }
    match layout {
        // `TWO_INPUT_ITEM`: the left and right slots take, the front gives, the back is the charge slot.
        Layout::ChemInfuser | Layout::Mixer => {
            items = [Data::None; 6];
            items[LEFT] = Data::Input1;
            items[RIGHT] = Data::Input2;
            items[FRONT] = Data::Output;
            items[BACK] = Data::Energy;
        }
        // `TWO_OUTPUT_ITEM`: the front takes, the left and right give, the back is the charge slot.
        Layout::Separator => {
            items = [Data::None; 6];
            items[FRONT] = Data::Input;
            items[LEFT] = Data::Output1;
            items[RIGHT] = Data::Output2;
            items[BACK] = Data::Energy;
        }
        Layout::ChemTank => items = [Data::None; 6],
        // The drain slot on top, the full one at the bottom, the charge slot at the back.
        Layout::Pump(_) => {
            items = [Data::None; 6];
            items[TOP] = Data::Input;
            items[BOTTOM] = Data::Output;
            items[BACK] = Data::Energy;
        }
        _ => {}
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
        // The infuser: left and right take the two chemicals, the front gives the third, and it pushes it out.
        Layout::ChemInfuser | Layout::Mixer => {
            *chem = [Data::None; 6];
            chem[LEFT] = Data::Input1;
            chem[RIGHT] = Data::Input2;
            chem[FRONT] = Data::Output;
            eject[Transmission::Chemicals as usize] = true;
        }
        // The washer: `OUT_EJECT`, and its fluid tank takes in everywhere.
        Layout::Washer => {
            *chem = [Data::Input; 6];
            chem[RIGHT] = Data::Output;
            eject[Transmission::Chemicals as usize] = true;
            sides[Transmission::Fluids as usize] = [Data::Input; 6];
        }
        // The rotary condensentrator (`AttachedSideConfig.ROTARY`): the gas in everywhere and out on the left, the fluid in
        // everywhere and out on the right, both pushing out (the mode decides which one does).
        Layout::Rotary => {
            *chem = [Data::Input; 6];
            chem[LEFT] = Data::Output;
            eject[Transmission::Chemicals as usize] = true;
            sides[Transmission::Fluids as usize] = [Data::Input; 6];
            sides[Transmission::Fluids as usize][RIGHT] = Data::Output;
            eject[Transmission::Fluids as usize] = true;
        }
        // The reaction chamber (`AttachedSideConfig.REACTION`): the fluid in everywhere, the gas in everywhere and out on the right, pushing out.
        Layout::Reaction => {
            *chem = [Data::Input; 6];
            chem[RIGHT] = Data::Output;
            eject[Transmission::Chemicals as usize] = true;
            sides[Transmission::Fluids as usize] = [Data::Input; 6];
        }
        // The separator: the left gas out on the left, the right gas out on the right.
        Layout::Separator => {
            *chem = [Data::None; 6];
            chem[LEFT] = Data::Output1;
            chem[RIGHT] = Data::Output2;
            eject[Transmission::Chemicals as usize] = true;
            sides[Transmission::Fluids as usize] = [Data::Input; 6];
        }
        // The factories' tank: only in (injecting, purifying), or in everywhere and out on the right (compressing, infusing).
        Layout::Factory(_, form) if form >= 3 => {
            *chem = [Data::Input; 6];
            if form == 4 {
                chem[RIGHT] = Data::Output;
            }
        }
        // A tank gives and takes on every side.
        Layout::ChemTank => {
            *chem = [Data::InputOutput; 6];
            eject[Transmission::Chemicals as usize] = true;
        }
        // `AttributeSideConfig.PUMP`: the tank gives on top.
        Layout::Pump(form) => sides[Transmission::Fluids as usize][TOP] = if form == 0 { Data::Output } else { Data::Input },
        Layout::Electric | Layout::Sawmill | Layout::Combiner | Layout::Factory(..) => {}
    }
    let mut cfg = Config { sides, eject };
    enforce(layout, &mut cfg);
    cfg
}

/// A side the machine cannot use (the mixer's top) holds nothing, whatever a click did.
pub fn enforce(layout: Layout, cfg: &mut Config) {
    if layout == Layout::Mixer {
        for t in 0..5 {
            cfg.sides[t][TOP] = Data::None;
        }
    }
}

/// Whether `side` of the machine cannot be configured.
pub fn disabled(layout: Layout, side: usize) -> bool {
    layout == Layout::Mixer && side == TOP
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
    if matches!(layout, Layout::Factory(..)) {
        return &[];
    }
    match (layout, data) {
        (Layout::Rotary, Data::Energy) => &[4],
        (Layout::Pump(_), Data::Input) => &[0],
        (Layout::Pump(_), Data::Output) => &[1],
        (Layout::Pump(_), Data::InputOutput) => &[0, 1],
        (Layout::Pump(_), Data::Energy) => &[2],
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
        (l, Data::Input) if l.extra_slot() => &[0],
        (l, Data::Output) if l.extra_slot() => &[1],
        (l, Data::InputOutput) if l.extra_slot() => &[0, 1],
        (l, Data::Extra) if l.extra_slot() => &[2],
        // The oxidizer's and the crystallizer's two slots.
        (Layout::Oxidizer | Layout::Crystallizer, Data::Input) => &[0],
        (Layout::Oxidizer | Layout::Crystallizer, Data::Output) => &[1],
        (Layout::Oxidizer | Layout::Crystallizer, Data::InputOutput) => &[0, 1],
        // A tank's drain and fill slots.
        (Layout::ChemTank, Data::Input) => &[0],
        (Layout::ChemTank, Data::Output) => &[1],
        (Layout::ChemTank, Data::InputOutput) => &[0, 1],
        // The infuser's two chemical item slots in and the one out.
        (Layout::ChemInfuser | Layout::Mixer, Data::Input1) => &[0],
        (Layout::ChemInfuser | Layout::Mixer, Data::Input2) => &[1],
        (Layout::ChemInfuser | Layout::Mixer, Data::Output) => &[2],
        (Layout::ChemInfuser | Layout::Mixer, Data::InputOutput) => &[0, 1, 2],
        // The washer's fluid slot, then its two output slots.
        (Layout::Washer, Data::Input) => &[0],
        (Layout::Washer, Data::Output) => &[1, 2],
        (Layout::Washer, Data::InputOutput) => &[0, 1, 2],
        // The reaction chamber's input and output slots, and its charge slot.
        (Layout::Reaction, Data::Input) => &[0],
        (Layout::Reaction, Data::Output) => &[1],
        (Layout::Reaction, Data::InputOutput) => &[0, 1],
        // The rotary's gas and fluid slots in (0, 2) and out (1, 3).
        (Layout::Rotary, Data::Input) => &[0, 2],
        (Layout::Rotary, Data::Output) => &[1, 3],
        (Layout::Rotary, Data::InputOutput) => &[0, 1, 2, 3],
        // The separator's fluid slot and its two gas slots.
        (Layout::Separator, Data::Input) => &[0],
        (Layout::Separator, Data::Output1) => &[1],
        (Layout::Separator, Data::Output2) => &[2],
        (Layout::Separator, Data::InputOutput) => &[0, 1, 2],
        _ => &[],
    }
}

/// How a factory lays its slots out: `form` 0 is an input and an output a line, 1 adds an extra input all the lines share (after the
/// lines, before the charge slot), 2 has a main and a secondary output for each line. The slots of each line come one after the other.
pub fn factory_has_extra(form: u8) -> bool {
    form == 1 || form >= 3
}

pub fn factory_slots_per_line(form: u8) -> usize {
    if form == 2 { 3 } else { 2 }
}

/// The slots of a factory of `lines` lines in all, the charge slot last.
pub fn factory_slot_count(lines: usize, form: u8) -> usize {
    lines * factory_slots_per_line(form) + usize::from(factory_has_extra(form)) + 1
}

/// The slots of a factory a data type reaches: its inputs, its outputs, its extra input and its charge slot.
fn factory_slots(lines: u8, form: u8, data: Data) -> Vec<u32> {
    let (n, per) = (usize::from(lines), factory_slots_per_line(form));
    let at = |line: usize, k: usize| (line * per + k) as u32;
    let outs = |line: usize| (1..per).map(move |k| at(line, k));
    match data {
        Data::Input => (0..n).map(|i| at(i, 0)).collect(),
        Data::Output => (0..n).flat_map(outs).collect(),
        Data::InputOutput => (0..n * per).map(|i| i as u32).collect(),
        Data::Extra if factory_has_extra(form) => vec![(n * per) as u32],
        Data::Energy => vec![(factory_slot_count(n, form) - 1) as u32],
        _ => Vec::new(),
    }
}

/// The slots of a machine a data type reaches, one list for each face (`slots_of`'s answer, for a factory built from its lines).
pub fn slots_vec(layout: Layout, data: Data) -> Vec<u32> {
    match layout {
        Layout::Factory(lines, form) => factory_slots(lines, form, data),
        _ => slots_of(layout, data).to_vec(),
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
        faces[face_of(facing, side)] = slots_vec(layout, *data);
    }
    faces
}

/// What each face of the world does with the machine's tanks, in the order of its tanks: none, in, out or both (`FlowSide`
/// numbers 0 to 3). The tanks of a machine of `process` are the one its chemical goes into, then the one it comes out in;
/// of the infuser, left, right, centre; of the washer, the fluid, the slurry and the clean slurry; of the separator, the
/// fluid and the two gases. A tank gives by itself only where the machine pushes (`pushes_chemicals`) and its eject is on.
pub fn tank_flow(layout: Layout, facing: &str, cfg: &Config, mode: bool) -> Vec<[u8; 6]> {
    let chem = cfg.side(Transmission::Chemicals);
    let fluid = cfg.side(Transmission::Fluids);
    let pushes = layout.pushes_chemicals() && cfg.ejects(Transmission::Chemicals);
    let count = match layout {
        Layout::ChemInfuser | Layout::Mixer | Layout::Washer | Layout::Separator => 3,
        Layout::Reaction => 3,
        Layout::Dissolution | Layout::Rotary => 2,
        _ => 1,
    };
    let mut out = vec![[0u8; 6]; count];
    for side in 0..6 {
        let face = face_of(facing, side);
        let (c, f) = (chem[side], fluid[side]);
        let takes = |d: Data| matches!(d, Data::Input | Data::InputOutput);
        let gives = |d: Data| matches!(d, Data::Output | Data::InputOutput) && pushes;
        let both = |take: bool, give: bool| u8::from(take) + if give { 2 } else { 0 };
        match layout {
            Layout::Dissolution => {
                out[0][face] = both(takes(c), false);
                out[1][face] = both(false, gives(c));
            }
            Layout::ChemInfuser | Layout::Mixer => {
                out[0][face] = both(matches!(c, Data::Input1 | Data::InputOutput), false);
                out[1][face] = both(matches!(c, Data::Input2 | Data::InputOutput), false);
                out[2][face] = both(false, matches!(c, Data::Output | Data::InputOutput) && pushes);
            }
            Layout::Washer => {
                out[0][face] = both(takes(f), false);
                out[1][face] = both(takes(c), false);
                out[2][face] = both(false, gives(c));
            }
            // The fluid tank takes in and the gas tank gives out in the mode that decondensentrates, and the other way round;
            // each gives by itself only while its eject is on.
            Layout::Rotary => {
                let push = |t: Transmission| cfg.ejects(t);
                out[0][face] = both(mode && takes(f), !mode && matches!(f, Data::Output | Data::InputOutput) && push(Transmission::Fluids));
                out[1][face] = both(!mode && takes(c), mode && matches!(c, Data::Output | Data::InputOutput) && push(Transmission::Chemicals));
            }
            // The fluid and the gas it takes in, and the gas it makes (given by the output side, with the eject on).
            Layout::Reaction => {
                out[0][face] = both(takes(f), false);
                out[1][face] = both(takes(c) && !matches!(c, Data::Output), false);
                out[2][face] = both(false, matches!(c, Data::Output | Data::InputOutput) && pushes);
            }
            Layout::Separator => {
                out[0][face] = both(takes(f), false);
                out[1][face] = both(false, matches!(c, Data::Output1 | Data::InputOutput) && pushes);
                out[2][face] = both(false, matches!(c, Data::Output2 | Data::InputOutput) && pushes);
            }
            // The pump's tank gives through the sides its fluid configuration says.
            Layout::Pump(0) => out[0][face] = both(false, matches!(f, Data::Output | Data::InputOutput)),
            Layout::Pump(_) => out[0][face] = both(takes(f), false),
            _ => out[0][face] = both(takes(c), gives(c)),
        }
    }
    out
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
        assert_eq!(tank_flow(Layout::Infuser, "north", &c, false), vec![[1; 6]]);
        // The oxidizer gives on its right (the west face facing north) and pushes there; its eject off, it does not.
        let o = default_config(Layout::Oxidizer);
        assert_eq!(tank_flow(Layout::Oxidizer, "north", &o, false)[0][4], 2);
        let mut quiet = o;
        quiet.eject[Transmission::Chemicals as usize] = false;
        assert_eq!(tank_flow(Layout::Oxidizer, "north", &quiet, false)[0], [0; 6]);
        // The dissolution chamber takes by one tank and gives by the other.
        let d = tank_flow(Layout::Dissolution, "north", &default_config(Layout::Dissolution), false);
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
    fn the_tool_goes_through_the_originals_eight_modes() {
        let mut m = ToolMode::Items;
        let mut seen = vec![m];
        for _ in 0..8 {
            m = m.next();
            seen.push(m);
        }
        assert_eq!(
            seen,
            [ToolMode::Items, ToolMode::Fluids, ToolMode::Chemicals, ToolMode::Energy, ToolMode::Heat, ToolMode::Empty, ToolMode::Rotate, ToolMode::Wrench, ToolMode::Items]
        );
        assert_eq!(ToolMode::Empty.transmission(), None);
        assert_eq!(ToolMode::Energy.transmission(), Some(Transmission::Energy));
        assert_eq!(ToolMode::Wrench.label(), "Wrench");
        assert_eq!(ToolMode::Fluids.label(), "Configurate (Fluids)");
        // The mode round-trips through the item's components, keeping the others.
        assert_eq!(mode_of(""), ToolMode::Items);
        assert_eq!(mode_of("{}"), ToolMode::Items);
        assert_eq!(mode_of("{\"mekanism:configurator_mode\":5}"), ToolMode::Empty);
        assert_eq!(mode_of("{\"minecraft:damage\":3,\"mekanism:configurator_mode\":7}"), ToolMode::Wrench);
        assert_eq!(with_mode("", ToolMode::Rotate), "{\"mekanism:configurator_mode\":6}");
        assert_eq!(with_mode("{\"minecraft:damage\":3}", ToolMode::Energy), "{\"minecraft:damage\":3,\"mekanism:configurator_mode\":3}");
        assert_eq!(
            with_mode("{\"mekanism:configurator_mode\":1,\"minecraft:damage\":3}", ToolMode::Wrench),
            "{\"mekanism:configurator_mode\":7,\"minecraft:damage\":3}"
        );
        assert_eq!(mode_of(&with_mode("{\"minecraft:damage\":3}", ToolMode::Heat)), ToolMode::Heat);
        // The split is structural, not a substring search: the mode's name inside another component's string value
        // is not a mode, and nested compounds, lists and single quotes survive a write byte-for-byte.
        assert_eq!(mode_of("{\"minecraft:lore\":\"mekanism:configurator_mode 5\"}"), ToolMode::Items);
        assert_eq!(mode_of("{\"minecraft:lore\":[\"mekanism:configurator_mode: 7\"]}"), ToolMode::Items);
        assert_eq!(
            with_mode("{'minecraft:enchantments':[{id:\"minecraft:sharpness\",lvl:3s}],\"minecraft:damage\":3}", ToolMode::Heat),
            "{'minecraft:enchantments':[{id:\"minecraft:sharpness\",lvl:3s}],\"minecraft:damage\":3,\"mekanism:configurator_mode\":4}"
        );
        assert_eq!(mode_of("{mekanism:configurator_mode:5}"), ToolMode::Empty);
        // Two component strings are the same stack's when they only differ in the mode (or in nothing at all).
        assert!(same_components("{\"mekanism:configurator_mode\":3}", "{\"mekanism:configurator_mode\":5}"));
        assert!(same_components("{\"minecraft:damage\":3}", "{\"minecraft:damage\":3,\"mekanism:configurator_mode\":1}"));
        assert!(same_components("", "{}"));
        assert!(!same_components("{\"minecraft:damage\":3}", "{\"minecraft:damage\":4}"));
        assert!(!same_components("{\"minecraft:damage\":3}", "{\"minecraft:unbreaking\":1}"));
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

    #[test]
    fn the_mixers_top_stays_disabled() {
        let mut c = default_config(Layout::Mixer);
        assert!(TRANSMISSIONS.iter().all(|t| c.side(*t)[TOP] == Data::None));
        assert!(disabled(Layout::Mixer, TOP) && !disabled(Layout::Mixer, FRONT) && !disabled(Layout::ChemInfuser, TOP));
        c.side_mut(Transmission::Energy)[TOP] = Data::Input;
        enforce(Layout::Mixer, &mut c);
        assert_eq!(c.side(Transmission::Energy)[TOP], Data::None);
    }
}
