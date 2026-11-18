//! `GuiPrecisionSawmill` (176 by 166): precision sawmill.
//! Java reference: `mekanism.client.gui.machine.GuiPrecisionSawmill` (lines 20-48).
//!
//! Layout:
//! - Up arrow at (60, 38)
//! - Vertical power bar at (164, 15), 6 by 54 frame, 4 by 52 fill
//! - Wide output slot at (111, 30), 42 by 26
//! - Progress bar at (78, 38), 25 by 9 (ProgressType.BAR)
//! - Energy tab at (-26, 137)
//! - Title centered at y = 6, inventory label at (8, 72)
//! - Slots: input at (56, 17), output 1 at (116, 35), output 2 at (132, 35), power at (56, 53)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, vertical_power_bar, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct PrecisionSawmill {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:precision_sawmill";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(16),
            energy: Some(EnergyTabSpec::for_machine(16, Some(5))),
            upgrades: Some(UpgradesSpec::new(3, 4, 16, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(PrecisionSawmill { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for PrecisionSawmill {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input at 56, 17; power at 56, 53
        c.blit_sprite("mekanism:slot/input", 55.0, 16.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 55.0, 52.0, 18.0, 18.0);

        // Wide output slot at (111, 30), 42 by 26 covering outputs at 116, 35 and 132, 35
        c.blit_sprite("mekanism:slot/output", 111.0, 30.0, 42.0, 26.0);

        // Up arrow at (60, 38)
        c.blit_sprite("mekanism:arrow/up", 60.0, 38.0, 8.0, 10.0);

        // Vertical power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        vertical_power_bar(c, 165.0, 16.0, power_frac);

        let power_rect = [164.0, 15.0, 6.0, 54.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        if inside(mouse, power_rect) {
            c.tooltip(&[(format!("Energy: {} / {}", energy, capacity).as_str(), WHITE)]);
        }

        // Progress bar at (78, 38): 25 by 9
        c.blit_sprite("mekanism:progress/bar/empty", 78.0, 38.0, 25.0, 9.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(19).max(1) as f32;
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        if prog_frac > 0.0 {
            let filled_w = (25.0 * prog_frac).floor();
            c.enable_scissor(78.0, 38.0, filled_w, 9.0);
            c.blit_sprite("mekanism:progress/bar/full", 78.0, 38.0, 25.0, 9.0);
            c.disable_scissor();
        }

        // Title and Inventory labels
        let title_text = host::translate("block.mekanism.precision_sawmill", &[]);
        title(c, &title_text, 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

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
    use horizon_mod_sdk::screen_format;

    #[test]
    fn test_precision_sawmill_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![50, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4000, 0, 200],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:precision_sawmill".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open precision sawmill");

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
    }

    #[test]
    fn test_precision_sawmill_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 45],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:precision_sawmill".into(),
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
