//! `GuiNutritionalLiquifier` (176 by 166): food input slot at (26, 36), paste container output slot at (110, 36),
//! container fill/drain slots at (155, 25) and (155, 56), charge slot at (155, 5), standard fluid gauge at (133, 13),
//! down arrow at (159, 44), large progress arrow at (54, 40), horizontal power bar at (115, 75),
//! side config, transporter, upgrade, energy, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiNutritionalLiquifier` (`GuiNutritionalLiquifier.java:21-49`)
//! - `mekanism.common.tile.machine.TileEntityNutritionalLiquifier`
//!
//! Numbers (`reaction.rs` / `plus.rs`): 0 progress, 1-2 energy (pair), 3-4 capacity (pair),
//! 18 fluid amount, 19 fluid kind, 24 redstone control mode, 38 security mode.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct NutritionalLiquifier {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let side_config = Some(Box::new(SideConfigWindow::for_machine(
        "mekanism:nutritional_liquifier",
        176,
        166,
    )) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine("mekanism:nutritional_liquifier", 176, 166)
        .map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome_spec = ChromeSpec {
        menu: "mekanism:nutritional_liquifier",
        redstone: Some(24),
        energy: Some(EnergyTabSpec::for_machine(24, None)),
        upgrades: Some(UpgradesSpec::new(4, 5, 24, &[0, 1])),
        warning: None,
        side_config,
        transporter,
    };
    let screen = NutritionalLiquifier {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for NutritionalLiquifier {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots
        slot_frame(c, 25.0, 35.0); // Food input
        slot_frame(c, 109.0, 35.0); // Container output
        slot_frame(c, 154.0, 24.0); // Container fill
        slot_frame(c, 154.0, 55.0); // Container drain
        slot_frame(c, 154.0, 4.0);  // Charge slot

        // Large progress arrow at (54, 40, 48, 8)
        c.blit_sprite("mekanism:progress/large_right/empty", 54.0, 40.0, 48.0, 8.0);
        let prog = data(0).max(0);
        let max_prog = 23; // default recipe ticks or data(23)
        let prog_frac = (prog as f32 / max_prog as f32).clamp(0.0, 1.0);
        let prog_w = (48.0 * prog_frac).floor();
        if prog_w > 0.0 {
            c.blit("mekanism:gui/progress/large_right/full", 54.0, 40.0, 0.0, 0.0, prog_w, 8.0, 48.0, 8.0);
        }

        // Down arrow at (159, 44)
        c.blit_sprite("mekanism:arrow/down", 159.0, 44.0, 8.0, 9.0);

        // Standard fluid gauge at (133, 13, 18, 60)
        c.blit_sprite("mekanism:gauge/normal", 133.0, 13.0, 18.0, 60.0);
        let amount = data(18).max(0);
        let frac = (amount as f32 / 10000.0).clamp(0.0, 1.0);
        let filled_h = (58.0 * frac).floor();
        if filled_h > 0.0 {
            c.fill(134.0, 14.0 + 58.0 - filled_h, 16.0, filled_h, 0xFF59_C15F); // Nutritional paste tint
        }
        c.blit("mekanism:gui/gauge/overlay/standard", 134.0, 14.0, 0.0, 0.0, 16.0, 58.0, 16.0, 58.0);
        if inside(mouse, [133.0, 13.0, 18.0, 60.0]) {
            let tip = if amount == 0 { "Empty".to_string() } else { format!("Paste: {} / 10000 mB", amount) };
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Horizontal power bar at (115, 75, 52, 4) with frame at (115, 75, 54, 6)
        c.blit_sprite("mekanism:bar/base", 115.0, 75.0, 54.0, 6.0);
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let p_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        let p_w = (52.0 * p_frac).floor();
        if p_w > 0.0 {
            c.blit("mekanism:gui/bar/horizontal_power", 116.0, 76.0, 0.0, 0.0, p_w, 4.0, 52.0, 4.0);
        }
        if inside(mouse, [115.0, 75.0, 54.0, 6.0]) {
            c.tooltip(&[(&format!("Energy: {} / {} FE", energy, capacity), WHITE)]);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.nutritional_liquifier", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

        // Chrome (tabs and modals: side config, transporter, upgrade, redstone, security)
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
    fn nutritional_liquifier_renders_and_handles_redstone_click() {
        let mut d = vec![0; 45];
        d[1] = 5000;
        d[3] = 40000;
        d[18] = 4000;
        d[24] = 0;
        host::set_menu(host::Menu {
            data: d,
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:nutritional_liquifier".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open nutritional liquifier");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));

        // Hover power bar at (120, 76)
        let hover_frame = Frame { mouse_x: 120.0, mouse_y: 76.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&hover_frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines, .. } if lines.iter().any(|(t, _)| t.contains("Energy: 5000 / 40000 FE")))));

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
