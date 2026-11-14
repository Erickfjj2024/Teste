//! `GuiLaserAmplifier` (176 by 166): energy gauge at (6, 10), delay, min, and max threshold labels
//! and text fields at (96, 28..58), output mode tab at (176, 109), energy tab, redstone control tab,
//! and security tab.
//!
//! Java source of truth:
//! - `mekanism.client.gui.GuiLaserAmplifier` (`GuiLaserAmplifier.java:26-114`)
//! - `mekanism.common.tile.laser.TileEntityLaserAmplifier`
//!
//! Numbers (`laser.rs`): 0-1 energy (pair), 2-3 capacity (pair), 4-5 min threshold (pair),
//! 6-7 max threshold (pair), 8 delay, 9 shot, 10 output mode, 11 redstone control mode, 12 security mode.
//! Strings: 0 delay text, 1 min threshold text, 2 max threshold text.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, TITLE, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, FieldFilter, FieldText, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub struct LaserAmplifier {
    chrome: Chrome,
    delay_input: String,
    min_input: String,
    max_input: String,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:laser_amplifier",
        redstone: Some(11),
        energy: Some(EnergyTabSpec::with_stored_and_capacity(0, (0, 1), (2, 3), None)),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = LaserAmplifier {
        chrome: Chrome::new(chrome_spec, 176.0),
        delay_input: String::new(),
        min_input: String::new(),
        max_input: String::new(),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for LaserAmplifier {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Energy gauge at (6, 10, 18, 60)
        c.blit_sprite("mekanism:gauge/normal", 6.0, 10.0, 18.0, 60.0);
        let (energy, capacity) = (pair(0, 1), pair(2, 3));
        let frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        let filled_h = (58.0 * frac).floor();
        if filled_h > 0.0 {
            c.fill(7.0, 11.0 + 58.0 - filled_h, 16.0, filled_h, 0xFF59_C15F);
        }
        c.blit("mekanism:gui/gauge/overlay/standard", 7.0, 11.0, 0.0, 0.0, 16.0, 58.0, 16.0, 58.0);
        if inside(mouse, [6.0, 10.0, 18.0, 60.0]) {
            c.tooltip(&[(&format!("Energy: {} / {} FE", energy, capacity), WHITE)]);
        }

        // Labels for delay, min, max
        let d_str = crate::elements::string(0);
        let min_str = crate::elements::string(1);
        let max_str = crate::elements::string(2);
        c.text(if d_str.is_empty() { "No delay" } else { &d_str }, 24.0, 30.0, TITLE, 0, 0.85);
        c.text(if min_str.is_empty() { "Min: 0 FE" } else { &min_str }, 24.0, 45.0, TITLE, 0, 0.85);
        c.text(if max_str.is_empty() { "Max: 5000000 FE" } else { &max_str }, 24.0, 60.0, TITLE, 0, 0.85);

        // Text fields: delay 7302, min 7300, max 7301
        c.field(7302, [96.0, 28.0, 36.0, 11.0], 4, FieldFilter::Digits, &self.delay_input, 0, WHITE);
        c.field(7300, [96.0, 43.0, 72.0, 11.0], 10, FieldFilter::Any, &self.min_input, 0, WHITE);
        c.field(7301, [96.0, 58.0, 72.0, 11.0], 10, FieldFilter::Any, &self.max_input, 0, WHITE);

        // Output mode tab at (176, 109, 26, 18)
        let mode = (data(10).max(0) as usize) % 3;
        let mode_rect = [176.0, 109.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", mode_rect, 0xFFFF_FFFF);
        let (icon, tip) = match mode {
            1 => ("mekanism:gui/amplifier_entity", "Redstone Output: Entity Detection"),
            2 => ("mekanism:gui/amplifier_contents", "Redstone Output: Energy Contents"),
            _ => ("mekanism:gui/amplifier_off", "Redstone Output: Off"),
        };
        c.blit(icon, 180.0, 109.0, 0.0, 0.0, 18.0, 18.0, 18.0, 18.0);
        c.area(7310, mode_rect, AreaKind::Tab, true, "Output Mode");
        if inside(mouse, mode_rect) {
            c.tooltip(&[(tip, WHITE)]);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.laser_amplifier", &[]), 24.0, 152.0, 5.0);
        inventory_label(c, 8.0, 73.0);

        // Chrome (tabs and modals)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        match event {
            InputEvent::FieldSubmitted(f) => {
                host::send_text(f.id, &f.text);
                match f.id {
                    7302 => self.delay_input.clear(),
                    7300 => self.min_input.clear(),
                    7301 => self.max_input.clear(),
                    _ => {}
                }
                true
            }
            InputEvent::FieldChanged(f) => {
                match f.id {
                    7302 => self.delay_input = f.text.clone(),
                    7300 => self.min_input = f.text.clone(),
                    7301 => self.max_input = f.text.clone(),
                    _ => {}
                }
                true
            }
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left && inside((p.x, p.y), [176.0, 109.0, 26.0, 18.0]) {
                    host::send_button(7310);
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
    use horizon_mod_sdk::screen::{FieldText, Modifiers, MouseButton, Pointer};

    #[test]
    fn laser_amplifier_renders_and_handles_output_tab_and_field() {
        let mut d = vec![0; 20];
        d[0] = 5000;
        d[2] = 50000;
        d[10] = 0; // off
        d[11] = 0; // redstone
        host::set_menu(host::Menu {
            data: d,
            strings: vec!["Delay: 20".into(), "Min: 1000 FE".into(), "Max: 50000 FE".into()],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:laser_amplifier".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open laser amplifier");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Delay: 20")));

        // Submit delay field
        let field_event = InputEvent::FieldSubmitted(FieldText { id: 7302, text: "40".into() });
        assert!(screen.input(&field_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["text 7302 \"40\"".to_string()]);

        // Click output mode tab
        let click_event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 115.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent2 = host::take_sent();
        assert_eq!(sent2, vec!["button 7310".to_string(), "click".to_string()]);
    }
}
