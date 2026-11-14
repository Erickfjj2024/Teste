//! `GuiFusionReactorController` (186 by 166) and its sub-windows (`GuiFusionReactorHeat`, `GuiFusionReactorFuel`, `GuiFusionReactorStats`).
//! Java sources:
//! - `java/generators/java/mekanism/generators/client/gui/GuiFusionReactorController.java`
//! - `java/generators/java/mekanism/generators/client/gui/GuiFusionReactorHeat.java`
//! - `java/generators/java/mekanism/generators/client/gui/GuiFusionReactorFuel.java`
//! - `java/generators/java/mekanism/generators/client/gui/GuiFusionReactorStats.java`
//!
//! Elements:
//! - Window dimensions: 186 x 166 (`imageWidth += 10`), titleLabelY = 5, inventoryLabelX = 13, inventoryLabelY = 74
//! - Main page:
//!   - "Formed" text at (13, 16)
//!   - Hohlraum slot at (84, 38)
//!   - Energy tab at (-26, 137, 26, 26): storing, producing
//!   - Heat tab at (-26, 109, 26, 26): dissipated
//!   - Right tabs:
//!     - Heat page tab at (186, 6, 26, 18)
//!     - Fuel page tab at (186, 34, 26, 18)
//!     - Stats page tab at (186, 62, 26, 18)
//!     - Security tab at (186, 90, 26, 18): data(37) from security_table.rs
//! - Heat page:
//!   - Plasma temperature gauge at (12, 50, 18, 60) and case gauge at (66, 50, 18, 60)
//!   - Water gauge (small 18x30) at (120, 84), steam gauge (small 18x30) at (156, 84), energy gauge (small 18x30) at (120, 46)
//!   - Transfer progress arrows at (34, 76), (88, 61), (88, 91)
//! - Fuel page:
//!   - Deuterium gauge (small 18x30) at (30, 64), fuel gauge (standard 18x60) at (84, 50), tritium gauge (small 18x30) at (138, 64)
//!   - Burning arrows at (52, 76) and (106, 76)
//!   - Injection rate line at y = 35 and textfield at (103, 115, 30, 12), field id 7200
//! - Stats page:
//!   - Strings 3 to 13 (air-cooled and water-cooled performance figures)
//!
//! Differences from Java / Não Feito:
//! - // NÃO FEITO: GuiWarningTab não suportada no servidor
use crate::elements::{base_panel, data, inside, inventory_label, pair, string, title, TITLE, WHITE};
use crate::security_table;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, FieldFilter, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

const TAB_SECURITY: u32 = 0xFFFF_A160;
const TAB_HEAT: u32 = 0xFFFF_5555;
const TAB_FUEL: u32 = 0xFFFF_AA00;
const TAB_STATS: u32 = 0xFF55_9EFF;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ActivePage {
    #[default]
    Controller,
    Heat,
    Fuel,
    Stats,
}

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(FusionReactorControllerScreen::new()),
        Setup {
            width: 186,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

pub struct FusionReactorControllerScreen {
    security_idx: Option<usize>,
    page: ActivePage,
    injection_rate_text: String,
}

impl FusionReactorControllerScreen {
    pub fn new() -> Self {
        let security_idx = security_table::SECURED
            .iter()
            .find(|(name, _)| *name == "mekanism:fusion_reactor_controller")
            .map(|(_, idx)| *idx);
        Self {
            security_idx,
            page: ActivePage::Controller,
            injection_rate_text: String::new(),
        }
    }

    fn render_gauge(
        &self,
        c: &mut Canvas,
        mouse: (f32, f32),
        rect: [f32; 4],
        overlay: &str,
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
        c.blit_sprite(overlay, inner[0], inner[1], inner[2], inner[3]);
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
}

impl MekScreen for FusionReactorControllerScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);

        match self.page {
            ActivePage::Heat => {
                base_panel(c, 186.0, 166.0);
                title(c, "Heat", 0.0, 186.0, 5.0);
                c.blit_sprite("mekanism:button/back", 6.0, 6.0, 12.0, 12.0);
                c.area(7210, [6.0, 6.0, 12.0, 12.0], AreaKind::Button, true, "Back");

                // Plasma temperature gauge at (12, 50, 18, 60)
                let p_frac = (data(29) as f32 / 1000.0).clamp(0.0, 1.0);
                c.blit_sprite("mekanism:gauge/normal", 12.0, 50.0, 18.0, 60.0);
                let p_fill = (58.0 * p_frac).floor();
                if p_fill > 0.0 {
                    c.fill(13.0, 51.0 + 58.0 - p_fill, 16.0, p_fill, 0xFFFF_3300);
                }
                c.blit_sprite("mekanism:gauge/overlay/standard", 13.0, 51.0, 16.0, 58.0);
                if inside(mouse, [12.0, 50.0, 18.0, 60.0]) {
                    let tip = format!("Plasma Temp: {} kK", data(21));
                    c.tooltip(&[(&tip, WHITE)]);
                }

                // Case temperature gauge at (66, 50, 18, 60)
                let c_frac = (data(30) as f32 / 1000.0).clamp(0.0, 1.0);
                c.blit_sprite("mekanism:gauge/normal", 66.0, 50.0, 18.0, 60.0);
                let c_fill = (58.0 * c_frac).floor();
                if c_fill > 0.0 {
                    c.fill(67.0, 51.0 + 58.0 - c_fill, 16.0, c_fill, 0xFFFF_8800);
                }
                c.blit_sprite("mekanism:gauge/overlay/standard", 67.0, 51.0, 16.0, 58.0);
                if inside(mouse, [66.0, 50.0, 18.0, 60.0]) {
                    let tip = format!("Case Temp: {} K", data(22) as f32 / 10.0);
                    c.tooltip(&[(&tip, WHITE)]);
                }

                // Small gauges at right:
                // Water gauge at (120, 84, 18, 30)
                self.render_gauge(
                    c,
                    mouse,
                    [120.0, 84.0, 18.0, 30.0],
                    "mekanism:gauge/overlay/small",
                    pair(6, 7),
                    pair(12, 13),
                    "Water",
                    "minecraft:block/water_still",
                    0xFF3F_76E4,
                );
                // Steam gauge at (156, 84, 18, 30)
                self.render_gauge(
                    c,
                    mouse,
                    [156.0, 84.0, 18.0, 30.0],
                    "mekanism:gauge/overlay/small",
                    pair(8, 9),
                    pair(14, 15),
                    "Steam",
                    "mekanism:liquid/liquid",
                    0xFFCC_CCCC,
                );
                // Energy gauge at (120, 46, 18, 30)
                let e = pair(25, 26);
                c.blit_sprite("mekanism:gauge/normal", 120.0, 46.0, 18.0, 30.0);
                c.blit_sprite("mekanism:gauge/overlay/small", 121.0, 47.0, 16.0, 28.0);
                if inside(mouse, [120.0, 46.0, 18.0, 30.0]) {
                    let tip = format!("Energy: {e} kFE");
                    c.tooltip(&[(&tip, WHITE)]);
                }

                // Progress arrows
                c.blit_sprite("mekanism:progress/small_right/empty", 34.0, 76.0, 18.0, 8.0);
                c.blit_sprite("mekanism:progress/small_right/empty", 88.0, 61.0, 18.0, 8.0);
                c.blit_sprite("mekanism:progress/small_right/empty", 88.0, 91.0, 18.0, 8.0);
                return;
            }
            ActivePage::Fuel => {
                base_panel(c, 186.0, 166.0);
                title(c, "Fuel", 0.0, 186.0, 5.0);
                c.blit_sprite("mekanism:button/back", 6.0, 6.0, 12.0, 12.0);
                c.area(7210, [6.0, 6.0, 12.0, 12.0], AreaKind::Button, true, "Back");

                // Injection rate
                c.text(&format!("Injection Rate: {}", data(23)), 6.0, 35.0, TITLE, 0, 1.0);
                c.text("Set:", 75.0, 117.0, TITLE, 0, 0.9);
                c.blit_sprite("mekanism:digital_text_input", 102.0, 114.0, 32.0, 14.0);
                c.field(7200, [103.0, 115.0, 30.0, 12.0], 2, FieldFilter::Digits, &self.injection_rate_text, 1, WHITE);

                // Gauges:
                // Deuterium (small) at (30, 64, 18, 30)
                self.render_gauge(
                    c,
                    mouse,
                    [30.0, 64.0, 18.0, 30.0],
                    "mekanism:gauge/overlay/small",
                    pair(0, 1),
                    1000,
                    "Deuterium",
                    "mekanism:liquid/liquid",
                    0xFFFF_DD4F,
                );
                // Fuel D-T (standard) at (84, 50, 18, 60)
                self.render_gauge(
                    c,
                    mouse,
                    [84.0, 50.0, 18.0, 60.0],
                    "mekanism:gauge/overlay/standard",
                    pair(4, 5),
                    1000,
                    "D-T Fuel",
                    "mekanism:liquid/liquid",
                    0xFFFF_383C,
                );
                // Tritium (small) at (138, 64, 18, 30)
                self.render_gauge(
                    c,
                    mouse,
                    [138.0, 64.0, 18.0, 30.0],
                    "mekanism:gauge/overlay/small",
                    pair(2, 3),
                    1000,
                    "Tritium",
                    "mekanism:liquid/liquid",
                    0xFF55_9EFF,
                );

                // Burning arrows
                c.blit_sprite("mekanism:progress/small_right/empty", 52.0, 76.0, 18.0, 8.0);
                c.blit_sprite("mekanism:progress/small_left/empty", 106.0, 76.0, 18.0, 8.0);
                return;
            }
            ActivePage::Stats => {
                base_panel(c, 186.0, 166.0);
                title(c, "Stats", 0.0, 186.0, 5.0);
                c.blit_sprite("mekanism:button/back", 6.0, 6.0, 12.0, 12.0);
                c.area(7210, [6.0, 6.0, 12.0, 12.0], AreaKind::Button, true, "Back");

                c.text("Passive Cooling:", 6.0, 20.0, TITLE, 0, 1.0);
                for i in 3..8 {
                    let s = string(i);
                    if !s.is_empty() {
                        c.text(&s, 6.0, 22.0 + 10.0 * (i - 2) as f32, TITLE, 0, 0.85);
                    }
                }

                c.text("Active Cooling:", 6.0, 86.0, TITLE, 0, 1.0);
                for i in 8..14 {
                    let s = string(i);
                    if !s.is_empty() {
                        c.text(&s, 6.0, 88.0 + 10.0 * (i - 7) as f32, TITLE, 0, 0.85);
                    }
                }
                return;
            }
            ActivePage::Controller => {}
        }

        // Main controller screen
        base_panel(c, 186.0, 166.0);
        title(c, "Fusion Reactor Controller", 0.0, 186.0, 5.0);
        inventory_label(c, 13.0, 74.0);

        // Formed label at (13, 16)
        c.text(&host::translate("gui.mekanism.formed", &[]), 13.0, 16.0, TITLE, 0, 1.0);

        // Hohlraum slot at (84, 38)
        c.blit_sprite("mekanism:slot/normal", 84.0, 38.0, 18.0, 18.0);

        // Energy tab at (-26, 137, 26, 26)
        let energy_rect = [-26.0, 137.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/energy_info", energy_rect[0], energy_rect[1], energy_rect[2], energy_rect[3]);
        c.area(7003, energy_rect, AreaKind::Tab, true, "Energy");
        if inside(mouse, energy_rect) {
            let storing = pair(25, 26);
            let prod = data(28);
            let tip = format!("Storing: {storing} kFE\nProducing: {prod} FE/t\nUnit: Forge Energy");
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Heat tab at (-26, 109, 26, 26)
        let heat_rect = [-26.0, 109.0, 26.0, 26.0];
        c.blit_sprite("mekanism:tab/heat_info_k", heat_rect[0], heat_rect[1], heat_rect[2], heat_rect[3]);
        c.area(7004, heat_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, heat_rect) {
            let tip = format!("Dissipated: {}.{}{} K/t", data(31), data(32), data(33));
            c.tooltip(&[(&tip, WHITE)]);
        }

        // Right tabs:
        // 1. Heat tab at (186, 6, 26, 18)
        let h_rect = [186.0, 6.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", h_rect, TAB_HEAT);
        c.blit_sprite("mekanism:tab/heat_info_k", 186.0 + 4.0, 6.0, 18.0, 18.0);
        c.area(7201, h_rect, AreaKind::Tab, true, "Heat");
        if inside(mouse, h_rect) {
            c.tooltip(&[("Heat", WHITE)]);
        }

        // 2. Fuel tab at (186, 34, 26, 18)
        let f_rect = [186.0, 34.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", f_rect, TAB_FUEL);
        c.blit_sprite("mekanism:button/radioactive", 186.0 + 4.0, 34.0, 18.0, 18.0);
        c.area(7202, f_rect, AreaKind::Tab, true, "Fuel");
        if inside(mouse, f_rect) {
            c.tooltip(&[("Fuel", WHITE)]);
        }

        // 3. Stats tab at (186, 62, 26, 18)
        let s_rect = [186.0, 62.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", s_rect, TAB_STATS);
        c.blit_sprite("mekanism:button/radio", 186.0 + 4.0, 62.0, 18.0, 18.0);
        c.area(7203, s_rect, AreaKind::Tab, true, "Stats");
        if inside(mouse, s_rect) {
            c.tooltip(&[("Stats", WHITE)]);
        }

        // 4. Security tab at (186, 90, 26, 18)
        if let Some(sec_idx) = self.security_idx {
            let sec_rect = [186.0, 90.0, 26.0, 18.0];
            c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
            let sec_mode = (data(sec_idx).max(0) as usize) % 3;
            let icon_sprite = match sec_mode {
                1 => "mekanism:security/private",
                2 => "mekanism:security/trusted",
                _ => "mekanism:security/public",
            };
            c.blit_sprite(icon_sprite, 186.0 + 4.0, 90.0, 18.0, 18.0);
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
                // Sub-page back button
                if self.page != ActivePage::Controller && inside(mouse, [6.0, 6.0, 12.0, 12.0]) {
                    self.page = ActivePage::Controller;
                    host::play_click();
                    return true;
                }
                if self.page != ActivePage::Controller {
                    return true;
                }

                // Main page tabs:
                // Heat sub-page tab
                if inside(mouse, [186.0, 6.0, 26.0, 18.0]) {
                    self.page = ActivePage::Heat;
                    host::play_click();
                    return true;
                }
                // Fuel sub-page tab
                if inside(mouse, [186.0, 34.0, 26.0, 18.0]) {
                    self.page = ActivePage::Fuel;
                    host::play_click();
                    return true;
                }
                // Stats sub-page tab
                if inside(mouse, [186.0, 62.0, 26.0, 18.0]) {
                    self.page = ActivePage::Stats;
                    host::play_click();
                    return true;
                }
                // Energy tab
                if inside(mouse, [-26.0, 137.0, 26.0, 26.0]) {
                    host::play_click();
                    return true;
                }
                // Heat tab
                if inside(mouse, [-26.0, 109.0, 26.0, 26.0]) {
                    host::play_click();
                    return true;
                }
                // Security tab
                if self.security_idx.is_some() && inside(mouse, [186.0, 90.0, 26.0, 18.0]) {
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
                if f.id == 7200 {
                    self.injection_rate_text = f.text.clone();
                    host::send_text(7200, &f.text);
                    return true;
                }
                false
            }
            InputEvent::FieldChanged(f) => {
                if f.id == 7200 {
                    self.injection_rate_text = f.text.clone();
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
    fn test_fusion_reactor_render_and_input() {
        let mut d = vec![0; 40];
        d[23] = 2; // injection rate 2
        d[28] = 50000; // producing 50,000 FE/t
        d[37] = 0; // security Public

        host::set_menu(host::Menu {
            data: d,
            strings: vec![],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:fusion_reactor_controller".into(),
            gui_width: 186.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open fusion controller");
        assert_eq!(setup.width, 186);
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

        // Test click Fuel tab (at 190, 40)
        let click_fuel = InputEvent::MousePressed(Pointer {
            x: 190.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_fuel));
        let sent_fuel = host::take_sent();
        assert_eq!(sent_fuel, vec!["click".to_string()]);

        // Submit injection rate field 7200
        let field_sub = InputEvent::FieldSubmitted(FieldText {
            id: 7200,
            text: "4".into(),
        });
        assert!(screen.input(&field_sub));
        let sent_field = host::take_sent();
        assert_eq!(sent_field, vec!["text 7200 \"4\"".to_string()]);
    }
}
