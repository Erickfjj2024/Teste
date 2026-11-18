//! The Module Tweaker (`GuiModuleTweaker`, `ModuleTweakerContainer`): the screen the Module Tweaker key (backslash, `data/mekanism/horizon/key/module_tweaker.json`) opens when the player has a
//! gear item with modules in the armor slots, the hotbar or the off hand (`hasTweakableItem`). The container shows those fourteen slots (the four armor pieces, the nine hotbar slots and the
//! off hand, none of them can be taken); a click on one picks the item, the list shows its installed modules, and the module picked there shows its settings (`GuiModuleScreen`), each of which
//! a click changes (the original's toggles and sliders).
//!
//! The screen's numbers: 0 the picked slot (0-13), 1 the picked module row (-1 none), 2 how many modules the item holds, 3-16 whether each slot holds a tweakable item. Strings: 0-15 the module
//! rows, then a label and a value for each of the 8 setting rows (16 + 2 i, 17 + 2 i).
//!
//! Not here: the ARGB colour window of the Color Modulation Unit (a click steps through the sixteen dye colours instead), the MekaSuit helmet Options window, the arrow keys that step through
//! the slots, the armor preview, the icons beside the modules (the rows are text) and the bar of the settings (at most 8 are shown, which is enough for every module).
use super::*;
use crate::modules::*;
use horizon_mod_sdk::server::{ItemMenu, ItemMenuUpdate};
use std::collections::HashMap;

pub const KEY: &str = "mekanism:module_tweaker";
pub const MENU: &str = "mekanism:module_tweaker";
/// The ids of the slot buttons, the module rows and the setting rows.
pub const SLOT_BASE: i32 = 9500;
pub const LIST_BASE: i32 = 9520;
pub const CONFIG_BASE: i32 = 9540;
pub const LIST_ROWS: usize = 16;
pub const CONFIG_ROWS: usize = 8;
/// The player inventory index of each of the container's slots: armor from the head down (`36 + slotType.ordinal() - 2`), the hotbar, the off hand.
pub const GEAR: [u32; 14] = [39, 38, 37, 36, 0, 1, 2, 3, 4, 5, 6, 7, 8, 40];
/// The dye colours a click on the colour setting steps through.
const DYES: [u32; 16] = [
    0xFFF9FFFE, 0xFFF9801D, 0xFFC74EBD, 0xFF3AB3DA, 0xFFFED83D, 0xFF80C71F, 0xFFF38BAA, 0xFF474F52, 0xFF9D9D97, 0xFF169C9C, 0xFF8932B8, 0xFF3C44AA, 0xFF835432, 0xFF5E7C16, 0xFFB02E26, 0xFF1D1D21,
];

/// One setting of a module (`ModuleConfig`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cfg {
    Enabled,
    HandleMode,
    RenderHud,
    Color,
    JetpackMode,
    Thrust,
    HoverThrust,
    Sprint,
    GravityBoost,
    Jump,
    Step,
    Range,
    FillHeld,
    ChargeSuit,
    ChargeInventory,
    Swim,
    Beneficial,
    Neutral,
    Harmful,
    Attack,
    Excavation,
    Blast,
    Extended,
    VeinRange,
    Farming,
    RequireTarget,
}

/// What the screen remembers for a player.
#[derive(Clone, Debug, Default)]
pub struct View {
    pub slot: usize,
    pub module: Option<usize>,
}

pub type Views = HashMap<String, View>;

/// A name the lang file gives a module (`module.mekanism.<name>`).
pub fn title(t: ModuleType) -> String {
    if t == ModuleType::FortuneUnit {
        return "Ore Refinement Unit".to_owned();
    }
    let id = t.id();
    let path = id.split(':').nth(1).unwrap_or(id);
    path.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().collect::<String>() + c.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether the item holds modules (`ModuleTweakerContainer.isTweakableItem`).
pub fn is_tweakable(item: &str) -> bool {
    ModuleType::EnergyUnit.can_install_on(item)
}

/// The settings of a module in the order the original lists them: enabled (unless the module cannot be disabled), the mode key, the HUD, then the module's own.
pub fn configs_of(t: ModuleType, installed: u8) -> Vec<Cfg> {
    let mut v = Vec::new();
    if t.can_be_disabled() {
        v.push(Cfg::Enabled);
    }
    if t.handles_mode_change() {
        v.push(Cfg::HandleMode);
    }
    if t.renders_hud() {
        v.push(Cfg::RenderHud);
    }
    match t {
        ModuleType::ColorModulationUnit => v.push(Cfg::Color),
        ModuleType::JetpackUnit => v.extend([Cfg::JetpackMode, Cfg::Thrust, Cfg::HoverThrust]),
        ModuleType::GravitationalModulatingUnit => v.push(Cfg::GravityBoost),
        ModuleType::LocomotiveBoostingUnit => v.push(Cfg::Sprint),
        ModuleType::HydraulicPropulsionUnit => v.extend([Cfg::Jump, Cfg::Step]),
        ModuleType::MagneticAttractionUnit => v.push(Cfg::Range),
        ModuleType::ElectrolyticBreathingUnit => v.push(Cfg::FillHeld),
        ModuleType::InhalationPurificationUnit => v.extend([Cfg::Beneficial, Cfg::Neutral, Cfg::Harmful]),
        ModuleType::ChargeDistributionUnit => v.extend([Cfg::ChargeSuit, Cfg::ChargeInventory]),
        // The swim boost is only offered at the most units (`BOOST_STACKS`).
        ModuleType::HydrostaticRepulsorUnit if installed >= 4 => v.push(Cfg::Swim),
        ModuleType::AttackAmplificationUnit => v.push(Cfg::Attack),
        ModuleType::ExcavationEscalationUnit => v.push(Cfg::Excavation),
        ModuleType::BlastingUnit => v.push(Cfg::Blast),
        ModuleType::VeinMiningUnit => v.extend([Cfg::Extended, Cfg::VeinRange]),
        ModuleType::FarmingUnit => v.push(Cfg::Farming),
        ModuleType::TeleportationUnit => v.push(Cfg::RequireTarget),
        _ => {}
    }
    v
}

fn label(c: Cfg) -> &'static str {
    match c {
        Cfg::Enabled => "Enabled",
        Cfg::HandleMode => "Handle Mode Key",
        Cfg::RenderHud => "Show in HUD",
        Cfg::Color => "ARGB Color",
        Cfg::JetpackMode => "Jetpack Mode",
        Cfg::Thrust => "Thrust Multiplier",
        Cfg::HoverThrust => "Hover Thrust Multiplier",
        Cfg::Sprint => "Sprint Boost",
        Cfg::GravityBoost => "Speed Boost",
        Cfg::Jump => "Jump Boost",
        Cfg::Step => "Step Assist",
        Cfg::Range => "Range",
        Cfg::FillHeld => "Fill Held",
        Cfg::ChargeSuit => "Charge Suit",
        Cfg::ChargeInventory => "Charge Inventory",
        Cfg::Swim => "Swim Boost",
        Cfg::Beneficial => "Remove Beneficial",
        Cfg::Neutral => "Remove Neutral",
        Cfg::Harmful => "Remove Harmful",
        Cfg::Attack => "Bonus Attack Damage",
        Cfg::Excavation => "Efficiency",
        Cfg::Blast => "Blast Radius",
        Cfg::Extended => "Extended Mode",
        Cfg::VeinRange => "Excavation Range",
        Cfg::Farming => "Farming Radius",
        Cfg::RequireTarget => "Requires Block Target",
    }
}

fn on_off(b: bool) -> String {
    if b { "On" } else { "Off" }.to_owned()
}

fn number(v: f32) -> String {
    if v.fract() == 0.0 { format!("{}", v as i64) } else { format!("{v}") }
}

fn area(radius: u8) -> String {
    format!("{0}x{0}", 2 * u32::from(radius) + 1)
}

/// How many values of an enum setting can be chosen (`createBounded(.., installed + n)`), at most the enum's size.
fn choices(c: Cfg, installed: u8) -> usize {
    let n = usize::from(installed);
    match c {
        Cfg::JetpackMode => 4,
        Cfg::GravityBoost => 5,
        Cfg::Attack | Cfg::Excavation => (n + 2).min(6),
        Cfg::Thrust | Cfg::HoverThrust | Cfg::Sprint | Cfg::Jump | Cfg::Step | Cfg::Range | Cfg::Blast | Cfg::VeinRange | Cfg::Farming => (n + 1).min(5),
        _ => 0,
    }
}

/// The place of an enum setting's value in its enum.
fn ordinal(m: &InstalledModule, c: Cfg) -> usize {
    usize::from(match c {
        Cfg::JetpackMode => m.jetpack_mode as u8,
        Cfg::Thrust => m.thrust_multiplier.ordinal(),
        Cfg::HoverThrust => m.hover_thrust_multiplier.ordinal(),
        Cfg::Sprint | Cfg::GravityBoost => m.sprint_boost.ordinal(),
        Cfg::Jump => m.jump_boost.ordinal(),
        Cfg::Step => m.step_assist.ordinal(),
        Cfg::Range => m.magnetic_range.ordinal(),
        Cfg::Attack => m.attack_damage.ordinal(),
        Cfg::Excavation => m.excavation_mode.ordinal(),
        Cfg::Blast => m.blast_radius.ordinal(),
        Cfg::VeinRange => m.excavation_range.ordinal(),
        Cfg::Farming => m.farming_radius.ordinal(),
        _ => 0,
    })
}

fn set_ordinal(m: &mut InstalledModule, c: Cfg, i: usize) {
    let i = i as u8;
    match c {
        Cfg::JetpackMode => m.jetpack_mode = [JetpackMode::Normal, JetpackMode::Hover, JetpackMode::Vector, JetpackMode::Disabled][usize::from(i).min(3)],
        Cfg::Thrust => m.thrust_multiplier = ThrustMultiplier::from_ordinal(i),
        Cfg::HoverThrust => m.hover_thrust_multiplier = ThrustMultiplier::from_ordinal(i),
        Cfg::Sprint | Cfg::GravityBoost => m.sprint_boost = SprintBoost::from_ordinal(i),
        Cfg::Jump => m.jump_boost = JumpBoost::from_ordinal(i),
        Cfg::Step => m.step_assist = StepAssist::from_ordinal(i),
        Cfg::Range => m.magnetic_range = MagneticRange::from_ordinal(i),
        Cfg::Attack => m.attack_damage = AttackDamage::from_ordinal(i),
        Cfg::Excavation => m.excavation_mode = ExcavationMode::from_ordinal(i),
        Cfg::Blast => m.blast_radius = BlastRadius::from_ordinal(i),
        Cfg::VeinRange => m.excavation_range = ExcavationRange::from_ordinal(i),
        Cfg::Farming => m.farming_radius = FarmingRadius::from_ordinal(i),
        _ => {}
    }
}

/// The value a setting shows (the enum's own text in the original).
pub fn value_text(m: &InstalledModule, c: Cfg) -> String {
    let i = ordinal(m, c);
    match c {
        Cfg::Enabled => on_off(m.enabled),
        Cfg::HandleMode => on_off(m.handle_mode_change),
        Cfg::RenderHud => on_off(m.render_hud),
        Cfg::Color => format!("#{:08X}", m.color_argb),
        Cfg::JetpackMode => ["Normal", "Hover", "Vector", "Disabled"][i.min(3)].to_owned(),
        Cfg::Thrust | Cfg::HoverThrust => number(ThrustMultiplier::from_ordinal(i as u8).multiplier()),
        Cfg::Sprint | Cfg::GravityBoost => number(SprintBoost::from_ordinal(i as u8).boost()),
        Cfg::Jump => number([0.0, 0.5, 1.0, 3.0, 5.0][i.min(4)]),
        Cfg::Step => number([0.0, 0.5, 1.0, 1.5, 2.0][i.min(4)]),
        Cfg::Range => number([0.0, 1.0, 3.0, 5.0, 10.0][i.min(4)]),
        Cfg::FillHeld => on_off(m.fill_held),
        Cfg::ChargeSuit => on_off(m.charge_suit),
        Cfg::ChargeInventory => on_off(m.charge_inventory),
        Cfg::Swim => on_off(m.swim_boost),
        Cfg::Beneficial => on_off(m.beneficial_effects),
        Cfg::Neutral => on_off(m.neutral_effects),
        Cfg::Harmful => on_off(m.harmful_effects),
        Cfg::Attack => ["0", "4", "8", "16", "24", "32"][i.min(5)].to_owned(),
        Cfg::Excavation => ["0", "4", "16", "32", "64", "128"][i.min(5)].to_owned(),
        Cfg::Blast => area([0, 1, 2, 3, 4][i.min(4)]),
        Cfg::Extended => on_off(m.extended_mode),
        Cfg::VeinRange => ["0", "2", "4", "6", "8"][i.min(4)].to_owned(),
        Cfg::Farming => area([0, 1, 2, 3, 4][i.min(4)]),
        Cfg::RequireTarget => on_off(m.require_target),
    }
}

/// A click on a setting: toggles a flag or steps an enum (a right click steps back). True when the container changed.
pub fn press(container: &mut ModuleContainer, t: ModuleType, c: Cfg, back: bool) -> bool {
    let Some(installed) = container.get(t).map(|m| m.installed) else { return false };
    if c == Cfg::Enabled {
        container.toggle_enabled(t);
        return true;
    }
    let Some(m) = container.get_mut(t) else { return false };
    match c {
        Cfg::HandleMode => m.handle_mode_change = !m.handle_mode_change,
        Cfg::RenderHud => m.render_hud = !m.render_hud,
        Cfg::FillHeld => m.fill_held = !m.fill_held,
        Cfg::ChargeSuit => m.charge_suit = !m.charge_suit,
        Cfg::ChargeInventory => m.charge_inventory = !m.charge_inventory,
        Cfg::Swim => m.swim_boost = !m.swim_boost,
        Cfg::Beneficial => m.beneficial_effects = !m.beneficial_effects,
        Cfg::Neutral => m.neutral_effects = !m.neutral_effects,
        Cfg::Harmful => m.harmful_effects = !m.harmful_effects,
        Cfg::Extended => m.extended_mode = !m.extended_mode,
        Cfg::RequireTarget => m.require_target = !m.require_target,
        Cfg::Color => {
            let at = DYES.iter().position(|d| *d == m.color_argb).map_or(0, |p| p + 1);
            m.color_argb = DYES[(at + if back { DYES.len() - 1 } else { 0 }) % DYES.len()];
        }
        _ => {
            let n = choices(c, installed);
            if n == 0 {
                return false;
            }
            let at = ordinal(m, c).min(n - 1);
            set_ordinal(m, c, (at + if back { n - 1 } else { 1 }) % n);
        }
    }
    if c == Cfg::HandleMode && m.handle_mode_change {
        // `ModuleContainer`: only one module of the item handles the mode key.
        for (other, om) in container.modules.iter_mut() {
            if *other != t && other.handles_mode_change() {
                om.handle_mode_change = false;
            }
        }
    }
    true
}

/// The container's stack for a slot of the screen.
fn gear(inv: &[Option<ItemStack>], i: usize) -> Option<ItemStack> {
    inv.get(GEAR[i] as usize).cloned().flatten()
}

/// Whether the player has an item to tweak (`hasTweakableItem`).
pub fn has_tweakable(inv: &[Option<ItemStack>]) -> bool {
    (0..GEAR.len()).any(|i| gear(inv, i).is_some_and(|s| is_tweakable(&s.item)))
}

/// The screen for the picked slot: the strings, the numbers and the container, once the view is valid.
pub fn screen(inv: &[Option<ItemStack>], view: &mut View) -> (Vec<String>, Vec<i32>) {
    let stack = gear(inv, view.slot).filter(|s| is_tweakable(&s.item));
    let container = stack.as_ref().map(|s| ModuleContainer::decode_from_components(&s.components));
    let list = container.as_ref().map(installed_list).unwrap_or_default();
    if view.module.is_some_and(|m| m >= list.len()) {
        view.module = None;
    }
    let mut texts = vec![String::new(); 16 + 2 * CONFIG_ROWS];
    for (i, (t, n)) in list.iter().take(LIST_ROWS).enumerate() {
        texts[i] = format!("{} ({}/{})", title(*t), n, t.max_stack());
    }
    if let (Some(row), Some(c)) = (view.module, container.as_ref()) {
        let (t, n) = list[row];
        if let Some(m) = c.get(t) {
            for (i, cfg) in configs_of(t, n).into_iter().take(CONFIG_ROWS).enumerate() {
                texts[16 + 2 * i] = label(cfg).to_owned();
                texts[17 + 2 * i] = value_text(m, cfg);
            }
        }
    }
    let mut data = vec![view.slot as i32, view.module.map_or(-1, |m| m as i32), list.len().min(LIST_ROWS) as i32];
    data.extend((0..GEAR.len()).map(|i| i32::from(gear(inv, i).is_some_and(|s| is_tweakable(&s.item)))));
    (texts, data)
}

fn installed_list(c: &ModuleContainer) -> Vec<(ModuleType, u8)> {
    crate::modification_station::installed_list(c)
}

impl Mekanism {
    /// The Module Tweaker key: opens the screen when the player has an item to tweak (`PacketOpenGui` with `hasTweakableItem`).
    pub(crate) fn module_tweaker_key(&mut self, player: &Player) -> bool {
        let Some(inv) = player_items::get_inventory(&player.id) else { return true };
        if !has_tweakable(&inv) {
            return true;
        }
        let first = (0..GEAR.len()).find(|i| gear(&inv, *i).is_some_and(|s| is_tweakable(&s.item))).unwrap_or(0);
        self.tweakers.insert(player.id.clone(), View { slot: first, module: None });
        self.item_menu_sent.remove(&(player.id.clone(), MENU.to_owned()));
        let _ = players::open_player_menu(&player.id, MENU);
        true
    }

    /// One tick of the open screen: the slots' items, the module rows and the settings.
    pub(crate) fn module_tweaker_tick(&mut self, player: &Player, menu: &ItemMenu) -> Option<ItemMenuUpdate> {
        let inv = player_items::get_inventory(&player.id)?;
        let view = self.tweakers.entry(player.id.clone()).or_default();
        let (texts, data) = screen(&inv, view);
        let items: Vec<Option<ItemStack>> = (0..GEAR.len()).map(|i| gear(&inv, i)).collect();
        let same_items = menu.items.len() == items.len() && menu.items.iter().zip(&items).all(|(a, b)| a == b);
        let key = (player.id.clone(), menu.menu.clone());
        if same_items && menu.menu_data == data && self.item_menu_sent.get(&key) == Some(&texts) {
            return None;
        }
        self.item_menu_sent.insert(key, texts.clone());
        Some(ItemMenuUpdate { items: Some(items), menu_data: Some(data), menu_text: Some(texts), close: false, menu_grid: None })
    }

    /// A press on a slot, a module row or a setting. Settings are written back to the picked item.
    pub(crate) fn module_tweaker_button(&mut self, player: &Player, id: i32) -> Option<ItemMenuUpdate> {
        let inv = player_items::get_inventory(&player.id)?;
        let view = self.tweakers.entry(player.id.clone()).or_default();
        if (SLOT_BASE..SLOT_BASE + GEAR.len() as i32).contains(&id) {
            let slot = (id - SLOT_BASE) as usize;
            if gear(&inv, slot).is_some_and(|s| is_tweakable(&s.item)) {
                *view = View { slot, module: None };
            }
        } else if (LIST_BASE..LIST_BASE + LIST_ROWS as i32).contains(&id) {
            view.module = Some((id - LIST_BASE) as usize);
        } else if (CONFIG_BASE..CONFIG_BASE + CONFIG_ROWS as i32).contains(&id) {
            let row = view.module?;
            let stack = gear(&inv, view.slot).filter(|s| is_tweakable(&s.item))?;
            let mut container = ModuleContainer::decode_from_components(&stack.components);
            let (t, n) = *installed_list(&container).get(row)?;
            let cfg = *configs_of(t, n).get((id - CONFIG_BASE) as usize)?;
            if press(&mut container, t, cfg, false) {
                let written = ItemStack { components: container.encode_to_components(&stack.components), ..stack.clone() };
                let _ = player_items::compare_and_set_slot(&player.id, GEAR[view.slot], Some(&stack), Some(&written));
            }
        }
        None
    }

    /// The icons cannot be taken: the screen puts them back (`mayPickup` is false).
    pub(crate) fn module_tweaker_taken(&mut self, player: &Player) -> Option<ItemMenuUpdate> {
        let inv = player_items::get_inventory(&player.id)?;
        let items: Vec<Option<ItemStack>> = (0..GEAR.len()).map(|i| gear(&inv, i)).collect();
        Some(ItemMenuUpdate { items: Some(items), menu_data: None, menu_text: None, close: false, menu_grid: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suit(modules: &[(ModuleType, u8)], item: &str) -> ItemStack {
        let mut c = ModuleContainer::new();
        for (t, n) in modules {
            c.add_module(item, *t, *n);
        }
        ItemStack { item: item.to_owned(), count: 1, components: c.encode_to_components("") }
    }

    #[test]
    fn the_settings_follow_the_module() {
        assert_eq!(configs_of(ModuleType::EnergyUnit, 1), vec![]);
        assert_eq!(configs_of(ModuleType::JetpackUnit, 1), vec![Cfg::Enabled, Cfg::HandleMode, Cfg::RenderHud, Cfg::JetpackMode, Cfg::Thrust, Cfg::HoverThrust]);
        assert!(!configs_of(ModuleType::HydrostaticRepulsorUnit, 3).contains(&Cfg::Swim));
        assert!(configs_of(ModuleType::HydrostaticRepulsorUnit, 4).contains(&Cfg::Swim));
    }

    #[test]
    fn a_click_steps_an_enum_within_the_installed_units_and_wraps() {
        let mut c = ModuleContainer::new();
        c.add_module("mekanism:mekasuit_pants", ModuleType::LocomotiveBoostingUnit, 1);
        // One unit: OFF and LOW (installed + 1), starting at LOW.
        assert_eq!(value_text(c.get(ModuleType::LocomotiveBoostingUnit).unwrap(), Cfg::Sprint), "0.05");
        assert!(press(&mut c, ModuleType::LocomotiveBoostingUnit, Cfg::Sprint, false));
        assert_eq!(value_text(c.get(ModuleType::LocomotiveBoostingUnit).unwrap(), Cfg::Sprint), "0");
        assert!(press(&mut c, ModuleType::LocomotiveBoostingUnit, Cfg::Sprint, false));
        assert_eq!(value_text(c.get(ModuleType::LocomotiveBoostingUnit).unwrap(), Cfg::Sprint), "0.05");
        assert!(press(&mut c, ModuleType::LocomotiveBoostingUnit, Cfg::Sprint, true));
        assert_eq!(c.get(ModuleType::LocomotiveBoostingUnit).unwrap().sprint_boost, SprintBoost::Off);
        // Three units: the back step wraps to the highest choice (installed + 1 = 4 of them).
        c.add_module("mekanism:mekasuit_pants", ModuleType::LocomotiveBoostingUnit, 2);
        assert!(press(&mut c, ModuleType::LocomotiveBoostingUnit, Cfg::Sprint, true));
        assert_eq!(c.get(ModuleType::LocomotiveBoostingUnit).unwrap().sprint_boost, SprintBoost::High);
    }

    #[test]
    fn the_choices_survive_the_item() {
        let mut c = ModuleContainer::new();
        c.add_module("mekanism:mekasuit_bodyarmor", ModuleType::JetpackUnit, 2);
        press(&mut c, ModuleType::JetpackUnit, Cfg::JetpackMode, false);
        press(&mut c, ModuleType::JetpackUnit, Cfg::Thrust, false);
        press(&mut c, ModuleType::JetpackUnit, Cfg::RenderHud, false);
        let back = ModuleContainer::decode_from_components(&c.encode_to_components(""));
        let m = back.get(ModuleType::JetpackUnit).unwrap();
        assert_eq!(m.jetpack_mode, JetpackMode::Hover);
        assert_eq!(m.thrust_multiplier, ThrustMultiplier::Fast);
        assert!(!m.render_hud);
        assert_eq!(m.installed, 2);
    }

    #[test]
    fn meka_tool_settings_are_kept_too() {
        let mut c = ModuleContainer::new();
        c.add_module("mekanism:meka_tool", ModuleType::AttackAmplificationUnit, 4);
        press(&mut c, ModuleType::AttackAmplificationUnit, Cfg::Attack, false);
        press(&mut c, ModuleType::AttackAmplificationUnit, Cfg::Attack, false);
        let back = ModuleContainer::decode_from_components(&c.encode_to_components(""));
        assert_eq!(back.get(ModuleType::AttackAmplificationUnit).unwrap().attack_damage, AttackDamage::Extreme);
    }

    #[test]
    fn only_one_module_handles_the_mode_key() {
        let mut c = ModuleContainer::new();
        c.add_module("mekanism:meka_tool", ModuleType::ExcavationEscalationUnit, 1);
        c.add_module("mekanism:meka_tool", ModuleType::BlastingUnit, 1);
        for t in [ModuleType::ExcavationEscalationUnit, ModuleType::BlastingUnit] {
            c.get_mut(t).unwrap().handle_mode_change = false;
        }
        assert!(press(&mut c, ModuleType::BlastingUnit, Cfg::HandleMode, false));
        assert!(c.get(ModuleType::BlastingUnit).unwrap().handle_mode_change);
        assert!(press(&mut c, ModuleType::ExcavationEscalationUnit, Cfg::HandleMode, false));
        assert!(!c.get(ModuleType::BlastingUnit).unwrap().handle_mode_change);
    }

    #[test]
    fn the_screen_lists_the_modules_and_the_settings_of_the_picked_one() {
        let mut inv: Vec<Option<ItemStack>> = vec![None; 41];
        inv[39] = Some(suit(&[(ModuleType::EnergyUnit, 3), (ModuleType::VisionEnhancementUnit, 1)], "mekanism:mekasuit_helmet"));
        inv[2] = Some(ItemStack { item: "minecraft:stone".into(), count: 1, components: String::new() });
        assert!(has_tweakable(&inv));
        let mut view = View { slot: 0, module: Some(1) };
        let (t, d) = screen(&inv, &mut view);
        assert_eq!(t[0], "Energy Unit (3/8)");
        assert_eq!(t[1], "Vision Enhancement Unit (1/4)");
        assert_eq!((t[16].as_str(), t[17].as_str()), ("Enabled", "Off"));
        assert_eq!(&d[..3], &[0, 1, 2]);
        assert_eq!(d[3], 1, "the helmet is tweakable");
        assert_eq!(d[3 + 6], 0, "the stone is not");
        assert!(!has_tweakable(&vec![None; 41]));
    }
}
