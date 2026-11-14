//! `GuiDimensionalStabilizer` (`mekanism:dimensional_stabilizer`, 176 by 166):
//! 5×5 chunk loading grid at (63, 19), center button at (83, 39) with radius enable/disable,
//! up arrow at (52, 28) and north label at (49, 41), energy slot at (143, 35) with power frame at (142, 34),
//! vertical power bar at (164, 15), energy tab at (-26, 137), redstone control tab at (176, 137),
//! and security tab at (176, 34).
//!
//! Java: `GuiDimensionalStabilizer.java:32-143`, `MekanismTileContainer<TileEntityDimensionalStabilizer>`,
//! `TileEntityDimensionalStabilizer.java`.
//!
//! Differences from Java:
//! - In Java, `GuiVisualsTab` is added to toggle visual chunk boundaries. The port does not have a visualizer component.
//!   // NÃO FEITO: a aba de visualização de chunks (GuiVisualsTab) não é sincronizada pelo servidor nem suportada no cliente.
//!
//! Server data (`dimensional_stabilizer.rs` / `convert.py` `stabilizer_menu`):
//! - `pair(0, 1)`: energy stored (FE)
//! - `pair(2, 3)`: energy capacity (FE)
//! - `data(4)`: active (1 or 0)
//! - `data(5)`: chunks loaded count
//! - `pair(6, 7)`: energy usage per tick (FE/t)
//! - `data(8)`: enable ring radius (1..=2, 0 if none)
//! - `data(9)`: disable ring radius (2..=1, 0 if none)
//! - `data(10)`: redstone control mode (0 disabled, 1 high, 2 low, 3 pulse)
//! - `pair(11, 12)`: energy needed to reach capacity (FE)
//! - `data(13 + x * 5 + z)`: chunk loading state for grid coordinate (x, z) in [0..5)
//! - `data(38)`: enable * 3 + disable
//! - `data(39)`: security mode (0 Public, 1 Private, 2 Trusted, from `security_table.rs`)
//!
//! Action buttons:
//! - 7500..7524: Toggle chunk loading at grid (x, z) (`BUTTON_TOGGLE_CHUNK_BASE + x * 5 + z`)
//! - 7530 + radius: Enable chunk loading for radius (`BUTTON_RING_ENABLE_BASE + radius`)
//! - 7540 + radius: Disable chunk loading for radius (`BUTTON_RING_DISABLE_BASE + radius`)
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, vertical_power_bar, TITLE, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const BUTTON_TOGGLE_CHUNK_BASE: i32 = 7500;
pub const BUTTON_RING_ENABLE_BASE: i32 = 7530;
pub const BUTTON_RING_DISABLE_BASE: i32 = 7540;

pub const CHUNK_BLUE: u32 = 0xFF36_6BD0; // EnumColor.DARK_BLUE
pub const CHUNK_DARK: u32 = 0xFF2A_2A2A;

pub struct DimensionalStabilizerScreen {
    chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:dimensional_stabilizer",
        redstone: Some(10),
        energy: Some(EnergyTabSpec::new(6, 11, 12, Some(4))),
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let chrome = Chrome::new(chrome_spec, 176.0);
    let screen = DimensionalStabilizerScreen { chrome };
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

impl MekScreen for DimensionalStabilizerScreen {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Title and Inventory labels (inventory label at y = 74 as in Java inventoryLabelY += 2)
        title(c, &host::translate("block.mekanism.dimensional_stabilizer", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 74.0);

        // Vertical power bar at (164, 15)
        let energy = pair(0, 1);
        let capacity = pair(2, 3);
        let fraction = if capacity > 0 { energy as f32 / capacity as f32 } else { 0.0 };
        vertical_power_bar(c, 165.0, 16.0, fraction);

        // Energy slot at (143, 35) with power frame at (142, 34)
        slot_frame(c, 142.0, 34.0);
        c.blit_sprite("mekanism:slot/power", 142.0, 34.0, 18.0, 18.0);

        // Up arrow at (52, 28) and North label "N" centered at (49, 41)
        c.blit_sprite("mekanism:arrow/up", 52.0, 28.0, 8.0, 12.0);
        c.text("N", 49.0, 41.0, TITLE, 0, 1.0);

        let mut hovered_tooltip: Option<Vec<(String, u32)>> = None;

        // 5×5 Chunk Loading Grid at (63 + 10*x, 19 + 10*z)
        for x in 0..5 {
            for z in 0..5 {
                let px = 63.0 + 10.0 * x as f32;
                let py = 19.0 + 10.0 * z as f32;
                let btn_rect = [px, py, 10.0, 10.0];
                let is_hover = inside(mouse, btn_rect);

                if x == 2 && z == 2 {
                    // Center chunk is always loaded
                    c.fill(px, py, 10.0, 10.0, CHUNK_BLUE);
                    c.area(
                        BUTTON_RING_ENABLE_BASE,
                        btn_rect,
                        AreaKind::Button,
                        true,
                        "Center Chunk",
                    );

                    if is_hover {
                        let mut lines = vec![("The chunk of the stabilizer is always loaded.".to_string(), WHITE)];
                        let enable_r = data(8);
                        let disable_r = data(9);
                        if enable_r > 0 {
                            lines.push((
                                format!("Left click to enable chunk loading for all chunks with a radius of {enable_r}."),
                                WHITE,
                            ));
                        }
                        if disable_r > 0 {
                            lines.push((
                                format!("Right click to disable chunk loading for all chunks with a radius of {disable_r}."),
                                WHITE,
                            ));
                        }
                        hovered_tooltip = Some(lines);
                    }
                } else {
                    let idx = x * 5 + z;
                    let loaded = data(13 + idx) == 1;
                    let color = if loaded { CHUNK_BLUE } else { CHUNK_DARK };
                    c.fill(px, py, 10.0, 10.0, color);
                    let btn_id = BUTTON_TOGGLE_CHUNK_BASE + idx as i32;
                    c.area(btn_id, btn_rect, AreaKind::Button, true, "Chunk");

                    if is_hover {
                        let dx = x as i32 - 2;
                        let dz = z as i32 - 2;
                        let action = if loaded { "off" } else { "on" };
                        let tip = format!("Toggle chunk loading {action} at ({dx:+}, {dz:+}) from here");
                        hovered_tooltip = Some(vec![(tip, WHITE)]);
                    }
                }
            }
        }

        // Render Chrome (energy tab, redstone tab, security tab)
        self.chrome.render(c, mouse);

        if let Some(lines) = hovered_tooltip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }

        match event {
            InputEvent::MousePressed(p) => {
                let mouse = (p.x, p.y);
                for x in 0..5 {
                    for z in 0..5 {
                        let px = 63.0 + 10.0 * x as f32;
                        let py = 19.0 + 10.0 * z as f32;
                        if inside(mouse, [px, py, 10.0, 10.0]) {
                            if x == 2 && z == 2 {
                                if p.button == MouseButton::Right {
                                    let disable_r = data(9);
                                    if disable_r > 0 {
                                        host::send_button(BUTTON_RING_DISABLE_BASE + disable_r);
                                        host::play_click();
                                        return true;
                                    }
                                } else {
                                    let enable_r = data(8);
                                    if enable_r > 0 {
                                        host::send_button(BUTTON_RING_ENABLE_BASE + enable_r);
                                        host::play_click();
                                        return true;
                                    }
                                }
                            } else {
                                let idx = x * 5 + z;
                                host::send_button(BUTTON_TOGGLE_CHUNK_BASE + idx as i32);
                                host::play_click();
                                return true;
                            }
                        }
                    }
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
    fn dimensional_stabilizer_renders_and_handles_buttons() {
        let mut test_data = vec![
            1000, // energy lo
            0,    // energy hi
            16000, // capacity lo
            0,    // capacity hi
            1,    // active
            1,    // chunks loaded
            2000, // energy per tick lo
            0,    // energy per tick hi
            1,    // enable radius
            0,    // disable radius
            0,    // redstone control
            15000, // needed lo
            0,    // needed hi
        ];
        // 25 chunks: center [2][2] is loaded
        for x in 0..5 {
            for z in 0..5 {
                test_data.push(if x == 2 && z == 2 { 1 } else { 0 });
            }
        }
        test_data.push(1 * 3 + 0); // data(38)
        test_data.push(0);         // data(39) security mode

        host::set_menu(host::Menu {
            data: test_data,
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:dimensional_stabilizer".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open dimensional stabilizer");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);
        assert!(!setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 70.0,
            mouse_y: 25.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        let commands = decoded.commands;

        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:bar/base")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:slot/power")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:arrow/up")));
        assert!(commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "N")));

        // Test clicking chunk (0, 0) -> id 7500
        let click_chunk = InputEvent::MousePressed(Pointer {
            x: 65.0,
            y: 22.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_chunk));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7500".to_string(), "click".to_string()]);

        // Test clicking center chunk (2, 2) at (85, 42) -> left click triggers enable radius 1 -> 7531
        let click_center = InputEvent::MousePressed(Pointer {
            x: 85.0,
            y: 42.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_center));
        let sent2 = host::take_sent();
        assert_eq!(sent2, vec!["button 7531".to_string(), "click".to_string()]);
    }
}
