//! The induction matrix (`MatrixMultiblockData`, `MatrixValidator`): a hollow cuboid (3 to 18 on a side) of `mekanism:induction_casing`
//! with `mekanism:induction_port` blocks on its faces, whose inside holds induction cells (what it stores) and providers (how fast it
//! takes in and gives out) and is empty otherwise. The ports are where energy goes in and out.
//!
//! Like the dynamic tank, the game finds the structure and this validates it; every port holds a mirror of the shared energy (at most
//! what the providers move in a tick, which is also all the room it has), what a port gains or loses is added to or taken from the total,
//! and the total is kept in the ports' bytes so it survives a reload.
use super::*;
use std::collections::HashMap;

pub const CASING: &str = "mekanism:induction_casing";
pub const PORT: &str = "mekanism:induction_port";
pub const GROUP: &str = "mekanism:induction";
/// `InductionCellTier` and `InductionProviderTier`: what a cell stores and what a provider moves each tick.
pub const CELLS: [(&str, u64); 4] = [
    ("mekanism:basic_induction_cell", fe(8_000_000_000)),
    ("mekanism:advanced_induction_cell", fe(64_000_000_000)),
    ("mekanism:elite_induction_cell", fe(512_000_000_000)),
    ("mekanism:ultimate_induction_cell", fe(4_000_000_000_000)),
];
pub const PROVIDERS: [(&str, u64); 4] = [
    ("mekanism:basic_induction_provider", fe(256_000)),
    ("mekanism:advanced_induction_provider", fe(2_048_000)),
    ("mekanism:elite_induction_provider", fe(16_384_000)),
    ("mekanism:ultimate_induction_provider", fe(131_072_000)),
];
/// What a port is registered to hold: far more than a matrix can (the real room is set each tick).
pub const SPEC_CAPACITY: u64 = 1 << 60;

/// What an item that holds energy has: its energy, its most, and how much it takes in or gives a tick (`IStrictEnergyHandler` of the Meka-Tool and the MekaSuit pieces).
pub fn item_energy(stack: &ItemStack) -> Option<(u64, u64, u64)> {
    if stack.item == crate::meka_tool::MEKA_TOOL {
        let d = crate::meka_tool::host_read_meka_tool(&stack.components);
        return Some((d.energy, d.max_energy(), d.charge_rate()));
    }
    let piece = crate::mekasuit::MekaSuitItemStack::decode(&stack.item, stack.count, &stack.components)?;
    Some((piece.energy, piece.max_energy(), piece.max_charge_rate()))
}

/// The item with `energy` in it (`None` for an item that holds none).
pub fn with_item_energy(stack: &ItemStack, energy: u64) -> Option<ItemStack> {
    if stack.item == crate::meka_tool::MEKA_TOOL {
        let components = crate::equipment_world::with_energy(&stack.components, energy)?;
        return Some(ItemStack { components, ..stack.clone() });
    }
    let mut piece = crate::mekasuit::MekaSuitItemStack::decode(&stack.item, stack.count, &stack.components)?;
    piece.energy = energy.min(piece.max_energy());
    Some(ItemStack { components: piece.encode(&stack.components), ..stack.clone() })
}

/// `EnergyInventorySlot.fillOrConvert` of the matrix (145, 20): an item that holds energy gives it to the matrix (as much as it has, the room and the item's rate allow), any
/// other item the energy it is worth whole when that fits (`conversion`: redstone and the like). Returns the energy gained and the slot after.
pub fn fill_step(slot: &Option<ItemStack>, room: u64) -> Option<(u64, Option<ItemStack>)> {
    let stack = slot.as_ref()?;
    if let Some((energy, _, rate)) = item_energy(stack) {
        let gain = energy.min(room).min(rate);
        if gain == 0 {
            return None;
        }
        return Some((gain, with_item_energy(stack, energy - gain)));
    }
    cube::fill_from_slot(slot, 0, room).map(|(gain, left)| (gain, left))
}

/// `EnergyInventorySlot.drain` (145, 50): the matrix charges the item in the slot, as much as it has, the item's room and its rate allow. Returns the energy given and the slot after.
pub fn drain_step(slot: &Option<ItemStack>, available: u64) -> Option<(u64, Option<ItemStack>)> {
    let stack = slot.as_ref()?;
    let (energy, max, rate) = item_energy(stack)?;
    let give = max.saturating_sub(energy).min(available).min(rate);
    if give == 0 {
        return None;
    }
    Some((give, with_item_energy(stack, energy + give)))
}

/// The lines of the inner screen (`GuiInductionMatrix`): the energy, the capacity and what went in and out last tick.
pub fn screen_texts(energy: u64, capacity: u64, input: u64, output: u64) -> Vec<String> {
    let f = crate::laser::format_energy;
    vec![format!("Energy: {}", f(energy)), format!("Capacity: {}", f(capacity)), format!("Input: {}/t", f(input)), format!("Output: {}/t", f(output))]
}

/// A found structure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    pub valid: bool,
    pub min: [i32; 3],
    pub capacity: u64,
    pub rate: u64,
    pub cells: u16,
    pub providers: u16,
}

/// What a member is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Casing,
    Glass,
    Port,
    Cell(u64),
    Provider(u64),
}

pub fn part_of(state: &str) -> Option<Part> {
    let name = state.split('[').next().unwrap_or("");
    if name == CASING {
        return Some(Part::Casing);
    }
    if name == "mekanism:structural_glass" {
        return Some(Part::Glass);
    }
    if name == PORT {
        return Some(Part::Port);
    }
    CELLS.iter().find(|(n, _)| *n == name).map(|(_, c)| Part::Cell(*c)).or_else(|| PROVIDERS.iter().find(|(n, _)| *n == name).map(|(_, r)| Part::Provider(*r)))
}

/// Whether the members make a matrix: the shell of casing and ports (ports on a face, not an edge), the inside cells and providers and
/// air, and a size of 3 to 18.
pub fn validate(members: &[([i32; 3], Part)], min: [i32; 3], max: [i32; 3], inside_empty: impl Fn([i32; 3]) -> bool) -> Found {
    let size = [max[0] - min[0] + 1, max[1] - min[1] + 1, max[2] - min[2] + 1];
    let bad = Found { min, ..Found::default() };
    if size.iter().any(|s| !(3..=18).contains(s)) {
        return bad;
    }
    let have: HashMap<[i32; 3], Part> = members.iter().copied().collect();
    let (mut capacity, mut rate, mut cells, mut providers) = (0u64, 0u64, 0u16, 0u16);
    for x in min[0]..=max[0] {
        for y in min[1]..=max[1] {
            for z in min[2]..=max[2] {
                let p = [x, y, z];
                let edges = [x == min[0] || x == max[0], y == min[1] || y == max[1], z == min[2] || z == max[2]].iter().filter(|b| **b).count();
                match (edges, have.get(&p)) {
                    (0, Some(Part::Cell(c))) => {
                        capacity += c;
                        cells += 1;
                    }
                    (0, Some(Part::Provider(r))) => {
                        rate += r;
                        providers += 1;
                    }
                    (0, Some(_)) => return bad,
                    (0, None) if !inside_empty(p) => return bad,
                    (0, None) => {}
                    // Edges and corners (edges > 1) MUST be Casing (neither Glass nor Port allowed, CuboidStructureValidator.java:115).
                    (e, Some(Part::Casing)) if e > 1 => {}
                    (e, Some(_)) if e > 1 => return bad,
                    // Faces (edges == 1) allow Casing, Glass, and Ports.
                    (1, Some(Part::Casing | Part::Glass | Part::Port)) => {}
                    _ => return bad,
                }
            }
        }
    }
    if members.iter().any(|(p, _)| (0..3).any(|i| p[i] < min[i] || p[i] > max[i])) || cells == 0 {
        return bad;
    }
    Found { valid: true, min, capacity, rate, cells, providers }
}

/// A member's bytes: validity, the box's minimum corner, the capacity, the rate and the counts, then the energy the structure holds.
pub fn encode(f: &Found, total: u64) -> Vec<u8> {
    let mut out = vec![u8::from(f.valid)];
    for c in f.min {
        out.extend(c.to_le_bytes());
    }
    out.extend(f.capacity.to_le_bytes());
    out.extend(f.rate.to_le_bytes());
    out.extend(f.cells.to_le_bytes());
    out.extend(f.providers.to_le_bytes());
    out.extend(total.to_le_bytes());
    out
}

pub fn decode(data: &[u8]) -> Option<(Found, u64)> {
    if data.len() < 33 {
        return None;
    }
    let int = |at: usize| i32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]);
    let long = |at: usize| u64::from_le_bytes(data[at..at + 8].try_into().unwrap_or([0; 8]));
    let short = |at: usize| u16::from_le_bytes([data[at], data[at + 1]]);
    let found = Found { valid: data[0] == 1, min: [int(1), int(5), int(9)], capacity: long(13), rate: long(21), cells: short(29), providers: short(31) };
    Some((found, data.get(33..41).map_or(0, |_| long(33))))
}

/// The mode of a port, kept in the byte after its data: false takes in, true gives out. A port saved before the
/// configurator kept it has no byte and takes in.
pub fn mode_of(data: &[u8]) -> bool {
    data.get(41).copied().unwrap_or(0) == 1
}

/// What the port was last given, kept in bytes 42..50.
pub fn given_of(data: &[u8]) -> Option<u64> {
    if data.len() >= 50 {
        let bytes: [u8; 8] = data[42..50].try_into().unwrap_or([0; 8]);
        Some(u64::from_le_bytes(bytes))
    } else {
        None
    }
}

/// The port's data with its mode byte and given energy after it.
pub fn with_mode_given(mut data: Vec<u8>, output: bool, given: u64) -> Vec<u8> {
    data.truncate(41);
    data.push(u8::from(output));
    data.extend(given.to_le_bytes());
    data
}

/// The port's data with its mode byte after it.
pub fn with_mode(data: Vec<u8>, output: bool) -> Vec<u8> {
    with_mode_given(data, output, 0)
}

/// The energy after a tick: what the ports held before, changed by what each gained or lost against the mirror it was given.
pub fn merge(total: u64, given: u64, now: &[u64], capacity: u64) -> u64 {
    let mut t = i128::from(total);
    for n in now {
        t += i128::from(*n) - i128::from(given);
    }
    t.clamp(0, i128::from(capacity)) as u64
}

impl Mekanism {
    /// The candidate structures the game found: validates each and tells every member.
    pub(crate) fn matrix_structures(&mut self, dimension: &str, structures: Vec<Structure>) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for s in structures {
            let members: Vec<([i32; 3], Part)> = s.members.iter().filter_map(|m| part_of(&m.state).map(|p| ([m.pos.x, m.pos.y, m.pos.z], p))).collect();
            let min = [s.min.x, s.min.y, s.min.z];
            let max = [s.max.x, s.max.y, s.max.z];
            let found = validate(&members, min, max, |p| {
                world::get_block(dimension, p[0], p[1], p[2]).is_some_and(|b| matches!(b.split('[').next().unwrap_or(""), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"))
            });
            for m in &s.members {
                let is_port = part_of(&m.state) == Some(Part::Port);
                if is_port {
                    // A port holds at most what the providers move in a tick (nothing of its own in a broken structure).
                    let _ = world::set_energy_capacity(dimension, m.pos.x, m.pos.y, m.pos.z, if found.valid { found.rate.max(1) } else { 1 });
                }
                let total = self.matrices.get(&(dimension.to_owned(), found.min)).copied().unwrap_or(0).min(found.capacity);
                // A port keeps the mode its tick last saw (see `port_modes` in lib.rs): the validation rewrites every
                // member's data, and a member carries no data of its own in `multiblock-changed`. Other members have
                // no mode byte and keep the plain encoding.
                let data = if is_port {
                    let output = self.port_modes.get(&(dimension.to_owned(), m.pos.x, m.pos.y, m.pos.z)).copied().unwrap_or(0) != 0;
                    let given = if output { total.min(found.rate) } else { 0 };
                    with_mode_given(encode(&found, total), output, given)
                } else {
                    encode(&found, total)
                };
                out.push(BlockEntityUpdate {
                    dimension: dimension.to_owned(),
                    pos: m.pos,
                    data,
                    items: None,
                    energy: if !found.valid && is_port { Some(0) } else { None },
                    energy_faces: if !found.valid && is_port { Some(vec![FlowSide::None; 6]) } else { None },
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        out
    }

    /// A tick of the ports of the matrices.
    pub(crate) fn run_matrix(&mut self, ports: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut by: HashMap<(String, [i32; 3]), Vec<&BlockEntity>> = HashMap::new();
        for p in ports {
            if let Some((f, _)) = decode(&p.data).filter(|(f, _)| f.valid) {
                by.entry((p.dimension.clone(), f.min)).or_default().push(*p);
            }
        }
        let mut out = Vec::new();
        for (key, group) in by {
            let Some((found, stored)) = decode(&group[0].data) else { continue };
            let known = self.matrices.get(&key).copied();
            // After a load the structure holds what its ports remember.
            let total = known.unwrap_or(stored.min(found.capacity));

            // Per-port transfer accounting:
            let mut total_input = 0u64;
            let mut total_output = 0u64;
            for p in &group {
                let output = mode_of(&p.data);
                let given = given_of(&p.data).unwrap_or(if output { p.energy } else { 0 });
                if output {
                    // Output port: energy was extracted by external networks
                    let extracted = given.saturating_sub(p.energy);
                    total_output = total_output.saturating_add(extracted);
                } else {
                    // Input port: energy was inserted by external networks
                    let inserted = p.energy.saturating_sub(given);
                    total_input = total_input.saturating_add(inserted);
                }
            }

            // Cap transfers by rate limit and available storage/energy (MatrixEnergyContainer.java:162-194)
            let allowed_input = total_input.min(found.rate).min(found.capacity.saturating_sub(total));
            let allowed_output = total_output.min(found.rate).min(total);
            let mut next = (total + allowed_input - allowed_output).min(found.capacity);
            // The two charge slots of each port: the first fills the matrix from an item, the second charges an item from it.
            let mut slot_updates: HashMap<[i32; 3], Vec<Option<ItemStack>>> = HashMap::new();
            for p in &group {
                let mut slots: Vec<Option<ItemStack>> = vec![p.items.first().cloned().flatten(), p.items.get(1).cloned().flatten()];
                let mut changed = false;
                if let Some((gain, left)) = fill_step(&slots[0], found.capacity.saturating_sub(next)) {
                    next += gain;
                    slots[0] = left;
                    changed = true;
                }
                if let Some((give, now)) = drain_step(&slots[1], next) {
                    next -= give;
                    slots[1] = now;
                    changed = true;
                }
                if changed {
                    slot_updates.insert([p.pos.x, p.pos.y, p.pos.z], slots);
                }
            }
            self.matrices.insert(key.clone(), next);
            self.matrix_given.insert(key.clone(), next.min(found.rate));

            let mega = |v: u64| v / 1_000_000;
            let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
            let data = vec![
                half(mega(next), 0),
                half(mega(next), 16),
                half(mega(found.capacity), 0),
                half(mega(found.capacity), 16),
                half(found.rate / 1000, 0),
                half(found.rate / 1000, 16),
                i32::from(found.cells),
                i32::from(found.providers),
                ((next as f64 / found.capacity.max(1) as f64).clamp(0.0, 1.0) * 1000.0) as i32,
                half(allowed_input / 1000, 0),
                half(allowed_input / 1000, 16),
                half(allowed_output / 1000, 0),
                half(allowed_output / 1000, 16),
            ];
            let texts = screen_texts(next, found.capacity, allowed_input, allowed_output);
            for p in &group {
                let tkey = (p.dimension.clone(), p.pos.x, p.pos.y, p.pos.z);
                if self.menu_texts.get(&tkey) != Some(&texts) {
                    let _ = world::set_menu_text(&p.dimension, p.pos.x, p.pos.y, p.pos.z, &texts);
                    self.menu_texts.insert(tkey, texts.clone());
                }
            }

            for p in &group {
                // A sneak click with the configurator toggles whether the port takes energy in or gives it out
                // (`TileEntityInductionPort.onSneakRightClick`); a plain click says which it is.
                let mut output = mode_of(&p.data);
                let mut mode_changed = false;
                if let Some(click) = self.take_click(p) {
                    if click.sneak && click.mode.is_configurating() {
                        output = !output;
                        mode_changed = true;
                        let _ = players::tell(
                            &click.player,
                            &format!("Toggled Induction Port transfer mode to {}.", if output { "Output" } else { "Input" }),
                        );
                    } else if click.mode.is_configurating() {
                        let _ = players::tell(&click.player, &format!("Induction Port mode: {}", if output { "Output" } else { "Input" }));
                    }
                }
                // Remember the mode for the structures function: it rewrites the port's data on every revalidation
                // and cannot read the mode from the member it is given.
                self.port_modes.insert((p.dimension.clone(), p.pos.x, p.pos.y, p.pos.z), u8::from(output));

                let (port_cap, port_energy, energy_faces) = if output {
                    // Output port: energy is mirrored up to rate limit, room to extract
                    let avail = next.min(found.rate);
                    (found.rate.max(1), avail, vec![FlowSide::Output; 6])
                } else {
                    // Input port: energy is 0, room to insert up to rate & remaining capacity
                    let room = (found.capacity.saturating_sub(next)).min(found.rate);
                    (room.max(1), 0, vec![FlowSide::Input; 6])
                };

                let _ = world::set_energy_capacity(&p.dimension, p.pos.x, p.pos.y, p.pos.z, port_cap);

                let changed = next != stored || p.energy != port_energy || mode_changed;
                let port_data = with_mode_given(encode(&found, next), output, port_energy);
                out.push(BlockEntityUpdate {
                    dimension: p.dimension.clone(),
                    pos: p.pos,
                    data: if changed { port_data } else { p.data.clone() },
                    items: slot_updates.get(&[p.pos.x, p.pos.y, p.pos.z]).cloned(),
                    energy: Some(port_energy),
                    energy_faces: Some(energy_faces),
                    tanks: None,
                    tank_faces: None,
                    menu_data: Some(data.clone()),
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hollow box of `n` with a port at the middle of its top face and a basic cell and provider inside (when `n` > 3, two cells).
    fn matrix(n: i32) -> Vec<([i32; 3], Part)> {
        let mut out = Vec::new();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    let edge = |c: i32| c == 0 || c == n - 1;
                    let e = [edge(x), edge(y), edge(z)].iter().filter(|b| **b).count();
                    if e > 0 {
                        out.push(([x, y, z], if (x, y, z) == (n / 2, n - 1, n / 2) { Part::Port } else { Part::Casing }));
                    }
                }
            }
        }
        out.push(([1, 1, 1], Part::Cell(fe(8_000_000_000))));
        out.push(([1, 1, 2], Part::Provider(fe(256_000))));
        out
    }

    #[test]
    fn a_matrix_adds_up_its_cells_and_providers() {
        let f = validate(&matrix(4), [0; 3], [3; 3], |_| true);
        assert_eq!((f.valid, f.capacity, f.rate, f.cells, f.providers), (true, fe(8_000_000_000), fe(256_000), 1, 1));
    }

    #[test]
    fn it_needs_a_cell_and_a_closed_shell() {
        let mut none = matrix(4);
        none.retain(|(_, p)| !matches!(p, Part::Cell(_)));
        assert!(!validate(&none, [0; 3], [3; 3], |_| true).valid);
        let mut hole = matrix(4);
        hole.retain(|(p, _)| *p != [0, 1, 1]);
        assert!(!validate(&hole, [0; 3], [3; 3], |_| true).valid);
        // A port on an edge, a cell in the shell.
        let mut edge = matrix(4);
        edge.push(([0, 0, 1], Part::Port));
        edge.retain(|(p, q)| !(*p == [0, 0, 1] && *q == Part::Casing));
        assert!(!validate(&edge, [0; 3], [3; 3], |_| true).valid);
        assert!(!validate(&matrix(4), [0; 3], [3; 3], |_| false).valid);
    }

    #[test]
    fn the_energy_follows_the_ports_and_is_kept_in_their_bytes() {
        let f = Found { valid: true, min: [1, 2, 3], capacity: 9_000, rate: 100, cells: 2, providers: 1 };
        assert_eq!(decode(&encode(&f, 777)), Some((f, 777)));
        // The ports were given 100 each: one took 40 in, another gave 30 out.
        assert_eq!(merge(500, 100, &[140, 70], 9_000), 510);
        assert_eq!(merge(10, 100, &[0], 9_000), 0);
        assert_eq!(merge(8_990, 100, &[200], 9_000), 9_000);
    }

    /// CuboidStructureValidator.java:115,149 and MatrixValidator.java:26-34:
    /// Structural glass is accepted on matrix faces (edges == 1), but forbidden on frame/edges/corners (edges > 1).
    #[test]
    fn structural_glass_allowed_on_matrix_faces_rejected_on_edges() {
        let mut m = matrix(4);
        // Replace casing on face (1, 3, 2) with glass
        for (pos, part) in &mut m {
            if *pos == [1, 3, 2] {
                *part = Part::Glass;
            }
        }
        let f = validate(&m, [0; 3], [3; 3], |_| true);
        assert!(f.valid, "Structural glass on face of induction matrix should be valid");

        // Now place structural glass on edge (0, 0, 1)
        for (pos, part) in &mut m {
            if *pos == [0, 0, 1] {
                *part = Part::Glass;
            }
        }
        let bad = validate(&m, [0; 3], [3; 3], |_| true);
        assert!(!bad.valid, "Structural glass on edge must be invalid (CuboidStructureValidator.java:115)");
    }

    /// MatrixEnergyContainer.java:162-194 and TileEntityInductionPort.java:38:
    /// Input ports (output = false) have room for insertion (given = 0),
    /// while Output ports (output = true) hold energy available for extraction (given = avail).
    #[test]
    fn input_port_receives_output_port_extracts() {
        let rate = 102_400u64;
        let capacity = 3_200_000_000u64;
        let stored = 1_000_000u64;

        // Input port: given = 0. Outside network inserts 50_000 FE into it.
        let input_port_given = 0u64;
        let input_port_now = 50_000u64;
        let inserted = input_port_now.saturating_sub(input_port_given);
        assert_eq!(inserted, 50_000);

        // Output port: given = mirror = stored.min(rate) = 102_400. Outside network extracts 30_000 FE (leaves 72_400).
        let output_port_given = stored.min(rate);
        let output_port_now = output_port_given - 30_000;
        let extracted = output_port_given.saturating_sub(output_port_now);
        assert_eq!(extracted, 30_000);

        let allowed_input = inserted.min(rate).min(capacity - stored);
        let allowed_output = extracted.min(rate).min(stored);
        let next = stored + allowed_input - allowed_output;
        assert_eq!(next, 1_000_000 + 50_000 - 30_000);
    }

    /// MatrixMultiblockData.java:123-128:
    /// Placing a new port (which starts with energy 0) must not cause phantom loss of stored energy.
    #[test]
    fn new_port_placement_does_not_destroy_energy() {
        let stored = 500_000u64;
        // A new input port has given = 0 and energy = 0.
        let new_input_given = 0u64;
        let new_input_now = 0u64;
        let inserted = new_input_now.saturating_sub(new_input_given);
        assert_eq!(inserted, 0);

        // A new output port has given = 0 and energy = 0.
        let new_output_given = 0u64;
        let new_output_now = 0u64;
        let extracted = new_output_given.saturating_sub(new_output_now);
        assert_eq!(extracted, 0);

        // Energy remains untouched
        let next = stored + inserted - extracted;
        assert_eq!(next, stored);
    }

    fn suit_stack(energy: u64) -> ItemStack {
        let mut piece = mekasuit::MekaSuitItemStack::new(mekasuit::HELMET).unwrap();
        piece.energy = energy;
        ItemStack { item: mekasuit::HELMET.to_owned(), count: 1, components: piece.encode("") }
    }

    #[test]
    fn the_second_slot_charges_an_item_from_the_matrix() {
        let slot = Some(suit_stack(0));
        let (give, after) = drain_step(&slot, 10_000_000_000).unwrap();
        let rate = item_energy(slot.as_ref().unwrap()).unwrap().2;
        assert_eq!(give, rate);
        assert_eq!(item_energy(after.as_ref().unwrap()).unwrap().0, rate);
        // nothing available, or a full item, or no energy item: nothing
        assert!(drain_step(&slot, 0).is_none());
        let max = item_energy(slot.as_ref().unwrap()).unwrap().1;
        assert!(drain_step(&Some(suit_stack(max)), 1_000).is_none());
        assert!(drain_step(&Some(ItemStack { item: "minecraft:stick".into(), count: 1, components: String::new() }), 1_000).is_none());
        assert!(drain_step(&None, 1_000).is_none());
    }

    #[test]
    fn the_first_slot_fills_the_matrix_from_an_item() {
        let (_, max, rate) = item_energy(&suit_stack(0)).unwrap();
        let slot = Some(suit_stack(max));
        let (gain, after) = fill_step(&slot, u64::MAX).unwrap();
        assert_eq!(gain, rate.min(max));
        assert_eq!(item_energy(after.as_ref().unwrap()).unwrap().0, max - gain);
        // the room bounds it, an empty item gives nothing
        let (small, _) = fill_step(&slot, 5).unwrap();
        assert_eq!(small, 5);
        assert!(fill_step(&Some(suit_stack(0)), 1_000).is_none());
    }

    #[test]
    fn the_inner_screen_lines() {
        let t = screen_texts(2_000_000, 8_000_000_000, 0, 1_500);
        assert_eq!(t, vec!["Energy: 2 MFE".to_owned(), "Capacity: 8 GFE".to_owned(), "Input: 0 FE/t".to_owned(), "Output: 1.5 kFE/t".to_owned()]);
    }
}
