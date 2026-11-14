//! `GuiWindGenerator` (176 by 166): armor slots holder on the left, charge slot at (143, 35),
//! inner screen at (48, 21, 80 by 44) showing energy, current generation power, output rate,
//! and sky blocked error if inactive, vertical power bar at (164, 15), wind state icon at (18, 35),
//! and energy, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.generators.client.gui.GuiWindGenerator` (`GuiWindGenerator.java:27-66`)
//! - `mekanism.generators.common.tile.TileEntityWindGenerator`
//!
//! Numbers (`generators.rs`): 0 active (1 yes, 0 no), 1-2 energy (pair), 3-4 capacity (pair),
//! 5 producing rate (FE/t), 15 redstone control mode, 31 security mode.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, inner_screen, inside, inventory_label, pair, slot_frame, title, vertical_power_bar, GREEN, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct WindGenerator {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:wind_generator",
        redstone: Some(15),
        energy: Some(EnergyTabSpec::with_stored_and_capacity(5, (1, 2), (3, 4), None)),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = WindGenerator {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for WindGenerator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Armor slots holder on left
        c.blit_sprite("mekanism:holder_left", -26.0, 6.0, 26.0, 98.0);

        // Charge slot at (143, 35) -> frame at (142, 34)
        slot_frame(c, 142.0, 34.0);

        // Power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let fraction = if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 };
        vertical_power_bar(c, 164.0, 15.0, fraction);

        // Wind state icon at (18, 35, 16, 16)
        let active = data(0) == 1;
        let wind_rect = [18.0, 35.0, 16.0, 16.0];
        let wind_tex = if active { "mekanism:gui/wind_on" } else { "mekanism:gui/wind_off" };
        c.blit(wind_tex, wind_rect[0], wind_rect[1], 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        if inside(mouse, wind_rect) {
            let tip = if active { "Wind Active" } else { "Wind Inactive" };
            c.tooltip(&[(tip, WHITE)]);
        }

        // Inner screen at (48, 21, 80, 44)
        inner_screen(c, [48.0, 21.0, 80.0, 44.0]);
        let energy_str = format!("Energy: {} / {} FE", energy, capacity);
        let power_str = format!("Power: {} FE/t", data(5));
        let output_str = "Output: 960 FE/t";
        c.text(&energy_str, 52.0, 24.0, GREEN, 0, 0.85);
        c.text(&power_str, 52.0, 34.0, GREEN, 0, 0.85);
        c.text(output_str, 52.0, 44.0, GREEN, 0, 0.85);
        if !active {
            c.text("Sky Blocked", 52.0, 54.0, 0xFFFF_5555, 0, 0.85);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanismgenerators.wind_generator", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

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
    fn wind_generator_renders_and_handles_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![
                1,       // 0: active
                1000, 0, // 1, 2: energy
                200000, 0,// 3, 4: capacity
                120,     // 5: producing
                0, 0, 0, 0,// 6..9
                0, 0, 0, 0, 0, // 10..14
                0,       // 15: redstone
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:wind_generator".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open wind generator");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text.contains("Power: 120 FE/t"))));

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
