//! `GuiPortableTeleporter` (`mekanism:portable_teleporter`, 176 by 172): the title at (22, 4), the status line at (7, 7),
//! the vertical power bar at (158, 26), the frequency selector at (27, 14) with base id 8300, and the Teleport button
//! at (42, 147, 92 by 20).
//! Java: `GuiPortableTeleporter.java` (lines 35-100), `PortableTeleporterContainer.java`.
//! Server data (`portable_teleporter.rs`): data 0 has frequency, 1 status (0 none, 1 no frame, 2 no destination,
//! 3 not enough energy, 4 ready), 2-3 energy, 4-5 capacity, 6 destination count, 7 public count, 8 private count.
//! Strings: `frequency::texts` (0 frequency, 1 owner, 2 security, followed by rows).
//!
//! Differences from Java:
//! - In Java, the status is an icon (`GuiTeleporterStatus`), while in the port it is colored text.
//! - The original does not show player inventory (`hide_inventory: true`) and has no item slots.
use crate::elements::{base_panel, data, inside, pair, title, vertical_power_bar};
use crate::frequency::Selector;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const BUTTON_TELEPORT: i32 = 8390;

pub struct PortableTeleporter {
    selector: Selector,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let screen = PortableTeleporter {
        selector: Selector::new(27.0, 14.0, 8300, 7, 8, 0),
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 176,
            height: 172,
            hide_inventory: true,
            own_labels: true,
        },
    ))
}

impl MekScreen for PortableTeleporter {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 172.0);

        // Power bar at (158, 26)
        let (energy, capacity) = (pair(2, 3), pair(4, 5));
        vertical_power_bar(
            c,
            158.0,
            26.0,
            if capacity == 0 {
                0.0
            } else {
                energy as f32 / capacity as f32
            },
        );

        // Title centered between (22, 152) at y = 4
        title(c, &host::translate("item.mekanism.portable_teleporter", &[]), 22.0, 130.0, 4.0);

        // Status text at (7, 7)
        let (text, colour) = match data(1) {
            0 => ("No frequency", 0xFFCD_0000),
            1 => ("No frame", 0xFFCD_0000),
            2 => ("No link", 0xFFCD_0000),
            3 => ("No energy", 0xFFCD_0000),
            _ => ("Ready", 0xFF00_AA00),
        };
        c.text(text, 7.0, 7.0, colour, 0, 0.8);

        // Frequency Selector at (27, 14)
        self.selector.render(c, mouse);

        // Teleport button at (42, 147, 92, 20)
        let ready = data(1) == 4 && data(0) != 0;
        let label = host::translate("gui.mekanism.teleport", &[]);
        let teleport_rect = [42.0, 147.0, 92.0, 20.0];
        crate::elements::button(c, BUTTON_TELEPORT, teleport_rect, &label, ready, mouse, Some(&label));
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left {
                    let teleport_rect = [42.0, 147.0, 92.0, 20.0];
                    let ready = data(1) == 4 && data(0) != 0;
                    if ready && inside((p.x, p.y), teleport_rect) {
                        host::send_button(BUTTON_TELEPORT);
                        host::play_click();
                        return true;
                    }
                    if self.selector.press((p.x, p.y)) {
                        return true;
                    }
                }
                false
            }
            InputEvent::FieldSubmitted(f) => self.selector.submitted(f.id, &f.text),
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
    fn portable_teleporter_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                1,  // has frequency
                4,  // ready
                100, 0, // energy
                400, 0, // capacity
                1,  // destinations
                1,  // public count
                0,  // private count
            ],
            strings: vec![
                "Frequency: Main".into(),
                "Owner: Steve".into(),
                "Security: Public".into(),
                "Main".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:portable_teleporter".into(),
            gui_width: 176.0,
            gui_height: 172.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open portable_teleporter");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 172);
        assert!(setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 50.0,
            mouse_y: 150.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Ready")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == BUTTON_TELEPORT)));

        // Test clicking the Teleport button
        let click_event = InputEvent::MousePressed(Pointer {
            x: 50.0,
            y: 150.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {BUTTON_TELEPORT}"), "click".to_string()]);
    }
}
