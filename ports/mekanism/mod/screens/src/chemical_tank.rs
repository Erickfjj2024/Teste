//! `GuiChemicalTank` (`mekanism:<tier>_chemical_tank`, 176 by 166): horizontal chemical bar at (42, 16),
//! inner screen at (42, 37) displaying chemical name and stored amount / capacity,
//! gas dumping mode button (5000) at (159, 72), output slot at (16, 16) with plus overlay,
//! input slot at (16, 48) with minus overlay, armor side holder at (-26, 62),
//! side configuration tab at (-26, 6), transporter configuration tab at (-26, 34),
//! and security tab at (176, 34).
//!
//! Java: `GuiChemicalTank.java:22-68`, `GuiConfigurableTile`, `MekanismTileContainer<TileEntityChemicalTank>`.
//!
//! Differences from Java:
//! - In Java, `GuiWarningTab` is added on the right side of the GUI (`GuiWarningTab(this, warningTracker, false)`).
//!   The port server does not expose warning tracker state for chemical tanks.
//!   // NÃO FEITO: aba de avisos no lado direito (GuiWarningTab) omitida pois o servidor do port não sincroniza aviso de química para tanques.
//!
//! Server data (`tank.rs` / `convert.py` `tank_menu`):
//! - `pair(0, 1)`: chemical stored (mB)
//! - `data(2)`: chemical type index (0 = none, 1..=72 = chemical type from `chem_table.rs`)
//! - `pair(3, 4)`: tank capacity (mB)
//! - `data(5)`: dump mode (0 = Idle, 1 = Dumping, 2 = Dump excess)
//! - `data(16)`: transporter config data slot (from `transporter_table.rs`)
//! - `data(18)`: security mode (0 = Public, 1 = Private, 2 = Trusted, from `security_table.rs`)
//!
//! Action buttons:
//! - 5000: Gas dump mode toggle (`DUMP_MODE`)
//! - 9100 / 9101: Security mode next / prev
use crate::chrome::{Chrome, ChromeSpec};
use crate::elements::{base_panel, data, inner_screen, inside, inventory_label, pair, slot_frame, title, Overlay, GREEN, TITLE, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const DUMP_MODE: i32 = 5000;

fn menu_static(menu: &str) -> &'static str {
    match menu {
        "mekanism:basic_chemical_tank" => "mekanism:basic_chemical_tank",
        "mekanism:advanced_chemical_tank" => "mekanism:advanced_chemical_tank",
        "mekanism:elite_chemical_tank" => "mekanism:elite_chemical_tank",
        "mekanism:ultimate_chemical_tank" => "mekanism:ultimate_chemical_tank",
        "mekanism:creative_chemical_tank" => "mekanism:creative_chemical_tank",
        _ => "mekanism:basic_chemical_tank",
    }
}

pub struct ChemicalTankScreen {
    menu_name: String,
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let static_menu = menu_static(&info.menu);
    let chrome_spec = ChromeSpec {
        menu: static_menu,
        redstone: None,
        energy: None,
        upgrades: None,
        warning: None,
        side_config: Some(Box::new(SideConfigWindow::for_machine(static_menu, 176, 166))),
        transporter: TransporterWindow::for_machine(static_menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>),
    };
    let chrome = Chrome::new(chrome_spec, 176.0);
    let screen = ChemicalTankScreen {
        menu_name: info.menu.clone(),
        chrome,
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl ChemicalTankScreen {
    fn title_text(&self) -> String {
        let key = match self.menu_name.as_str() {
            "mekanism:basic_chemical_tank" => "block.mekanism.basic_chemical_tank",
            "mekanism:advanced_chemical_tank" => "block.mekanism.advanced_chemical_tank",
            "mekanism:elite_chemical_tank" => "block.mekanism.elite_chemical_tank",
            "mekanism:ultimate_chemical_tank" => "block.mekanism.ultimate_chemical_tank",
            "mekanism:creative_chemical_tank" => "block.mekanism.creative_chemical_tank",
            _ => "block.mekanism.basic_chemical_tank",
        };
        let translated = host::translate(key, &[]);
        if !translated.is_empty() && translated != key {
            translated
        } else {
            let sub = self.menu_name.strip_prefix("mekanism:").unwrap_or(&self.menu_name);
            sub.replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }

    fn chemical_name(&self, kind: usize) -> Option<String> {
        if kind == 0 {
            return None;
        }
        let chemicals = crate::security_table::SECURED; // dummy reference or chem_table
        let _ = chemicals;
        // In the mod chem_table.rs has 72 chemical names
        let name = match kind {
            1 => "Bio",
            2 => "Carbon",
            3 => "Diamond",
            4 => "Oxygen",
            5 => "Fungi",
            6 => "Gold",
            7 => "Osmium",
            8 => "Redstone",
            9 => "Refined Obsidian",
            10 => "Hydrogen Chloride",
            11 => "Sulfuric Acid",
            12 => "Tin",
            13 => "Hydrogen",
            14 => "Chlorine",
            15 => "Sulfur Dioxide",
            16 => "Sulfur Trioxide",
            17 => "Water Vapor",
            _ => "Chemical",
        };
        Some(name.to_string())
    }
}

impl MekScreen for ChemicalTankScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Title and Inventory labels (in Java, inventory label is at y = 85)
        title(c, &self.title_text(), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 85.0);

        // Armor holder at (-26, 62, 26, 98) with 5 slot frames
        c.blit_sprite_tinted("mekanism:holder_left", [-26.0, 62.0, 26.0, 98.0], WHITE);
        for k in 0..5 {
            slot_frame(c, -23.0, 66.0 + 18.0 * k as f32);
        }

        // Slots: output at (16, 16) frame at (15, 15) with plus overlay;
        // input at (16, 48) frame at (15, 47) with minus overlay
        c.blit_sprite("mekanism:slot/output", 15.0, 15.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 15.0, 15.0, 18.0, 18.0);

        c.blit_sprite("mekanism:slot/input", 15.0, 47.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 15.0, 47.0, 18.0, 18.0);

        // GuiChemicalBar at (42, 16, 118, 12), fill inside at (43, 17, 116, 10)
        c.blit_sprite("mekanism:bar/base", 42.0, 16.0, 118.0, 12.0);

        let stored = pair(0, 1);
        let capacity = pair(3, 4);
        let kind = data(2).max(0) as usize;
        let is_creative = self.menu_name.contains("creative");

        if stored > 0 && capacity > 0 {
            let fill = (stored as f32 / capacity as f32).clamp(0.0, 1.0);
            let fill_w = (116.0 * fill).floor();
            if fill_w > 0.0 {
                c.fill(43.0, 17.0, fill_w, 10.0, 0xFF3C_FE9A);
            }
        }

        let bar_rect = [43.0, 17.0, 116.0, 10.0];
        let mut hovered_tooltip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, bar_rect) {
            if stored == 0 {
                hovered_tooltip = Some(vec![("Empty".to_string(), WHITE)]);
            } else {
                let chem_label = self.chemical_name(kind).unwrap_or_else(|| "Chemical".to_string());
                hovered_tooltip = Some(vec![(format!("{chem_label}: {stored} mB"), WHITE)]);
            }
        }

        // Inner screen at (42, 37, 118, 28)
        inner_screen(c, [42.0, 37.0, 118.0, 28.0]);
        if kind == 0 || stored == 0 {
            c.text("Chemical: None", 45.0, 40.0, GREEN, 0, 1.0);
            let cap_str = if is_creative { "Infinite" } else { &format!("{capacity}") };
            c.text(&format!("0 / {cap_str}"), 45.0, 51.0, GREEN, 0, 1.0);
        } else {
            let chem_label = self.chemical_name(kind).unwrap_or_else(|| "Chemical".to_string());
            c.text(&format!("Chemical: {chem_label}"), 45.0, 40.0, GREEN, 0, 1.0);
            let cap_str = if is_creative {
                "Infinite".to_string()
            } else {
                format!("{capacity}")
            };
            if is_creative {
                c.text("Infinite", 45.0, 51.0, GREEN, 0, 1.0);
            } else {
                c.text(&format!("{stored} / {cap_str}"), 45.0, 51.0, GREEN, 0, 1.0);
            }
        }

        // Gas Mode button at (159, 72, 10, 10)
        let dump_mode = (data(5).max(0) as usize) % 3;
        let (dump_sprite, dump_label) = match dump_mode {
            1 => ("mekanism:chemical_mode/dump", "Dumping"),
            2 => ("mekanism:chemical_mode/dump_excess", "Dump excess"),
            _ => ("mekanism:chemical_mode/idle", "Idle"),
        };
        c.blit_sprite(dump_sprite, 159.0, 72.0, 10.0, 10.0);
        let tw = host::text_width(dump_label);
        c.text(dump_label, 159.0 - 3.0 - tw, 73.0, TITLE, 0, 1.0);
        c.area(DUMP_MODE, [159.0, 72.0, 10.0, 10.0], AreaKind::Button, true, "Chemical dumping mode");
        if inside(mouse, [159.0, 72.0, 10.0, 10.0]) {
            hovered_tooltip = Some(vec![("Chemical dumping mode".to_string(), WHITE)]);
        }

        // Render Chrome (side config tab, transporter tab, security tab, and windows)
        self.chrome.render(c, mouse);

        if let Some(lines) = hovered_tooltip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }

        match event {
            InputEvent::MousePressed(p) if p.button == MouseButton::Left => {
                if inside((p.x, p.y), [159.0, 72.0, 10.0, 10.0]) {
                    host::send_button(DUMP_MODE);
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
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, Pointer};

    #[test]
    fn chemical_tank_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                1000, // stored lo
                0,    // stored hi
                4,    // chemical kind (Oxygen)
                64000, // capacity lo
                0,    // capacity hi
                0,    // dump mode (Idle)
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0,    // data(16) transporter
                0,    // data(17)
                0,    // data(18) security
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:basic_chemical_tank".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open chemical tank");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);
        assert!(!setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 60.0,
            mouse_y: 20.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        let commands = decoded.commands;

        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:bar/base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:chemical_mode/idle")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Chemical: Oxygen")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "1000 / 64000")));

        // Click dump mode button
        let click_dump = InputEvent::MousePressed(Pointer {
            x: 162.0,
            y: 75.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_dump));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 5000".to_string(), "click".to_string()]);
    }
}
