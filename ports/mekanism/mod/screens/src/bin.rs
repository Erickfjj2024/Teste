//! Bins (`mekanism:<tier>_bin`, 176 by 166): an input slot at (44, 34) and output stack at (116, 34),
//! the stored item count and capacity between them at (62, 38) and (62, 48), and toggle buttons for
//! "Eject down" (7700) at (8, 56) and "Lock item" (7701) at (98, 56).
//!
//! Java: In original Mekanism 1.21.1 Java, `TileEntityBin` has NO GUI (blocks are operated in-world
//! by clicking / punching). This screen was created for the port in `port/tools/convert.py` (`bin_menu`)
//! and `port/mod/src/bin.rs`.
//!
//! Server data (`bin.rs`):
//! - `data(0)`: `count & 0xFFFF` (lower 16 bits)
//! - `data(1)`: `(count >> 16) & 0xFFFF` (upper 16 bits)
//! - `data(2)`: `capacity & 0xFFFF` (lower 16 bits)
//! - `data(3)`: `(capacity >> 16) & 0xFFFF` (upper 16 bits)
//! - `data(4)`: `active` (eject down toggle: 1 on, 0 off)
//! - `data(5)`: `locked` (lock item toggle: 1 on, 0 off)
//!
//! Action buttons:
//! - 7700: Toggle active (`TOGGLE_ACTIVE`, Eject down)
//! - 7701: Toggle lock (`TOGGLE_LOCK`, Lock item)
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, TITLE, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const GREEN: u32 = 0xFF00_CD00;
const RED: u32 = 0xFFCD_0000;

pub const TOGGLE_ACTIVE: i32 = 7700;
pub const TOGGLE_LOCK: i32 = 7701;

pub struct BinScreen {
    menu_name: String,
}

pub fn open(info: &OpenInfo) -> Opened {
    let screen = BinScreen {
        menu_name: info.menu.clone(),
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl BinScreen {
    fn title_text(&self) -> String {
        let key = match self.menu_name.as_str() {
            "mekanism:basic_bin" => "block.mekanism.basic_bin",
            "mekanism:advanced_bin" => "block.mekanism.advanced_bin",
            "mekanism:elite_bin" => "block.mekanism.elite_bin",
            "mekanism:ultimate_bin" => "block.mekanism.ultimate_bin",
            "mekanism:creative_bin" => "block.mekanism.creative_bin",
            _ => "block.mekanism.basic_bin",
        };
        let translated = host::translate(key, &[]);
        if !translated.is_empty() && translated != key {
            translated
        } else {
            let sub = self.menu_name.strip_prefix("mekanism:").unwrap_or(&self.menu_name);
            sub.replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

impl MekScreen for BinScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Title and Inventory labels
        title(c, &self.title_text(), 0.0, 176.0, 5.0);
        inventory_label(c, 8.0, 73.0);

        // Slot frames: input at (44, 34) frame at (43, 33); output at (116, 34) frame at (115, 33)
        slot_frame(c, 43.0, 33.0);
        slot_frame(c, 115.0, 33.0);

        // "In" and "Out" labels above slots
        c.text("In", 38.0, 22.0, TITLE, 0, 0.8);
        c.text("Out", 112.0, 22.0, TITLE, 0, 0.8);

        // Stored count and capacity
        let count = pair(0, 1);
        let capacity = pair(2, 3);
        let count_str = format!("{count}");
        c.text(&count_str, 62.0, 38.0, GREEN, 0, 1.0);

        let cap_str = if self.menu_name.contains("creative") {
            "of Infinite".to_string()
        } else {
            format!("of {capacity}")
        };
        c.text(&cap_str, 62.0, 48.0, GREEN, 0, 0.8);

        // Eject down button at (8, 56, 70, 14)
        let eject_rect = [8.0, 56.0, 70.0, 14.0];
        let eject_hover = inside(mouse, eject_rect);
        let eject_sprite = if eject_hover { "mekanism:button_focus" } else { "mekanism:button" };
        c.blit_sprite(eject_sprite, 8.0, 56.0, 70.0, 14.0);
        c.text("Eject down", 12.0, 59.0, TITLE, 0, 1.0);
        let eject_active = data(4) == 1;
        if eject_active {
            c.text("ON", 62.0, 59.0, GREEN, 0, 1.0);
        } else {
            c.text("off", 62.0, 59.0, RED, 0, 1.0);
        }
        c.area(TOGGLE_ACTIVE, eject_rect, AreaKind::Button, true, "Eject down");
        if eject_hover {
            c.tooltip(&[("Eject down", WHITE)]);
        }

        // Lock item button at (98, 56, 70, 14)
        let lock_rect = [98.0, 56.0, 70.0, 14.0];
        let lock_hover = inside(mouse, lock_rect);
        let lock_sprite = if lock_hover { "mekanism:button_focus" } else { "mekanism:button" };
        c.blit_sprite(lock_sprite, 98.0, 56.0, 70.0, 14.0);
        c.text("Lock item", 102.0, 59.0, TITLE, 0, 1.0);
        let lock_active = data(5) == 1;
        if lock_active {
            c.text("ON", 152.0, 59.0, GREEN, 0, 1.0);
        } else {
            c.text("off", 152.0, 59.0, RED, 0, 1.0);
        }
        c.area(TOGGLE_LOCK, lock_rect, AreaKind::Button, true, "Lock item");
        if lock_hover {
            c.tooltip(&[("Lock item", WHITE)]);
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) if p.button == MouseButton::Left => {
                if inside((p.x, p.y), [8.0, 56.0, 70.0, 14.0]) {
                    host::send_button(TOGGLE_ACTIVE);
                    host::play_click();
                    return true;
                }
                if inside((p.x, p.y), [98.0, 56.0, 70.0, 14.0]) {
                    host::send_button(TOGGLE_LOCK);
                    host::play_click();
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
    fn bin_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                100, // count lo
                0,   // count hi
                0,   // cap lo (4096 = 0x1000)
                0x1000 >> 16,
                1,   // active
                0,   // locked
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:basic_bin".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open bin");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);
        assert!(!setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 20.0,
            mouse_y: 60.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        let commands = decoded.commands;

        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "100")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "In")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Out")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Eject down")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Lock item")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "ON")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "off")));

        // Click Eject down
        let click_eject = InputEvent::MousePressed(Pointer {
            x: 20.0,
            y: 60.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_eject));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7700".to_string(), "click".to_string()]);

        // Click Lock item
        let click_lock = InputEvent::MousePressed(Pointer {
            x: 120.0,
            y: 60.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_lock));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7701".to_string(), "click".to_string()]);
    }
}
