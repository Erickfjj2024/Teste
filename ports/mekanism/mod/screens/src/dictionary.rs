//! `GuiDictionary` (`mekanism:dictionary`, 176 by 171): the target slot at (6, 6) with its frame at (5, 5), the title
//! centered with offset from the target slot, the item info text at (28, 17), the inner screen at (7, 29) holding the
//! scrollable list of tags (162 by 42, 4 rows of 10px), the inventory label at (8, 75), the right-aligned "Tag Type" label,
//! and the Tag Type dropdown at (113, 73) displaying "Item".
//! Java: `GuiDictionary.java` (lines 30-114), `DictionaryContainer.java`.
//! Server data (`item_menus.rs`): `data[0]` is how many tags, `string(0)` is what is shown, `string(1..)` are the sorted tags.
//!
//! Differences from Java:
//! - In Java, `GuiDictionary` has a dropdown with other tag types (block, fluid, chemical...), but the server port only supports Item tags (`item_menus.rs`).
//! - In Java, the target slot is a ghost target on the client, whereas in the port it is container slot 0.
use crate::elements::{GREEN, TITLE, WHITE, base_panel, data, inner_screen, inside, inventory_label, slot_frame, string, title};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::widgets::ScrollList;
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, TEXT_RIGHT, host};

pub struct Dictionary {
    scroll_list: ScrollList,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let screen = Dictionary {
        scroll_list: ScrollList::new(7, 29, 162, 42, 10),
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 176,
            height: 171,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for Dictionary {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 171.0);

        // Slot frame for the target item at (5, 5)
        slot_frame(c, 5.0, 5.0);

        // Title centered between target slot (x = 22) and right edge (x = 176) at y = 5
        title(c, &host::translate("item.mekanism.dictionary", &[]), 22.0, 154.0, 5.0);

        // Target item info header (string 0) at (28, 17)
        let header = string(0);
        if !header.is_empty() {
            c.text(&header, 28.0, 17.0, TITLE, 0, 0.75);
        }

        // Tags scroll list at (7, 29, 162, 42)
        inner_screen(c, [7.0, 29.0, 162.0, 42.0]);
        let total_tags = data(0).max(0) as usize;
        self.scroll_list.set_rows(total_tags);
        self.scroll_list.scroll.draw_bar(c, "mekanism:gui/scroll_list", 6, 6);
        self.scroll_list.draw_selection(c, total_tags);

        for (idx, x, y) in self.scroll_list.shown(total_tags) {
            let tag_name = string(1 + idx);
            c.text(&tag_name, x + 3.0, y + 1.0, GREEN, 0, 0.85);
            c.area(
                7000 + idx as i32,
                [x, y, 154.0, 10.0],
                AreaKind::Row,
                true,
                &tag_name,
            );
        }

        // Inventory label at (8, 75)
        inventory_label(c, 8.0, 75.0);

        // Right-aligned "Tag Type:" label before dropdown (at 108, y = 75)
        let tag_type_label = host::translate("dictionary.mekanism.tag_type", &[]);
        c.text(&tag_type_label, 108.0, 75.0, TITLE, TEXT_RIGHT, 1.0);

        // Dropdown button at (113, 73, 56, 14) showing "Item"
        let dropdown_rect = [113.0, 73.0, 56.0, 14.0];
        let hovered = inside(mouse, dropdown_rect);
        let button_sprite = if hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(button_sprite, dropdown_rect[0], dropdown_rect[1], dropdown_rect[2], dropdown_rect[3]);
        let item_text = host::translate("dictionary.mekanism.item", &[]);
        c.text(
            &item_text,
            dropdown_rect[0] + dropdown_rect[2] / 2.0,
            dropdown_rect[1] + (dropdown_rect[3] - 8.0) / 2.0,
            WHITE,
            horizon_mod_sdk::screen::TEXT_CENTER,
            1.0,
        );
        c.area(7050, dropdown_rect, AreaKind::Button, true, &item_text);
        if hovered {
            let tip = host::translate("dictionary.mekanism.item.desc", &[]);
            c.tooltip(&[(&tip, WHITE)]);
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left {
                    let total_tags = data(0).max(0) as usize;
                    if let Some(sel) = self.scroll_list.press(p.x as f64, p.y as f64, total_tags) {
                        if let Some(row) = sel {
                            host::send_button(7000 + row as i32);
                        }
                        host::play_click();
                        return true;
                    }
                    if inside((p.x, p.y), [113.0, 73.0, 56.0, 14.0]) {
                        host::play_click();
                        return true;
                    }
                }
                false
            }
            InputEvent::MouseDragged(d) => {
                if self.scroll_list.scroll.dragging {
                    let total_tags = data(0).max(0) as usize;
                    self.scroll_list.drag(d.y as f64, total_tags);
                    return true;
                }
                false
            }
            InputEvent::MouseReleased(_) => {
                if self.scroll_list.scroll.dragging {
                    self.scroll_list.release();
                    return true;
                }
                false
            }
            InputEvent::MouseScrolled(w) => {
                let total_tags = data(0).max(0) as usize;
                if self.scroll_list.wheel(w.x as f64, w.y as f64, w.delta as f64, total_tags) {
                    return true;
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
    use horizon_mod_sdk::screen::{Modifiers, Pointer};

    #[test]
    fn dictionary_renders_and_handles_clicks() {
        host::set_menu(host::Menu {
            data: vec![2],
            strings: vec![
                "minecraft:iron_ingot: 2 tags".into(),
                "#c:ingots".into(),
                "#c:ingots/iron".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:dictionary".into(),
            gui_width: 176.0,
            gui_height: 171.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open dictionary");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 171);
        assert!(!setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 10.0,
            mouse_y: 10.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        let commands = decoded.commands;
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "minecraft:iron_ingot: 2 tags")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "#c:ingots")));

        // Test click on a tag row
        let click_event = InputEvent::MousePressed(Pointer {
            x: 20.0,
            y: 35.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7000".to_string(), "click".to_string()]);
    }
}
