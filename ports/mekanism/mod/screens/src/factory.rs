//! `GuiFactory` (`java/main/java/mekanism/client/gui/machine/GuiFactory.java:25-82`):
//! The unified factory screen supporting all 4 tiers (Basic, Advanced, Elite, Ultimate) and 9 machine types
//! (Smelting, Enriching, Crushing, Compressing, Combining, Purifying, Injecting, Infusing, Sawing),
//! totaling 36 menu variants (`<tier>_<tipo>_factory`).
//!
//! # Layout and Elements:
//! - Base panel: 176x166, 176x177 (chemical), 176x187 (sawing), or 210 width for Ultimate tier (`GuiFactory.java:32-44`).
//! - Player inventory slots and label: `inventoryLabelX = 26` for Ultimate, 8 otherwise.
//!   `inventoryLabelY = 85` (chemical), 95 (sawing), 75 (others) (`GuiFactory.java:34, 37, 39, 43`).
//! - Energy power slot at (6, 12).
//! - Extra item slot at (6, 56) for combining (form 1) and chemical factories (form >= 3), with `slot/overlay/plus` for chemical.
//! - Machine process lines (3 for basic, 5 for advanced, 7 for elite, 9 for ultimate) with:
//!   - Input slot at (x - 1, 12)
//!   - Output slot at (x - 1, 56)
//!   - Secondary output slot at (x - 1, 76) for Sawing factories (`GuiFactory.java:35`)
//!   - Progress arrow (down) at (4 + x, 33) (`GuiFactory.java:69`)
//! - Power bar at (width - 12, 16): height 73 for Sawing, 52 otherwise (`GuiFactory.java:53`).
//! - Secondary chemical resource bar at (7, 76, width 138 / 172, height 4) and Dump button at (148 / 182, 76, 21x10)
//!   for chemical factories (`GuiFactory.java:58-62`, `GuiDumpButton.java:20`, `GuiChemicalBar.java:22`).
//! - Auto-sort tab at (-26, 62, 26, 35) with `button/sorting` icon and centered "On"/"Off" status text (`GuiSortingTab.java:21-28`).
//! - Machine chrome: Left tabs (Side Config, Transporter Config, Warning, Energy) and Right tabs (Upgrades, Security, Redstone),
//!   plus modal window overlays for Side Configuration, Transporter Configuration, and Upgrades.
//!
//! # Differences from Java:
//! - In Java, the sorting tab is a `GuiInsetElement` of height 35 with scrolling text. Here it renders the static
//!   "On" or "Off" text centered inside the tab body according to data slot state.
//! - Chemical colors and names are mapped locally from the standard Mekanism registry definitions.
//!
//! // NÃO FEITO: aviso individual por linha de processo (WarningType::INPUT_DOESNT_PRODUCE_OUTPUT por linha no GuiFactory.java:72) pois o servidor envia apenas o status global da máquina no slot de dados 5.
//! // NÃO FEITO: interação com Gauge Dropper na barra de químicos (GuiTankBar.java:93-109) pois o SDK de telas não expõe o item na mão/cursor nem PacketDropperUse.
//! // NÃO FEITO: integração com Recipe Viewer / JEI nas setas de progresso (GuiProgress.java:104).

use crate::chrome::{colors, Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, inside, slot_frame, Overlay, TITLE, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

/// All 36 factory menu names supported by `GuiFactory`.
pub const ALL_MENUS: [&str; 36] = [
    "mekanism:basic_smelting_factory",
    "mekanism:basic_enriching_factory",
    "mekanism:basic_crushing_factory",
    "mekanism:basic_compressing_factory",
    "mekanism:basic_combining_factory",
    "mekanism:basic_purifying_factory",
    "mekanism:basic_injecting_factory",
    "mekanism:basic_infusing_factory",
    "mekanism:basic_sawing_factory",
    "mekanism:advanced_smelting_factory",
    "mekanism:advanced_enriching_factory",
    "mekanism:advanced_crushing_factory",
    "mekanism:advanced_compressing_factory",
    "mekanism:advanced_combining_factory",
    "mekanism:advanced_purifying_factory",
    "mekanism:advanced_injecting_factory",
    "mekanism:advanced_infusing_factory",
    "mekanism:advanced_sawing_factory",
    "mekanism:elite_smelting_factory",
    "mekanism:elite_enriching_factory",
    "mekanism:elite_crushing_factory",
    "mekanism:elite_compressing_factory",
    "mekanism:elite_combining_factory",
    "mekanism:elite_purifying_factory",
    "mekanism:elite_injecting_factory",
    "mekanism:elite_infusing_factory",
    "mekanism:elite_sawing_factory",
    "mekanism:ultimate_smelting_factory",
    "mekanism:ultimate_enriching_factory",
    "mekanism:ultimate_crushing_factory",
    "mekanism:ultimate_compressing_factory",
    "mekanism:ultimate_combining_factory",
    "mekanism:ultimate_purifying_factory",
    "mekanism:ultimate_injecting_factory",
    "mekanism:ultimate_infusing_factory",
    "mekanism:ultimate_sawing_factory",
];

/// The chemicals supported in Mekanism tanks, in order of their screen index (from 1).
pub const CHEMICALS: [&str; 72] = [
    "mekanism:bio",
    "mekanism:carbon",
    "mekanism:diamond",
    "mekanism:oxygen",
    "mekanism:fungi",
    "mekanism:gold",
    "mekanism:osmium",
    "mekanism:redstone",
    "mekanism:refined_obsidian",
    "mekanism:hydrogen_chloride",
    "mekanism:sulfuric_acid",
    "mekanism:tin",
    "mekanism:hydrogen",
    "mekanism:chlorine",
    "mekanism:sulfur_dioxide",
    "mekanism:sulfur_trioxide",
    "mekanism:water_vapor",
    "mekanism:cyan",
    "mekanism:gray",
    "mekanism:green",
    "mekanism:brown",
    "mekanism:light_blue",
    "mekanism:purple",
    "mekanism:white",
    "mekanism:light_gray",
    "mekanism:magenta",
    "mekanism:orange",
    "mekanism:pink",
    "mekanism:red",
    "mekanism:yellow",
    "mekanism:black",
    "mekanism:blue",
    "mekanism:lime",
    "mekanism:lithium",
    "mekanism:brine",
    "mekanism:antimatter",
    "mekanism:aqua",
    "mekanism:dark_red",
    "mekanism:clean_copper",
    "mekanism:dirty_copper",
    "mekanism:clean_gold",
    "mekanism:dirty_gold",
    "mekanism:clean_iron",
    "mekanism:dirty_iron",
    "mekanism:plutonium",
    "mekanism:spent_nuclear_waste",
    "mekanism:polonium",
    "mekanism:clean_lead",
    "mekanism:dirty_lead",
    "mekanism:clean_osmium",
    "mekanism:dirty_osmium",
    "mekanism:clean_tin",
    "mekanism:dirty_tin",
    "mekanism:clean_uranium",
    "mekanism:hydrofluoric_acid",
    "mekanism:fissile_fuel",
    "mekanism:dirty_uranium",
    "mekanism:uranium_oxide",
    "mekanism:uranium_hexafluoride",
    "mekanism:ethene",
    "mekanism:sodium",
    "mekanism:steam",
    "mekanism:superheated_sodium",
    "minecraft:water",
    "minecraft:lava",
    "mekanismgenerators:bioethanol",
    "mekanism:heavy_water",
    "mekanism:nutritional_paste",
    "mekanism:nuclear_waste",
    "mekanism:deuterium",
    "mekanism:tritium",
    "mekanism:fusion_fuel",
];

/// Resolves RGB color for a chemical from its identifier.
pub fn chemical_color(id: &str) -> u32 {
    let name = id.strip_prefix("mekanism:").unwrap_or(id);
    match name {
        "bio" => 0x5A4630,
        "fungi" => 0x74656A,
        "tin" => 0xCCCCD9,
        "gold" => 0xF2CD67,
        "refined_obsidian" => 0x7C00ED,
        "diamond" => 0x6CEDD8,
        "redstone" => 0xB30505,
        "carbon" => 0x2C2C2C,
        "hydrogen" => 0xFFFFFF,
        "oxygen" => 0x6CE2FF,
        "steam" | "water_vapor" => 0x6CE2FF,
        "chlorine" => 0xCFE800,
        "sulfur_dioxide" => 0xA99D90,
        "sulfur_trioxide" => 0xCE6C6C,
        "sulfuric_acid" => 0x82802B,
        "hydrogen_chloride" => 0xA8F1E9,
        "hydrofluoric_acid" => 0xC6C7BD,
        "uranium_oxide" => 0xE1F573,
        "uranium_hexafluoride" => 0x809960,
        "ethene" => 0xEACCF9,
        "sodium" => 0xE9FEF4,
        "superheated_sodium" => 0xD19469,
        "brine" => 0xFEEF9C,
        "lithium" => 0xEBA400,
        "osmium" => 0x52BDCA,
        "fissile_fuel" => 0x2E332F,
        "nuclear_waste" => 0x4F412A,
        "spent_nuclear_waste" => 0x262015,
        "plutonium" => 0x1F919C,
        "polonium" => 0x1B9E7B,
        "antimatter" => 0xA464B3,
        "cyan" => 0x2D9CDB,
        "gray" => 0x4F4F4F,
        "green" => 0x27AE60,
        "brown" => 0x8D6E63,
        "light_blue" | "aqua" => 0x56CCF2,
        "purple" => 0x9B51E0,
        "white" => 0xFFFFFF,
        "light_gray" => 0xBDBDBD,
        "magenta" => 0xBB6BD9,
        "orange" => 0xF2994A,
        "pink" => 0xF48FB1,
        "red" | "dark_red" => 0xEB5757,
        "yellow" => 0xF2C94C,
        "black" => 0x1A1A1A,
        "blue" => 0x2F80ED,
        "lime" => 0x6FCF97,
        "clean_copper" => 0xB87333,
        "dirty_copper" => 0x8B4513,
        "clean_gold" => 0xF2CD67,
        "dirty_gold" => 0xD4AF37,
        "clean_iron" => 0xD8D8D8,
        "dirty_iron" => 0x808080,
        "clean_lead" => 0x708090,
        "dirty_lead" => 0x4F4F4F,
        "clean_osmium" => 0x52BDCA,
        "dirty_osmium" => 0x4682B4,
        "clean_tin" => 0xCCCCD9,
        "dirty_tin" => 0x808080,
        "clean_uranium" => 0x7FFF00,
        "dirty_uranium" => 0x556B2F,
        "minecraft:water" => 0x3F76E4,
        "minecraft:lava" => 0xFFA500,
        _ => 0xFFFFFF,
    }
}

/// Chemical color by 1-based index from menu data.
pub fn chemical_color_by_index(index: i32) -> u32 {
    if index <= 0 || index as usize > CHEMICALS.len() {
        return 0xFFFF_FFFF;
    }
    chemical_color(CHEMICALS[index as usize - 1])
}

/// Chemical display name by 1-based index from menu data.
pub fn chemical_name_by_index(index: i32) -> String {
    if index <= 0 || index as usize > CHEMICALS.len() {
        return "Unknown".to_string();
    }
    let id = CHEMICALS[index as usize - 1];
    let name = id.split(':').nth(1).unwrap_or(id);
    let trans = host::translate(&format!("chemical.mekanism.{name}"), &[]);
    if !trans.is_empty() && !trans.starts_with("chemical.") {
        return trans;
    }
    name.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The 4 factory tiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactoryTier {
    Basic,
    Advanced,
    Elite,
    Ultimate,
}

impl FactoryTier {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "basic" => Some(Self::Basic),
            "advanced" => Some(Self::Advanced),
            "elite" => Some(Self::Elite),
            "ultimate" => Some(Self::Ultimate),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Advanced => "advanced",
            Self::Elite => "elite",
            Self::Ultimate => "ultimate",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Advanced => "Advanced",
            Self::Elite => "Elite",
            Self::Ultimate => "Ultimate",
        }
    }

    pub fn processes(self) -> usize {
        match self {
            Self::Basic => 3,
            Self::Advanced => 5,
            Self::Elite => 7,
            Self::Ultimate => 9,
        }
    }

    pub fn base_x(self) -> f32 {
        match self {
            Self::Basic => 55.0,
            Self::Advanced => 35.0,
            Self::Elite => 29.0,
            Self::Ultimate => 27.0,
        }
    }

    pub fn mult(self) -> f32 {
        match self {
            Self::Basic => 38.0,
            Self::Advanced => 26.0,
            Self::Elite => 19.0,
            Self::Ultimate => 19.0,
        }
    }

    pub fn width(self) -> f32 {
        match self {
            Self::Ultimate => 210.0,
            _ => 176.0,
        }
    }

    pub fn inventory_x(self) -> f32 {
        match self {
            Self::Ultimate => 26.0,
            _ => 8.0,
        }
    }
}

/// The 9 factory process kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactoryKind {
    Smelting,
    Enriching,
    Crushing,
    Compressing,
    Combining,
    Purifying,
    Injecting,
    Infusing,
    Sawing,
}

impl FactoryKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "smelting" => Some(Self::Smelting),
            "enriching" => Some(Self::Enriching),
            "crushing" => Some(Self::Crushing),
            "compressing" => Some(Self::Compressing),
            "combining" => Some(Self::Combining),
            "purifying" => Some(Self::Purifying),
            "injecting" => Some(Self::Injecting),
            "infusing" => Some(Self::Infusing),
            "sawing" => Some(Self::Sawing),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Smelting => "smelting",
            Self::Enriching => "enriching",
            Self::Crushing => "crushing",
            Self::Compressing => "compressing",
            Self::Combining => "combining",
            Self::Purifying => "purifying",
            Self::Injecting => "injecting",
            Self::Infusing => "infusing",
            Self::Sawing => "sawing",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Smelting => "Smelting",
            Self::Enriching => "Enriching",
            Self::Crushing => "Crushing",
            Self::Compressing => "Compressing",
            Self::Combining => "Combining",
            Self::Purifying => "Purifying",
            Self::Injecting => "Injecting",
            Self::Infusing => "Infusing",
            Self::Sawing => "Sawing",
        }
    }

    pub fn form(self) -> u8 {
        match self {
            Self::Crushing | Self::Enriching | Self::Smelting => 0,
            Self::Combining => 1,
            Self::Sawing => 2,
            Self::Injecting | Self::Purifying => 3,
            Self::Compressing | Self::Infusing => 4,
        }
    }

    pub fn has_extra_slot(self) -> bool {
        self.form() == 1 || self.form() >= 3
    }

    pub fn has_secondary_bar(self) -> bool {
        self.form() >= 3
    }

    pub fn has_secondary_output(self) -> bool {
        self.form() == 2
    }

    pub fn height(self) -> f32 {
        if self.has_secondary_output() {
            187.0
        } else if self.has_secondary_bar() {
            177.0
        } else {
            166.0
        }
    }

    pub fn inventory_y(self) -> f32 {
        if self.has_secondary_output() {
            105.0
        } else if self.has_secondary_bar() {
            95.0
        } else {
            85.0
        }
    }

    pub fn inventory_label_y(self) -> f32 {
        if self.has_secondary_output() {
            95.0
        } else if self.has_secondary_bar() {
            85.0
        } else {
            75.0
        }
    }

    pub fn tank_unit_capacity(self) -> u64 {
        match self {
            Self::Infusing => 1000,
            _ => 210,
        }
    }

    pub fn supported_upgrades(self) -> Vec<usize> {
        match self {
            Self::Purifying | Self::Injecting => vec![0, 1, 2, 3], // Speed, Energy, Chemical, Muffling
            _ => vec![0, 1, 3],                                    // Speed, Energy, Muffling
        }
    }
}

/// Factory screen structure implementing `MekScreen`.
pub struct FactoryScreen {
    pub tier: FactoryTier,
    pub kind: FactoryKind,
    pub chrome: Chrome,
}

impl FactoryScreen {
    fn is_modal_open(&self) -> bool {
        self.chrome.upgrade_window_open
            || self.chrome.spec.side_config.as_ref().is_some_and(|w| w.is_open())
            || self.chrome.spec.transporter.as_ref().is_some_and(|w| w.is_open())
    }
}

/// Opens any of the 36 factory menus by dispatching according to `info.menu`.
pub fn open(info: &OpenInfo) -> Opened {
    let menu_static = ALL_MENUS
        .iter()
        .copied()
        .find(|&m| m == info.menu)
        .ok_or_else(|| format!("unknown factory menu: {}", info.menu))?;

    let stripped = info.menu.strip_prefix("mekanism:").unwrap_or(&info.menu);
    let parts: Vec<&str> = stripped.split('_').collect();
    if parts.len() != 3 || parts[2] != "factory" {
        return Err(format!("invalid factory menu name: {}", info.menu));
    }
    let tier = FactoryTier::parse(parts[0]).ok_or_else(|| format!("unknown tier: {}", parts[0]))?;
    let kind = FactoryKind::parse(parts[1]).ok_or_else(|| format!("unknown kind: {}", parts[1]))?;

    let width = tier.width();
    let height = kind.height();
    let lines = tier.processes();

    let base_slots = lines * (if kind.has_secondary_output() { 3 } else { 2 })
        + (if kind.has_extra_slot() { 1 } else { 0 })
        + 1;
    let input_slot = base_slots as u32;
    let output_slot = (base_slots + 1) as u32;

    let spec = ChromeSpec {
        menu: menu_static,
        redstone: Some(16),
        energy: Some(EnergyTabSpec::for_machine(16, Some(5))),
        upgrades: Some(UpgradesSpec::new(input_slot, output_slot, 16, &kind.supported_upgrades())),
        warning: Some(5),
        side_config: Some(Box::new(SideConfigWindow::for_machine(menu_static, width as i32, height as i32))),
        transporter: TransporterWindow::for_machine(menu_static, width as i32, height as i32).map(|w| Box::new(w) as Box<dyn Overlay>),
    };

    let chrome = Chrome::new(spec, width);
    let screen = FactoryScreen {
        tier,
        kind,
        chrome,
    };

    Ok((
        Box::new(screen),
        Setup {
            width: width as u32,
            height: height as u32,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for FactoryScreen {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn close(&mut self) {
        self.chrome.close_all_windows();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        let width = self.tier.width();
        let height = self.kind.height();
        let lines = self.tier.processes();
        let inv_x = self.tier.inventory_x();
        let inv_y = self.kind.inventory_y();
        let is_modal = self.is_modal_open();

        // 1. Base panel background
        base_panel(c, width, height);

        // 2. Player inventory slots (3 rows of 9, and 1 hotbar row)
        for r in 0..3 {
            for col in 0..9 {
                slot_frame(c, inv_x - 1.0 + 18.0 * col as f32, inv_y - 1.0 + 18.0 * r as f32);
            }
        }
        for col in 0..9 {
            slot_frame(c, inv_x - 1.0 + 18.0 * col as f32, inv_y + 57.0);
        }

        // 3. Power slot at (6, 12)
        c.blit_sprite("mekanism:slot/power", 6.0, 12.0, 18.0, 18.0);

        // 4. Extra input slot at (6, 56) (if combining or chemical factory)
        if self.kind.has_extra_slot() {
            c.blit_sprite("mekanism:slot/extra", 6.0, 56.0, 18.0, 18.0);
            if self.kind.has_secondary_bar() {
                c.blit_sprite("mekanism:slot/overlay/plus", 6.0, 56.0, 18.0, 18.0);
            }
        }

        // 5. Machine process lines: input, output, secondary output (sawing), progress arrow
        let base_x = self.tier.base_x();
        let mult = self.tier.mult();
        let max_ticks = crate::elements::data(19).max(1) as f32;

        for i in 0..lines {
            let x = base_x + i as f32 * mult;
            // Input slot
            c.blit_sprite("mekanism:slot/input", x - 1.0, 12.0, 18.0, 18.0);
            // Output slot
            c.blit_sprite("mekanism:slot/output", x - 1.0, 56.0, 18.0, 18.0);
            // Sawing secondary output slot
            if self.kind.has_secondary_output() {
                c.blit_sprite("mekanism:slot/output", x - 1.0, 76.0, 18.0, 18.0);
            }
            // Progress arrow (down)
            c.blit_sprite("mekanism:progress/down/empty", 4.0 + x, 33.0, 8.0, 20.0);
            let progress = crate::elements::data(32 + i).max(0) as f32;
            if progress > 0.0 {
                let fill_h = (20.0 * (progress / max_ticks).clamp(0.0, 1.0)).floor();
                if fill_h > 0.0 {
                    c.blit_sprite("mekanism:progress/down/full", 4.0 + x, 33.0, 8.0, fill_h);
                }
            }
        }

        // 6. Vertical power bar at (width - 12, 16)
        let bar_h = if self.kind.has_secondary_output() { 73.0 } else { 52.0 };
        let bar_x = width - 12.0;
        let bar_y = 16.0;
        c.blit_sprite("mekanism:bar/base", bar_x, bar_y, 6.0, bar_h + 2.0);

        let energy = crate::elements::pair(1, 2);
        let capacity = crate::elements::pair(3, 4);
        let frac = if capacity > 0 { (energy as f32 / capacity as f32).clamp(0.0, 1.0) } else { 0.0 };
        let filled = (bar_h * frac).floor();
        if filled > 0.0 {
            if bar_h == 52.0 {
                c.blit("mekanism:gui/bar/vertical_power", bar_x + 1.0, bar_y + 1.0 + 52.0 - filled, 0.0, 52.0 - filled, 4.0, filled, 4.0, 52.0);
            } else {
                c.blit_region(
                    "mekanism:gui/bar/vertical_power",
                    [bar_x + 1.0, bar_y + 1.0 + bar_h - filled, 4.0, filled],
                    [0.0, 52.0 - (52.0 * frac).floor(), 4.0, (52.0 * frac).floor()],
                    [4.0, 52.0],
                    WHITE,
                );
            }
        }
        if !is_modal && inside(mouse, [bar_x, bar_y, 6.0, bar_h + 2.0]) {
            let energy_tooltip = format!("Energy: {} / {} FE", energy, capacity);
            c.tooltip(&[(&energy_tooltip, WHITE)]);
        }

        // 7. Secondary Chemical Resource Bar & Dump Button (if compressing, injecting, purifying, infusing)
        if self.kind.has_secondary_bar() {
            let chem_bar_w = if self.tier == FactoryTier::Ultimate { 172.0 } else { 138.0 };
            c.blit_sprite("mekanism:bar/base", 7.0, 76.0, chem_bar_w + 2.0, 6.0);

            let amount_slot = 32 + lines;
            let chem_slot = 33 + lines;
            let amount = crate::elements::data(amount_slot).max(0) as u64;
            let chem_idx = crate::elements::data(chem_slot);
            let max_cap = self.kind.tank_unit_capacity() * lines as u64;

            if amount > 0 {
                let fill_w = (chem_bar_w * (amount as f32 / max_cap as f32).clamp(0.0, 1.0)).floor();
                if fill_w > 0.0 {
                    let color = chemical_color_by_index(chem_idx);
                    c.fill(8.0, 77.0, fill_w, 4.0, 0xFF00_0000 | color);
                }
            }

            if !is_modal && inside(mouse, [7.0, 76.0, chem_bar_w + 2.0, 6.0]) {
                if amount == 0 {
                    let empty_str = host::translate("gui.mekanism.empty", &[]);
                    let text = if empty_str.is_empty() || empty_str.starts_with("gui.") {
                        "Empty"
                    } else {
                        &empty_str
                    };
                    c.tooltip(&[(text, WHITE)]);
                } else {
                    let name = chemical_name_by_index(chem_idx);
                    let chem_tooltip = format!("{}: {} mB", name, amount);
                    c.tooltip(&[(&chem_tooltip, WHITE)]);
                }
            }

            // Dump button
            let dump_x = if self.tier == FactoryTier::Ultimate { 182.0 } else { 148.0 };
            let dump_rect = [dump_x, 76.0, 21.0, 10.0];
            let dump_hovered = !is_modal && inside(mouse, dump_rect);
            c.blit_sprite("mekanism:button/dump", dump_rect[0], dump_rect[1], dump_rect[2], dump_rect[3]);
            c.area(4000, dump_rect, AreaKind::Button, true, "Dump");
            if dump_hovered {
                let dump_str = host::translate("gui.mekanism.dump", &[]);
                let text = if dump_str.is_empty() || dump_str.starts_with("gui.") {
                    "Dump"
                } else {
                    &dump_str
                };
                c.tooltip(&[(text, WHITE)]);
            }
        }

        // 8. Title and Inventory Label
        let title_key = format!("block.mekanism.{}_{}_factory", self.tier.name(), self.kind.name());
        let title_trans = host::translate(&title_key, &[]);
        let title_text = if !title_trans.is_empty() && !title_trans.starts_with("block.") {
            title_trans
        } else {
            format!("{} {} Factory", self.tier.display_name(), self.kind.display_name())
        };
        crate::elements::title(c, &title_text, 0.0, width, 4.0);
        crate::elements::inventory_label(c, inv_x, self.kind.inventory_label_y());

        // 9. Auto-sort Tab at (-26, 62, 26, 35)
        let sort_rect = [-26.0, 62.0, 26.0, 35.0];
        c.blit_sprite_tinted("mekanism:holder_left", sort_rect, colors::TAB_CONFIGURATION);
        c.blit_sprite("mekanism:button/sorting", -22.0, 65.0, 18.0, 18.0);

        let sort_slot = 32 + lines + if self.kind.has_secondary_bar() { 2 } else { 0 };
        let is_sorting = crate::elements::data(sort_slot) != 0;
        let sort_label = if is_sorting { "On" } else { "Off" };
        c.text(sort_label, -13.0, 86.0, TITLE, horizon_mod_sdk::screen::TEXT_CENTER, 1.0);
        c.area(4100, sort_rect, AreaKind::Tab, true, "Auto-sort");
        if !is_modal && inside(mouse, sort_rect) {
            let sort_str = host::translate("auto.sort", &[]);
            let text = if sort_str.is_empty() || sort_str.starts_with("auto.") {
                "Auto-sort"
            } else {
                &sort_str
            };
            c.tooltip(&[(text, WHITE)]);
        }

        // 10. Chrome: tabs around screen and modal window overlays
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }

        let is_modal = self.is_modal_open();
        if is_modal {
            return false;
        }

        match event {
            InputEvent::MousePressed(p) => {
                let mouse = (p.x, p.y);
                if self.kind.has_secondary_bar() {
                    let dump_x = if self.tier == FactoryTier::Ultimate { 182.0 } else { 148.0 };
                    let dump_rect = [dump_x, 76.0, 21.0, 10.0];
                    if inside(mouse, dump_rect) {
                        host::send_button(4000);
                        host::play_click();
                        return true;
                    }
                }
                let sort_rect = [-26.0, 62.0, 26.0, 35.0];
                if inside(mouse, sort_rect) {
                    host::send_button(4100);
                    host::play_click();
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::{decode, Command};
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};

    fn make_open_info(menu: &str) -> OpenInfo {
        OpenInfo {
            menu: menu.to_string(),
            gui_width: 0.0,
            gui_height: 0.0,
            touch: false,
        }
    }

    #[test]
    fn test_all_36_factories_open_successfully() {
        for menu in ALL_MENUS {
            let (_screen, setup) = open(&make_open_info(menu))
                .unwrap_or_else(|e| panic!("failed to open {}: {}", menu, e));

            assert!(setup.own_labels);
            assert!(!setup.hide_inventory);

            let stripped = menu.strip_prefix("mekanism:").unwrap();
            let is_ultimate = stripped.starts_with("ultimate");
            let is_sawing = stripped.contains("sawing");
            let is_chem = stripped.contains("compressing")
                || stripped.contains("infusing")
                || stripped.contains("injecting")
                || stripped.contains("purifying");

            let expected_width = if is_ultimate { 210 } else { 176 };
            let expected_height = if is_sawing {
                187
            } else if is_chem {
                177
            } else {
                166
            };

            assert_eq!(setup.width, expected_width, "width mismatch for {}", menu);
            assert_eq!(setup.height, expected_height, "height mismatch for {}", menu);
        }
    }

    #[test]
    fn test_basic_smelting_factory_render_and_auto_sort_click() {
        let _ = host::take_sent();
        let mut data = vec![0; 45];
        // Energy: 500 FE, capacity: 1000 FE
        data[1] = 500;
        data[2] = 0;
        data[3] = 1000;
        data[4] = 0;
        data[5] = 1; // status active
        data[16] = 1; // redstone control mode HIGH
        data[19] = 200; // ticks required
        data[32] = 50; // line 0 progress
        data[33] = 100; // line 1 progress
        data[34] = 150; // line 2 progress
        data[35] = 0; // sort_at = 32 + 3 = 35: Off

        host::set_menu(host::Menu {
            data,
            strings: vec![],
            grid: vec![],
            slots: vec![],
            carried: None,
        });

        let (mut screen, setup) = open(&make_open_info("mekanism:basic_smelting_factory"))
            .expect("open basic smelting factory");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame {
            mouse_x: 10.0,
            mouse_y: 10.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = decode(&bytes).expect("decode canvas");

        // Check base panel sprite
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        // Check sorting button sprite
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:button/sorting")));
        // Check sorting Off text
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Off")));
        // Check title
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Basic Smelting Factory")));

        // Click auto-sort tab at (-26, 62, 26, 35)
        let sort_click = InputEvent::MousePressed(Pointer {
            x: -15.0,
            y: 75.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&sort_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 4100".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_basic_compressing_factory_dump_button_and_chemical_bar() {
        let _ = host::take_sent();
        let mut data = vec![0; 45];
        data[1] = 800; // energy
        data[3] = 1000; // capacity
        data[19] = 200; // ticks
        data[32 + 3] = 150; // chem amount (150 mB)
        data[33 + 3] = 4; // chem index 4 = oxygen
        data[37] = 1; // sort_at = 32 + 3 + 2 = 37: On

        host::set_menu(host::Menu {
            data,
            strings: vec![],
            grid: vec![],
            slots: vec![],
            carried: None,
        });

        let (mut screen, setup) = open(&make_open_info("mekanism:basic_compressing_factory"))
            .expect("open basic compressing factory");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 177);

        let frame = Frame {
            mouse_x: 10.0,
            mouse_y: 10.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = decode(&bytes).expect("decode canvas");

        // Check chemical dump button sprite
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:button/dump")));
        // Check slot overlay plus
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:slot/overlay/plus")));
        // Check sorting On text
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "On")));

        // Click dump button at (148, 76, 21, 10)
        let dump_click = InputEvent::MousePressed(Pointer {
            x: 155.0,
            y: 80.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&dump_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 4000".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_ultimate_sawing_factory_render_elements() {
        let _ = host::take_sent();
        let mut data = vec![0; 50];
        data[1] = 5000;
        data[3] = 10000;
        data[19] = 200;
        for i in 0..9 {
            data[32 + i] = (i as i32 + 1) * 20;
        }

        host::set_menu(host::Menu {
            data,
            strings: vec![],
            grid: vec![],
            slots: vec![],
            carried: None,
        });

        let (mut screen, setup) = open(&make_open_info("mekanism:ultimate_sawing_factory"))
            .expect("open ultimate sawing factory");

        assert_eq!(setup.width, 210);
        assert_eq!(setup.height, 187);

        let frame = Frame {
            mouse_x: 10.0,
            mouse_y: 10.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = decode(&bytes).expect("decode canvas");

        // Verify title
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Ultimate Sawing Factory")));

        // Count output slots (9 primary + 9 secondary = 18 output slot blits)
        let output_slots_count = decoded.commands.iter().filter(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:slot/output")).count();
        assert_eq!(output_slots_count, 18);

        // Count empty progress arrows (9 lines)
        let progress_arrows_count = decoded.commands.iter().filter(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:progress/down/empty")).count();
        assert_eq!(progress_arrows_count, 9);
    }

    #[test]
    fn test_factory_chrome_tabs_and_windows() {
        let _ = host::take_sent();
        let mut data = vec![0; 45];
        data[16] = 2; // redstone control mode LOW
        // security mode at slot 38 for basic_crushing_factory
        data[38] = 1; // private

        host::set_menu(host::Menu {
            data,
            strings: vec![],
            grid: vec![],
            slots: vec![],
            carried: None,
        });

        let (mut screen, _) = open(&make_open_info("mekanism:basic_crushing_factory"))
            .expect("open basic crushing factory");

        // 1. Open upgrades window by clicking upgrade tab at (176, 6, 26, 18)
        let upgrade_tab_click = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 12.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&upgrade_tab_click));

        // Render while upgrade window is open
        let frame = Frame {
            mouse_x: 0.0,
            mouse_y: 0.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Upgrades")));

        // Press Escape to close upgrade window
        let esc_key = InputEvent::KeyPressed(horizon_mod_sdk::screen::Key {
            key: "key.keyboard.escape".into(),
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&esc_key));

        // Render again: upgrade window should no longer be drawn
        let mut canvas2 = Canvas::new();
        screen.render(&frame, &mut canvas2);
        let bytes2 = canvas2.finish();
        let decoded2 = decode(&bytes2).expect("decode canvas");
        assert!(!decoded2.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Upgrades")));

        let _ = host::take_sent();

        // 2. Click Security tab at (176, 34, 26, 18)
        let sec_tab_click = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&sec_tab_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);

        // 3. Click Redstone tab at (176, 137, 26, 18)
        let redstone_tab_click = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 145.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&redstone_tab_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 6000".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_chemical_name_and_color_resolution() {
        assert_eq!(chemical_color("mekanism:hydrogen"), 0xFFFFFF);
        assert_eq!(chemical_color("mekanism:oxygen"), 0x6CE2FF);
        assert_eq!(chemical_color("mekanism:redstone"), 0xB30505);

        assert_eq!(chemical_color_by_index(1), 0x5A4630); // bio
        assert_eq!(chemical_color_by_index(4), 0x6CE2FF); // oxygen

        assert_eq!(chemical_name_by_index(4), "Oxygen");
        assert_eq!(chemical_name_by_index(1), "Bio");
        assert_eq!(chemical_name_by_index(10), "Hydrogen Chloride");
    }
}
