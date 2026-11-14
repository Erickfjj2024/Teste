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
                format!("{amount} / {capacity} mB")
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
            let tip = match edit_mode {
                1 => "Container Edit Mode: Fill only",
                2 => "Container Edit Mode: Empty only",
                _ => "Container Edit Mode: Both (fill and empty)",
            };
            c.tooltip(&[(tip, WHITE)]);
        }

        // Security Tab at (176, 6, 26, 18)
        if let Some(sec_idx) = self.security_idx {
            let sec_rect = [176.0, 6.0, 26.0, 18.0];
            c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
            let sec_mode = (data(sec_idx).max(0) as usize) % 3;
            let icon_sprite = match sec_mode {
                1 => "mekanism:security/private",
                2 => "mekanism:security/trusted",
                _ => "mekanism:security/public",
            };
            c.blit_sprite(icon_sprite, 176.0 + 4.0, 6.0, 18.0, 18.0);
            c.area(9100, sec_rect, AreaKind::Tab, true, "Security");
            if inside(mouse, sec_rect) {
                let text = match sec_mode {
                    1 => "Security: Private",
                    2 => "Security: Trusted",
                    _ => "Security: Public",
                };
                c.tooltip(&[(text, WHITE)]);
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
                // Security tab
                if self.security_idx.is_some() && inside(mouse, [176.0, 6.0, 26.0, 18.0]) {
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
    }
}
