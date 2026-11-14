//! `GuiModificationStation` (176 by 230): element holder at (27, 19, 120 by 74) for installed
//! modules list, module slot at (35, 118), gear container slot at (125, 118), charge slot at (151, 21),
//! large progress arrow at (65, 123), vertical power bar at (156, 40), and energy, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.client.gui.GuiModificationStation` (`GuiModificationStation.java:25-67`)
//! - `mekanism.common.tile.TileEntityModificationStation`
//!
//! Numbers (`modification_station.rs::menu_numbers`): 0 progress ticks, 1 max progress ticks,
//! 2-3 energy (pair), 4-5 capacity (pair), 6 redstone control mode, 7-8 needed energy (pair),
//! 9 energy usage per tick, 10 installed module count, 11 security mode. Strings: installed module names.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, element_holder, inside, inventory_label, pair, slot_frame, string, title, vertical_power_bar, TITLE, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub struct ModificationStation {
    chrome: Chrome,
    selected_row: Option<usize>,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:modification_station",
        redstone: Some(6),
        energy: Some(EnergyTabSpec::with_stored_and_capacity(9, (2, 3), (4, 5), None)),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = ModificationStation {
        chrome: Chrome::new(chrome_spec, 176.0),
        selected_row: None,
    };
    Ok((Box::new(screen), Setup { width: 176, height: 230, hide_inventory: false, own_labels: true }))
}

impl MekScreen for ModificationStation {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 230.0);

        // Modules element holder at (27, 19, 120, 74)
        element_holder(c, [27.0, 19.0, 120.0, 74.0]);
        let mod_count = (data(10).max(0) as usize).min(6);
        for i in 0..mod_count {
            let row_rect = [28.0, 21.0 + 12.0 * i as f32, 118.0, 12.0];
            let is_sel = self.selected_row == Some(i);
            if is_sel {
                c.fill(row_rect[0], row_rect[1], row_rect[2], row_rect[3], 0x403C_FE9A);
            }
            let mod_name = string(i);
            c.text(if mod_name.is_empty() { "Module" } else { &mod_name }, 31.0, 22.0 + 12.0 * i as f32, TITLE, 0, 0.85);
            c.area(7800 + i as i32, row_rect, AreaKind::Row, true, "Remove module");
        }

        // Slots
        slot_frame(c, 34.0, 117.0); // Module slot
        c.blit_sprite("mekanism:slot/overlay/module", 34.0, 117.0, 18.0, 18.0);
        slot_frame(c, 124.0, 117.0); // Gear slot
        slot_frame(c, 150.0, 20.0);  // Charge slot

        // Large progress arrow at (65, 123, 48, 8)
        c.blit_sprite("mekanism:progress/large_right/empty", 65.0, 123.0, 48.0, 8.0);
        let prog = data(0);
        let max_prog = data(1);
        let prog_frac = if max_prog <= 0 { 0.0 } else { (prog as f32 / max_prog as f32).clamp(0.0, 1.0) };
        let prog_w = (48.0 * prog_frac).floor();
        if prog_w > 0.0 {
            c.blit("mekanism:gui/progress/large_right/full", 65.0, 123.0, 0.0, 0.0, prog_w, 8.0, 48.0, 8.0);
        }
        if inside(mouse, [65.0, 123.0, 48.0, 8.0]) {
            c.tooltip(&[(&format!("Installing: {}%", (prog_frac * 100.0).round() as i32), WHITE)]);
        }

        // Power bar at (156, 40)
        let (energy, capacity) = (pair(2, 3), pair(4, 5));
        let p_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        vertical_power_bar(c, 156.0, 40.0, p_frac);

        // Title and inventory label
        title(c, &host::translate("block.mekanism.modification_station", &[]), 24.0, 152.0, 6.0);
        inventory_label(c, 8.0, 138.0);

        // Chrome (tabs and modals)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        if let InputEvent::MousePressed(p) = event {
            if p.button == MouseButton::Left {
                let mod_count = (data(10).max(0) as usize).min(6);
                for i in 0..mod_count {
                    let row_rect = [28.0, 21.0 + 12.0 * i as f32, 118.0, 12.0];
                    if inside((p.x, p.y), row_rect) {
                        self.selected_row = Some(i);
                        let id = if p.modifiers.shift { 7820 + i as i32 } else { 7800 + i as i32 };
                        host::send_button(id);
                        host::play_click();
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};

    #[test]
    fn modification_station_renders_and_handles_module_click() {
        let mut d = vec![0; 25];
        d[2] = 5000;
        d[4] = 40000;
        d[6] = 0;  // redstone
        d[10] = 2; // 2 modules
        host::set_menu(host::Menu {
            data: d,
            strings: vec!["Solar Recharging Unit".into(), "Energy Unit".into()],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:modification_station".into(),
            gui_width: 176.0,
            gui_height: 230.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open modification station");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 230);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Solar Recharging Unit")));

        // Click on first module row at (30, 25)
        let click_event = InputEvent::MousePressed(Pointer {
            x: 30.0,
            y: 25.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7800".to_string(), "click".to_string()]);
    }
}
