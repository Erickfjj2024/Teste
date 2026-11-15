//! `GuiQuantumEntangloporter` (176 by 240): quantum entangloporter.
//! Java reference: `mekanism.client.gui.GuiQuantumEntangloporter` (lines 1-68).
//! Container: `mekanism.common.inventory.container.tile.MekanismTileContainer<TileEntityQuantumEntangloporter>`.
//! Tile: `mekanism.common.tile.TileEntityQuantumEntangloporter`.
//! Server: `port/mod/src/entangloporter.rs`.
//!
//! Layout:
//! - Window width 176, height 240
//! - Title centered at y = 4
//! - Frequency selector at (27, 14) (`frequency_selector`, FREQ_BASE = 8200)
//! - Shared item buffer slot at (153, 7) with slot frame at (152, 6)
//! - Energy tab at (-26, 137, 26, 26) showing stored energy of the selected frequency
//! - Security tab at (176, 34, 26, 18) via Chrome (data slot 13)
//! - Inventory label at (8, 147) (imageHeight 240 - 93 = 147)
//!
//! Differences from Java / Fidelity notes:
//! - The original has no slot on its screen; the port exposes the frequency's shared item slot at (153, 7).
//! - // NÃO FEITO: aba de calor (GuiHeatTab) não transferida no port (transferência de calor não modelada).
//! - // NÃO FEITO: aba de aviso (GuiWarningTab) no lado direito não aplicável sem avisos do tile.
use crate::chrome::{Chrome, ChromeSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, WHITE};
use crate::frequency::Selector;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct QuantumEntangloporter {
    pub chrome: Chrome,
    pub selector: Selector,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:quantum_entangloporter";
    let selector = Selector::new(27.0, 14.0, 8200, 11, 12, 0);
    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: None,
            energy: None,
            upgrades: None,
            warning: None,
            side_config: None,
            transporter: None,
        },
        176.0,
    );

    Ok((
        Box::new(QuantumEntangloporter { chrome, selector }),
        Setup {
            width: 176,
            height: 240,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for QuantumEntangloporter {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 240.0);

        // Shared item buffer slot at (153, 7)
        slot_frame(c, 152.0, 6.0);

        // Title centered at y = 4
        let title_text = host::translate("block.mekanism.quantum_entangloporter", &[]);
        title(c, &title_text, 0.0, 176.0, 4.0);

        // Energy tab at (-26, 137, 26, 26)
        let energy_rect = [-26.0, 137.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/energy_info", -26.0, 137.0, 26.0, 26.0);
        c.area(-1, energy_rect, AreaKind::Tab, true, "Energy");

        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, energy_rect) {
            let kfe = if data(0) == 0 { 0 } else { pair(1, 2) };
            let storing_str = format!("{kfe} kFE");
            let storing = host::translate("gui.mekanism.storing", &[storing_str]);
            let rate_str = "0 FE".to_string();
            let rate = host::translate("matrix.mekanism.input_rate", &[rate_str]);
            hovered_tip = Some(vec![(storing, WHITE), (rate, WHITE)]);
        }

        // Frequency selector at (27, 14)
        self.selector.render(c, mouse);

        // Inventory label at (8, 147)
        inventory_label(c, 8.0, 147.0);

        // Tooltip if hovered
        if let Some(lines) = hovered_tip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }

        // Chrome (handles security tab at 176, 34)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        match event {
            InputEvent::MousePressed(p) => self.selector.press((p.x, p.y)),
            InputEvent::FieldSubmitted(f) => self.selector.submitted(f.id, &f.text),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};
    use horizon_mod_sdk::screen_format::{self, Command};

    #[test]
    fn test_quantum_entangloporter_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                1, 500, 0, 100, 0, 200, 0, 0, 0, 2, 64, 1, 0, 0,
            ],
            strings: vec![
                "Frequency: Alpha".into(),
                "Owner: Player".into(),
                "Security: Public".into(),
                "Alpha".into(),
            ],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:quantum_entangloporter".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 240.0,
        })
        .expect("open quantum entangloporter");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 240);

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
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:slot/normal")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:tab/energy_info")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:inner_screen")));

        // Hover energy tab at (-10, 145)
        let mut c_hover = Canvas::new();
        screen.render(
            &Frame {
                mouse_x: -10.0,
                mouse_y: 145.0,
                partial_tick: 0.0,
                time_ms: 0,
            },
            &mut c_hover,
        );
        let bytes_h = c_hover.finish();
        let frame_h = screen_format::decode(&bytes_h).expect("decode hover frame");
        assert!(frame_h.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines, .. } if lines.iter().any(|(t, _)| t.contains("gui.mekanism.storing") || t.contains("Storing")))));
    }

    #[test]
    fn test_quantum_entangloporter_security_tab_click() {
        host::set_menu(host::Menu {
            data: vec![0; 20],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:quantum_entangloporter".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 240.0,
        })
        .expect("open");

        // Click security tab at (180, 40)
        let event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        let handled = screen.input(&event);
        assert!(handled);
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100", "click"]);
    }

    #[test]
    fn test_quantum_entangloporter_delete_click() {
        host::set_menu(host::Menu {
            data: vec![0; 20],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:quantum_entangloporter".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 240.0,
        })
        .expect("open");

        // Delete button in frequency selector:
        // x = 27 + 62 = 89, y = 14 + 113 = 127, w = 60, h = 18
        let event = InputEvent::MousePressed(Pointer {
            x: 100.0,
            y: 130.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        let handled = screen.input(&event);
        assert!(handled);
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 8232", "click"]);
    }
}
