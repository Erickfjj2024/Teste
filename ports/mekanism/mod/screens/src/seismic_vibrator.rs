//! `GuiSeismicVibrator` (176 by 166): charge slot at (143, 35), inner screen at (16, 23, 112 by 40)
//! displaying active vibrating status and current chunk coordinates, vertical power bar at (164, 15),
//! and energy, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiSeismicVibrator` (`GuiSeismicVibrator.java:19-47`)
//! - `mekanism.common.tile.machine.TileEntitySeismicVibrator`
//!
//! Numbers (`seismic_vibrator.rs::menu_numbers`): 0 vibrating (1 yes, 0 no), 1-2 energy (pair),
//! 3-4 capacity (pair), 5 energy usage per tick (FE/t), 6 redstone control mode, 7-8 needed energy (pair),
//! 9 security mode. Strings: 0 chunk string ("Chunk: x, z").

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, inner_screen, inventory_label, pair, slot_frame, string, title, vertical_power_bar, GREEN};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct SeismicVibrator {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:seismic_vibrator",
        redstone: Some(6),
        energy: Some(EnergyTabSpec::new(5, 7, 8, None)),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = SeismicVibrator {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for SeismicVibrator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Charge slot at (143, 35) -> frame at (142, 34)
        slot_frame(c, 142.0, 34.0);

        // Inner screen at (16, 23, 112, 40)
        inner_screen(c, [16.0, 23.0, 112.0, 40.0]);
        let vibrating = data(0) == 1;
        let status_text = if vibrating { "Vibrating" } else { "Idle" };
        let chunk_text = string(0);
        c.text(status_text, 20.0, 27.0, GREEN, 0, 0.85);
        c.text(if chunk_text.is_empty() { "Chunk: 0, 0" } else { &chunk_text }, 20.0, 37.0, GREEN, 0, 0.85);

        // Power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let fraction = if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 };
        vertical_power_bar(c, 164.0, 15.0, fraction);

        // Title and inventory label
        title(c, &host::translate("block.mekanism.seismic_vibrator", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 74.0);

        // Chrome (tabs and modals)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        self.chrome.input(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};

    #[test]
    fn seismic_vibrator_renders_and_handles_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![
                1,       // 0: vibrating
                4000, 0, // 1, 2: energy
                50000, 0,// 3, 4: capacity
                20,      // 5: usage
                0,       // 6: redstone
                46000, 0,// 7, 8: needed
                0,       // 9: security
            ],
            strings: vec!["Chunk: 3, -5".into()],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:seismic_vibrator".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open seismic vibrator");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Chunk: 3, -5")));

        // Click redstone tab
        let click_event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 140.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 6000".to_string(), "click".to_string()]);
    }
}
