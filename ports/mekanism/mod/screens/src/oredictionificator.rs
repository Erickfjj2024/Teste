//! `GuiOredictionificator` (236 by 230): element holder at (9, 17, 204 by 68) for three filter rows,
//! new filter input field at (12, 88), large progress arrow at (94, 119), input slot at (56, 115),
//! output slot at (164, 115), and redstone and security tabs on the right.
//!
//! Java source of truth:
//! - `mekanism.client.gui.machine.GuiOredictionificator` (`GuiOredictionificator.java:27-95`)
//! - `mekanism.common.tile.machine.TileEntityOredictionificator`
//!
//! Numbers (`oredictionificator.rs::menu_numbers`): 0 did_process (1 yes, 0 no), 1 redstone control mode,
//! 2 filter count, 3-5 filter state (1 enabled, 0 disabled) for first 3 filters, 6 security mode.
//! Strings: 2*i tag name, 2*i + 1 result item name.

use crate::chrome::{Chrome, ChromeSpec};
use crate::elements::{base_panel, data, element_holder, inside, slot_frame, string, title, GREEN, TITLE, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, FieldFilter, FieldText, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub struct Oredictionificator {
    chrome: Chrome,
    new_filter_input: String,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:oredictionificator",
        redstone: Some(1),
        energy: None,
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = Oredictionificator {
        chrome: Chrome::new(chrome_spec, 236.0),
        new_filter_input: String::new(),
    };
    Ok((Box::new(screen), Setup { width: 236, height: 230, hide_inventory: false, own_labels: true }))
}

impl MekScreen for Oredictionificator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 236.0, 230.0);

        // Filter list holder at (9, 17, 204, 68)
        element_holder(c, [9.0, 17.0, 204.0, 68.0]);
        let filter_count = (data(2).max(0) as usize).min(3);
        for i in 0..3 {
            let row_rect = [10.0, 18.0 + 22.0 * i as f32, 202.0, 22.0];
            let hovered = inside(mouse, row_rect);
            let sprite = if hovered { "mekanism:button_focus" } else { "mekanism:button" };
            c.blit_sprite(sprite, row_rect[0], row_rect[1], row_rect[2], row_rect[3]);

            if i < filter_count {
                let is_on = data(3 + i) == 1;
                let color = if is_on { GREEN } else { 0xFFFF_5555 };
                let tag = string(2 * i);
                let res = string(2 * i + 1);
                c.text(if tag.is_empty() { "#tag" } else { &tag }, 14.0, 21.0 + 22.0 * i as f32, color, 0, 0.85);
                c.text(if res.is_empty() { "result" } else { &res }, 14.0, 31.0 + 22.0 * i as f32, TITLE, 0, 0.85);
                c.area(7700 + i as i32, row_rect, AreaKind::Button, true, "Toggle filter");
                if hovered {
                    c.tooltip(&[("Click to toggle. Shift-click for next result.", WHITE)]);
                }
            }
        }

        // Scroll bar at (213, 17, 14, 90)
        // NÃO FEITO: janela modal GuiOredictionificatorFilter e busca de tags no SDK/servidor (o port usa campo de texto inline)
        element_holder(c, [213.0, 17.0, 14.0, 90.0]);
        c.blit("mekanism:gui/scroll_bar", 214.0, 18.0, 12.0, 0.0, 12.0, 15.0, 24.0, 15.0);

        // New filter holder at (9, 85, 204, 22) and textfield
        element_holder(c, [9.0, 85.0, 204.0, 22.0]);
        c.text("New filter: type a tag (e.g. c:ores/iron) and enter", 14.0, 75.0, 0xFF80_8080, 0, 0.75);
        c.field(7790, [12.0, 88.0, 198.0, 16.0], 100, FieldFilter::Id, &self.new_filter_input, 0, WHITE);

        // Large progress arrow at (94, 119, 48, 8)
        c.blit_sprite("mekanism:progress/large_right/empty", 94.0, 119.0, 48.0, 8.0);
        if data(0) == 1 {
            c.blit("mekanism:gui/progress/large_right/full", 94.0, 119.0, 0.0, 0.0, 48.0, 8.0, 48.0, 8.0);
        }
        if inside(mouse, [94.0, 119.0, 48.0, 8.0]) {
            c.tooltip(&[("Converting", WHITE)]);
        }

        // Input slot at (56, 115) -> frame at (55, 114)
        slot_frame(c, 55.0, 114.0);
        // Output slot at (164, 115) -> frame at (163, 114)
        slot_frame(c, 163.0, 114.0);

        // Title and inventory label (inv at x=38, y=148)
        title(c, &host::translate("block.mekanism.oredictionificator", &[]), 0.0, 236.0, 6.0);
        c.draw_string(&host::translate("container.inventory", &[]), 38.0, 136.0, TITLE, false);

        // Chrome (tabs on right side at x = 236)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        match event {
            InputEvent::FieldSubmitted(f) if f.id == 7790 => {
                host::send_text(7790, &f.text);
                self.new_filter_input.clear();
                true
            }
            InputEvent::FieldChanged(f) if f.id == 7790 => {
                self.new_filter_input = f.text.clone();
                true
            }
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left {
                    let filter_count = (data(2).max(0) as usize).min(3);
                    for i in 0..filter_count {
                        let row_rect = [10.0, 18.0 + 22.0 * i as f32, 202.0, 22.0];
                        if inside((p.x, p.y), row_rect) {
                            let id = if p.modifiers.shift { 7740 + i as i32 } else { 7700 + i as i32 };
                            host::send_button(id);
                            host::play_click();
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{FieldText, Modifiers, MouseButton, Pointer};

    #[test]
    fn oredictionificator_renders_and_handles_filter_click_and_field() {
        let mut d = vec![0; 10];
        d[0] = 1; // did_process
        d[1] = 0; // redstone
        d[2] = 2; // 2 filters
        d[3] = 1; // filter 0 on
        d[4] = 0; // filter 1 off
        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "c:ores/copper".into(), "minecraft:copper_ingot".into(),
                "c:ores/iron".into(), "minecraft:iron_ingot".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:oredictionificator".into(),
            gui_width: 236.0,
            gui_height: 230.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open oredictionificator");
        assert_eq!(setup.width, 236);
        assert_eq!(setup.height, 230);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "c:ores/copper")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Blit { texture, .. } if texture == "mekanism:gui/scroll_bar")));

        // Submit new filter field
        let field_event = InputEvent::FieldSubmitted(FieldText { id: 7790, text: "c:ores/tin".into() });
        assert!(screen.input(&field_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["text 7790 \"c:ores/tin\"".to_string()]);

        // Click filter row 0
        let click_event = InputEvent::MousePressed(Pointer {
            x: 20.0,
            y: 25.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent2 = host::take_sent();
        assert_eq!(sent2, vec!["button 7700".to_string(), "click".to_string()]);
    }
}
