//! `GuiInductionMatrix` (176 by 166): energized induction matrix multiblock port screen.
//! Java source: `java/main/java/mekanism/client/gui/GuiInductionMatrix.java`
//!
//! Elements:
//! - Window dimensions: 176 x 166, titleLabelY = 5, inventoryLabelX = 8, inventoryLabelY = 74
//! - Armor holder at (-26, 36, 26, 98) on the left
//! - Element holder at (141, 16, 26, 56) with charge slot at (145, 20) [PLUS] and discharge slot at (145, 50) [MINUS]
//! - Medium energy gauge at (7, 16, 34, 56): energy pair (0, 1) MFE, capacity pair (2, 3) MFE, level data(8) in thousandths
//! - Inner screen at (49, 21, 84, 46): strings 0..3 (energy, capacity, input rate, output rate)
//! - Energy tab at (-26, 137, 26, 26): storing, input, output
//! - Security tab at (176, 6, 26, 18): data(13) from security_table.rs
//!
//! Differences from Java / Não Feito:
//! - // NÃO FEITO: GuiMatrixStats (segunda página de estatísticas da matriz) não implementada no servidor port/mod/src/induction.rs
//! - // NÃO FEITO: GuiWarningTab não suportada no servidor
use crate::elements::{self, base_panel, data, inside, inventory_label, pair, string, title, GREEN, WHITE};
use crate::security_table;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const TAB_SECURITY: u32 = 0xFFFF_A160;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(InductionMatrixScreen::new()),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct InductionMatrixScreen {
    security_idx: Option<usize>,
}

impl InductionMatrixScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:induction_port")
            .map(|(_, idx)| *idx);
        Self { security_idx }
    }
}

impl MekScreen for InductionMatrixScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Armor holder at left: (-26, 36, 26, 98)
        c.blit_sprite("mekanism:holder_left", -26.0, 36.0, 26.0, 98.0);

        // Title and inventory label
        title(c, &host::translate("block.mekanism.induction_port", &[]), 0.0, 176.0, 5.0);
        inventory_label(c, 8.0, 74.0);

        // Element holder at (141, 16, 26, 56)
        elements::element_holder(c, [141.0, 16.0, 26.0, 56.0]);
        // Top charge slot at (145, 20) with plus overlay
        c.blit_sprite("mekanism:slot/normal", 145.0, 20.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 145.0, 20.0, 18.0, 18.0);
        // Bottom discharge slot at (145, 50) with minus overlay
        c.blit_sprite("mekanism:slot/normal", 145.0, 50.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 145.0, 50.0, 18.0, 18.0);

        // Medium Energy Gauge at (7, 16, 34, 56), inner (8, 17, 32, 54)
        c.blit_sprite("mekanism:gauge/normal", 7.0, 16.0, 34.0, 56.0);
        let energy = pair(0, 1);
        let capacity = pair(2, 3);
        let fraction = (data(8) as f32 / 1000.0).clamp(0.0, 1.0);
        let filled = (52.0 * fraction).floor();
        if filled > 0.0 {
            c.blit(
                "mekanism:gui/bar/vertical_power",
                8.0,
                17.0 + 52.0 - filled,
                0.0,
                52.0 - filled,
                32.0,
                filled,
                4.0,
                52.0,
            );
        }
        c.blit_sprite("mekanism:gauge/overlay/medium", 8.0, 17.0, 32.0, 54.0);
        if inside(mouse, [7.0, 16.0, 34.0, 56.0]) {
            let tip = format!("Energy: {energy} / {capacity} MFE");
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Inner screen at (49, 21, 84, 46)
        elements::inner_screen(c, [49.0, 21.0, 84.0, 46.0]);
        for k in 0..4 {
            let s = string(k);
            if !s.is_empty() {
                c.text(&s, 52.0, 24.0 + 11.0 * k as f32, GREEN, 0, 0.8);
            }
        }

        // Energy tab at (-26, 137, 26, 26)
        let energy_rect = [-26.0, 137.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/energy_info", energy_rect[0], energy_rect[1], energy_rect[2], energy_rect[3]);
        c.area(7001, energy_rect, AreaKind::Tab, true, "Energy");
        if inside(mouse, energy_rect) {
            let in_rate = pair(9, 10);
            let out_rate = pair(11, 12);
            let tip = format!("Storing: {energy} / {capacity} MFE\nInput: {in_rate} kFE/t\nOutput: {out_rate} kFE/t");
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Security tab at (176, 6, 26, 18)
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
                // Energy tab
                if inside(mouse, [-26.0, 137.0, 26.0, 26.0]) {
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
    fn test_induction_matrix_render_and_input() {
        let mut d = vec![0; 15];
        // energy: 500 MFE
        d[0] = 500;
        // capacity: 1000 MFE
        d[2] = 1000;
        // level: 500
        d[8] = 500;
        // input: 50 kFE/t
        d[9] = 50;
        // output: 25 kFE/t
        d[11] = 25;
        // security: 0 (public) at index 13
        d[13] = 0;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "Energy: 500 MFE".into(),
                "Capacity: 1,000 MFE".into(),
                "Input: 50 kFE/t".into(),
                "Output: 25 kFE/t".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:induction_port".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open induction matrix");
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Energy: 500 MFE")));

        // Test click on security tab
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }
}
