//! `GuiThermalEvaporationController` (196 by 166): thermal evaporation plant multiblock controller screen.
//! Java source: `java/main/java/mekanism/client/gui/GuiThermalEvaporationController.java`
//!
//! Elements:
//! - Window dimensions: 196 x 166 (`imageWidth += 20`), titleLabelY = 6, inventoryLabelX = 18, inventoryLabelY = 74
//! - Two standard fluid gauges (18x60):
//!   - Input at (6, 13): pair (0, 1), capacity pair (2, 3), chem index 6
//!   - Output at (172, 13): pair (4, 5), capacity pair (6, 7), chem index 7
//! - Bucket slots with down arrows:
//!   - Input slots at (28, 20) and (28, 51), arrow at (32, 39)
//!   - Output slots at (152, 20) and (152, 51), arrow at (156, 39)
//! - Inner screen at (48, 19, 100, 40): strings 0..3 (Formed, Height, Temperature, Production)
//! - Horizontal rate bar at (58, 62, 80, 10), inner (59, 63, 78, 8): temperature over 3,000 K
//! - Heat tab at (-26, 109, 26, 26): dissipated rate (data 12..14)
//! - Security tab at (196, 6, 26, 18): data(15) from security_table.rs
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
        Box::new(ThermalEvaporationControllerScreen::new()),
        Setup {
            width: 196,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct ThermalEvaporationControllerScreen {
    security_idx: Option<usize>,
}

impl ThermalEvaporationControllerScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:thermal_evaporation_controller")
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
                format!("{label}: {}", host::translate("gui.mekanism.empty", &[]))
            } else {
                format!("{label}: {amount} / {capacity} mB")
            };
            c.tooltip(&[(&tip, WHITE)]);
        }
    }
}

impl MekScreen for ThermalEvaporationControllerScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 196.0, 166.0);

        // Title centered between gauges (x = 24 to 172)
        title(c, "Thermal Evaporation Plant", 24.0, 148.0, 6.0);
        inventory_label(c, 18.0, 74.0);

        // Input and Output Gauges
        let in_amount = pair(0, 1);
        let in_cap = pair(2, 3);
        let out_amount = pair(4, 5);
        let out_cap = pair(6, 7);

        self.render_gauge(
            c,
            mouse,
            [6.0, 13.0, 18.0, 60.0],
            in_amount,
            in_cap,
            "Input",
            "minecraft:block/water_still",
            0xFF3F_76E4,
        );
        self.render_gauge(
            c,
            mouse,
            [172.0, 13.0, 18.0, 60.0],
            out_amount,
            out_cap,
            "Output",
            "mekanism:liquid/liquid",
            0xFFFE_EF9C,
        );

        // Input slots & arrow
        c.blit_sprite("mekanism:slot/input", 27.0, 19.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 27.0, 50.0, 18.0, 18.0);
        c.blit_sprite("mekanism:arrow/down", 32.0, 39.0, 8.0, 8.0);

        // Output slots & arrow
        c.blit_sprite("mekanism:slot/input", 151.0, 19.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 151.0, 50.0, 18.0, 18.0);
        c.blit_sprite("mekanism:arrow/down", 156.0, 39.0, 8.0, 8.0);

        // Inner screen at (48, 19, 100, 40)
        elements::inner_screen(c, [48.0, 19.0, 100.0, 40.0]);
        for k in 0..4 {
            let s = string(k);
            if !s.is_empty() {
                c.text(&s, 51.0, 22.0 + 9.0 * k as f32, GREEN, 0, 0.85);
            }
        }

        // Horizontal rate bar at (58, 62, 80, 10), inner (59, 63, 78, 8)
        c.blit_sprite("mekanism:bar/base", 58.0, 62.0, 80.0, 10.0);
        let temp = data(8) as f32;
        let frac = (temp / 3000.0).clamp(0.0, 1.0);
        let filled = (78.0 * frac).floor();
        if filled > 0.0 {
            c.blit(
                "mekanism:gui/bar/horizontal_rate",
                59.0,
                63.0,
                0.0,
                0.0,
                filled,
                8.0,
                78.0,
                8.0,
            );
        }
        if inside(mouse, [58.0, 62.0, 80.0, 10.0]) {
            let tip = format!("Temperature: {temp} K");
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Heat tab at (-26, 109, 26, 26)
        let heat_rect = [-26.0, 109.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/heat_info_k", heat_rect[0], heat_rect[1], heat_rect[2], heat_rect[3]);
        c.area(7005, heat_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, heat_rect) {
            let tip = format!("Dissipated: {}.{}{} K/t", data(12), data(13), data(14));
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Security tab at (196, 6, 26, 18)
        if let Some(sec_idx) = self.security_idx {
            let sec_rect = [196.0, 6.0, 26.0, 18.0];
            c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
            let sec_mode = (data(sec_idx).max(0) as usize) % 3;
            let icon_sprite = match sec_mode {
                1 => "mekanism:security/private",
                2 => "mekanism:security/trusted",
                _ => "mekanism:security/public",
            };
            c.blit_sprite(icon_sprite, 196.0 + 4.0, 6.0, 18.0, 18.0);
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
                // Heat tab
                if inside(mouse, [-26.0, 109.0, 26.0, 26.0]) {
                    host::play_click();
                    return true;
                }
                // Security tab
                if self.security_idx.is_some() && inside(mouse, [196.0, 6.0, 26.0, 18.0]) {
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
    fn test_evaporation_controller_render_and_input() {
        let mut d = vec![0; 20];
        // Input: 10000 / 64000
        d[0] = 10000;
        d[2] = 64000;
        // Output: 5000 / 64000
        d[4] = 5000;
        d[6] = 64000;
        // Temperature: 300 K
        d[8] = 300;
        // Security mode: 0 (public) at index 15
        d[15] = 0;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "Formed".into(),
                "Height: 18".into(),
                "Temperature: 300.0 K".into(),
                "Production: 25.0 mB/t".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:thermal_evaporation_controller".into(),
            gui_width: 196.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open evaporation controller");
        assert_eq!(setup.width, 196);
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Formed")));

        // Test click on security tab
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 200.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }
}
