//! `GuiFluidTank` (`mekanism:<tier>_fluid_tank`, 176 by 166): wide fluid gauge at (48, 18),
//! input slot at (146, 19) with frame at (145, 18) and input overlay, output slot at (146, 51)
//! with frame at (145, 50) and output overlay, armor side holder at (-26, 62),
//! container edit mode tab at (176, 138), and security tab at (176, 34).
//!
//! Java: `GuiFluidTank.java:14-36`, `MekanismTileContainer<TileEntityFluidTank>`.
//!
//! Server data (`tank.rs` / `convert.py` `tank_menu`):
//! - `pair(0, 1)`: fluid stored (mB)
//! - `data(2)`: fluid / chemical type index (0 = empty, 1..=72 = chemical type)
//! - `pair(3, 4)`: tank capacity (mB)
//! - `data(5)`: edit mode (0 = Both, 1 = Fill, 2 = Empty)
//! - `data(6)`: security mode (0 = Public, 1 = Private, 2 = Trusted)
//!
//! Action buttons:
//! - 5100: Edit mode next (`EDIT_MODE_NEXT`)
//! - 5101: Edit mode previous (`EDIT_MODE_PREV`)
//! - 9100: Security mode next (`SECURITY_NEXT`)
//! - 9101: Security mode previous (`SECURITY_PREV`)
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const EDIT_MODE_NEXT: i32 = 5100;
pub const EDIT_MODE_PREV: i32 = 5101;
pub const SECURITY_NEXT: i32 = 9100;
pub const SECURITY_PREV: i32 = 9101;

pub const TAB_CONTAINER_EDIT_MODE: u32 = 0xFF36_6BD0; // SpecialColors.TAB_CONTAINER_EDIT_MODE
pub const TAB_SECURITY: u32 = 0xFFFF_A160;

pub struct FluidTankScreen {
    menu_name: String,
}

pub fn open(info: &OpenInfo) -> Opened {
    let screen = FluidTankScreen {
        menu_name: info.menu.clone(),
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl FluidTankScreen {
    fn title_text(&self) -> String {
        let key = match self.menu_name.as_str() {
            "mekanism:basic_fluid_tank" => "block.mekanism.basic_fluid_tank",
            "mekanism:advanced_fluid_tank" => "block.mekanism.advanced_fluid_tank",
            "mekanism:elite_fluid_tank" => "block.mekanism.elite_fluid_tank",
            "mekanism:ultimate_fluid_tank" => "block.mekanism.ultimate_fluid_tank",
            "mekanism:creative_fluid_tank" => "block.mekanism.creative_fluid_tank",
            _ => "block.mekanism.basic_fluid_tank",
        };
        let translated = host::translate(key, &[]);
        if !translated.is_empty() && translated != key {
            translated
        } else {
            let sub = self.menu_name.strip_prefix("mekanism:").unwrap_or(&self.menu_name);
            sub.replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

impl MekScreen for FluidTankScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Title and Inventory labels
        title(c, &self.title_text(), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

        // Armor holder at (-26, 62, 26, 98) with 5 slot frames
        c.blit_sprite_tinted("mekanism:holder_left", [-26.0, 62.0, 26.0, 98.0], WHITE);
        for k in 0..5 {
            slot_frame(c, -23.0, 66.0 + 18.0 * k as f32);
        }

        // Slots: input at (146, 19), frame at (145, 18); output at (146, 51), frame at (145, 50)
        c.blit_sprite("mekanism:slot/input", 145.0, 18.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/input", 145.0, 18.0, 18.0, 18.0);

        c.blit_sprite("mekanism:slot/output", 145.0, 50.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/output", 145.0, 50.0, 18.0, 18.0);

        // Wide Fluid Gauge at (48, 18, 66, 50)
        c.blit_sprite("mekanism:gauge/normal", 48.0, 18.0, 66.0, 50.0);

        let stored = pair(0, 1);
        let capacity = pair(3, 4);
        let kind = data(2);
        let gauge_rect = [49.0, 19.0, 64.0, 48.0];

        if stored > 0 && capacity > 0 {
            let fill = (stored as f32 / capacity as f32).clamp(0.0, 1.0);
            let fluid_color = if kind > 0 {
                crate::factory::chemical_color_by_index(kind)
            } else {
                0xFF3F_76E4
            };
            c.fluid("minecraft:block/water_still", gauge_rect, fill, fluid_color);
        }

        c.blit_sprite("mekanism:gauge/overlay/wide", 49.0, 19.0, 64.0, 48.0);

        let mut tooltip_lines: Option<Vec<(String, u32)>> = None;

        if inside(mouse, gauge_rect) {
            if stored == 0 {
                let empty_trans = host::translate("gui.mekanism.empty", &[]);
                let empty_text = if !empty_trans.is_empty() && !empty_trans.starts_with("gui.") {
                    empty_trans
                } else {
                    "Empty".to_string()
                };
                tooltip_lines = Some(vec![(empty_text, WHITE)]);
            } else {
                let label = if kind == 0 {
                    "Fluid".to_string()
                } else {
                    crate::factory::chemical_name_by_index(kind)
                };
                tooltip_lines = Some(vec![(format!("{label}: {stored} mB"), WHITE)]);
            }
        }

        // Edit Mode Tab at (176, 138, 26, 18) (GuiContainerEditModeTab.java:31-62, SpecialColors.TAB_CONTAINER_EDIT_MODE = 0xFF366BD0)
        let edit_rect = [176.0, 138.0, 26.0, 18.0];
        let edit_mode = (data(5).max(0) as usize) % 3;
        c.blit_sprite_tinted("mekanism:holder_right", edit_rect, TAB_CONTAINER_EDIT_MODE);
        let edit_sprite = match edit_mode {
            1 => "mekanism:container_edit_mode/fill",
            2 => "mekanism:container_edit_mode/empty",
            _ => "mekanism:container_edit_mode/both",
        };
        c.blit_sprite(edit_sprite, 180.0, 138.0, 18.0, 18.0);
        c.area(EDIT_MODE_NEXT, edit_rect, AreaKind::Tab, true, "Edit mode");
        if inside(mouse, edit_rect) {
            let key = match edit_mode {
                1 => "fluid_container.mekanism.fill",
                2 => "fluid_container.mekanism.empty",
                _ => "fluid_container.mekanism.both",
            };
            let trans = host::translate(key, &[]);
            let tip = if !trans.is_empty() && !trans.starts_with("fluid_container.") {
                trans
            } else {
                match edit_mode {
                    1 => "Fill".to_string(),
                    2 => "Empty".to_string(),
                    _ => "Both".to_string(),
                }
            };
            tooltip_lines = Some(vec![(tip, WHITE)]);
        }

        // Security Tab at (176, 34, 26, 18)
        let sec_rect = [176.0, 34.0, 26.0, 18.0];
        let sec_mode = (data(6).max(0) as usize) % 3;
        c.blit_sprite_tinted("mekanism:holder_right", sec_rect, TAB_SECURITY);
        let sec_sprite = match sec_mode {
            1 => "mekanism:security/private",
            2 => "mekanism:security/protected",
            _ => "mekanism:security/public",
        };
        c.blit_sprite(sec_sprite, 180.0, 34.0, 18.0, 18.0);
        c.area(SECURITY_NEXT, sec_rect, AreaKind::Tab, true, "Security");
        if inside(mouse, sec_rect) {
            let tip = match sec_mode {
                1 => "Security: Private",
                2 => "Security: Trusted",
                _ => "Security: Public",
            };
            tooltip_lines = Some(vec![(tip.to_string(), WHITE)]);
        }

        if let Some(lines) = tooltip_lines {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                let mouse = (p.x, p.y);
                if inside(mouse, [176.0, 138.0, 26.0, 18.0]) {
                    if p.button == MouseButton::Right {
                        host::send_button(EDIT_MODE_PREV);
                    } else {
                        host::send_button(EDIT_MODE_NEXT);
                    }
                    host::play_click();
                    return true;
                }
                if inside(mouse, [176.0, 34.0, 26.0, 18.0]) {
                    if p.button == MouseButton::Right {
                        host::send_button(SECURITY_PREV);
                    } else {
                        host::send_button(SECURITY_NEXT);
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
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, Pointer};

    #[test]
    fn fluid_tank_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                5000, // stored lo
                0,    // stored hi
                1,    // fluid kind
                32000, // capacity lo
                0,    // capacity hi
                0,    // edit mode (both)
                0,    // security mode (public)
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:basic_fluid_tank".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open fluid tank");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);
        assert!(!setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 60.0,
            mouse_y: 30.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        let commands = decoded.commands;

        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:gauge/normal")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:gauge/overlay/wide")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Fluid { sprite, .. } if sprite == "minecraft:block/water_still")));
        // Check edit mode tab tinted with SpecialColors.TAB_CONTAINER_EDIT_MODE
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, argb, .. } if sprite == "mekanism:holder_right" && *argb == TAB_CONTAINER_EDIT_MODE)));

        // Click Edit Mode tab (left click)
        let click_edit = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 145.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_edit));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 5100".to_string(), "click".to_string()]);

        // Click Security tab (right click)
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 40.0,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9101".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_fluid_tank_edit_mode_hover_and_toggle_prev() {
        host::set_menu(host::Menu {
            data: vec![
                0, // stored lo = 0 (empty)
                0, // stored hi
                0, // fluid kind
                32000, // capacity lo
                0, // capacity hi
                1, // edit mode (Fill)
                0, // security mode (public)
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:basic_fluid_tank".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, _setup) = open(&info).expect("open fluid tank");

        // Hover over edit mode tab at (185, 145)
        let frame_edit = Frame {
            mouse_x: 185.0,
            mouse_y: 145.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame_edit, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");

        // Check tooltip has Fill (fluid_container.mekanism.fill) without "Edit mode:" prefix
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "Fill" || t == "fluid_container.mekanism.fill"))));

        // Right click edit mode tab sends EDIT_MODE_PREV (5101)
        let click_edit_prev = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 145.0,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_edit_prev));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 5101".to_string(), "click".to_string()]);
    }
}
