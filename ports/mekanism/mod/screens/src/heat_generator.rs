//! `GuiHeatGenerator` (176 by 166): armor slots holder on the left, lava fuel slot at (17, 35),
//! charge slot at (143, 35), wide fluid gauge (lava) at (55, 18), heat tab at (-26, 109),
//! vertical power bar at (164, 15), and energy, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.generators.client.gui.GuiHeatGenerator` (`GuiHeatGenerator.java:26-62`)
//! - `mekanism.generators.common.tile.TileEntityHeatGenerator`
//!
//! Numbers (`generators.rs`): 0 active, 1-2 energy (pair), 3-4 capacity (pair), 5 producing rate (FE/t),
//! 6 temperature (Kelvin), 7 transfer loss, 8 environment loss, 10-11 lava tank amount (pair),
//! 13-14 lava tank capacity (pair), 15 redstone control mode, 31 security mode.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, vertical_power_bar, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct HeatGenerator {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:heat_generator",
        redstone: Some(15),
        energy: Some(EnergyTabSpec::with_stored_and_capacity(5, (1, 2), (3, 4), None)),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = HeatGenerator {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for HeatGenerator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Armor holder on left
        c.blit_sprite("mekanism:holder_left", -26.0, 6.0, 26.0, 98.0);

        // Fuel slot at (17, 35) -> frame at (16, 34)
        slot_frame(c, 16.0, 34.0);
        // Charge slot at (143, 35) -> frame at (142, 34)
        slot_frame(c, 142.0, 34.0);

        // Power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let fraction = if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 };
        vertical_power_bar(c, 164.0, 15.0, fraction);

        // Wide Fluid Gauge (Lava) at (55, 18, 66, 50)
        c.blit_sprite("mekanism:gauge/normal", 55.0, 18.0, 66.0, 50.0);
        let (lava_amount, lava_cap) = (pair(10, 11), pair(13, 14));
        let lava_frac = if lava_cap == 0 { 0.0 } else { (lava_amount as f32 / lava_cap as f32).clamp(0.0, 1.0) };
        let lava_height = (48.0 * lava_frac).floor();
        if lava_height > 0.0 {
            c.fill(56.0, 19.0 + 48.0 - lava_height, 64.0, lava_height, 0xFFFF_6A00);
        }
        c.blit("mekanism:gui/gauge/overlay/wide", 56.0, 19.0, 0.0, 0.0, 64.0, 48.0, 64.0, 48.0);
        if inside(mouse, [55.0, 18.0, 66.0, 50.0]) {
            let tip = if lava_amount == 0 {
                "Empty".to_string()
            } else {
                format!("Lava: {} / {} mB", lava_amount, lava_cap)
            };
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Heat tab at (-26, 109, 26, 26)
        let heat_tab_rect = [-26.0, 109.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/heat_info_k", heat_tab_rect[0], heat_tab_rect[1], heat_tab_rect[2], heat_tab_rect[3]);
        c.area(-2, heat_tab_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, heat_tab_rect) {
            let tip = format!("Temperature: {} K\nTransferred: {}/t\nDissipated: {}/t", data(6), data(7), data(8));
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.heat_generator", &[]), 0.0, 176.0, 6.0);
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
    fn heat_generator_renders_and_handles_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![
                1,       // 0: active
                3000, 0, // 1, 2: energy
                160000, 0,// 3, 4: capacity
                200,     // 5: producing
                350,     // 6: temp
                10, 5, 0,// 7..9: losses
                8000, 0, 0, // 10..12: tank
                24000, 0, // 13, 14: tank cap
                0,       // 15: redstone
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:heat_generator".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open heat generator");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:tab/heat_info_k")));

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
