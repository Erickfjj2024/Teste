//! `GuiPersonalStorageTile` (`mekanism:personal_chest`, `mekanism:personal_barrel`, 176 by 222):
//! 54 container slots (6 rows of 9 slots at 8, 18), inventory label at y = 128 (imageHeight - 94),
//! and security tab at (176, 34).
//!
//! Java: `GuiPersonalStorageTile.java:10-25`, `GuiPersonalStorageItem.java:11-32`,
//! `MekanismTileContainer<TileEntityPersonalStorage>`, `PersonalStorageManager.java:150-154`.
//!
//! Server data (`personal_storage.rs` / `convert.py` `personal_storage_menu`):
//! - `data(0)`: security mode (0 = Public, 1 = Private, 2 = Trusted, from `security_table.rs`)
//! - `data(1)`: open count
//!
//! Action buttons:
//! - 9100: Security mode next (`SECURITY_NEXT`)
//! - 9101: Security mode previous (`SECURITY_PREV`)
use crate::elements::{base_panel, data, inside, inventory_label, slot_frame, title, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const SECURITY_NEXT: i32 = 9100;
pub const SECURITY_PREV: i32 = 9101;
pub const TAB_SECURITY: u32 = 0xFFFF_A160;

pub struct PersonalStorageScreen {
    menu_name: String,
}

pub fn open(info: &OpenInfo) -> Opened {
    let screen = PersonalStorageScreen {
        menu_name: info.menu.clone(),
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 176,
            height: 222,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl PersonalStorageScreen {
    fn title_text(&self) -> String {
        let key = if self.menu_name == "mekanism:personal_barrel" {
            "block.mekanism.personal_barrel"
        } else {
            "block.mekanism.personal_chest"
        };
        let translated = host::translate(key, &[]);
        if !translated.is_empty() && translated != key {
            translated
        } else if self.menu_name == "mekanism:personal_barrel" {
            "Personal Barrel".to_string()
        } else {
            "Personal Chest".to_string()
        }
    }
}

impl MekScreen for PersonalStorageScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 222.0);

        // Title and Inventory labels (inventory label at y = 128 as in Java imageHeight - 94)
        title(c, &self.title_text(), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 128.0);

        // 54 container slot frames (6 rows of 9 columns at 7 + 18*x, 17 + 18*y)
        for y in 0..6 {
            for x in 0..9 {
                slot_frame(c, 7.0 + 18.0 * x as f32, 17.0 + 18.0 * y as f32);
            }
        }

        // Security tab at (176, 34, 26, 18)
        let sec_rect = [176.0, 34.0, 26.0, 18.0];
        let sec_mode = (data(0).max(0) as usize) % 3;
        c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
        let sec_sprite = match sec_mode {
            1 => "mekanism:security/private",
            2 => "mekanism:security/protected",
            _ => "mekanism:security/public",
        };
        c.blit_sprite(sec_sprite, 180.0, 34.0, 18.0, 18.0);
        c.area(SECURITY_NEXT, sec_rect, AreaKind::Tab, true, "Security");

        if inside(mouse, sec_rect) {
            let tip = match sec_mode {
                1 => "Security: Private",
                2 => "Security: Trusted",
                _ => "Security: Public",
            };
            c.tooltip(&[(tip, WHITE)]);
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                if inside((p.x, p.y), [176.0, 34.0, 26.0, 18.0]) {
                    if p.button == MouseButton::Right {
                        host::send_button(SECURITY_PREV);
                    } else {
                        host::send_button(SECURITY_NEXT);
                    }
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
    fn personal_storage_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                0, // security mode (public)
                1, // open count
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:personal_chest".into(),
            gui_width: 176.0,
            gui_height: 222.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open personal chest");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 222);
        assert!(!setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 50.0,
            mouse_y: 50.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        let commands = decoded.commands;

        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:security/public")));

        // Count slot frames: 54 container slots
        let slot_frames = commands
            .iter()
            .filter(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:slot/normal"))
            .count();
        assert_eq!(slot_frames, 54);

        // Click security tab
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }
}
