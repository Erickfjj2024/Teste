//! `GuiDynamicTank` (176 by 166): dynamic tank multiblock valve screen.
//! Java source: `java/main/java/mekanism/client/gui/GuiDynamicTank.java`
//!
//! Elements:
//! - Window dimensions: 176 x 166, titleLabelY = 6, inventoryLabelX = 8, inventoryLabelY = 74
//! - Armor holder at (-26, 62, 26, 98) on the left
//! - Element holder at (141, 16, 26, 56) with input slot at (145, 20), output slot at (145, 50), down arrow at (150, 39)
//! - Medium merged gauge at (7, 16, 34, 56): amount pair (0, 1), chemical index 2, capacity pair (3, 4)
//! - Inner screen at (49, 21, 84, 46): 4 text lines (strings 0..3: name/empty, amount, capacity)
//! - Container edit mode tab at (176, 138, 26, 18): mode data(5) (0 both, 1 fill, 2 empty), button 5100 (alt 5101)
//! - Security tab at (176, 6, 26, 18): data(6) from security_table.rs
//!
//! Differences from Java / Não Feito:
//! - // NÃO FEITO: GuiWarningTab (sistema de alertas de tanque) não suportado no servidor
use crate::elements::{self, base_panel, data, inside, inventory_label, pair, string, title, GREEN, WHITE};
use crate::security_table;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const TAB_CONTAINER_EDIT_MODE: u32 = 0xFF36_6BD0;
const TAB_SECURITY: u32 = 0xFFFF_A160;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(DynamicTankScreen::new()),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct DynamicTankScreen {
    security_idx: Option<usize>,
}

impl DynamicTankScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:dynamic_valve")
            .map(|(_, idx)| *idx);
        Self { security_idx }
    }
}

impl MekScreen for DynamicTankScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Armor holder at left: (-26, 62, 26, 98)
        c.blit_sprite("mekanism:holder_left", -26.0, 62.0, 26.0, 98.0);

        // Title and inventory label
        title(c, &host::translate("gui.mekanism.dynamic_tank", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 74.0);

        // Element holder at (141, 16, 26, 56)
        elements::element_holder(c, [141.0, 16.0, 26.0, 56.0]);
        // Input slot at (145, 20) with input overlay
        c.blit_sprite("mekanism:slot/normal", 145.0, 20.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/input", 145.0, 20.0, 18.0, 18.0);
        // Output slot at (145, 50) with output overlay
        c.blit_sprite("mekanism:slot/normal", 145.0, 50.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/output", 145.0, 50.0, 18.0, 18.0);
        // Down arrow at (150, 39)
        c.blit_sprite("mekanism:arrow/down", 150.0, 39.0, 8.0, 8.0);

        // Medium Gauge at (7, 16, 34, 56), inner (8, 17, 32, 54)
        c.blit_sprite("mekanism:gauge/normal", 7.0, 16.0, 34.0, 56.0);
        let amount = pair(0, 1);
        let capacity = pair(3, 4);
        let inner = [8.0, 17.0, 32.0, 54.0];
        let fill = if capacity == 0 {
            0.0
        } else {
            (amount as f32 / capacity as f32).clamp(0.0, 1.0)
        };
        if fill > 0.0 {
            c.fluid("mekanism:liquid/liquid", inner, fill, 0xFF3F_76E4);
        }
        c.blit_sprite("mekanism:gauge/overlay/medium", inner[0], inner[1], inner[2], inner[3]);
        if inside(mouse, [7.0, 16.0, 34.0, 56.0]) {
            let tip = if amount == 0 {
                host::translate("gui.mekanism.empty", &[])
            } else {
                host::translate("generic.mekanism.stored.mb", &["Fluid".to_string(), amount.to_string()])
            };
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Inner screen at (49, 21, 84, 46)
        elements::inner_screen(c, [49.0, 21.0, 84.0, 46.0]);
        for k in 0..4 {
            let s = string(k);
            if !s.is_empty() {
                c.text(&s, 52.0, 24.0 + 10.0 * k as f32, GREEN, 0, 0.85);
            }
        }

        // Container Edit Mode Tab at (176, 138, 26, 18)
        let edit_rect = [176.0, 138.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", edit_rect, TAB_CONTAINER_EDIT_MODE);
        let edit_mode = (data(5).max(0) as usize) % 3;
        let mode_sprite = match edit_mode {
            1 => "mekanism:container_edit_mode/fill",
            2 => "mekanism:container_edit_mode/empty",
            _ => "mekanism:container_edit_mode/both",
        };
        c.blit_sprite(mode_sprite, 176.0 + 4.0, 138.0, 18.0, 18.0);
        c.area(5100, edit_rect, AreaKind::Tab, true, "Container Edit Mode");
        if inside(mouse, edit_rect) {
            let key = match edit_mode {
                1 => "fluid_container.mekanism.fill",
                2 => "fluid_container.mekanism.empty",
                _ => "fluid_container.mekanism.both",
            };
            let tip = host::translate(key, &[]);
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Security Tab at (176, 34, 26, 18) - Java GuiSecurityTab defaults to y = 34
        if let Some(sec_idx) = self.security_idx {
            let sec_rect = [176.0, 34.0, 26.0, 18.0];
            c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
            let sec_mode = (data(sec_idx).max(0) as usize) % 3;
            let icon_sprite = match sec_mode {
                1 => "mekanism:security/private",
                2 => "mekanism:security/protected",
                _ => "mekanism:security/public",
            };
            c.blit_sprite(icon_sprite, 176.0 + 4.0, 34.0, 18.0, 18.0);
            c.area(9100, sec_rect, AreaKind::Tab, true, "Security");
            if inside(mouse, sec_rect) {
                let mode_key = match sec_mode {
                    1 => "security.mekanism.private",
                    2 => "security.mekanism.trusted",
                    _ => "security.mekanism.public",
                };
                let text = host::translate("tooltip.mekanism.security", &[host::translate(mode_key, &[])]);
                c.tooltip(&[(&text, WHITE)]);
            }
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                let mouse = (p.x, p.y);
                // Container edit mode tab: left click 5100, right click 5101
                if inside(mouse, [176.0, 138.0, 26.0, 18.0]) {
                    if p.button == MouseButton::Right {
                        host::send_button(5101);
                    } else {
                        host::send_button(5100);
                    }
                    host::play_click();
                    return true;
                }
                // Security tab at y = 34
                if self.security_idx.is_some() && inside(mouse, [176.0, 34.0, 26.0, 18.0]) {
                    if p.button == MouseButton::Right {
                        host::send_button(9101);
                    } else {
                        host::send_button(9100);
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
    use horizon_mod_sdk::screen::{
        format::Command, host, InputEvent, Modifiers, MouseButton, OpenInfo, Pointer,
    };

    #[test]
    fn test_dynamic_tank_render_and_input() {
        let mut d = vec![0; 10];
        // amount: 16000
        d[0] = 16000;
        // capacity: 32000
        d[3] = 32000;
        // edit mode: 1 (fill)
        d[5] = 1;
        // security mode: 0 (public) at index 6
        d[6] = 0;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "Water:".into(),
                "16,000 mB".into(),
                "Capacity:".into(),
                "32,000 mB".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:dynamic_valve".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open dynamic tank");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Water:")));

        // Test click on edit mode tab
        let click_edit = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 140.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_edit));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 5100".to_string(), "click".to_string()]);

        // Test click on security tab at y = 34..52
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 38.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent_sec = host::take_sent();
        assert_eq!(sent_sec, vec!["button 9100".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_dynamic_tank_empty_gauge_and_protected_security() {
        let mut d = vec![0; 10];
        // amount: 0 (empty)
        d[0] = 0;
        d[3] = 32000;
        // edit mode: 0 (both)
        d[5] = 0;
        // security mode: 2 (trusted -> protected) at index 6
        d[6] = 2;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:dynamic_valve".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, _) = open(&info).expect("open screen");

        // Hover over gauge at (10, 20) -> should show empty tooltip
        let frame = Frame {
            mouse_x: 10.0,
            mouse_y: 20.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let decoded = horizon_mod_sdk::screen::format::decode(&canvas.finish()).expect("decode");

        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "gui.mekanism.empty"))));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:security/protected")));

        // Hover over container edit mode tab at (180, 140) -> should show fluid_container.mekanism.both
        let frame_edit = Frame {
            mouse_x: 180.0,
            mouse_y: 140.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas2 = Canvas::new();
        screen.render(&frame_edit, &mut canvas2);
        let decoded2 = horizon_mod_sdk::screen::format::decode(&canvas2.finish()).expect("decode");
        assert!(decoded2.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "fluid_container.mekanism.both"))));
    }
}
