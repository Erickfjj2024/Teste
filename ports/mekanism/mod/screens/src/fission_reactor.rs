//! `GuiFissionReactor` (195 by 255): fission reactor multiblock port screen.
//! Java source: `java/generators/java/mekanism/generators/client/gui/GuiFissionReactor.java`
//!
//! Elements:
//! - Window dimensions: 195 x 255 (`imageWidth = 195`, `imageHeight += 89`), titleLabelY = 5, inventoryLabelX = 6, inventoryLabelY = 163
//! - Gauges (STANDARD 18x60):
//!   - Coolant at (6, 13): pair (4, 5), capacity pair (6, 7), chem 17 (aqua)
//!   - Fuel at (25, 13): pair (0, 1), capacity pair (2, 3), chem 16 (dark green)
//!   - Heated Coolant at (152, 13): pair (8, 9), capacity pair (10, 11), chem 18 (orange)
//!   - Waste at (171, 13): pair (12, 13), capacity pair (14, 15), chem 19 (brown)
//! - Inner screen at (45, 17, 105, 56):
//!   - Status: Active (#55FF55) / Disabled (#FF5555) based on data(26)
//!   - string(0): Burn Rate
//!   - string(1): Heating Rate
//!   - string(2): Temperature (color band data 30)
//!   - string(3): Damage (color band data 31)
//! - Activate button at (6, 75, 81, 16) and SCRAM button at (89, 75, 81, 16): both send button 7100
//! - Big light indicator at (173, 76, 12, 12): enabled when active, disabled otherwise
//! - Temperature horizontal rate bar at (5, 102, 183, 8)
//! - Heat graph well at (5, 123, 185, 38)
//! - Heat tab at (-26, 109, 26, 26): dissipated rate (data 32..34)
//! - Stats tab at (195, 6, 26, 18): opens modal stats page (`GuiFissionReactorStats`)
//!   - Strings 4..6: Heat Statistics
//!   - Strings 7..8: Fuel Statistics
//!   - Current Burn Rate bar at (6, 115, 183, 8)
//!   - Rate Limit textfield at (77, 128, 54, 12), field id 7110
//! - Security tab at (195, 34, 26, 18): data(36) from security_table.rs
//!
//! Differences from Java / Não Feito:
//! - // NÃO FEITO: GuiWarningTab não suportada no servidor
use crate::elements::{self, base_panel, data, inside, inventory_label, pair, string, title, GREEN, TITLE, WHITE};
use crate::security_table;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const TAB_SECURITY: u32 = 0xFFFF_A160;
const TAB_STATS: u32 = 0xFF55_9EFF;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(FissionReactorScreen::new()),
        Setup {
            width: 195,
            height: 255,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct FissionReactorScreen {
    security_idx: Option<usize>,
    stats_open: bool,
    rate_limit_text: String,
}

impl FissionReactorScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:fission_reactor_port")
            .map(|(_, idx)| *idx);
        Self {
            security_idx,
            stats_open: false,
            rate_limit_text: String::new(),
        }
    }

    fn render_gauge(
        &self,
        c: &mut Canvas,
        mouse: (f32, f32),
        rect: [f32; 4],
        amount: u64,
        capacity: u64,
        label_key: &str,
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
            let label = host::translate(label_key, &[]);
            let tip = if amount == 0 {
                host::translate("gui.mekanism.empty", &[])
            } else {
                host::translate("generic.mekanism.stored.mb", &[label, amount.to_string()])
            };
            c.tooltip(&[(&tip, WHITE)]);
        }
    }
}

impl MekScreen for FissionReactorScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);

        if self.stats_open {
            // Stats window
            base_panel(c, 195.0, 255.0);
            title(c, "Fission Reactor Statistics", 0.0, 195.0, 5.0);
            // Close / back button at (6, 6, 12, 12)
            c.blit_sprite("mekanism:button/back", 6.0, 6.0, 12.0, 12.0);
            c.area(7102, [6.0, 6.0, 12.0, 12.0], AreaKind::Button, true, "Back");

            c.text(&host::translate("fission.mekanismgenerators.heat_statistics", &[]), 6.0, 20.0, TITLE, 0, 1.0);
            c.text(&string(4), 6.0, 32.0, TITLE, 0, 0.9);
            c.text(&string(5), 6.0, 42.0, TITLE, 0, 0.9);
            c.text(&string(6), 6.0, 52.0, TITLE, 0, 0.9);

            c.text(&host::translate("fission.mekanismgenerators.fuel_statistics", &[]), 6.0, 68.0, TITLE, 0, 1.0);
            c.text(&string(7), 6.0, 80.0, TITLE, 0, 0.9);
            c.text(&string(8), 6.0, 90.0, TITLE, 0, 0.9);

            c.text(&host::translate("fission.mekanismgenerators.burn_rate", &[]), 6.0, 104.0, TITLE, 0, 0.9);
            c.blit_sprite("mekanism:bar/base", 5.0, 114.0, 185.0, 10.0);
            let burn_val = data(23);
            let max_val = data(35).max(1);
            let burn_fill = (burn_val as f32 / max_val as f32).clamp(0.0, 1.0);
            let b_width = (183.0 * burn_fill).floor();
            if b_width > 0.0 {
                c.fill(6.0, 115.0, b_width, 8.0, 0xFFFF_A500);
            }
            if inside(mouse, [5.0, 114.0, 185.0, 10.0]) {
                let tip = format!("Burn Rate: {} (x0.01 mB/t)", burn_val);
                c.tooltip(&[(&tip, WHITE)]);
            }

            c.text(&host::translate("fission.mekanismgenerators.set_rate_limit", &[]), 5.0, 131.0, TITLE, 0, 0.9);
            c.blit_sprite("mekanism:digital_text_input", 76.0, 127.0, 56.0, 14.0);
            c.field(7110, [77.0, 128.0, 54.0, 12.0], 8, horizon_mod_sdk::screen::FieldFilter::Any, &self.rate_limit_text, 1, WHITE);
            inventory_label(c, 6.0, 163.0);
            return;
        }

        // Main reactor page
        base_panel(c, 195.0, 255.0);
        title(c, "Fission Reactor", 0.0, 195.0, 5.0);
        inventory_label(c, 6.0, 163.0);

        // Gauges
        let fuel = pair(0, 1);
        let fuel_cap = pair(2, 3);
        let coolant = pair(4, 5);
        let coolant_cap = pair(6, 7);
        let hot = pair(8, 9);
        let hot_cap = pair(10, 11);
        let waste = pair(12, 13);
        let waste_cap = pair(14, 15);

        // Coolant (aqua) at (6, 13)
        self.render_gauge(
            c,
            mouse,
            [6.0, 13.0, 18.0, 60.0],
            coolant,
            coolant_cap,
            "fission.mekanismgenerators.coolant_tank",
            "mekanism:liquid/liquid",
            0xFF00_FFFF,
        );
        // Fuel (dark green) at (25, 13)
        self.render_gauge(
            c,
            mouse,
            [25.0, 13.0, 18.0, 60.0],
            fuel,
            fuel_cap,
            "fission.mekanismgenerators.fuel_tank",
            "mekanism:liquid/liquid",
            0xFF00_8000,
        );
        // Heated Coolant (orange) at (152, 13)
        self.render_gauge(
            c,
            mouse,
            [152.0, 13.0, 18.0, 60.0],
            hot,
            hot_cap,
            "fission.mekanismgenerators.heated_coolant_tank",
            "mekanism:liquid/liquid",
            0xFFFF_A500,
        );
        // Waste (brown) at (171, 13)
        self.render_gauge(
            c,
            mouse,
            [171.0, 13.0, 18.0, 60.0],
            waste,
            waste_cap,
            "fission.mekanismgenerators.waste_tank",
            "mekanism:liquid/liquid",
            0xFF8B_4513,
        );

        // Inner screen at (45, 17, 105, 56)
        elements::inner_screen(c, [45.0, 17.0, 105.0, 56.0]);
        let state = data(26);
        let is_active = state == 1;
        let status_color = if is_active { 0xFF55_FF55 } else { 0xFFFF_5555 };
        let status_text = if is_active { "Status: Active" } else { "Status: Disabled" };
        c.text(status_text, 48.0, 20.0, status_color, 0, 0.85);

        let s0 = string(0);
        let s1 = string(1);
        let s2 = string(2);
        let s3 = string(3);
        let burn_str = if s0.is_empty() { format!("Burn Rate: {} mB/t", data(23) as f32 / 100.0) } else { s0 };
        let heat_str = if s1.is_empty() { format!("Heating Rate: {} mB/t", data(25)) } else { s1 };
        let temp_str = if s2.is_empty() { format!("Temperature: {} K", data(20) as f32 / 10.0) } else { s2 };
        let dmg_str = if s3.is_empty() { format!("Damage: {}%", data(21)) } else { s3 };

        c.text(&burn_str, 48.0, 30.0, GREEN, 0, 0.85);
        c.text(&heat_str, 48.0, 40.0, GREEN, 0, 0.85);
        let temp_colors = [0xFF55_FF55, 0xFFFF_FF55, 0xFFFF_AA00, 0xFFFF_5555, 0xFFAA_0000];
        let t_color = temp_colors.get((data(30).max(0) as usize) % temp_colors.len()).copied().unwrap_or(GREEN);
        c.text(&temp_str, 48.0, 50.0, t_color, 0, 0.85);
        let dmg_colors = [0xFF55_FF55, 0xFFFF_FF55, 0xFFFF_AA00, 0xFFAA_0000];
        let d_color = dmg_colors.get((data(31).max(0) as usize) % dmg_colors.len()).copied().unwrap_or(GREEN);
        c.text(&dmg_str, 48.0, 60.0, d_color, 0, 0.85);

        // Buttons
        let activate_active = state == 0;
        let scram_active = state == 1;

        elements::button(
            c,
            7100,
            [6.0, 75.0, 81.0, 16.0],
            "Activate",
            activate_active,
            mouse,
            if state == 2 {
                Some("Forced off: the reactor melted down and cannot restart until its rods and fuel are reset")
            } else {
                Some("Activate")
            },
        );

        elements::button(c, 7100, [89.0, 75.0, 81.0, 16.0], "SCRAM", scram_active, mouse, Some("SCRAM"));

        // Big Light at (173, 76, 12, 12)
        let light_sprite = if is_active {
            "mekanism:light/big/enabled"
        } else {
            "mekanism:light/big/disabled"
        };
        c.blit_sprite(light_sprite, 173.0, 76.0, 12.0, 12.0);

        // Temperature label and bar at (5, 102, 185, 10)
        c.text(&host::translate("temperature.mekanism.long", &["".to_string()]), 5.0, 93.0, TITLE, 0, 0.9);
        c.blit_sprite("mekanism:bar/base", 5.0, 102.0, 185.0, 10.0);
        let temp_fraction = (data(20) as f32 / 18000.0).clamp(0.0, 1.0);
        let t_bar_width = (183.0 * temp_fraction).floor();
        if t_bar_width > 0.0 {
            c.fill(6.0, 103.0, t_bar_width, 8.0, 0xFFFF_3300);
        }
        if inside(mouse, [5.0, 102.0, 185.0, 10.0]) {
            let tip = format!("Temperature: {} K", data(20) as f32 / 10.0);
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Heat Graph label and well at (5, 123, 185, 38)
        c.text(&host::translate("fission.mekanismgenerators.heat_graph", &[]), 5.0, 114.0, TITLE, 0, 0.9);
        elements::inner_screen(c, [5.0, 123.0, 185.0, 38.0]);

        // Heat Tab at (-26, 109, 26, 26)
        let heat_rect = [-26.0, 109.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/heat_info_k", heat_rect[0], heat_rect[1], heat_rect[2], heat_rect[3]);
        c.area(7002, heat_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, heat_rect) {
            let tip = host::translate("gui.mekanism.dissipated", &[format!("{}.{}{} K", data(32), data(33), data(34))]);
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Stats Tab at (195, 6, 26, 18)
        let stats_rect = [195.0, 6.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", stats_rect, TAB_STATS);
        c.blit_sprite("mekanism:button/radio", 195.0 + 4.0, 6.0, 18.0, 18.0);
        c.area(7101, stats_rect, AreaKind::Tab, true, "Stats");
        if inside(mouse, stats_rect) {
            c.tooltip(&[("Fission Reactor Statistics", WHITE)]);
        }

        // Security tab at (195, 34, 26, 18)
        if let Some(sec_idx) = self.security_idx {
            let sec_rect = [195.0, 34.0, 26.0, 18.0];
            c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
            let sec_mode = (data(sec_idx).max(0) as usize) % 3;
            let icon_sprite = match sec_mode {
                1 => "mekanism:security/private",
                2 => "mekanism:security/protected",
                _ => "mekanism:security/public",
            };
            c.blit_sprite(icon_sprite, 195.0 + 4.0, 34.0, 18.0, 18.0);
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
                if self.stats_open {
                    // Back button at (6, 6, 12, 12)
                    if inside(mouse, [6.0, 6.0, 12.0, 12.0]) {
                        self.stats_open = false;
                        host::play_click();
                        return true;
                    }
                    return true;
                }
                // Stats tab at (195, 6, 26, 18)
                if inside(mouse, [195.0, 6.0, 26.0, 18.0]) {
                    self.stats_open = true;
                    host::play_click();
                    return true;
                }
                // Activate button at (6, 75, 81, 16)
                if data(26) == 0 && inside(mouse, [6.0, 75.0, 81.0, 16.0]) {
                    host::send_button(7100);
                    host::play_click();
                    return true;
                }
                // SCRAM button at (89, 75, 81, 16)
                if data(26) == 1 && inside(mouse, [89.0, 75.0, 81.0, 16.0]) {
                    host::send_button(7100);
                    host::play_click();
                    return true;
                }
                // Heat tab at (-26, 109, 26, 26)
                if inside(mouse, [-26.0, 109.0, 26.0, 26.0]) {
                    host::play_click();
                    return true;
                }
                // Security tab at (195, 34, 26, 18)
                if self.security_idx.is_some() && inside(mouse, [195.0, 34.0, 26.0, 18.0]) {
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
            InputEvent::FieldSubmitted(f) => {
                if f.id == 7110 {
                    self.rate_limit_text = f.text.clone();
                    host::send_text(7110, &f.text);
                    return true;
                }
                false
            }
            InputEvent::FieldChanged(f) => {
                if f.id == 7110 {
                    self.rate_limit_text = f.text.clone();
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
        format::Command, host, FieldText, InputEvent, Modifiers, MouseButton, OpenInfo, Pointer,
    };

    #[test]
    fn test_fission_reactor_render_and_input() {
        let mut d = vec![0; 40];
        d[20] = 3731; // 373.1 K
        d[26] = 0; // Off
        d[36] = 0; // Security mode Public

        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "Burn Rate: 0.00 mB/t".into(),
                "Heating Rate: 0 mB/t".into(),
                "Temperature: 373.1 K".into(),
                "Damage: 0%".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:fission_reactor_port".into(),
            gui_width: 195.0,
            gui_height: 255.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open fission reactor");
        assert_eq!(setup.width, 195);
        assert_eq!(setup.height, 255);
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Status: Disabled")));

        // Test click Activate button (at 10, 80)
        let click_act = InputEvent::MousePressed(Pointer {
            x: 10.0,
            y: 80.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_act));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7100".to_string(), "click".to_string()]);

        // Test click Stats tab (at 200, 10)
        let click_stats = InputEvent::MousePressed(Pointer {
            x: 200.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_stats));
        let sent_stats = host::take_sent();
        assert_eq!(sent_stats, vec!["click".to_string()]);

        // Submit rate limit field 7110
        let field_sub = InputEvent::FieldSubmitted(FieldText {
            id: 7110,
            text: "10.0".into(),
        });
        assert!(screen.input(&field_sub));
        let sent_field = host::take_sent();
        assert_eq!(sent_field, vec!["text 7110 \"10.0\"".to_string()]);
    }

    #[test]
    fn test_fission_reactor_rate_bar_and_protected_security() {
        let mut d = vec![0; 40];
        // Coolant: 0 (empty)
        d[4] = 0;
        d[6] = 10000;
        // Security mode: 2 (trusted -> protected) at index 36
        d[36] = 2;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:fission_reactor_port".into(),
            gui_width: 195.0,
            gui_height: 255.0,
            touch: false,
        };
        let (mut screen, _) = open(&info).expect("open screen");

        let frame = Frame {
            mouse_x: 10.0,
            mouse_y: 20.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let decoded = horizon_mod_sdk::screen::format::decode(&canvas.finish()).expect("decode");

        // Empty gauge tooltip
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "gui.mekanism.empty"))));
        // Protected security sprite
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:security/protected")));
        // Temperature rate bar base 185x10 at y=102
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && rect[0] == 5.0 && rect[1] == 102.0 && rect[2] == 185.0 && rect[3] == 10.0)));
    }
}
