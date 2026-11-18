//! `GuiChemicalWasher` (176 by 166): chemical washer.
//! Java reference: `mekanism.client.gui.machine.GuiChemicalWasher` (lines 23-56).
//!
//! Layout:
//! - Side holder at (176, 66), 26 by 57, holding fluid input (180, 71) and output (180, 102) slots
//! - Down arrow at (184, 91), 8 by 9
//! - Horizontal power bar at (115, 75), 54 by 6 frame, 52 by 4 fill
//! - Input fluid gauge at (7, 13), standard 16 by 58 (18 by 60 frame, capacity 10,000 mB)
//! - Input slurry gauge at (28, 13), standard 16 by 58 (18 by 60 frame, capacity 10,000 mB)
//! - Output clean slurry gauge at (131, 13), standard 16 by 58 (18 by 60 frame, capacity 10,000 mB)
//! - Progress bar at (64, 39), 48 by 8 (ProgressType.LARGE_RIGHT)
//! - Energy tab at (-26, 137)
//! - Title centered at y = 4; inventory label omitted (matches Java)
//! - Slots: fluid input bucket at (180, 71), fluid output container at (180, 102),
//!   slurry output container at (152, 56) with minus overlay, power at (152, 14)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chemical_oxidizer::{horizontal_power_bar, render_chemical_gauge};
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, pair, title, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct ChemicalWasher {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:chemical_washer";
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
        Box::new(ChemicalWasher { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for ChemicalWasher {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Side holder at (176, 66, 26, 57) drawn before slots
        c.blit_sprite("mekanism:holder_right", 176.0, 66.0, 26.0, 57.0);

        // Down arrow between the fluid slots at (184, 91, 8, 9)
        c.blit_sprite("mekanism:arrow/down", 184.0, 91.0, 8.0, 9.0);

        // Slots: fluid input (180, 71), slurry output (152, 56), fluid output (180, 102), power (152, 14)
        c.blit_sprite("mekanism:slot/input", 179.0, 70.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/extra", 151.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 151.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 179.0, 101.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 151.0, 13.0, 18.0, 18.0);

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

        // Input Fluid gauge at (7, 13), 16 by 58 inside 18 by 60 (t0, data 16, 17)
        let fluid_amt = data(16).max(0) as u64;
        let fluid_kind = data(17).max(0) as usize;
        let fluid_tip = render_chemical_gauge(
            c,
            mouse,
            7.0,
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

        // Input Slurry chemical gauge at (28, 13), 16 by 58 inside 18 by 60 (t1, data 18, 19)
        let in_amt = data(18).max(0) as u64;
        let in_kind = data(19).max(0) as usize;
        let in_tip = render_chemical_gauge(
            c,
            mouse,
            28.0,
            13.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            in_amt,
            10000,
            in_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = in_tip;
        }

        // Output Clean Slurry chemical gauge at (131, 13), 16 by 58 inside 18 by 60 (t2, data 20, 21)
        let out_amt = data(20).max(0) as u64;
        let out_kind = data(21).max(0) as usize;
        let out_tip = render_chemical_gauge(
            c,
            mouse,
            131.0,
            13.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            out_amt,
            10000,
            out_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = out_tip;
        }

        // Progress bar at (64, 39): 48 by 8 (LARGE_RIGHT)
        c.blit_sprite("mekanism:progress/large_right/empty", 64.0, 39.0, 48.0, 8.0);
        let active = data(0) != 0;
        if active {
            c.blit_sprite("mekanism:progress/large_right/full", 64.0, 39.0, 48.0, 8.0);
        }

        // Title centered across 176 at y = 4
        let title_text = host::translate("block.mekanism.chemical_washer", &[]);
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
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};
    use horizon_mod_sdk::screen_format::{self, Command};

    #[test]
    fn test_chemical_washer_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                1, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10000, 64, 2000, 42, 1000, 41, 0, 0, 0, 4000, 0, 100,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:chemical_washer".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open chemical washer");

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
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:arrow/down")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:progress/large_right/full")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && *rect == [115.0, 75.0, 54.0, 6.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Area { label, rect, .. } if label == "Energy" && *rect == [115.0, 75.0, 54.0, 6.0])));
    }

    #[test]
    fn test_chemical_washer_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:chemical_washer".into(),
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
