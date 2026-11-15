//! `GuiFluidicPlenisher` (176 by 166): input fluid container slot at (28, 20), output slot at (28, 51),
//! charge slot at (143, 35), standard fluid gauge at (6, 13), down arrow at (32, 39), inner screen
//! at (54, 23, 80 by 42) displaying energy, finished status, and fluid status, vertical power bar
//! at (164, 15), and energy, upgrade, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiFluidicPlenisher` (`GuiFluidicPlenisher.java:26-66`)
//! - `mekanism.common.tile.machine.TileEntityFluidicPlenisher`
//!
//! Numbers (`pump.rs`): 0 operating, 1-2 energy (pair), 3-4 capacity (pair), 16 fluid amount, 17 fluid kind,
//! 24 redstone control mode, 40 security mode. Strings: 0 energy line, 1 finished line, 2 fluid line.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inner_screen, inside, inventory_label, pair, slot_frame, string, title, vertical_power_bar, GREEN, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct FluidicPlenisher {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:fluidic_plenisher",
        redstone: Some(24),
        energy: Some(EnergyTabSpec::for_machine(24, None)),
        upgrades: Some(UpgradesSpec::new(3, 4, 24, &[0, 1])),
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = FluidicPlenisher {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for FluidicPlenisher {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots
        slot_frame(c, 27.0, 19.0);
        slot_frame(c, 27.0, 50.0);
        slot_frame(c, 142.0, 34.0);

        // Down arrow at (32, 39)
        c.blit_sprite("mekanism:arrow/down", 32.0, 39.0, 8.0, 9.0);

        // Standard fluid gauge at (6, 13, 18, 60)
        c.blit_sprite("mekanism:gauge/normal", 6.0, 13.0, 18.0, 60.0);
        let amount = data(16).max(0);
        let frac = (amount as f32 / 10000.0).clamp(0.0, 1.0);
        let filled_h = (58.0 * frac).floor();
        if filled_h > 0.0 {
            c.fill(7.0, 14.0 + 58.0 - filled_h, 16.0, filled_h, 0xFF36_6BD0);
        }
        c.blit("mekanism:gui/gauge/overlay/standard", 7.0, 14.0, 0.0, 0.0, 16.0, 58.0, 16.0, 58.0);
        if inside(mouse, [6.0, 13.0, 18.0, 60.0]) {
            let tip = if amount == 0 { "No fluid".to_string() } else { format!("Fluid: {} / 10000 mB", amount) };
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Inner screen at (54, 23, 80, 42)
        inner_screen(c, [54.0, 23.0, 80.0, 42.0]);
        let line0 = string(0);
        let line1 = string(1);
        let line2 = string(2);
        c.text(if line0.is_empty() { "0/100000" } else { &line0 }, 57.0, 26.0, GREEN, 0, 0.85);
        c.text(if line1.is_empty() { "Finished: No" } else { &line1 }, 57.0, 35.0, GREEN, 0, 0.85);
        c.text(if line2.is_empty() { "No fluid" } else { &line2 }, 57.0, 44.0, GREEN, 0, 0.85);

        // Power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let fraction = if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 };
        vertical_power_bar(c, 164.0, 15.0, fraction);
        if inside(mouse, [164.0, 15.0, 6.0, 54.0]) {
            c.tooltip(&[(&format!("Energy: {} / {} FE", energy, capacity), WHITE)]);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.fluidic_plenisher", &[]), 0.0, 176.0, 5.0);
        inventory_label(c, 8.0, 74.0);

        // Chrome (tabs and modal windows)
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
    fn fluidic_plenisher_renders_and_handles_redstone_click() {
        let mut d = vec![0; 45];
        d[1] = 6000;
        d[3] = 40000;
        d[16] = 5000;
        d[24] = 0;
        host::set_menu(host::Menu {
            data: d,
            strings: vec!["6000/40000".into(), "Finished: Yes".into(), "Water: 5000 mB".into()],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:fluidic_plenisher".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open fluidic plenisher");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Finished: Yes")));

        // Hover power bar
        let hover_frame = Frame { mouse_x: 165.0, mouse_y: 20.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&hover_frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines, .. } if lines.iter().any(|(t, _)| t.contains("Energy: 6000 / 40000 FE")))));

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
