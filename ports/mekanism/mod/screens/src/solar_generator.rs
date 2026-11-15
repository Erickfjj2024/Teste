//! `GuiSolarGenerator` (176 by 166): the title, armor slots holder on the left, charge slot at (143, 35),
//! inner screen at (40, 23, 96 by 40) displaying energy stored, production rate, and max output,
//! vertical power bar at (164, 15), sun state texture at (18, 35), and energy and redstone tabs.
//! Handles both `mekanism:solar_generator` and `mekanism:advanced_solar_generator`.
//!
//! Java source of truth:
//! - `mekanism.generators.client.gui.GuiSolarGenerator` (`GuiSolarGenerator.java:23-53`)
//! - `mekanism.generators.common.tile.TileEntitySolarGenerator`
//! - `mekanism.generators.common.tile.TileEntityAdvancedSolarGenerator`
//!
//! Numbers (`generators.rs`): 0 active, 1-2 energy (pair), 3-4 capacity (pair), 5 producing rate (FE/t),
//! 6 sun visible (1 yes, 0 no), 15 redstone control mode, 31 security mode.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, inner_screen, inside, inventory_label, pair, slot_frame, title, vertical_power_bar, GREEN, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct SolarGenerator {
    menu: &'static str,
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let menu: &'static str = match info.menu.as_str() {
        "mekanism:advanced_solar_generator" => "mekanism:advanced_solar_generator",
        _ => "mekanism:solar_generator",
    };
    let chrome_spec = ChromeSpec {
        menu,
        redstone: Some(15),
        energy: Some(EnergyTabSpec::with_stored_and_capacity(5, (1, 2), (3, 4), None)),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = SolarGenerator {
        menu,
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for SolarGenerator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Armor slots holder on left: (-26, 6, 26, 98)
        c.blit_sprite("mekanism:holder_left", -26.0, 6.0, 26.0, 98.0);

        // Charge slot at (143, 35) -> frame at (142, 34)
        slot_frame(c, 142.0, 34.0);

        // Power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let fraction = if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 };
        vertical_power_bar(c, 164.0, 15.0, fraction);

        // Sun state texture at (18, 35, 16, 16)
        let sun = data(6) == 1;
        let sun_rect = [18.0, 35.0, 16.0, 16.0];
        let sun_tex = if sun { "mekanism:gui/sees_sun" } else { "mekanism:gui/no_sun" };
        c.blit(sun_tex, sun_rect[0], sun_rect[1], 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        if inside(mouse, sun_rect) {
            let tip = if sun { "Sun Visible" } else { "No Sun" };
            c.tooltip(&[(tip, WHITE)]);
        }

        // Inner screen at (40, 23, 96, 40)
        inner_screen(c, [40.0, 23.0, 96.0, 40.0]);
        let energy_str = format!("Energy: {} / {} FE", energy, capacity);
        let producing_str = format!("Producing: {} FE/t", data(5));
        let max_output_val = if self.menu == "mekanism:advanced_solar_generator" { 600 } else { 100 };
        let output_str = format!("Output: {} FE/t", max_output_val);
        c.text(&energy_str, 44.0, 27.0, GREEN, 0, 0.85);
        c.text(&producing_str, 44.0, 37.0, GREEN, 0, 0.85);
        c.text(&output_str, 44.0, 47.0, GREEN, 0, 0.85);

        // Title and inventory label
        let title_key = if self.menu == "mekanism:advanced_solar_generator" {
            "block.mekanism.advanced_solar_generator"
        } else {
            "block.mekanism.solar_generator"
        };
        title(c, &host::translate(title_key, &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

        // Render generic chrome (tabs and modals)
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
    fn solar_generator_renders_and_handles_redstone_tab_click() {
        host::set_menu(host::Menu {
            data: vec![
                1,      // 0: active
                5000, 0,// 1, 2: energy
                96000, 0,// 3, 4: capacity
                50,     // 5: producing
                1,      // 6: sun
                0, 0, 0,// 7, 8, 9: losses
                0, 0, 0, 0, 0, // 10..14: tank
                0,      // 15: redstone
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:solar_generator".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open solar generator");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 20.0, mouse_y: 20.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text.contains("Producing: 50 FE/t"))));

        // Click on Redstone tab at (176, 137)
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
