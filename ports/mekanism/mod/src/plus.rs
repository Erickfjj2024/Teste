//! The machines of `continuous` and the storage tanks: their ticks, buttons and what they keep. They sit beside `run` in
//! `lib.rs` and share its configuration clicks.
use super::*;
use crate::tank::{DumpMode, EditMode};

/// Menu button ids past the side configuration window's (1000 to 3049) and the infuser's dump button (4000).
pub const DUMP_LEFT: i32 = 5000;
pub const DUMP_RIGHT: i32 = 5001;
pub const EDIT_MODE: i32 = 5100;

/// The fluids an ingredient names (`"#minecraft:water"`, `"#c:brine"`), as the ids the tanks hold.
fn fluids_of(ingredient: &serde_json::Value) -> Vec<String> {
    match ingredient {
        serde_json::Value::String(s) => {
            let s = s.trim_start_matches('#');
            vec![match s {
                "c:brine" => "mekanism:brine".to_owned(),
                "c:lithium" => "mekanism:lithium".to_owned(),
                other => other.to_owned(),
            }]
        }
        serde_json::Value::Array(l) => l.iter().flat_map(fluids_of).collect(),
        _ => Vec::new(),
    }
}

/// The recipes of a machine of `continuous`.
pub fn load_cont_book(shape: continuous::Shape) -> continuous::Book {
    use continuous::{Recipe, Shape};
    let mut book = continuous::Book::default();
    let kind = match shape {
        Shape::Infuser => "mekanism:chemical_infusing",
        Shape::Washer => "mekanism:washing",
        Shape::Separator => "mekanism:separating",
    };
    for r in recipe_types::recipes_of_type(kind) {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&r.json) else { continue };
        let amount = |v: &serde_json::Value| v["amount"].as_u64().unwrap_or(1);
        match shape {
            Shape::Infuser => {
                let (l, r) = (&json["left_input"], &json["right_input"]);
                let Some(out) = json["output"]["id"].as_str() else { continue };
                book.add(
                    &chemicals_of(&l["ingredient"]),
                    &chemicals_of(&r["ingredient"]),
                    Recipe { a: amount(l), b: amount(r), outs: vec![(out.to_owned(), amount(&json["output"]))] },
                    true,
                );
            }
            Shape::Washer => {
                let (f, c) = (&json["fluid_input"], &json["chemical_input"]);
                let Some(out) = json["output"]["id"].as_str() else { continue };
                book.add(
                    &fluids_of(&f["ingredient"]),
                    &chemicals_of(&c["ingredient"]),
                    Recipe { a: amount(f), b: amount(c), outs: vec![(out.to_owned(), amount(&json["output"]))] },
                    false,
                );
            }
            Shape::Separator => {
                let f = &json["input"];
                let (l, rt) = (&json["output"]["left"], &json["output"]["right"]);
                let (Some(lo), Some(ro)) = (l["id"].as_str(), rt["id"].as_str()) else { continue };
                book.add(
                    &fluids_of(&f["ingredient"]),
                    &[],
                    Recipe { a: amount(f), b: 0, outs: vec![(lo.to_owned(), amount(l)), (ro.to_owned(), amount(rt))] },
                    false,
                );
            }
        }
    }
    book
}

fn held(e: &BlockEntity, i: usize) -> process::Tank {
    e.tanks.get(i).map_or_else(process::Tank::default, |t| process::Tank { chemical: t.substance.clone(), amount: t.amount })
}

fn as_tank(t: &process::Tank) -> Tank {
    Tank { substance: t.chemical.clone(), amount: t.amount }
}

/// The screen's three numbers about a big tank: its amount (low and high 16 bits) and its chemical, then its capacity.
fn big_tank(t: &process::Tank, cap: u64) -> [i32; 5] {
    let half = |v: u64, s: u32| ((v >> s) & 0xFFFF) as i32;
    let v = t.amount.min(u64::from(u32::MAX));
    let c = cap.min(u64::from(u32::MAX));
    [half(v, 0), half(v, 16), if t.amount == 0 { 0 } else { chem::index(&t.chemical) }, half(c, 0), half(c, 16)]
}

impl Mekanism {
    /// Loads a continuous machine's recipes when it has none (asked again every second until the game lends them).
    fn refresh_cont(&mut self, def: &Def, tick: u64) {
        let Some(spec) = def.cont else { return };
        let empty = self.conts.get(def.block).is_none_or(continuous::Book::is_empty);
        let due = self.tried.get(def.block).is_none_or(|t| tick.saturating_sub(*t) >= 20);
        if empty && due {
            self.tried.insert(def.block, tick);
            self.conts.insert(def.block, load_cont_book(spec.shape));
        }
    }

    /// A tick of the chemical infuser, the washer or the electrolytic separator.
    pub(crate) fn run_continuous(&mut self, def: &Def, e: &BlockEntity, tick: u64) -> Option<BlockEntityUpdate> {
        let spec = def.cont?;
        self.refresh_cont(def, tick);
        let layout = def.layout;
        let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        let progress = e.data.get(..2).map_or(0, |b| u32::from(u16::from_le_bytes([b[0], b[1]])));
        let stored = Config::decode(e.data.get(2..).unwrap_or(&[]), layout);
        let mut cfg = stored.unwrap_or_else(|| config::default_config(layout));
        if stored.is_none() {
            let front = cube::facing_name(front_for(e));
            if front != facing {
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                facing = front.to_owned();
            }
        }
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut dirty = stored.is_none() || !self.applied.contains(&key);
        if let Some(click) = self.take_click(e) {
            dirty |= self.configure(&mut cfg, &facing, layout, &click);
        }
        // The two gases of a separator can be dumped (`GasMode`), kept after the configuration.
        let mut modes = [DumpMode::from_byte(e.data.get(37).copied().unwrap_or(0)), DumpMode::from_byte(e.data.get(38).copied().unwrap_or(0))];
        let mut tanks = [held(e, 0), held(e, 1), held(e, 2)];
        let slot = |i: usize| stack(e.items.get(i).unwrap_or(&None));
        // The fluid slot: a full bucket empties into the fluid tank (the emptied one goes to the washer's output slot, or
        // stays where the full one was in the separator's).
        let mut items: Vec<Option<Stack>> = (0..e.items.len()).map(slot).collect();
        let mut items_changed = false;
        if layout == Layout::Washer && e.items.len() >= 3 {
            let (mut i0, mut o2) = (items[0].clone(), items[2].clone());
            if tank::buckets(&mut i0, &mut o2, &mut tanks[0], spec.caps[0], EditMode::Fill) {
                items[0] = i0;
                items[2] = o2;
                items_changed = true;
            }
        } else if layout == Layout::Separator && !e.items.is_empty() {
            let mut i0 = items[0].clone();
            let mut sink = None;
            if tank::buckets(&mut i0, &mut sink, &mut tanks[0], spec.caps[0], EditMode::Fill) {
                // The emptied bucket takes the place of the full one.
                items[0] = sink;
                items_changed = true;
            }
        }
        let book = self.conts.entry(def.block).or_default();
        let io = continuous::Io { tanks: tanks.clone(), energy: e.energy };
        let (next, status) = continuous::step(&spec, book, &io);
        let mut tanks = next.tanks.clone();
        if def.layout == Layout::Separator {
            tank::dump(&mut tanks[1], spec.caps[1], modes[0], 8, 1024);
            tank::dump(&mut tanks[2], spec.caps[2], modes[1], 8, 1024);
        }
        // Buttons asked for since the last tick.
        if let Some(ids) = self.buttons.remove(&key) {
            for id in ids {
                match id {
                    DUMP_LEFT => modes[0] = modes[0].next(),
                    DUMP_RIGHT => modes[1] = modes[1].next(),
                    _ => {}
                }
            }
        }
        let active = status == Status::Running;
        let lit = e.state.contains("lit=true");
        if active != lit {
            let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if active { "true" } else { "false" });
        }
        // The tanks that take from outside take only what the machine's recipes use.
        if !self.filtered.contains(&key)
            && let Some(book) = self.conts.get(def.block).filter(|b| !b.is_empty())
        {
            let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 0, &book.accepted(0));
            if spec.shape != continuous::Shape::Separator {
                let _ = world::set_tank_filter(&e.dimension, e.pos.x, e.pos.y, e.pos.z, 1, &book.accepted(1));
            }
            self.filtered.insert(key.clone());
        }
        let energy_faces = dirty.then(|| self.apply_config(e, layout, &facing, &cfg));
        let capacity = def.usage * STORED_TICKS * if def.layout == Layout::Separator { 25 } else { 1 };
        let mut data = menu_data(u32::from(active), next.energy, capacity, status, &cfg);
        for t in &tanks {
            data.extend(chem::screen(t));
        }
        data.extend([modes[0] as i32, modes[1] as i32]);
        let mut saved = machine_data(progress, &cfg);
        saved.extend([modes[0] as u8, modes[1] as u8]);
        let tanks_changed = tanks != io.tanks || e.tanks.len() != 3;
        if !items_changed && !dirty && !tanks_changed && next.energy == e.energy && data == e.menu_data && saved == e.data {
            return None;
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
            items: items_changed.then(|| items.iter().map(item).collect()),
            energy: Some(next.energy),
            energy_faces,
            tanks: tanks_changed.then(|| tanks.iter().map(as_tank).collect()),
            tank_faces: dirty.then(|| self.tank_sides(layout, &facing, &cfg)),
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }

    /// A tick of a storage tank.
    pub(crate) fn run_tank(&mut self, def: &Def, e: &BlockEntity) -> Option<BlockEntityUpdate> {
        let t = def.tank?;
        let fluid = t.fluid;
        let layout = def.layout;
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        let mut facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        // A fluid tank keeps its edit mode, a chemical tank its dump mode and then its configuration.
        let mode_byte = e.data.first().copied().unwrap_or(0);
        let stored = if fluid { Some(()) } else { Config::decode(e.data.get(1..).unwrap_or(&[]), layout).map(|_| ()) };
        let mut cfg = if fluid {
            config::default_config(Layout::Electric)
        } else {
            Config::decode(e.data.get(1..).unwrap_or(&[]), layout).unwrap_or_else(|| config::default_config(layout))
        };
        let mut edit = EditMode::from_byte(mode_byte);
        let mut dump_mode = DumpMode::from_byte(mode_byte);
        if stored.is_none() && !fluid {
            let front = cube::facing_name(front_for(e));
            if front != facing {
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", front);
                facing = front.to_owned();
            }
        }
        let mut dirty = stored.is_none() || !self.applied.contains(&key);
        if !fluid && let Some(click) = self.take_click(e) {
            dirty |= self.configure(&mut cfg, &facing, layout, &click);
        }
        if let Some(ids) = self.buttons.remove(&key) {
            for id in ids {
                if fluid && id == EDIT_MODE {
                    edit = edit.next();
                    dirty = true;
                } else if fluid && id == EDIT_MODE + 1 {
                    edit = edit.previous();
                    dirty = true;
                } else if !fluid && id == DUMP_LEFT {
                    dump_mode = dump_mode.next();
                }
            }
        }
        let mut tank_now = held(e, 0);
        let before = tank_now.clone();
        let mut items: Vec<Option<Stack>> = (0..e.items.len()).map(|i| stack(e.items.get(i).unwrap_or(&None))).collect();
        let mut items_changed = false;
        if fluid && items.len() >= 2 {
            let (mut i, mut o) = (items[0].clone(), items[1].clone());
            if tank::buckets(&mut i, &mut o, &mut tank_now, t.capacity, edit) {
                items[0] = i;
                items[1] = o;
                items_changed = true;
            }
        }
        if t.creative {
            tank::creative(&mut tank_now, t.capacity);
        }
        if !fluid && !t.creative {
            // A chemical tank loses 1/400 of its capacity a tick while dumping, at most its transfer rate.
            tank::dump(&mut tank_now, t.capacity, dump_mode, t.capacity / 400, t.rate);
        }
        let mut saved = vec![if fluid { edit as u8 } else { dump_mode as u8 }];
        if !fluid {
            saved.extend(cfg.encode());
        }
        let mut data = big_tank(&tank_now, t.capacity).to_vec();
        data.push(if fluid { edit as i32 } else { dump_mode as i32 });
        if !fluid {
            data.extend(cfg.screen_words());
        }
        // A fluid tank gives and takes by its edit mode; a chemical tank by its configuration.
        let faces: Vec<FlowSide> = if fluid {
            let side = match edit {
                EditMode::Both => FlowSide::Both,
                EditMode::Fill => FlowSide::Input,
                EditMode::Empty => FlowSide::Output,
            };
            vec![side; 6]
        } else {
            config::tank_flow(layout, &facing, &cfg)[0].iter().map(|f| flow_side(*f)).collect()
        };
        if !self.filtered.contains(&key) {
            // A fluid tank takes in fluids only, a chemical tank chemicals only (the kinds keep them apart).
            self.filtered.insert(key.clone());
        }
        let tanks_changed = tank_now != before || e.tanks.is_empty();
        if !items_changed && !dirty && !tanks_changed && data == e.menu_data && saved == e.data {
            return None;
        }
        self.applied.insert(key);
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
            items: items_changed.then(|| items.iter().map(item).collect()),
            energy: None,
            energy_faces: None,
            tanks: tanks_changed.then(|| vec![as_tank(&tank_now)]),
            tank_faces: dirty.then(|| vec![faces]),
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }

    /// A button of a tank's or a continuous machine's screen: it is applied on the machine's next tick, but a click of the
    /// side configuration window is applied now (as for the other machines).
    pub(crate) fn plus_button(&mut self, def: &Def, id: i32, e: &BlockEntity) -> Option<BlockEntityUpdate> {
        let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
        if matches!(id, DUMP_LEFT | DUMP_RIGHT | 5100 | 5101) {
            self.buttons.entry(key).or_default().push(id);
            return None;
        }
        let layout = def.layout;
        let facing = sides::property(&e.state, "facing").unwrap_or("north").to_owned();
        // A chemical tank keeps its mode byte first; a continuous machine its progress bytes.
        let skip = if def.kind == Kind::Tank { 1 } else { 2 };
        let mut cfg = Config::decode(e.data.get(skip..).unwrap_or(&[]), layout).unwrap_or_else(|| config::default_config(layout));
        match config::click_of(id)? {
            config::Click::Side { t, side, back } => {
                let now = cfg.side(t)[side];
                cfg.side_mut(t)[side] = config::cycle(layout, t, now, back);
            }
            config::Click::Eject(t) => cfg.eject[t as usize] = !cfg.eject[t as usize],
            config::Click::Clear { t, back } => {
                let target = config::clear_target(layout, t, cfg.side(t), back);
                *cfg.side_mut(t) = [target; 6];
            }
        }
        let faces = self.apply_config(e, layout, &facing, &cfg);
        // The configuration's words replace those in the screen's numbers (6 to 15).
        let mut data = e.menu_data.clone();
        for (i, w) in cfg.screen_words().iter().enumerate() {
            if let Some(slot) = data.get_mut(6 + i) {
                *slot = *w;
            }
        }
        let mut saved = e.data.clone();
        let encoded = cfg.encode();
        saved.truncate(skip);
        saved.resize(skip, 0);
        saved.extend(&encoded);
        if def.kind == Kind::Continuous && e.data.len() > skip + encoded.len() {
            saved.extend(&e.data[skip + encoded.len()..]);
        }
        Some(BlockEntityUpdate {
            dimension: e.dimension.clone(),
            pos: e.pos,
            data: saved,
            items: None,
            energy: None,
            energy_faces: Some(faces),
            tanks: None,
            tank_faces: Some(self.tank_sides(layout, &facing, &cfg)),
            menu_data: Some(data),
            transmitter_faces: None,
        })
    }
}
