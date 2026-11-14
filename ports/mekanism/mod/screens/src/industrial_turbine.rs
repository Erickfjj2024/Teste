//! `GuiIndustrialTurbine` (190 by 166): industrial turbine multiblock valve screen.
//! Java source: `java/generators/java/mekanism/generators/client/gui/GuiIndustrialTurbine.java`
//!
//! Elements:
//! - Window dimensions: 190 x 166 (`imageWidth += 14`), titleLabelY = 5, inventoryLabelX = 15, inventoryLabelY = 74
//! - Medium steam gauge at (6, 13, 34, 60): pair (6, 7), capacity pair (8, 9), chem index 10
//! - Steam rate bar at (40, 13, 6, 58): data(22) / 1000.0
//! - Inner screen at (50, 18, 126, 50): strings 0..3 (Production, Flow Rate, Capacity, Max Flow)
//! - Power bar at (178, 16, 6, 54): data(23) / 1000.0, energy data(24) kFE
//! - Steam dump mode button at (173, 72, 10, 10): data(21) (0 idle, 1 excess, 2 dump), sends button 7400
//! - Energy tab at (-26, 137, 26, 26): storing data(24) kFE, producing data(11) FE/t
//! - Security tab at (190, 6, 26, 18): data(25) from security_table.rs
//!
//! Differences from Java / Não Feito:
//! - // NÃO FEITO: GuiTurbineStats (segunda página com estatores, pás, bobinas e condensadores) não implementada no servidor
//! - // NÃO FEITO: GuiWarningTab não suportada no servidor
use crate::elements::{self, base_panel, data, inside, inventory_label, pair, string, title, vertical_power_bar, GREEN, TITLE, WHITE};
use crate::security_table;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const TAB_SECURITY: u32 = 0xFFFF_A160;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(IndustrialTurbineScreen::new()),
        Setup {
            width: 190,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct IndustrialTurbineScreen {
    security_idx: Option<usize>,
}

impl IndustrialTurbineScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:turbine_valve")
            .map(|(_, idx)| *idx);
        Self { security_idx }
    }
}

impl MekScreen for IndustrialTurbineScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 190.0, 166.0);

        // Title and inventory label
        title(c, &host::translate("turbine.mekanismgenerators.industrial_turbine", &[]), 0.0, 190.0, 5.0);
        inventory_label(c, 15.0, 74.0);

        // Medium Steam Gauge at (6, 13, 34, 60), inner (7, 14, 32, 58)
        c.blit_sprite("mekanism:gauge/normal", 6.0, 13.0, 34.0, 60.0);
        let steam = pair(6, 7);
        let steam_cap = pair(8, 9);
        let inner = [7.0, 14.0, 32.0, 58.0];
        let fill = if steam_cap == 0 {
            0.0
        } else {
            (steam as f32 / steam_cap as f32).clamp(0.0, 1.0)
        };
        if fill > 0.0 {
            c.fluid("mekanism:liquid/liquid", inner, fill, 0xFFCC_CCCC);
        }
        c.blit_sprite("mekanism:gauge/overlay/medium", inner[0], inner[1], inner[2], inner[3]);
        if inside(mouse, [6.0, 13.0, 34.0, 60.0]) {
            let tip = if steam == 0 {
                host::translate("gui.mekanism.empty", &[])
            } else {
                host::translate("generic.mekanism.stored.mb", &["Steam".to_string(), steam.to_string()])
            };
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Steam rate bar at (40, 13, 8, 60), inner (41, 14, 6, 58)
        c.blit_sprite("mekanism:bar/base", 40.0, 13.0, 8.0, 60.0);
        let rate_frac = (data(22) as f32 / 1000.0).clamp(0.0, 1.0);
        let r_fill = (58.0 * rate_frac).floor();
        if r_fill > 0.0 {
            c.blit(
                "mekanism:gui/bar/vertical_rate",
                41.0,
                14.0 + 58.0 - r_fill,
                0.0,
                58.0 - r_fill,
                6.0,
                r_fill,
                6.0,
                58.0,
            );
        }
        if inside(mouse, [40.0, 13.0, 8.0, 60.0]) {
            let tip = host::translate("turbine.mekanismgenerators.steam_input", &[data(22).to_string()]);
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Inner screen at (50, 18, 126, 50)
        elements::inner_screen(c, [50.0, 18.0, 126.0, 50.0]);
        for k in 0..4 {
            let s = string(k);
            if !s.is_empty() {
                c.text(&s, 53.0, 21.0 + 11.0 * k as f32, GREEN, 0, 0.85);
            }
        }

        // Power bar at (178, 16, 6, 54)
        let power_frac = (data(23) as f32 / 1000.0).clamp(0.0, 1.0);
        vertical_power_bar(c, 179.0, 17.0, power_frac);
        if inside(mouse, [178.0, 16.0, 6.0, 54.0]) {
            let tip = format!("Energy: {} kFE", data(24));
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Steam dump mode button at (173, 72, 10, 10)
        let mode = (data(21).max(0) as usize) % 3;
        let mode_texture = match mode {
            1 => "mekanism:gui/gas_mode_excess",
            2 => "mekanism:gui/gas_mode_dump",
            _ => "mekanism:gui/gas_mode_idle",
        };
        c.blit(mode_texture, 173.0, 72.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0);
        let mode_label_key = match mode {
            1 => "chemical_mode.mekanism.dumping_excess",
            2 => "chemical_mode.mekanism.dumping",
            _ => "chemical_mode.mekanism.idle",
        };
        let mode_label = host::translate(mode_label_key, &[]);
        let label_w = host::text_width(&mode_label);
        c.draw_string(&mode_label, 173.0 - 2.0 - label_w, 73.0, TITLE, false);
        c.area(7400, [173.0, 72.0, 10.0, 10.0], AreaKind::Button, true, "Dump Mode");
        if inside(mouse, [173.0, 72.0, 10.0, 10.0]) {
            let warning = host::translate("turbine.mekanismgenerators.tooltip.steamdump.warning", &[]);
            if mode == 1 {
                let line1 = host::translate("turbine.mekanismgenerators.tooltip.steamdump_excess", &[]);
                c.tooltip(&[(&line1, WHITE), (&warning, 0xFFFF_5555)]);
            } else if mode == 2 {
                let line1 = host::translate("turbine.mekanismgenerators.tooltip.steamdump", &[]);
                c.tooltip(&[(&line1, WHITE), (&warning, 0xFFFF_5555)]);
            }
        }

        // Energy tab at (-26, 137, 26, 26)
        let energy_rect = [-26.0, 137.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/energy_info", energy_rect[0], energy_rect[1], energy_rect[2], energy_rect[3]);
        c.area(7006, energy_rect, AreaKind::Tab, true, "Energy");
        if inside(mouse, energy_rect) {
            let line1 = host::translate("gui.mekanism.storing", &[format!("{} kFE", data(24))]);
            let line2 = host::translate("generators.mekanismgenerators.producing.amount", &[format!("{} FE", data(11))]);
            c.tooltip(&[(&line1, WHITE), (&line2, WHITE)]);
        }

        // Security tab at (190, 34, 26, 18) - Java GuiSecurityTab defaults to y = 34
        if let Some(sec_idx) = self.security_idx {
            let sec_rect = [190.0, 34.0, 26.0, 18.0];
            c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
            let sec_mode = (data(sec_idx).max(0) as usize) % 3;
            let icon_sprite = match sec_mode {
                1 => "mekanism:security/private",
                2 => "mekanism:security/protected",
                _ => "mekanism:security/public",
            };
            c.blit_sprite(icon_sprite, 190.0 + 4.0, 34.0, 18.0, 18.0);
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
                // Dump mode button
                if inside(mouse, [173.0, 72.0, 10.0, 10.0]) {
                    host::send_button(7400);
                    host::play_click();
                    return true;
                }
                // Energy tab
                if inside(mouse, [-26.0, 137.0, 26.0, 26.0]) {
                    host::play_click();
                    return true;
                }
                // Security tab at y = 34
                if self.security_idx.is_some() && inside(mouse, [190.0, 34.0, 26.0, 18.0]) {
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
    fn test_industrial_turbine_render_and_input() {
        let mut d = vec![0; 30];
        // steam: 1000 / 5000
        d[6] = 1000;
        d[8] = 5000;
        // producing: 2000 FE/t
        d[11] = 2000;
        // steam mode: 0 (idle)
        d[21] = 0;
        // security: 0 (public) at index 25
        d[25] = 0;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![
                "Production: 2,000 FE/t".into(),
                "Flow Rate: 100 mB/t".into(),
                "Capacity: 5,000 mB".into(),
                "Max Flow: 500 mB/t".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:turbine_valve".into(),
            gui_width: 190.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open industrial turbine");
        assert_eq!(setup.width, 190);
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
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Production: 2,000 FE/t")));

        // Test click on dump mode button (at 175, 75)
        let click_dump = InputEvent::MousePressed(Pointer {
            x: 175.0,
            y: 75.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_dump));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7400".to_string(), "click".to_string()]);

        // Test click on security tab at y = 34..52
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 195.0,
            y: 38.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent2 = host::take_sent();
        assert_eq!(sent2, vec!["button 9100".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_industrial_turbine_dump_mode_and_security_protected() {
        let mut d = vec![0; 30];
        // steam: 0 (empty)
        d[6] = 0;
        d[8] = 5000;
        // dump mode: 1 (dumping excess)
        d[21] = 1;
        // security: 2 (trusted -> protected) at index 25
        d[25] = 2;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:turbine_valve".into(),
            gui_width: 190.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, _) = open(&info).expect("open screen");

        // Hover over steam dump button at (175, 75)
        let frame_dump = Frame {
            mouse_x: 175.0,
            mouse_y: 75.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame_dump, &mut canvas);
        let decoded = horizon_mod_sdk::screen::format::decode(&canvas.finish()).expect("decode");

        // Should render steam dump excess tooltip with warning line
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, color)| t == "turbine.mekanismgenerators.tooltip.steamdump.warning" && *color == 0xFFFF_5555))));
        // Security sprite should be protected
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:security/protected")));
        // Steam rate bar base should be 8x60
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && rect[2] == 8.0 && rect[3] == 60.0)));
    }
}
