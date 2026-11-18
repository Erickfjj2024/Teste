//! `GuiLaserTractorBeam` (176 by 166): 27 inventory slots (3 rows of 9) from (8, 16) to (152, 52),
//! player inventory at (8, 84), and security tab on the right.
//!
//! Java source of truth:
//! - `mekanism.client.gui.GuiLaserTractorBeam` (`GuiLaserTractorBeam.java:10-23`)
//! - `mekanism.common.tile.laser.TileEntityLaserTractorBeam`
//!
//! Numbers (`laser.rs`): 0 security mode.

use crate::chrome::{Chrome, ChromeSpec};
use crate::elements::{base_panel, inventory_label, slot_frame, title};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct LaserTractorBeam {
    chrome: Chrome,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:laser_tractor_beam",
        redstone: None,
        energy: None,
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = LaserTractorBeam {
        chrome: Chrome::new(chrome_spec, 176.0),
    };
    Ok((Box::new(screen), Setup { width: 176, height: 166, hide_inventory: false, own_labels: true }))
}

impl MekScreen for LaserTractorBeam {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // 27 item inventory slots (3 rows of 9) at (8 + 18*col, 16 + 18*row) -> frames at (7+18*col, 15+18*row)
        for row in 0..3 {
            for col in 0..9 {
                slot_frame(c, 7.0 + 18.0 * col as f32, 15.0 + 18.0 * row as f32);
            }
        }

        // Title and inventory label
        title(c, &host::translate("block.mekanism.laser_tractor_beam", &[]), 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

        // Chrome (security tab)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        self.chrome.input(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};

    #[test]
    fn laser_tractor_beam_renders_and_handles_security_click() {
        host::set_menu(host::Menu {
            data: vec![0], // 0: security
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:laser_tractor_beam".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open laser tractor beam");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));

        // Click security tab at (176, 34)
        let click_event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 9100".to_string(), "click".to_string()]);
    }
}
