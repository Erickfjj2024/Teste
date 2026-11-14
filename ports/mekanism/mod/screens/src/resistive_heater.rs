//! `GuiResistiveHeater` (176 by 166): charge slot at (15, 35), inner screen at (48, 23, 80 by 42)
//! displaying temperature and current energy usage, digital input field for target energy usage
//! at (50, 51, 76 by 12), vertical power bar at (164, 15), heat tab at (-26, 109), and energy,
//! redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiResistiveHeater` (`GuiResistiveHeater.java:29-92`)
//! - `mekanism.common.tile.machine.TileEntityResistiveHeater`
//!
//! Numbers (`resistive_heater.rs::screen_numbers`): 0-1 energy (pair), 2-3 capacity (pair),
//! 4-5 energyPerTick (pair), 6-7 energyUsed (pair), 8 temperature (Kelvin), 9 redstone control mode,
//! 10-11 needed energy (pair), 12 last_transfer_loss, 13 last_environment_loss, 14 security mode.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, inner_screen, inside, inventory_label, pair, slot_frame, title, vertical_power_bar, GREEN, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, FieldFilter, Frame, InputEvent, OpenInfo, Setup, host};

pub struct ResistiveHeater {
    chrome: Chrome,
    usage_text: String,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:resistive_heater",
        redstone: Some(9),
        energy: Some(EnergyTabSpec::new(6, 10, 11, None)),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = ResistiveHeater {
        chrome: Chrome::new(chrome_spec, 176.0),
        usage_text: String::new(),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for ResistiveHeater {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Charge slot at (15, 35) -> frame at (14, 34)
        slot_frame(c, 14.0, 34.0);

        // Power bar at (164, 15)
        let (energy, capacity) = (pair(0, 1), pair(2, 3));
        let fraction = if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 };
        vertical_power_bar(c, 164.0, 15.0, fraction);

        // Inner screen at (48, 23, 80, 42)
        inner_screen(c, [48.0, 23.0, 80.0, 42.0]);
        let temp_str = format!("Temperature: {} K", data(8));
        let usage_val = pair(4, 5);
        let usage_str = format!("Usage: {} FE/t", usage_val);
        c.text(&temp_str, 52.0, 27.0, GREEN, 0, 0.85);
        c.text(&usage_str, 52.0, 37.0, GREEN, 0, 0.85);

        // Text field for setting energy usage at (50, 51, 76, 12)
        c.field(7600, [50.0, 51.0, 76.0, 12.0], 7, FieldFilter::Digits, &self.usage_text, 0, WHITE);

        // Heat tab at (-26, 109, 26, 26)
        let heat_tab_rect = [-26.0, 109.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/heat_info_k", heat_tab_rect[0], heat_tab_rect[1], heat_tab_rect[2], heat_tab_rect[3]);
        c.area(-2, heat_tab_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, heat_tab_rect) {
            let tip = format!("Temperature: {} K\nTransferred: {}/t\nDissipated: {}/t\nUnit: Kelvin", data(8), data(12), data(13));
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.resistive_heater", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 74.0);

        // Chrome (tabs and modals)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        match event {
            InputEvent::FieldSubmitted(f) if f.id == 7600 => {
                host::send_text(7600, &f.text);
                self.usage_text.clear();
                true
            }
            InputEvent::FieldChanged(f) if f.id == 7600 => {
                self.usage_text = f.text.clone();
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{FieldText, Modifiers, MouseButton, Pointer};

    #[test]
    fn resistive_heater_renders_and_handles_field_and_redstone() {
        host::set_menu(host::Menu {
            data: vec![
                5000, 0, // 0, 1: energy
                100000, 0,// 2, 3: capacity
                100, 0,  // 4, 5: usage per tick
                100, 0,  // 6, 7: usage
                450,     // 8: temp
                0,       // 9: redstone
                95000, 0,// 10, 11: needed
                10, 5,   // 12, 13: losses
                0,       // 14: security
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:resistive_heater".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open resistive heater");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Field { id, .. } if *id == 7600)));

        // Submit usage textfield
        let field_event = InputEvent::FieldSubmitted(FieldText { id: 7600, text: "250".into() });
        assert!(screen.input(&field_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["text 7600 \"250\"".to_string()]);

        // Click redstone tab
        let click_event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 140.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent2 = host::take_sent();
        assert_eq!(sent2, vec!["button 6000".to_string(), "click".to_string()]);
    }
}
