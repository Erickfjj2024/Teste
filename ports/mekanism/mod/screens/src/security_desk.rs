//! `GuiSecurityDesk` (`mekanism:security_desk`, 176 by 230): the title at (0, 4), the trusted players list at (13, 13,
//! 122 by 42), the "Trusted Players" subheading at (74, 57), the Add player text field at (35, 68, 99 by 11), the Remove
//! button at (13, 81, 122 by 20), the security mode status line at (13, 103), the three mode buttons (Public, Private,
//! Trusted) at (13, 113), (54, 113), and (95, 113), the three element holders on the right for lock slot, override button
//! with security light, and unlock slot, and the player inventory at (8, 148).
//! Java: `GuiSecurityDesk.java` (lines 40-195), `TileEntitySecurityDesk.java`.
//! Server data (`security_desk.rs`):
//! - `data[0]`: mode (0 public, 1 private, 2 trusted)
//! - `data[1]`: override (1 on, 0 off)
//! - `data[2]`: viewer is owner (1 yes, 0 no)
//! - `data[3]`: count of trusted names
//! - `data[4]`: selected row (-1 none)
//! - `data[5]`: security mode for security tab
//! Strings: 0..data[3] trusted player names.
//!
//! Differences from Java:
//! - In Java, the lock and unlock slots manage item security frequencies and ownership; in the port, item ownership is not tracked.
//! - Security override button and light indicate global security frequency override status for all owner machines.
use crate::elements::{GREEN, SUBHEADING, TITLE, WHITE, base_panel, button, data, element_holder, inner_screen, inside, inventory_label, slot_frame, string, title};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, FieldFilter, Frame, InputEvent, MouseButton, OpenInfo, Setup, TEXT_CENTER, TEXT_RIGHT, host};

pub const ROW_BASE: i32 = 9200;
pub const REMOVE: i32 = 9270;
pub const OVERRIDE: i32 = 9271;
pub const MODE_PUBLIC: i32 = 9272;
pub const MODE_PRIVATE: i32 = 9273;
pub const MODE_TRUSTED: i32 = 9274;
pub const NAME_BOX: i32 = 9275;

pub struct SecurityDesk;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(SecurityDesk),
        Setup {
            width: 176,
            height: 230,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for SecurityDesk {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 230.0);

        // Title centered at y = 4
        title(c, &host::translate("block.mekanism.security_desk", &[]), 0.0, 176.0, 4.0);

        // Inventory label at (8, 136)
        inventory_label(c, 8.0, 136.0);

        let is_owner = data(2) != 0;
        let mode = data(0);
        let overridden = data(1) != 0;
        let trusted_count = data(3).max(0) as usize;
        let selected_row = data(4);

        // Right side element holders and slots
        // Top holder (141, 13, 26, 37): lock slot at (146, 18) and Public icon
        element_holder(c, [141.0, 13.0, 26.0, 37.0]);
        slot_frame(c, 145.0, 17.0);
        c.blit("mekanism:gui/public", 145.0, 32.0, 0.0, 0.0, 18.0, 18.0, 18.0, 18.0);

        // Middle holder (141, 54, 26, 34): override button (146, 59, 16, 16) and security light
        element_holder(c, [141.0, 54.0, 26.0, 34.0]);
        let override_rect = [146.0, 59.0, 16.0, 16.0];
        let override_hovered = is_owner && inside(mouse, override_rect);
        let override_sprite = if !is_owner {
            "mekanism:button_inactive"
        } else if override_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(override_sprite, override_rect[0], override_rect[1], override_rect[2], override_rect[3]);
        c.blit("mekanism:gui/button/exclamation", 146.0, 59.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
        let override_tip = if overridden {
            "Security override: On"
        } else {
            "Security override: Off"
        };
        c.area(OVERRIDE, override_rect, AreaKind::Button, is_owner, override_tip);
        if override_hovered {
            c.tooltip(&[(override_tip, WHITE)]);
        }

        // Security light indicator at (144, 77): green if overridden, red if not, yellow if not owner
        let light_color = if !is_owner {
            0xFFEE_EE00 // yellow
        } else if overridden {
            0xFF00_AA00 // green
        } else {
            0xFFCD_0000 // red
        };
        c.fill(150.0, 78.0, 8.0, 4.0, light_color);

        // Bottom holder (141, 92, 26, 37): unlock slot at (146, 97) and Private icon
        element_holder(c, [141.0, 92.0, 26.0, 37.0]);
        slot_frame(c, 145.0, 96.0);
        c.blit("mekanism:gui/private", 145.0, 111.0, 0.0, 0.0, 18.0, 18.0, 18.0, 18.0);

        // Left side: Trusted list at (13, 13, 122, 42)
        inner_screen(c, [13.0, 13.0, 122.0, 42.0]);
        for row in 0..trusted_count.min(4) {
            let y = 14.0 + 10.0 * row as f32;
            let text = string(row);
            if selected_row == row as i32 {
                c.fill(14.0, y, 120.0, 10.0, 0x403C_FE9A);
            }
            c.text(&text, 16.0, y + 1.0, GREEN, 0, 0.85);
            c.area(
                ROW_BASE + row as i32,
                [14.0, y, 120.0, 10.0],
                AreaKind::Row,
                is_owner,
                &text,
            );
        }

        // Subheading "Trusted Players" at (74, 57)
        let trusted_sub = host::translate("gui.mekanism.security.trusted_players", &[]);
        c.text(&trusted_sub, 74.0, 57.0, SUBHEADING, TEXT_CENTER, 0.85);

        // Text field "Add:" at (35, 68, 99, 11)
        let add_label = host::translate("gui.mekanism.security.add", &[]);
        c.text(&add_label, 33.0, 69.0, TITLE, TEXT_RIGHT, 1.0);
        inner_screen(c, [35.0, 68.0, 99.0, 11.0]);
        c.field(NAME_BOX, [36.0, 69.0, 97.0, 9.0], 16, FieldFilter::Any, "", 1, WHITE);

        // Remove button at (13, 81, 122, 20)
        let can_remove = is_owner && selected_row >= 0;
        let remove_label = host::translate("gui.mekanism.remove", &[]);
        button(
            c,
            REMOVE,
            [13.0, 81.0, 122.0, 20.0],
            &remove_label,
            can_remove,
            mouse,
            Some("Remove the selected player"),
        );

        // Security mode text at (13, 103)
        let mode_desc = match mode {
            0 => "Security: Public",
            1 => "Security: Private",
            _ => "Security: Trusted",
        };
        c.text(mode_desc, 13.0, 103.0, TITLE, 0, 0.85);

        // Three mode buttons at (13, 113), (54, 113), and (95, 113), 40 by 16
        for (k, (id, m_val, icon, tip)) in [
            (MODE_PUBLIC, 0, "public", "Public"),
            (MODE_PRIVATE, 1, "private", "Private"),
            (MODE_TRUSTED, 2, "trusted", "Trusted"),
        ]
        .into_iter()
        .enumerate()
        {
            let bx = 13.0 + 41.0 * k as f32;
            let rect = [bx, 113.0, 40.0, 16.0];
            let active = is_owner && mode != m_val;
            let hovered = active && inside(mouse, rect);
            let sprite = if !active {
                "mekanism:button_inactive"
            } else if hovered {
                "mekanism:button_focus"
            } else {
                "mekanism:button"
            };
            c.blit_sprite(sprite, rect[0], rect[1], rect[2], rect[3]);
            c.blit(
                &format!("mekanism:gui/button/{icon}"),
                rect[0] - 1.0,
                rect[1],
                0.0,
                0.0,
                40.0,
                16.0,
                40.0,
                16.0,
            );
            c.area(id, rect, AreaKind::Button, active, tip);
            if hovered {
                c.tooltip(&[(tip, WHITE)]);
            }
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left {
                    let is_owner = data(2) != 0;
                    let mode = data(0);

                    // Row clicks
                    let trusted_count = data(3).max(0) as usize;
                    for row in 0..trusted_count.min(4) {
                        let y = 14.0 + 10.0 * row as f32;
                        if inside((p.x, p.y), [14.0, y, 120.0, 10.0]) && is_owner {
                            host::send_button(ROW_BASE + row as i32);
                            host::play_click();
                            return true;
                        }
                    }

                    // Remove button
                    if inside((p.x, p.y), [13.0, 81.0, 122.0, 20.0]) && is_owner && data(4) >= 0 {
                        host::send_button(REMOVE);
                        host::play_click();
                        return true;
                    }

                    // Override button
                    if inside((p.x, p.y), [146.0, 59.0, 16.0, 16.0]) && is_owner {
                        host::send_button(OVERRIDE);
                        host::play_click();
                        return true;
                    }

                    // Mode buttons
                    for (k, (id, m_val)) in [(MODE_PUBLIC, 0), (MODE_PRIVATE, 1), (MODE_TRUSTED, 2)]
                        .into_iter()
                        .enumerate()
                    {
                        let bx = 13.0 + 41.0 * k as f32;
                        if inside((p.x, p.y), [bx, 113.0, 40.0, 16.0]) && is_owner && mode != m_val {
                            host::send_button(id);
                            host::play_click();
                            return true;
                        }
                    }
                }
                false
            }
            InputEvent::FieldSubmitted(f) => {
                if f.id == NAME_BOX && data(2) != 0 && !f.text.trim().is_empty() {
                    host::send_text(NAME_BOX, f.text.trim());
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
    use horizon_mod_sdk::screen::{FieldText, Modifiers, Pointer};

    #[test]
    fn security_desk_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                0, // mode Public
                0, // override off
                1, // viewer is owner
                2, // 2 trusted
                0, // selected row 0
                0, // security tab mode
            ],
            strings: vec!["Alice".into(), "Bob".into()],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:security_desk".into(),
            gui_width: 176.0,
            gui_height: 230.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open security_desk");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 230);
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
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Alice")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Bob")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Security: Public")));

        // Test click on Remove button (selection is row 0)
        let remove_click = InputEvent::MousePressed(Pointer {
            x: 20.0,
            y: 90.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&remove_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {REMOVE}"), "click".to_string()]);

        // Test click on Override button
        let override_click = InputEvent::MousePressed(Pointer {
            x: 150.0,
            y: 65.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&override_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {OVERRIDE}"), "click".to_string()]);

        // Test submit text in Name Box
        let submit_event = InputEvent::FieldSubmitted(FieldText {
            id: NAME_BOX,
            text: "Charlie".into(),
        });
        assert!(screen.input(&submit_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("text {NAME_BOX} \"Charlie\"")]);
    }
}
