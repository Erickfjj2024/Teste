//! `GuiRotaryCondensentrator` (176 by 166): rotary condensentrator.
//! Java reference: `mekanism.client.gui.machine.GuiRotaryCondensentrator` (lines 30-90).
//!
//! Layout:
//! - Down arrow at (159, 44), 8 by 9
//! - Horizontal power bar at (115, 75), 54 by 6 frame, 52 by 4 fill
//! - Left gas gauge at (25, 13), standard 16 by 58 (18 by 60 frame, capacity 10,000 mB)
//! - Right fluid gauge at (133, 13), standard 16 by 58 (18 by 60 frame, capacity 10,000 mB)
//! - Toggle button at (4, 4), 18 by 18 (id 5200), button/toggle or toggle_flipped
//! - Progress arrow at (64, 39), 48 by 8:
//!   - Mode 0 (Condensentrating): ProgressType.LARGE_RIGHT
//!   - Mode 1 (Decondensentrating): ProgressType.LARGE_LEFT
//! - Mode label text at (4, 74): "Condensentrating" / "Decondensentrating"
//! - Energy tab at (-26, 137)
//! - Title centered at y = 4; inventory label omitted (matches Java)
//! - Slots: gas input at (5, 25) with plus overlay, gas output at (5, 56),
//!   fluid input at (155, 25), fluid output at (155, 56), power at (155, 5)
//! - Upgrade window: slots 5 (in) and 6 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chemical_oxidizer::{horizontal_power_bar, render_chemical_gauge};
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, pair, title, Overlay, TITLE, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const ROTARY_MODE_ID: i32 = 5200;

pub struct RotaryCondensentrator {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:rotary_condensentrator";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(24),
            energy: Some(EnergyTabSpec::for_machine(24, Some(5))),
            upgrades: Some(UpgradesSpec::new(5, 6, 24, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(RotaryCondensentrator { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for RotaryCondensentrator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Down arrow at (159, 44, 8, 9)
        c.blit_sprite("mekanism:arrow/down", 159.0, 44.0, 8.0, 9.0);

        // Slots: gas in (5, 25), gas out (5, 56), fluid in (155, 25), fluid out (155, 56), power (155, 5)
        c.blit_sprite("mekanism:slot/input", 4.0, 24.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 4.0, 24.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 4.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/input", 154.0, 24.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 154.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 154.0, 4.0, 18.0, 18.0);

        // Horizontal power bar at (115, 75)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        horizontal_power_bar(c, 115.0, 75.0, power_frac);

        let power_rect = [115.0, 75.0, 54.0, 6.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Left Gas gauge at (25, 13), 16 by 58 inside 18 by 60 (t1, data 18, 19, capacity 10,000)
        let gas_amt = data(18).max(0) as u64;
        let gas_kind = data(19).max(0) as usize;
        let gas_tip = render_chemical_gauge(
            c,
            mouse,
            25.0,
            13.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            gas_amt,
            10000,
            gas_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = gas_tip;
        }

        // Right Fluid gauge at (133, 13), 16 by 58 inside 18 by 60 (t0, data 16, 17, capacity 10,000)
        let fluid_amt = data(16).max(0) as u64;
        let fluid_kind = data(17).max(0) as usize;
        let fluid_tip = render_chemical_gauge(
            c,
            mouse,
            133.0,
            13.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            fluid_amt,
            10000,
            fluid_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = fluid_tip;
        }

        // Rotary mode: data(20) from server or data(22) from JSON (0 = Condensentrating, 1 = Decondensentrating)
        let mode = if data(20) == 1 || data(22) == 1 { 1 } else { 0 };
        let active = data(0) != 0;

        // Progress bar at (64, 39): 48 by 8 (LARGE_RIGHT when mode 0, LARGE_LEFT when mode 1)
        if mode == 0 {
            c.blit_sprite("mekanism:progress/large_right/empty", 64.0, 39.0, 48.0, 8.0);
            if active {
                c.blit_sprite("mekanism:progress/large_right/full", 64.0, 39.0, 48.0, 8.0);
            }
        } else {
            c.blit_sprite("mekanism:progress/large_left/empty", 64.0, 39.0, 48.0, 8.0);
            if active {
                c.blit_sprite("mekanism:progress/large_left/full", 64.0, 39.0, 48.0, 8.0);
            }
        }

        // Toggle button at (4, 4, 18, 18)
        let toggle_sprite = if mode == 1 {
            "mekanism:button/toggle_flipped"
        } else {
            "mekanism:button/toggle"
        };
        c.blit_sprite(toggle_sprite, 4.0, 4.0, 18.0, 18.0);
        let toggle_tip = host::translate("condensentrator.mekanism.toggle", &[]);
        c.area(ROTARY_MODE_ID, [4.0, 4.0, 18.0, 18.0], AreaKind::Button, true, &toggle_tip);
        if inside(mouse, [4.0, 4.0, 18.0, 18.0]) && hovered_tip.is_none() {
            hovered_tip = Some(vec![(toggle_tip, WHITE)]);
        }

        // Mode label text at (4, 74)
        let mode_text = if mode == 1 {
            host::translate("condensentrator.mekanism.decondensentrating", &[])
        } else {
            host::translate("condensentrator.mekanism.condensentrating", &[])
        };
        c.text(&mode_text, 4.0, 74.0, TITLE, 0, 1.0);

        // Title centered at y = 4
        let title_text = host::translate("block.mekanism.rotary_condensentrator", &[]);
        title(c, &title_text, 0.0, 176.0, 4.0);

        // Tooltip if hovered
        if let Some(lines) = hovered_tip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }

        // Chrome (tabs and overlays)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }

        if let InputEvent::MousePressed(p) = event {
            if p.button == MouseButton::Left && inside((p.x, p.y), [4.0, 4.0, 18.0, 18.0]) {
                host::send_button(ROTARY_MODE_ID);
                host::play_click();
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};
    use horizon_mod_sdk::screen_format::{self, Command};

    #[test]
    fn test_rotary_condensentrator_render_mode_condensentrating() {
        host::set_menu(host::Menu {
            data: vec![
                1, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1000, 64, 2000, 13, 0, 0, 0, 0, 0, 4000, 0, 100,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:rotary_condensentrator".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open rotary condensentrator");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let mut c = Canvas::new();
        screen.render(
            &Frame {
                mouse_x: 10.0,
                mouse_y: 10.0,
                partial_tick: 0.0,
                time_ms: 0,
            },
            &mut c,
        );
        let bytes = c.finish();
        let frame = screen_format::decode(&bytes).expect("decode frame");
        assert!(!frame.commands.is_empty());
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:button/toggle")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:progress/large_right/full")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && *rect == [115.0, 75.0, 54.0, 6.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Area { label, rect, .. } if label == "Energy" && *rect == [115.0, 75.0, 54.0, 6.0])));
    }

    #[test]
    fn test_rotary_condensentrator_render_mode_decondensentrating() {
        host::set_menu(host::Menu {
            data: vec![
                1, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1000, 64, 2000, 13, 1, 0, 0, 0, 0, 4000, 0, 100,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:rotary_condensentrator".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open rotary condensentrator");

        let mut c = Canvas::new();
        screen.render(
            &Frame {
                mouse_x: 10.0,
                mouse_y: 10.0,
                partial_tick: 0.0,
                time_ms: 0,
            },
            &mut c,
        );
        let bytes = c.finish();
        let frame = screen_format::decode(&bytes).expect("decode frame");
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:button/toggle_flipped")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:progress/large_left/full")));
    }

    #[test]
    fn test_rotary_condensentrator_toggle_mode_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:rotary_condensentrator".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open");

        // Click toggle mode button at (4, 4, 18, 18)
        let event = InputEvent::MousePressed(Pointer {
            x: 10.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 5200", "click"]);
    }

    #[test]
    fn test_rotary_condensentrator_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:rotary_condensentrator".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open");

        let event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 140.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        let handled = screen.input(&event);
        assert!(handled);
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 6000", "click"]);
    }
}
