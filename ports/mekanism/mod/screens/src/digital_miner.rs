//! `GuiDigitalMiner` (176 by 242): 27 buffer item slots at (8, 92), charge slot at (152, 20),
//! inner screen at (7, 19, 77 by 69) displaying running status, search state, and blocks to mine,
//! auto eject and silk touch digital switches at (19, 56) and (57, 56), Start, Stop, Config,
//! and Reset buttons at (87, 19..70), vertical power bar at (157, 39, 47 high),
//! and energy, upgrade, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiDigitalMiner` (`GuiDigitalMiner.java:40-143`)
//! - `mekanism.common.tile.machine.TileEntityDigitalMiner`
//!
//! Numbers (`miner.rs`): 0 running, 1-2 energy (pair), 3-4 capacity (pair), 5 radius,
//! 8 eject, 9 silk, 11 to_mine queue count, 13 status (0 stopped, 1 searching, 2 mining, 3 no energy, 4 finished),
//! 16 redstone control mode, 16 security mode.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, button, data, inner_screen, inside, inventory_label, pair, slot_frame, title, vertical_power_bar, GREEN, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub struct DigitalMiner {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:digital_miner",
        redstone: Some(16),
        energy: Some(EnergyTabSpec::with_stored_and_capacity(0, (1, 2), (3, 4), None)),
        upgrades: Some(UpgradesSpec::new(28, 29, 16, &[0, 1])),
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = DigitalMiner {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 242, hide_inventory: false, own_labels: true }))
}

impl MekScreen for DigitalMiner {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 242.0);

        // 27 buffer slots (3 rows of 9) at (8, 92) -> frames at (7 + 18*col, 91 + 18*row)
        for row in 0..3 {
            for col in 0..9 {
                slot_frame(c, 7.0 + 18.0 * col as f32, 91.0 + 18.0 * row as f32);
            }
        }

        // Charge slot at (152, 20) -> frame at (151, 19)
        slot_frame(c, 151.0, 19.0);

        // Inner screen at (7, 19, 77, 69)
        inner_screen(c, [7.0, 19.0, 77.0, 69.0]);
        let status = data(13);
        let (status_label, status_color) = match status {
            3 => ("Low Power", 0xFFFF_5555),
            1 | 2 => ("Running", GREEN),
            _ => ("Idle", GREEN),
        };
        let search_label = match status {
            1 | 2 => "Searching...",
            3 => "Paused",
            4 => "Finished",
            _ => "Not ready",
        };
        let to_mine_str = format!("To mine: {}", data(11));
        c.text(status_label, 10.0, 22.0, status_color, 0, 0.85);
        c.text(search_label, 10.0, 32.0, GREEN, 0, 0.85);
        c.text(&to_mine_str, 10.0, 42.0, GREEN, 0, 0.85);

        // Digital switches at (19, 56), (38, 56) and (57, 56)
        let eject_on = data(8) == 1;
        let eject_rect = [19.0, 56.0, 16.0, 16.0];
        let eject_sprite = if eject_on { "mekanism:button_focus" } else { "mekanism:button" };
        c.blit_sprite(eject_sprite, eject_rect[0], eject_rect[1], eject_rect[2], eject_rect[3]);
        c.blit("mekanism:gui/switch/eject", 22.0, 59.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0);
        c.area(7807, eject_rect, AreaKind::Button, true, "Auto eject");
        if inside(mouse, eject_rect) {
            c.tooltip(&[(&host::translate("gui.mekanism.auto_eject", &[]), WHITE)]);
        }

        // NÃO FEITO: auto pull no servidor/menu_data (o port não suporta auto pull no servidor)
        let pull_rect = [38.0, 56.0, 16.0, 16.0];
        c.blit_sprite("mekanism:button", pull_rect[0], pull_rect[1], pull_rect[2], pull_rect[3]);
        c.blit("mekanism:gui/switch/input", 41.0, 59.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0);
        c.area(7809, pull_rect, AreaKind::Button, false, "Auto pull");
        if inside(mouse, pull_rect) {
            c.tooltip(&[(&host::translate("gui.mekanism.auto_pull", &[]), WHITE)]);
        }

        let silk_on = data(9) == 1;
        let silk_rect = [57.0, 56.0, 16.0, 16.0];
        let silk_sprite = if silk_on { "mekanism:button_focus" } else { "mekanism:button" };
        c.blit_sprite(silk_sprite, silk_rect[0], silk_rect[1], silk_rect[2], silk_rect[3]);
        c.blit("mekanism:gui/switch/silk", 60.0, 59.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0);
        c.area(7808, silk_rect, AreaKind::Button, true, "Silk touch");
        if inside(mouse, silk_rect) {
            c.tooltip(&[(&host::translate("gui.mekanism.miner.silk", &[]), WHITE)]);
        }

        // Power bar at (157, 39, 47 high) with base at (157, 39, 6, 49)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let fraction = if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 };
        c.blit_sprite("mekanism:bar/base", 157.0, 39.0, 6.0, 49.0);
        let filled_h = (47.0 * fraction.clamp(0.0, 1.0)).floor();
        if filled_h > 0.0 {
            c.blit("mekanism:gui/bar/vertical_power", 158.0, 40.0 + 47.0 - filled_h, 0.0, 47.0 - filled_h, 4.0, filled_h, 4.0, 47.0);
        }
        if inside(mouse, [157.0, 39.0, 6.0, 49.0]) {
            c.tooltip(&[(&format!("Energy: {} / {} FE", energy, capacity), WHITE)]);
        }

        // Action buttons at (87, 19..70, 61, 18)
        let start_active = status == 0 || status == 4;
        let stop_active = status == 1 || status == 2;
        let config_active = status == 0;
        let start_text = host::translate("gui.mekanism.start", &[]);
        let stop_text = host::translate("gui.mekanism.stop", &[]);
        let config_text = host::translate("gui.mekanism.config", &[]);
        let reset_text = host::translate("gui.mekanism.reset", &[]);
        button(c, 7800, [87.0, 19.0, 61.0, 18.0], if start_text.is_empty() { "Start" } else { &start_text }, start_active, mouse, None);
        button(c, 7801, [87.0, 36.0, 61.0, 18.0], if stop_text.is_empty() { "Stop" } else { &stop_text }, stop_active, mouse, None);
        button(c, 7802, [87.0, 53.0, 61.0, 18.0], if config_text.is_empty() { "Config" } else { &config_text }, config_active, mouse, None);
        button(c, 7803, [87.0, 70.0, 61.0, 18.0], if reset_text.is_empty() { "Reset" } else { &reset_text }, true, mouse, None);

        // Title and inventory label
        title(c, &host::translate("block.mekanism.digital_miner", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 148.0);

        // Chrome (tabs and modals)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        if let InputEvent::MousePressed(p) = event {
            if p.button == MouseButton::Left {
                let status = data(13);
                let start_active = status == 0 || status == 4;
                let stop_active = status == 1 || status == 2;
                let config_active = status == 0;
                let mouse = (p.x, p.y);

                if start_active && inside(mouse, [87.0, 19.0, 61.0, 18.0]) {
                    host::send_button(7800);
                    host::play_click();
                    return true;
                }
                if stop_active && inside(mouse, [87.0, 36.0, 61.0, 18.0]) {
                    host::send_button(7801);
                    host::play_click();
                    return true;
                }
                if config_active && inside(mouse, [87.0, 53.0, 61.0, 18.0]) {
                    host::send_button(7802);
                    host::play_click();
                    return true;
                }
                if inside(mouse, [87.0, 70.0, 61.0, 18.0]) {
                    host::send_button(7803);
                    host::play_click();
                    return true;
                }
                if inside(mouse, [19.0, 56.0, 16.0, 16.0]) {
                    host::send_button(7807);
                    host::play_click();
                    return true;
                }
                if inside(mouse, [57.0, 56.0, 16.0, 16.0]) {
                    host::send_button(7808);
                    host::play_click();
                    return true;
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
    fn digital_miner_renders_and_handles_start_button() {
        let mut d = vec![0; 45];
        d[1] = 10000;
        d[3] = 40000;
        d[11] = 42; // to mine
        d[13] = 0;  // stopped
        d[16] = 0;  // redstone
        host::set_menu(host::Menu {
            data: d,
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:digital_miner".into(),
            gui_width: 176.0,
            gui_height: 242.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open digital miner");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 242);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "To mine: 42")));

        // Hover power bar at (158, 42)
        let hover_frame = Frame { mouse_x: 158.0, mouse_y: 42.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&hover_frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines, .. } if lines.iter().any(|(t, _)| t.contains("Energy: 10000 / 40000 FE")))));

        // Click Start button at (90, 25)
        let click_event = InputEvent::MousePressed(Pointer {
            x: 90.0,
            y: 25.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7800".to_string(), "click".to_string()]);
    }
}
