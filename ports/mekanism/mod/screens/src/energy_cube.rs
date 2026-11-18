//! `GuiEnergyCube` (`mekanism:<tier>_energy_cube`, 176 by 166): wide energy gauge at (55, 18),
//! discharge slot at (17, 35) with minus overlay, charge slot at (143, 35) with plus overlay,
//! armor side holder on the right at (176, 36) with 5 slot frames,
//! side configuration tab at (-26, 6), transporter configuration tab at (-26, 34),
//! energy tab at (-26, 137), and security tab shifted upwards to (176, 6).
//!
//! Java: `GuiEnergyCube.java:19-48`, `GuiConfigurableTile`, `MekanismTileContainer<TileEntityEnergyCube>`.
//!
//! Server data (`cube.rs` / `convert.py` `energy_cube_panel`):
//! - `data(0)`: input rate (FE/t)
//! - `pair(1, 2)`: energy stored (FE)
//! - `pair(3, 4)`: energy capacity (FE)
//! - `data(17)`: transporter config data slot (from `transporter_table.rs`)
//! - `data(19)`: security mode (0 = Public, 1 = Private, 2 = Trusted, from `security_table.rs`)
use crate::chrome::{Chrome, ChromeSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

fn menu_static(menu: &str) -> &'static str {
    match menu {
        "mekanism:basic_energy_cube" => "mekanism:basic_energy_cube",
        "mekanism:advanced_energy_cube" => "mekanism:advanced_energy_cube",
        "mekanism:elite_energy_cube" => "mekanism:elite_energy_cube",
        "mekanism:ultimate_energy_cube" => "mekanism:ultimate_energy_cube",
        "mekanism:creative_energy_cube" => "mekanism:creative_energy_cube",
        _ => "mekanism:basic_energy_cube",
    }
}

pub struct EnergyCubeScreen {
    menu_name: String,
    chrome: Chrome,
    joules: bool,
}

pub fn open(info: &OpenInfo) -> Opened {
    let static_menu = menu_static(&info.menu);
    let chrome_spec = ChromeSpec {
        menu: static_menu,
        redstone: None,
        energy: None, // Custom energy tab handled directly in screen for exact Java/convert.py match
        upgrades: None,
        warning: None,
        side_config: Some(Box::new(SideConfigWindow::for_machine(static_menu, 176, 166))),
        transporter: TransporterWindow::for_machine(static_menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>),
    };
    let chrome = Chrome::new(chrome_spec, 176.0);
    let screen = EnergyCubeScreen {
        menu_name: info.menu.clone(),
        chrome,
        joules: false,
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

impl EnergyCubeScreen {
    fn title_text(&self) -> String {
        let key = match self.menu_name.as_str() {
            "mekanism:basic_energy_cube" => "block.mekanism.basic_energy_cube",
            "mekanism:advanced_energy_cube" => "block.mekanism.advanced_energy_cube",
            "mekanism:elite_energy_cube" => "block.mekanism.elite_energy_cube",
            "mekanism:ultimate_energy_cube" => "block.mekanism.ultimate_energy_cube",
            "mekanism:creative_energy_cube" => "block.mekanism.creative_energy_cube",
            _ => "block.mekanism.basic_energy_cube",
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

    fn max_output_rate(&self) -> u64 {
        match self.menu_name.as_str() {
            "mekanism:basic_energy_cube" => 4_000,
            "mekanism:advanced_energy_cube" => 16_000,
            "mekanism:elite_energy_cube" => 64_000,
            "mekanism:ultimate_energy_cube" => 256_000,
            "mekanism:creative_energy_cube" => 1_000_000_000,
            _ => 4_000,
        }
    }
}

impl MekScreen for EnergyCubeScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Title and Inventory labels
        title(c, &self.title_text(), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

        // Armor holder on the right at (176, 36, 26, 98) with 5 slot frames
        c.blit_sprite_tinted("mekanism:holder_right", [176.0, 36.0, 26.0, 98.0], WHITE);
        for k in 0..5 {
            slot_frame(c, 179.0, 40.0 + 18.0 * k as f32);
        }

        // Slots: discharge slot at (17, 35) frame at (16, 34) with minus overlay;
        // charge slot at (143, 35) frame at (142, 34) with plus overlay
        c.blit_sprite("mekanism:slot/normal", 16.0, 34.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 16.0, 34.0, 18.0, 18.0);

        c.blit_sprite("mekanism:slot/normal", 142.0, 34.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 142.0, 34.0, 18.0, 18.0);

        // Wide Energy Gauge at (55, 18, 66, 50)
        c.blit_sprite("mekanism:gauge/normal", 55.0, 18.0, 66.0, 50.0);

        let stored = pair(1, 2);
        let capacity = pair(3, 4);
        let gauge_rect = [56.0, 19.0, 64.0, 48.0];

        let mut hovered_tooltip: Option<Vec<(String, u32)>> = None;

        if capacity > 0 && stored > 0 {
            let fill = (stored as f32 / capacity as f32).clamp(0.0, 1.0);
            let fill_h = (48.0 * fill).floor();
            if fill_h > 0.0 {
                c.blit("mekanism:gui/energy_gauge_fill", 56.0, 19.0 + 48.0 - fill_h, 0.0, 48.0 - fill_h, 64.0, fill_h, 64.0, 48.0);
            }
        }
        c.blit_sprite("mekanism:gauge/overlay/wide", 56.0, 19.0, 64.0, 48.0);

        if inside(mouse, gauge_rect) {
            hovered_tooltip = Some(vec![(format!("Energy: {stored} / {capacity}"), WHITE)]);
        }

        // Energy tab at (-26, 137, 26, 26) (GuiEnergyCube.java:38-39: MATRIX_INPUT_RATE & MAX_OUTPUT)
        let energy_rect = [-26.0, 137.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/energy_info", -26.0, 137.0, 26.0, 26.0);
        c.area(-1, energy_rect, AreaKind::Tab, true, "Energy");
        if inside(mouse, energy_rect) {
            let input_rate = data(0).max(0) as u64;
            let max_output = self.max_output_rate();
            let (in_val, out_val, unit_val) = if self.joules {
                (
                    format!("{} J", (input_rate as f64 * 2.5).round() as u64),
                    format!("{} J", (max_output as f64 * 2.5).round() as u64),
                    "Unit: Joules",
                )
            } else {
                (
                    format!("{input_rate} FE"),
                    format!("{max_output} FE"),
                    "Unit: Forge Energy",
                )
            };
            let in_trans = host::translate("matrix.mekanism.input_rate", &[in_val.clone()]);
            let in_line = if !in_trans.is_empty() && !in_trans.starts_with("matrix.") {
                in_trans
            } else {
                format!("Input: {in_val}/t")
            };
            let out_trans = host::translate("gui.mekanism.max_output", &[out_val.clone()]);
            let out_line = if !out_trans.is_empty() && !out_trans.starts_with("gui.") {
                out_trans
            } else {
                format!("Max Output: {out_val}/t")
            };
            hovered_tooltip = Some(vec![
                (in_line, WHITE),
                (out_line, WHITE),
                (unit_val.to_string(), WHITE),
            ]);
        }

        // Render Chrome (side config, transporter, security tab at y=6)
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
                if inside((p.x, p.y), [-26.0, 137.0, 26.0, 26.0]) {
                    self.joules = !self.joules;
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
    fn energy_cube_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                500, // input rate
                1000, // stored lo
                0,    // stored hi
                4000, // capacity lo
                0,    // capacity hi
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0,    // data(17) transporter
                0,    // data(18)
                0,    // data(19) security
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:basic_energy_cube".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open energy cube");
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:gauge/normal")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:gauge/overlay/wide")));

        // Hover over energy tab at (-15.0, 145.0) and verify tooltip commands
        let frame_energy_hover = Frame {
            mouse_x: -15.0,
            mouse_y: 145.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas_hover = Canvas::new();
        screen.render(&frame_energy_hover, &mut canvas_hover);
        let bytes_hover = canvas_hover.finish();
        let decoded_hover = horizon_mod_sdk::screen::format::decode(&bytes_hover).expect("decode canvas hover");
        assert!(decoded_hover.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(text, _)| text.contains("Input: 500 FE/t") || text.contains("matrix.mekanism.input_rate")))));
        assert!(decoded_hover.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(text, _)| text.contains("Max Output: 4000 FE/t") || text.contains("gui.mekanism.max_output")))));

        // Toggle energy tab unit
        let click_energy = InputEvent::MousePressed(Pointer {
            x: -15.0,
            y: 145.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_energy));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["click".to_string()]);

        // Click security tab at y=6
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }
}
