//! `GuiChemicalOxidizer` (176 by 166): chemical oxidizer.
//! Java reference: `mekanism.client.gui.machine.GuiChemicalOxidizer` (lines 20-48).
//!
//! Layout:
//! - Horizontal power bar at (115, 75), 54 by 6 frame, 52 by 4 fill
//! - Chemical gauge at (131, 13), standard 16 by 58 (18 by 60 frame)
//! - Progress bar at (64, 40), 48 by 8 (ProgressType.LARGE_RIGHT)
//! - Energy tab at (-26, 137)
//! - Title centered at y = 5, inventory label at (8, 72)
//! - Slots: input at (26, 36), extra at (152, 55), power at (152, 14)
//! - Upgrade window: slots 2 (in) and 3 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

#[derive(Clone, Copy, Debug)]
pub struct ChemMeta {
    pub texture: &'static str,
    pub tint: u32,
    pub label: &'static str,
}

pub const CHEMICAL_COUNT: usize = 72;
pub const CHEMICAL_METAS: [ChemMeta; CHEMICAL_COUNT] = [
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_bio", tint: 0xFFFF_FFFF, label: "Bio" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_base", tint: 0xFF2C_2C2C, label: "Carbon" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_base", tint: 0xFF6C_EDD8, label: "Diamond" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF6C_E2FF, label: "Oxygen" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_fungi", tint: 0xFFFF_FFFF, label: "Fungi" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_base", tint: 0xFFF2_CD67, label: "Gold" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF52_BDCA, label: "Osmium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_base", tint: 0xFFB3_0505, label: "Redstone" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_base", tint: 0xFF7C_00ED, label: "Refined Obsidian" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFA8_F1E9, label: "Hydrogen Chloride" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF82_802B, label: "Sulfuric Acid" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_infuse_type_base", tint: 0xFFCC_CCD9, label: "Tin" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Hydrogen" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFCF_E800, label: "Chlorine" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFA9_9D90, label: "Sulfur Dioxide" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFCE_6C6C, label: "Sulfur Trioxide" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_steam", tint: 0xFFFF_FFFF, label: "Water Vapor" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Cyan" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Gray" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Green" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Brown" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Light Blue" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Purple" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "White" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Light Gray" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Magenta" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Orange" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Pink" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Red" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Yellow" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Black" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Blue" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Lime" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFEB_A400, label: "Lithium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFE_EF9C, label: "Brine" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFA4_64B3, label: "Antimatter" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Aqua" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Dark Red" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_clean", tint: 0xFFAA_4B19, label: "Clean Copper" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_dirty", tint: 0xFFAA_4B19, label: "Dirty Copper" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_clean", tint: 0xFFF2_CD67, label: "Clean Gold" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_dirty", tint: 0xFFF2_CD67, label: "Dirty Gold" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_clean", tint: 0xFFAF_8E77, label: "Clean Iron" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_dirty", tint: 0xFFAF_8E77, label: "Dirty Iron" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF1F_919C, label: "Plutonium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF26_2015, label: "Spent Nuclear Waste" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF1B_9E7B, label: "Polonium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_clean", tint: 0xFF3A_404A, label: "Clean Lead" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_dirty", tint: 0xFF3A_404A, label: "Dirty Lead" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_clean", tint: 0xFF1E_79C3, label: "Clean Osmium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_dirty", tint: 0xFF1E_79C3, label: "Dirty Osmium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_clean", tint: 0xFFCC_CCD9, label: "Clean Tin" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_dirty", tint: 0xFFCC_CCD9, label: "Dirty Tin" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_clean", tint: 0xFF46_664F, label: "Clean Uranium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFC6_C7BD, label: "Hydrofluoric Acid" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF2E_332F, label: "Fissile Fuel" },
    ChemMeta { texture: "mekanism:gui/chem_mek_chemical_slurry_dirty", tint: 0xFF46_664F, label: "Dirty Uranium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFE1_F573, label: "Uranium Oxide" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF80_9960, label: "Uranium Hexafluoride" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFEA_CCF9, label: "Ethene" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFE9_FEF4, label: "Sodium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_steam", tint: 0xFFFF_FFFF, label: "Steam" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFD1_9469, label: "Superheated Sodium" },
    ChemMeta { texture: "mekanism:gui/fluid_water", tint: 0xFF3F_76E4, label: "Water" },
    ChemMeta { texture: "mekanism:gui/fluid_lava", tint: 0xFFFF_FFFF, label: "Lava" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFCE_EB3D, label: "Bioethanol" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF0D_1455, label: "Heavy Water" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFEB_6CA3, label: "Nutritional Paste" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFF4F_412A, label: "Nuclear Waste" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Deuterium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Tritium" },
    ChemMeta { texture: "mekanism:gui/chem_mek_liquid_liquid", tint: 0xFFFF_FFFF, label: "Fusion Fuel" },
];

pub fn chemical_meta(which: usize) -> Option<&'static ChemMeta> {
    if which > 0 && which <= CHEMICAL_COUNT {
        Some(&CHEMICAL_METAS[which - 1])
    } else {
        None
    }
}

/// Renders a chemical or fluid gauge (border, fluid content, and overlay).
pub fn render_chemical_gauge(
    c: &mut Canvas,
    mouse: (f32, f32),
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    overlay_sprite: &str,
    amount: u64,
    capacity: u64,
    chem_idx: usize,
) -> Option<Vec<(String, u32)>> {
    c.blit_sprite("mekanism:gauge/normal", x, y, w + 2.0, h + 2.0);

    let meta = chemical_meta(chem_idx);
    if let Some(m) = meta {
        if amount > 0 && capacity > 0 {
            let fill = (amount as f32 / capacity as f32).clamp(0.0, 1.0);
            c.fluid(m.texture, [x + 1.0, y + 1.0, w, h], fill, m.tint);
        }
    }

    c.blit_sprite(overlay_sprite, x + 1.0, y + 1.0, w, h);

    let r = [x, y, w + 2.0, h + 2.0];
    c.area(-1, r, AreaKind::Other, true, "Gauge");
    if inside(mouse, r) {
        if let Some(m) = meta {
            if amount > 0 {
                return Some(vec![(format!("{}: {} mB", m.label, amount), WHITE)]);
            }
        }
        return Some(vec![("Empty".to_string(), WHITE)]);
    }
    None
}

/// Renders a horizontal power bar at (x, y) with 54 by 6 frame and 52 by 4 fill.
pub fn horizontal_power_bar(c: &mut Canvas, x: f32, y: f32, fraction: f32) {
    c.blit_sprite("mekanism:bar/base", x, y, 54.0, 6.0);
    let filled = (52.0 * fraction.clamp(0.0, 1.0)).floor();
    if filled > 0.0 {
        c.blit("mekanism:gui/bar/horizontal_power", x + 1.0, y + 1.0, 0.0, 0.0, filled, 4.0, 52.0, 4.0);
    }
}

pub struct ChemicalOxidizer {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:chemical_oxidizer";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(20),
            energy: Some(EnergyTabSpec::for_machine(20, Some(5))),
            upgrades: Some(UpgradesSpec::new(2, 3, 20, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(ChemicalOxidizer { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for ChemicalOxidizer {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input at (26, 36), extra at (152, 55), power at (152, 14)
        c.blit_sprite("mekanism:slot/input", 25.0, 35.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/extra", 151.0, 54.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 151.0, 54.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 151.0, 13.0, 18.0, 18.0);

        // Horizontal power bar at (115, 75)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        horizontal_power_bar(c, 115.0, 75.0, power_frac);

        let power_rect = [115.0, 75.0, 54.0, 6.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Chemical output gauge at (131, 13), 16 by 58 inside 18 by 60
        let chem_amount = data(18).max(0) as u64;
        let chem_kind = data(19).max(0) as usize;
        let gauge_tip = render_chemical_gauge(c, mouse, 131.0, 13.0, 16.0, 58.0, "mekanism:gauge/overlay/standard", chem_amount, 10000, chem_kind);
        if hovered_tip.is_none() {
            hovered_tip = gauge_tip;
        }

        // Progress bar at (64, 40): 48 by 8 (LARGE_RIGHT)
        c.blit_sprite("mekanism:progress/large_right/empty", 64.0, 40.0, 48.0, 8.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(23).max(1) as f32;
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        if prog_frac > 0.0 {
            let filled_w = (48.0 * prog_frac).floor();
            c.enable_scissor(64.0, 40.0, filled_w, 8.0);
            c.blit_sprite("mekanism:progress/large_right/full", 64.0, 40.0, 48.0, 8.0);
            c.disable_scissor();
        }

        // Title centered at y = 5
        let title_text = host::translate("block.mekanism.chemical_oxidizer", &[]);
        title(c, &title_text, 0.0, 176.0, 5.0);

        // Inventory label at (8, 72)
        inventory_label(c, 8.0, 72.0);

        // Tooltip if hovered
        if let Some(lines) = hovered_tip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }

        // Chrome (tabs and overlays)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};
    use horizon_mod_sdk::screen_format::{self, Command};

    #[test]
    fn test_chemical_oxidizer_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                50, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2500, 4, 0, 4000, 0, 100,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:chemical_oxidizer".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open chemical oxidizer");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let mut c = Canvas::new();
        screen.render(
            &Frame {
                mouse_x: 10.0,
                mouse_y: 10.0,
                partial_tick: 0.0,
                time_ms: 0,
            },
            &mut c,
        );
        let bytes = c.finish();
        let frame = screen_format::decode(&bytes).expect("decode frame");
        assert!(!frame.commands.is_empty());
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && *rect == [115.0, 75.0, 54.0, 6.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Area { label, rect, .. } if label == "Energy" && *rect == [115.0, 75.0, 54.0, 6.0])));
    }

    #[test]
    fn test_chemical_oxidizer_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:chemical_oxidizer".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open");

        let event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 140.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        let handled = screen.input(&event);
        assert!(handled);
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 6000", "click"]);
    }
}
