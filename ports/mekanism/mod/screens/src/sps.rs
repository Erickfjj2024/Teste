//! `GuiSPS` (176 by 182): supercritical phase shifter multiblock port screen.
//! Java source: `java/main/java/mekanism/client/gui/GuiSPS.java`
//!
//! Elements:
//! - Window dimensions: 176 x 182 (`imageHeight += 16`), titleLabelY = 5, inventoryLabelX = 8, inventoryLabelY = 90
//! - Two standard chemical gauges (18x60):
//!   - Input (Polonium) at (7, 17): pair (0, 1), capacity pair (2, 3), chem index 8
//!   - Output (Antimatter) at (151, 17): pair (4, 5), capacity pair (6, 7), chem index 9
//! - Inner screen at (27, 17, 122, 60): strings 0..2 (Status, Energy Input, Process Rate)
//! - Dynamic horizontal rate bar at (7, 79, 162, 10), inner (8, 80, 160, 8): percentage data(10), purple gradient
//! - Security tab at (176, 6, 26, 18): data(15) from security_table.rs
//!
//! Differences from Java / Não Feito:
//! - // NÃO FEITO: GuiWarningTab não suportada no servidor
use crate::elements::{self, base_panel, data, inside, inventory_label, pair, string, title, GREEN, WHITE};
use crate::security_table;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const TAB_SECURITY: u32 = 0xFFFF_A160;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(SPSScreen::new()),
        Setup {
            width: 176,
            height: 182,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct SPSScreen {
    security_idx: Option<usize>,
}

impl SPSScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:sps_port")
            .map(|(_, idx)| *idx);
        Self { security_idx }
    }

    fn render_gauge(
        &self,
        c: &mut Canvas,
        mouse: (f32, f32),
        rect: [f32; 4],
        amount: u64,
        capacity: u64,
        label: &str,
        fluid_sprite: &str,
        tint: u32,
    ) {
        c.blit_sprite("mekanism:gauge/normal", rect[0], rect[1], rect[2], rect[3]);
        let inner = [rect[0] + 1.0, rect[1] + 1.0, rect[2] - 2.0, rect[3] - 2.0];
        let fill = if capacity == 0 {
            0.0
        } else {
            (amount as f32 / capacity as f32).clamp(0.0, 1.0)
        };
        if fill > 0.0 {
            c.fluid(fluid_sprite, inner, fill, tint);
        }
        c.blit_sprite("mekanism:gauge/overlay/standard", inner[0], inner[1], inner[2], inner[3]);
        if inside(mouse, rect) {
            let tip = if amount == 0 {
                host::translate("gui.mekanism.empty", &[])
            } else {
                host::translate("generic.mekanism.stored.mb", &[label.to_string(), amount.to_string()])
            };
            c.tooltip(&[(&tip, WHITE)]);
        }
    }
}

impl MekScreen for SPSScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 182.0);

        // Title and inventory label (titleLabelY = 6 in Minecraft AbstractContainerScreen)
        title(c, &host::translate("block.mekanism.sps_port", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 90.0);

        // 2 Gauges: Input (polonium) at (7, 17) and Output (antimatter) at (151, 17)
        let in_amount = pair(0, 1);
        let in_cap = pair(2, 3);
        let out_amount = pair(4, 5);
        let out_cap = pair(6, 7);

        self.render_gauge(
            c,
            mouse,
            [7.0, 17.0, 18.0, 60.0],
            in_amount,
            in_cap,
            "Polonium",
            "mekanism:liquid/liquid",
            0xFF00_FFFF,
        );
        self.render_gauge(
            c,
            mouse,
            [151.0, 17.0, 18.0, 60.0],
            out_amount,
            out_cap,
            "Antimatter",
            "mekanism:liquid/liquid",
            0xFF8A_2BE2,
        );

        // Inner screen at (27, 17, 122, 60)
        elements::inner_screen(c, [27.0, 17.0, 122.0, 60.0]);
        let s0 = string(0);
        let s1 = string(1);
        let s2 = string(2);
        let status_str = if s0.is_empty() { "Status: Idle".to_string() } else { s0 };
        c.text(&status_str, 30.0, 20.0, GREEN, 0, 0.85);
        if !s1.is_empty() {
            c.text(&s1, 30.0, 31.0, GREEN, 0, 0.85);
        }
        if !s2.is_empty() {
            c.text(&s2, 30.0, 42.0, GREEN, 0, 0.85);
        }

        // Dynamic horizontal rate bar at (7, 79, 162, 10), inner (8, 80, 160, 8)
        c.blit_sprite("mekanism:bar/base", 7.0, 79.0, 162.0, 10.0);
        let progress = (data(10).clamp(0, 100) as f32) / 100.0;
        let bar_width = (160.0 * progress).floor();
        if bar_width > 0.0 {
            c.gradient(
                8.0,
                80.0,
                bar_width,
                8.0,
                0xFF3C_2D4A,
                0xFF64_1EAA,
            );
        }
        if inside(mouse, [7.0, 79.0, 162.0, 10.0]) {
            let tip = host::translate("gui.mekanism.progress", &[format!("{}%", data(10))]);
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Security tab at (176, 34, 26, 18) - Java GuiSecurityTab defaults to y = 34
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
                let text = host::translate("gui.mekanism.security", &[host::translate(mode_key, &[])]);
                c.tooltip(&[(&text, WHITE)]);
            }
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                let mouse = (p.x, p.y);
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
    fn test_sps_render_and_input() {
        let mut d = vec![0; 20];
        // Input: 1000 / 2000
        d[0] = 1000;
        d[2] = 2000;
        // Progress: 75%
        d[10] = 75;
        // Security mode: 0 (public) at index 15
        d[15] = 0;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "Status: Active".into(),
                "Energy Input: 50.00 MFE/t".into(),
                "Process Rate: 10 mB/t".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:sps_port".into(),
            gui_width: 176.0,
            gui_height: 182.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open sps");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 182);
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Status: Active")));

        // Test click on security tab at y = 34..52
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 38.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_sps_gauge_tooltip_and_protected_security() {
        let mut d = vec![0; 20];
        // Empty input (0 / 2000)
        d[0] = 0;
        d[2] = 2000;
        // Output filled (500 / 2000)
        d[4] = 500;
        d[6] = 2000;
        // Security mode: 2 (trusted -> protected sprite) at index 15
        d[15] = 2;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:sps_port".into(),
            gui_width: 176.0,
            gui_height: 182.0,
            touch: false,
        };
        let (mut screen, _) = open(&info).expect("open sps");

        // Hover over input gauge at (10, 20) -> should show empty
        let frame_empty = Frame {
            mouse_x: 10.0,
            mouse_y: 20.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame_empty, &mut canvas);
        let decoded = horizon_mod_sdk::screen::format::decode(&canvas.finish()).expect("decode");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "gui.mekanism.empty"))));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:security/protected")));
    }
}
