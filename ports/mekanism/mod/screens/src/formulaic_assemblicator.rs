//! `GuiFormulaicAssemblicator` (176 by 230): formula slot at (6, 26), 18 input slots at (8, 98),
//! 3x3 crafting grid at (26, 17), large output slot at (115, 16), charge slot at (152, 76),
//! tall progress arrow at (86, 43), recipe check/cross overlay at (88, 22), six control buttons
//! (Encode 7900, Stock Control 7901, Empty/Fill 7902, Craft Single 7903, Craft All 7904, Auto 7905),
//! vertical power bar at (159, 15), and side config, transporter, upgrade, energy, redstone, and security tabs.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiFormulaicAssemblicator` (`GuiFormulaicAssemblicator.java:34-144`)
//! - `mekanism.common.tile.machine.TileEntityFormulaicAssemblicator`
//!
//! Numbers (`assemblicator.rs::menu_numbers`): 0 ticks worked, 1 ticks a craft takes, 2-3 energy (pair),
//! 4-5 capacity (pair), 6 auto mode, 7 stock control, 8 valid formula, 9 recipe in the grid,
//! 10 redstone control mode, 11-12 energy still needed (pair), 13 can encode, 14 energy a tick, 15 security mode.

use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, vertical_power_bar, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub struct FormulaicAssemblicator {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let side_config = Some(Box::new(SideConfigWindow::for_machine(
        "mekanism:formulaic_assemblicator",
        176,
        230,
    )) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine("mekanism:formulaic_assemblicator", 176, 230)
        .map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome_spec = ChromeSpec {
        menu: "mekanism:formulaic_assemblicator",
        redstone: Some(10),
        energy: Some(EnergyTabSpec::with_stored_and_capacity(14, (2, 3), (4, 5), None)),
        upgrades: Some(UpgradesSpec::new(35, 36, 10, &[0, 1])),
        warning: None,
        side_config,
        transporter,
    };
    let screen = FormulaicAssemblicator {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 230, hide_inventory: false, own_labels: true }))
}

impl MekScreen for FormulaicAssemblicator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 230.0);

        // Formula slot at (6, 26) -> frame at (5, 25)
        slot_frame(c, 5.0, 25.0);
        c.blit_sprite("mekanism:slot/overlay/formula", 5.0, 25.0, 18.0, 18.0);

        // 18 input slots (2 rows of 9) at (8 + 18*col, 98 + 18*row) -> frames at (7+18*col, 97+18*row)
        for row in 0..2 {
            for col in 0..9 {
                slot_frame(c, 7.0 + 18.0 * col as f32, 97.0 + 18.0 * row as f32);
            }
        }

        // 3x3 crafting grid at (26 + 18*col, 17 + 18*row) -> frames at (25+18*col, 16+18*row)
        for row in 0..3 {
            for col in 0..3 {
                slot_frame(c, 25.0 + 18.0 * col as f32, 16.0 + 18.0 * row as f32);
            }
        }

        // Large output slot at (115, 16, 36 by 54)
        c.blit("mekanism:gui/slot/output_large", 115.0, 16.0, 0.0, 0.0, 36.0, 54.0, 36.0, 54.0);

        // Charge slot at (152, 76) -> frame at (151, 75)
        slot_frame(c, 151.0, 75.0);

        // Tall progress arrow at (86, 43, 28 by 15)
        c.blit_sprite("mekanism:progress/tall_right/empty", 86.0, 43.0, 28.0, 15.0);
        let ticks = data(0);
        let max_ticks = data(1);
        let prog_frac = if max_ticks <= 0 { 0.0 } else { (ticks as f32 / max_ticks as f32).clamp(0.0, 1.0) };
        let prog_w = (28.0 * prog_frac).floor();
        if prog_w > 0.0 {
            c.blit("mekanism:gui/progress/tall_right/full", 86.0, 43.0, 0.0, 0.0, prog_w, 15.0, 28.0, 15.0);
        }

        // Recipe check / cross overlay at (88, 22, 18, 18)
        let has_recipe = data(9) == 1;
        let overlay_sprite = if has_recipe { "mekanism:slot/overlay/check" } else { "mekanism:slot/overlay/x" };
        c.blit_sprite(overlay_sprite, 88.0, 22.0, 18.0, 18.0);

        // Power bar at (159, 15)
        let (energy, capacity) = (pair(2, 3), pair(4, 5));
        let p_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        vertical_power_bar(c, 159.0, 15.0, p_frac);

        // Buttons
        let auto_mode = data(6) == 1;
        let stock_ctrl = data(7) == 1;
        let valid_formula = data(8) == 1;
        let can_encode = data(13) == 1;

        // 1. Encode formula button at (7, 45, 14, 14), id 7900
        let enc_act = can_encode;
        c.blit_sprite(if enc_act { "mekanism:button" } else { "mekanism:button_inactive" }, 7.0, 45.0, 14.0, 14.0);
        c.blit("mekanism:gui/button/encode_formula", 7.0, 45.0, 0.0, 0.0, 14.0, 14.0, 14.0, 14.0);
        c.area(7900, [7.0, 45.0, 14.0, 14.0], AreaKind::Button, enc_act, "Encode Formula");
        if inside(mouse, [7.0, 45.0, 14.0, 14.0]) {
            c.tooltip(&[("Encode Formula", WHITE)]);
        }

        // 2. Stock control button at (26, 75, 16, 16), id 7901
        let stock_act = valid_formula;
        c.blit_sprite(if stock_act { "mekanism:button" } else { "mekanism:button_inactive" }, 26.0, 75.0, 16.0, 16.0);
        c.blit("mekanism:gui/button/stock_control", 26.0, 75.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        c.area(7901, [26.0, 75.0, 16.0, 16.0], AreaKind::Button, stock_act, "Stock Control");
        if inside(mouse, [26.0, 75.0, 16.0, 16.0]) {
            let tip = if stock_ctrl { "Stock Control: On" } else { "Stock Control: Off" };
            c.tooltip(&[(tip, WHITE)]);
        }

        // 3. Fill/Empty button at (44, 75, 16, 16), id 7902
        let fill_act = !auto_mode;
        c.blit_sprite(if fill_act { "mekanism:button" } else { "mekanism:button_inactive" }, 44.0, 75.0, 16.0, 16.0);
        let fill_icon = if valid_formula { "mekanism:gui/button/empty" } else { "mekanism:gui/button/fill" };
        c.blit(fill_icon, 44.0, 75.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        c.area(7902, [44.0, 75.0, 16.0, 16.0], AreaKind::Button, fill_act, "Fill/Empty");
        if inside(mouse, [44.0, 75.0, 16.0, 16.0]) {
            let tip = if valid_formula { "Empty Grid" } else { "Fill Grid" };
            c.tooltip(&[(tip, WHITE)]);
        }

        // 4. Craft single at (71, 75, 16, 16), id 7903
        let craft_act = !auto_mode && has_recipe;
        c.blit_sprite(if craft_act { "mekanism:button" } else { "mekanism:button_inactive" }, 71.0, 75.0, 16.0, 16.0);
        c.blit("mekanism:gui/button/craft_single", 71.0, 75.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        c.area(7903, [71.0, 75.0, 16.0, 16.0], AreaKind::Button, craft_act, "Craft Single Item");
        if inside(mouse, [71.0, 75.0, 16.0, 16.0]) {
            c.tooltip(&[("Craft Single Item", WHITE)]);
        }

        // 5. Craft available at (89, 75, 16, 16), id 7904
        c.blit_sprite(if craft_act { "mekanism:button" } else { "mekanism:button_inactive" }, 89.0, 75.0, 16.0, 16.0);
        c.blit("mekanism:gui/button/craft_available", 89.0, 75.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        c.area(7904, [89.0, 75.0, 16.0, 16.0], AreaKind::Button, craft_act, "Craft Available Items");
        if inside(mouse, [89.0, 75.0, 16.0, 16.0]) {
            c.tooltip(&[("Craft Available Items", WHITE)]);
        }

        // 6. Auto-mode toggle at (107, 75, 16, 16), id 7905
        let auto_act = valid_formula;
        c.blit_sprite(if auto_act { "mekanism:button" } else { "mekanism:button_inactive" }, 107.0, 75.0, 16.0, 16.0);
        c.blit("mekanism:gui/button/auto_toggle", 107.0, 75.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        c.area(7905, [107.0, 75.0, 16.0, 16.0], AreaKind::Button, auto_act, "Auto Mode");
        if inside(mouse, [107.0, 75.0, 16.0, 16.0]) {
            let tip = if auto_mode { "Auto-Mode: On" } else { "Auto-Mode: Off" };
            c.tooltip(&[(tip, WHITE)]);
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.formulaic_assemblicator", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 136.0);

        // Chrome (tabs and modal windows)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        if let InputEvent::MousePressed(p) = event {
            if p.button == MouseButton::Left {
                let mouse = (p.x, p.y);
                let auto_mode = data(6) == 1;
                let valid_formula = data(8) == 1;
                let has_recipe = data(9) == 1;
                let can_encode = data(13) == 1;

                if can_encode && inside(mouse, [7.0, 45.0, 14.0, 14.0]) {
                    host::send_button(7900);
                    host::play_click();
                    return true;
                }
                if valid_formula && inside(mouse, [26.0, 75.0, 16.0, 16.0]) {
                    host::send_button(7901);
                    host::play_click();
                    return true;
                }
                if !auto_mode && inside(mouse, [44.0, 75.0, 16.0, 16.0]) {
                    host::send_button(7902);
                    host::play_click();
                    return true;
                }
                if !auto_mode && has_recipe && inside(mouse, [71.0, 75.0, 16.0, 16.0]) {
                    host::send_button(7903);
                    host::play_click();
                    return true;
                }
                if !auto_mode && has_recipe && inside(mouse, [89.0, 75.0, 16.0, 16.0]) {
                    host::send_button(7904);
                    host::play_click();
                    return true;
                }
                if valid_formula && inside(mouse, [107.0, 75.0, 16.0, 16.0]) {
                    host::send_button(7905);
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
    fn formulaic_assemblicator_renders_and_handles_fill_button() {
        let mut d = vec![0; 45];
        d[2] = 5000;  // energy lo
        d[4] = 40000; // capacity lo
        d[6] = 0;     // auto mode off
        d[8] = 0;     // no formula
        d[9] = 1;     // has recipe
        d[10] = 0;    // redstone
        host::set_menu(host::Menu {
            data: d,
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:formulaic_assemblicator".into(),
            gui_width: 176.0,
            gui_height: 230.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open formulaic assemblicator");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 230);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == 7902)));

        // Click Fill/Empty button at (45, 76)
        let click_event = InputEvent::MousePressed(Pointer {
            x: 45.0,
            y: 76.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7902".to_string(), "click".to_string()]);
    }
}
