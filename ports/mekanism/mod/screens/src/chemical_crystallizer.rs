//! `GuiChemicalCrystallizer` (176 by 166): chemical crystallizer.
//! Java reference: `mekanism.client.gui.machine.GuiChemicalCrystallizer` (lines 26-95).
//!
//! Layout:
//! - Vertical power bar at (157, 23), 6 by 54 frame, 4 by 52 fill at (158, 24)
//! - Input chemical gauge at (7, 4), standard 16 by 58 (18 by 60 frame, capacity 10,000 mB)
//! - Progress bar at (53, 61), 48 by 8 (ProgressType.LARGE_RIGHT)
//! - Inner screen at (31, 13, 115, 42) holding chemical name, recipe output, and ore preview slot at (128, 13)
//! - Energy tab at (-26, 137)
//! - Title centered between x = 25 and 152 at y = 4; inventory label omitted (matches Java)
//! - Slots: chemical item input at (8, 65) with plus overlay, output at (129, 57), power at (152, 5)
//! - Upgrade window: slots 2 (in) and 3 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chemical_oxidizer::{chemical_meta, render_chemical_gauge};
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inner_screen, inside, pair, title, vertical_power_bar, Overlay, GREEN, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

/// Returns crystallizing product information (display label and item ID) for a chemical index.
fn crystallizing_product(chem_kind: usize) -> Option<(&'static str, &'static str)> {
    match chem_kind {
        // Slurry crystallization products (indices from chem_table.rs)
        41 => Some(("Copper Crystal", "mekanism:crystal_copper")),
        43 => Some(("Gold Crystal", "mekanism:crystal_gold")),
        45 => Some(("Iron Crystal", "mekanism:crystal_iron")),
        50 => Some(("Lead Crystal", "mekanism:crystal_lead")),
        52 => Some(("Osmium Crystal", "mekanism:crystal_osmium")),
        54 => Some(("Tin Crystal", "mekanism:crystal_tin")),
        56 => Some(("Uranium Crystal", "mekanism:crystal_uranium")),
        35 => Some(("Salt", "mekanism:salt")),
        34 => Some(("Lithium Dust", "mekanism:dust_lithium")),
        _ => None,
    }
}

pub struct ChemicalCrystallizer {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:chemical_crystallizer";
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
        Box::new(ChemicalCrystallizer { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for ChemicalCrystallizer {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input (8, 65) with plus overlay, output (129, 57), power (152, 5)
        c.blit_sprite("mekanism:slot/extra", 7.0, 64.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 7.0, 64.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 128.0, 56.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 151.0, 4.0, 18.0, 18.0);

        // Vertical power bar at (157, 23)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        vertical_power_bar(c, 158.0, 24.0, power_frac);

        let power_rect = [157.0, 23.0, 6.0, 54.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Input Chemical gauge at (7, 4), 16 by 58 inside 18 by 60
        let chem_amount = data(16).max(0) as u64;
        let chem_kind = data(17).max(0) as usize;
        let gauge_tip = render_chemical_gauge(
            c,
            mouse,
            7.0,
            4.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            chem_amount,
            10000,
            chem_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = gauge_tip;
        }

        // Progress bar at (53, 61): 48 by 8 (LARGE_RIGHT)
        c.blit_sprite("mekanism:progress/large_right/empty", 53.0, 61.0, 48.0, 8.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(23).max(1) as f32;
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        if prog_frac > 0.0 {
            let filled_w = (48.0 * prog_frac).floor();
            c.enable_scissor(53.0, 61.0, filled_w, 8.0);
            c.blit_sprite("mekanism:progress/large_right/full", 53.0, 61.0, 48.0, 8.0);
            c.disable_scissor();
        }

        // Inner screen at (31, 13, 115, 42)
        inner_screen(c, [31.0, 13.0, 115.0, 42.0]);
        c.blit_sprite("mekanism:slot/dark", 128.0, 13.0, 18.0, 18.0);

        if chem_amount > 0 && chem_kind > 0 {
            if let Some(m) = chemical_meta(chem_kind) {
                c.text(m.label, 34.0, 16.0, GREEN, 0, 1.0);
                if let Some((prod_label, prod_item)) = crystallizing_product(chem_kind) {
                    c.text(&format!("({prod_label})"), 34.0, 26.0, GREEN, 0, 1.0);
                    c.item(prod_item, "", 1, 129.0, 14.0, 0, 1.0);
                } else {
                    c.text(&format!("({})", m.label), 34.0, 26.0, GREEN, 0, 1.0);
                }
            }
        }

        // Title centered between input gauge (x = 25) and energy slot (x = 152) at y = 4
        let title_text = host::translate("block.mekanism.chemical_crystallizer", &[]);
        title(c, &title_text, 25.0, 127.0, 4.0);

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
    fn test_chemical_crystallizer_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                50, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2500, 41, 0, 0, 0, 4000, 0, 100,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:chemical_crystallizer".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open chemical crystallizer");

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
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:slot/dark")));
    }

    #[test]
    fn test_chemical_crystallizer_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:chemical_crystallizer".into(),
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
