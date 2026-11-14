//! `GuiThermoelectricBoiler` (218 by 166): thermoelectric boiler multiblock valve screen.
//! Java source: `java/main/java/mekanism/client/gui/GuiThermoelectricBoiler.java`
//!
//! Elements:
//! - Window dimensions: 218 x 166 (`imageWidth += 42`), titleLabelY = 5, inventoryLabelX = 29, inventoryLabelY = 74
//! - Inner screen at (54, 23, 110, 40): temperature, boil rate, max boil rate (strings 0..2)
//! - Vertical rate bars:
//!   - Boil rate at (44, 13, 6, 58)
//!   - Max boil at (166, 13, 6, 58)
//! - Four standard gauges (18x60):
//!   - Superheated Coolant at (6, 13): pair (8, 9), capacity pair (10, 11), chem index 18
//!   - Water at (26, 13): pair (0, 1), capacity pair (2, 3), chem index 16
//!   - Steam at (174, 13): pair (4, 5), capacity pair (6, 7), chem index 17
//!   - Cooled Coolant at (194, 13): pair (12, 13), capacity pair (14, 15), chem index 19
//! - Heat tab at (-26, 109, 26, 26): dissipated rate
//! - Security tab at (218, 6, 26, 18)
//!
//! Differences from Java / Não Feito:
//! - // NÃO FEITO: GuiBoilerStats (segunda página com estatísticas detalhadas de ebulição) não foi implementado no servidor port/mod/src/boiler.rs
//! - // NÃO FEITO: GuiWarningTab (sistema de avisos do multiblock) não suportado no servidor port/mod/src/boiler.rs
use crate::elements::{self, base_panel, data, inside, inventory_label, pair, string, title, GREEN, WHITE};
use crate::security_table;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const TAB_SECURITY: u32 = 0xFFFF_A160;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(ThermoelectricBoilerScreen::new()),
        Setup {
            width: 218,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct ThermoelectricBoilerScreen {
    security_idx: Option<usize>,
}

impl ThermoelectricBoilerScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:boiler_valve")
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
                format!("{label}: {}", host::translate("gui.mekanism.empty", &[]))
            } else {
                format!("{label}: {amount} / {capacity} mB")
            };
            c.tooltip(&[(&tip, WHITE)]);
        }
    }

    fn render_rate_bar(&self, c: &mut Canvas, mouse: (f32, f32), x: f32, y: f32, level: f32, tip: &str) {
        c.blit_sprite("mekanism:bar/base", x, y, 6.0, 58.0);
        let filled = (56.0 * level.clamp(0.0, 1.0)).floor();
        if filled > 0.0 {
            c.blit(
                "mekanism:gui/bar/vertical_rate",
                x + 1.0,
                y + 57.0 - filled,
                0.0,
                58.0 - filled,
                4.0,
                filled,
                6.0,
                58.0,
            );
        }
        if inside(mouse, [x, y, 6.0, 58.0]) {
            c.tooltip(&[(tip, WHITE)]);
        }
    }
}

impl MekScreen for ThermoelectricBoilerScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 218.0, 166.0);

        // Title and inventory label
        title(c, &host::translate("boiler.mekanism.thermoelectric_boiler", &[]), 0.0, 218.0, 5.0);
        inventory_label(c, 29.0, 74.0);

        // Inner screen at (54, 23, 110, 40)
        elements::inner_screen(c, [54.0, 23.0, 110.0, 40.0]);
        let s0 = string(0);
        let s1 = string(1);
        let s2 = string(2);
        let temp_str = if s0.is_empty() { format!("Temperature: {} K", data(20) as f32 / 10.0) } else { s0 };
        let boil_str = if s1.is_empty() { format!("Boil Rate: {} mB/t", data(21)) } else { s1 };
        let max_str = if s2.is_empty() { format!("Max Boil: {} mB/t", data(22)) } else { s2 };
        c.text(&temp_str, 57.0, 26.0, GREEN, 0, 0.85);
        c.text(&boil_str, 57.0, 37.0, GREEN, 0, 0.85);
        c.text(&max_str, 57.0, 48.0, GREEN, 0, 0.85);

        // 4 Gauges
        let water = pair(0, 1);
        let water_cap = pair(2, 3);
        let steam = pair(4, 5);
        let steam_cap = pair(6, 7);
        let hot = pair(8, 9);
        let hot_cap = pair(10, 11);
        let cold = pair(12, 13);
        let cold_cap = pair(14, 15);

        // Superheated Coolant (orange)
        self.render_gauge(
            c,
            mouse,
            [6.0, 13.0, 18.0, 60.0],
            hot,
            hot_cap,
            "boiler.mekanism.heated_coolant_tank",
            "mekanism:liquid/liquid",
            0xFFFF_A500,
        );
        // Water (indigo / blue)
        self.render_gauge(
            c,
            mouse,
            [26.0, 13.0, 18.0, 60.0],
            water,
            water_cap,
            "boiler.mekanism.water_tank",
            "minecraft:block/water_still",
            0xFF3F_76E4,
        );
        // Steam (gray)
        self.render_gauge(
            c,
            mouse,
            [174.0, 13.0, 18.0, 60.0],
            steam,
            steam_cap,
            "boiler.mekanism.steam_tank",
            "mekanism:liquid/liquid",
            0xFFCC_CCCC,
        );
        // Cooled Coolant (aqua)
        self.render_gauge(
            c,
            mouse,
            [194.0, 13.0, 18.0, 60.0],
            cold,
            cold_cap,
            "boiler.mekanism.coolant_tank",
            "mekanism:liquid/liquid",
            0xFF00_FFFF,
        );

        // Rate bars
        let boil_rate = data(21);
        let max_boil = data(22);
        let boil_lvl = if max_boil == 0 { 0.0 } else { boil_rate as f32 / max_boil as f32 };
        let boil_tip = format!("Boil Rate: {boil_rate} mB/t");
        self.render_rate_bar(c, mouse, 44.0, 13.0, boil_lvl, &boil_tip);

        let max_lvl = data(25) as f32 / 1000.0;
        let max_tip = format!("Max Boil: {max_boil} mB/t");
        self.render_rate_bar(c, mouse, 166.0, 13.0, max_lvl, &max_tip);

        // Heat tab at (-26, 109, 26, 26)
        let heat_rect = [-26.0, 109.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/heat_info_k", heat_rect[0], heat_rect[1], heat_rect[2], heat_rect[3]);
        c.area(7000, heat_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, heat_rect) {
            let loss_w = data(26);
            let loss_t = data(27);
            let loss_h = data(28);
            let tip = format!("Dissipated: {loss_w}.{loss_t}{loss_h} K/t");
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Security tab at (218, 6, 26, 18)
        if let Some(sec_idx) = self.security_idx {
            let sec_rect = [218.0, 6.0, 26.0, 18.0];
            c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
            let mode = (data(sec_idx).max(0) as usize) % 3;
            let icon_sprite = match mode {
                1 => "mekanism:security/private",
                2 => "mekanism:security/trusted",
                _ => "mekanism:security/public",
            };
            c.blit_sprite(icon_sprite, 218.0 + 4.0, 6.0, 18.0, 18.0);
            c.area(9100, sec_rect, AreaKind::Tab, true, "Security");
            if inside(mouse, sec_rect) {
                let text = match mode {
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
                // Security tab
                if self.security_idx.is_some() && inside(mouse, [218.0, 6.0, 26.0, 18.0]) {
                    if p.button == MouseButton::Right {
                        host::send_button(9101);
                    } else {
                        host::send_button(9100);
                    }
                    host::play_click();
                    return true;
                }
                // Heat tab
                if inside(mouse, [-26.0, 109.0, 26.0, 26.0]) {
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
    fn test_boiler_render_and_input() {
        let mut d = vec![0; 30];
        // water: 5000 / 10000
        d[0] = 5000;
        d[2] = 10000;
        // boil rate 100, max 200
        d[21] = 100;
        d[22] = 200;
        d[25] = 500;
        // security mode public = 0 at slot 29
        d[29] = 0;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "Temperature: 373.15 K".into(),
                "Boil Rate: 100 mB/t".into(),
                "Max Boil: 200 mB/t".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:boiler_valve".into(),
            gui_width: 218.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open boiler");
        assert_eq!(setup.width, 218);
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Boil Rate: 100 mB/t")));

        // Test click on security tab
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 220.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }
}
