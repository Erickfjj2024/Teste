//! `GuiFuelwoodHeater` (176 by 166): fuel slot at (15, 29), inner screen at (48, 23, 80 by 28)
//! showing temperature and remaining burn time, burning flame indicator at (144, 31),
//! heat tab at (-26, 109), and security tab on the right.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiFuelwoodHeater` (`GuiFuelwoodHeater.java:21-66`)
//! - `mekanism.common.tile.machine.TileEntityFuelwoodHeater`
//!
//! Numbers (`fuelwood_heater.rs::screen_numbers`): 0 temperature (Kelvin), 1 burn_time, 2 max_burn_time,
//! 3 last_transfer_loss, 4 last_environment_loss, 5 security mode.

use crate::chrome::{Chrome, ChromeSpec};
use crate::elements::{base_panel, data, inner_screen, inside, inventory_label, slot_frame, title, GREEN, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct FuelwoodHeater {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:fuelwood_heater",
        redstone: None,
        energy: None,
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = FuelwoodHeater {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for FuelwoodHeater {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Fuel slot at (15, 29) -> frame at (14, 28)
        slot_frame(c, 14.0, 28.0);

        // Inner screen at (48, 23, 80, 28)
        inner_screen(c, [48.0, 23.0, 80.0, 28.0]);
        let temp_str = host::translate("temperature.mekanism.short", &[format!("{}K", data(0))]);
        let fuel_str = host::translate("gui.mekanism.fuel", &[data(1).to_string()]);
        c.text(&temp_str, 52.0, 27.0, GREEN, 0, 0.85);
        c.text(&fuel_str, 52.0, 37.0, GREEN, 0, 0.85);

        // Flame indicator at (144, 31, 13, 13)
        c.blit_sprite("mekanism:progress/flame/empty", 144.0, 31.0, 13.0, 13.0);
        let burn_time = data(1);
        let max_burn = data(2);
        let frac = if max_burn <= 0 { 0.0 } else { (burn_time as f32 / max_burn as f32).clamp(0.0, 1.0) };
        let filled_h = (13.0 * frac).floor();
        if filled_h > 0.0 {
            c.blit("mekanism:gui/progress/flame/full", 144.0, 31.0 + 13.0 - filled_h, 0.0, 13.0 - filled_h, 13.0, filled_h, 13.0, 13.0);
        }
        if inside(mouse, [144.0, 31.0, 13.0, 13.0]) {
            let tip = format!("Burning: {}%", (frac * 100.0).round() as i32);
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Heat tab at (-26, 109, 26, 26)
        let heat_tab_rect = [-26.0, 109.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/heat_info_k", heat_tab_rect[0], heat_tab_rect[1], heat_tab_rect[2], heat_tab_rect[3]);
        c.area(-2, heat_tab_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, heat_tab_rect) {
            let temp_line = host::translate("temperature.mekanism.short", &[format!("{}K", data(0))]);
            let trans_line = host::translate("gui.mekanism.transferred_rate", &[format!("{}K", data(3))]);
            let diss_line = host::translate("gui.mekanism.dissipated", &[format!("{}K", data(4))]);
            c.tooltip(&[(&temp_line, WHITE), (&trans_line, WHITE), (&diss_line, WHITE)]);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.fuelwood_heater", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

        // Chrome (security tab)
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
    fn fuelwood_heater_renders_and_handles_security_click() {
        host::set_menu(host::Menu {
            data: vec![
                400, // 0: temp
                50,  // 1: burn_time
                100, // 2: max_burn
                5,   // 3: transfer
                2,   // 4: env
                0,   // 5: security
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:fuelwood_heater".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open fuelwood heater");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: -20.0, mouse_y: 115.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "temperature.mekanism.short")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "gui.mekanism.inventory" || text == "container.inventory")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "gui.mekanism.transferred_rate"))));

        // Click security tab at (176, 34)
        let click_event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }
}
