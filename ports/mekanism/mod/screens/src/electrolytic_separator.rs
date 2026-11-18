//! `GuiElectrolyticSeparator` (176 by 166): electrolytic separator.
//! Java reference: `mekanism.client.gui.machine.GuiElectrolyticSeparator` (lines 22-55).
//!
//! Layout:
//! - Vertical power bar at (164, 15), 6 by 54 frame, 4 by 52 fill at (165, 16)
//! - Input fluid gauge at (5, 10), standard 16 by 58 (18 by 60 frame, capacity 24,000 mB)
//! - Left gas output gauge at (58, 18), small 16 by 28 (18 by 30 frame, capacity 2,400 mB)
//! - Right gas output gauge at (100, 18), small 16 by 28 (18 by 30 frame, capacity 2,400 mB)
//! - Bidirectional progress arrow at (80, 30), 16 by 6 (ProgressType.BI)
//! - Gas dumping mode buttons: left at (7, 72) (id 5000), right at (159, 72) (id 5001)
//! - Energy tab at (-26, 137)
//! - Title centered between x = 23 and 176 at y = 6; inventory label omitted (matches Java)
//! - Slots: fluid input at (26, 35), left gas output at (59, 52), right gas output at (101, 52), power at (143, 35)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chemical_oxidizer::render_chemical_gauge;
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, pair, title, vertical_power_bar, Overlay, TITLE, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const DUMP_LEFT_ID: i32 = 5000;
pub const DUMP_RIGHT_ID: i32 = 5001;

pub struct ElectrolyticSeparator {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:electrolytic_separator";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(24),
            energy: Some(EnergyTabSpec::for_machine(24, Some(5))),
            upgrades: Some(UpgradesSpec::new(3, 4, 24, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(ElectrolyticSeparator { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for ElectrolyticSeparator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: fluid input (26, 35), left gas output (59, 52), right gas output (101, 52), power (143, 35)
        c.blit_sprite("mekanism:slot/input", 25.0, 34.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 58.0, 51.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 100.0, 51.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 142.0, 34.0, 18.0, 18.0);

        // Vertical power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        vertical_power_bar(c, 165.0, 16.0, power_frac);

        let power_rect = [164.0, 15.0, 6.0, 54.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Input Fluid gauge at (5, 10), 16 by 58 inside 18 by 60 (t0, data 16, 17, capacity 24,000)
        let fluid_amt = data(16).max(0) as u64;
        let fluid_kind = data(17).max(0) as usize;
        let fluid_tip = render_chemical_gauge(
            c,
            mouse,
            5.0,
            10.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            fluid_amt,
            24000,
            fluid_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = fluid_tip;
        }

        // Left Gas output gauge at (58, 18), 16 by 28 inside 18 by 30 (t1, data 18, 19, capacity 2,400)
        let left_amt = data(18).max(0) as u64;
        let left_kind = data(19).max(0) as usize;
        let left_tip = render_chemical_gauge(
            c,
            mouse,
            58.0,
            18.0,
            16.0,
            28.0,
            "mekanism:gauge/overlay/small",
            left_amt,
            2400,
            left_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = left_tip;
        }

        // Right Gas output gauge at (100, 18), 16 by 28 inside 18 by 30 (t2, data 20, 21, capacity 2,400)
        let right_amt = data(20).max(0) as u64;
        let right_kind = data(21).max(0) as usize;
        let right_tip = render_chemical_gauge(
            c,
            mouse,
            100.0,
            18.0,
            16.0,
            28.0,
            "mekanism:gauge/overlay/small",
            right_amt,
            2400,
            right_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = right_tip;
        }

        // Bidirectional progress bar at (80, 30): 16 by 6 (BI)
        c.blit_sprite("mekanism:progress/bidirectional/empty", 80.0, 30.0, 16.0, 6.0);
        let active = data(0) != 0;
        if active {
            c.blit_sprite("mekanism:progress/bidirectional/full", 80.0, 30.0, 16.0, 6.0);
        }

        // Gas Dumping Mode buttons
        // Left Gas Mode at (7, 72): button 5000, mode in data(22)
        let left_mode = (data(22).max(0) as usize) % 3;
        let (left_sprite, left_label) = match left_mode {
            1 => ("mekanism:chemical_mode/dump", "Dumping"),
            2 => ("mekanism:chemical_mode/dump_excess", "Dump excess"),
            _ => ("mekanism:chemical_mode/idle", "Idle"),
        };
        c.blit_sprite(left_sprite, 7.0, 72.0, 10.0, 10.0);
        c.text(left_label, 18.0, 73.0, TITLE, 0, 1.0);
        c.area(DUMP_LEFT_ID, [7.0, 72.0, 10.0, 10.0], AreaKind::Button, true, left_label);
        if inside(mouse, [7.0, 72.0, 10.0, 10.0]) && hovered_tip.is_none() {
            hovered_tip = Some(vec![(left_label.to_string(), WHITE)]);
        }

        // Right Gas Mode at (159, 72): button 5001, mode in data(23)
        let right_mode = (data(23).max(0) as usize) % 3;
        let (right_sprite, right_label) = match right_mode {
            1 => ("mekanism:chemical_mode/dump", "Dumping"),
            2 => ("mekanism:chemical_mode/dump_excess", "Dump excess"),
            _ => ("mekanism:chemical_mode/idle", "Idle"),
        };
        c.blit_sprite(right_sprite, 159.0, 72.0, 10.0, 10.0);
        let tw = host::text_width(right_label);
        c.text(right_label, 159.0 - 3.0 - tw, 73.0, TITLE, 0, 1.0);
        c.area(DUMP_RIGHT_ID, [159.0, 72.0, 10.0, 10.0], AreaKind::Button, true, right_label);
        if inside(mouse, [159.0, 72.0, 10.0, 10.0]) && hovered_tip.is_none() {
            hovered_tip = Some(vec![(right_label.to_string(), WHITE)]);
        }

        // Title centered between fluid gauge (x = 23) and panel edge (x = 176) at y = 6
        let title_text = host::translate("block.mekanism.electrolytic_separator", &[]);
        title(c, &title_text, 23.0, 153.0, 6.0);

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
            if p.button == MouseButton::Left {
                if inside((p.x, p.y), [7.0, 72.0, 10.0, 10.0]) {
                    host::send_button(DUMP_LEFT_ID);
                    host::play_click();
                    return true;
                }
                if inside((p.x, p.y), [159.0, 72.0, 10.0, 10.0]) {
                    host::send_button(DUMP_RIGHT_ID);
                    host::play_click();
                    return true;
                }
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
    fn test_electrolytic_separator_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                1, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12000, 64, 1200, 13, 1200, 4, 1, 2, 0, 4000, 0, 100,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:electrolytic_separator".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open electrolytic separator");

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
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:chemical_mode/dump")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:chemical_mode/dump_excess")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:progress/bidirectional/full")));
    }

    #[test]
    fn test_electrolytic_separator_dump_buttons() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:electrolytic_separator".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open");

        // Click left dump button
        let event_left = InputEvent::MousePressed(Pointer {
            x: 10.0,
            y: 75.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&event_left));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 5000", "click"]);

        // Click right dump button
        let event_right = InputEvent::MousePressed(Pointer {
            x: 162.0,
            y: 75.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&event_right));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 5001", "click"]);
    }

    #[test]
    fn test_electrolytic_separator_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:electrolytic_separator".into(),
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
