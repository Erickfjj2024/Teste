//! `GuiTeleporter` (176 by 240): the title, the energy slot at (153, 7) with the power bar at (158, 26), the status line at (7, 7) and the frequency selector at (27, 14). Reference screen of the
//! port (ported from the JSON screen the converter makes, `teleporter_menu`; compare it with the Java when you touch it). Numbers (`teleporter.rs` of the mod): 0 has a frequency, 1 the status
//! (0 none, 1 no frame, 2 no destination, 3 not enough energy, 4 ready), 2-3 the energy, 4-5 its capacity, 6 how many teleporters are on the frequency, 7 and 8 how many public and private
//! frequencies the lists hold. Strings: `frequency.rs`.
use crate::elements::{base_panel, data, inventory_label, pair, slot_frame, title, vertical_power_bar};
use crate::frequency::Selector;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

struct Teleporter {
    selector: Selector,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let screen = Teleporter { selector: Selector::new(27.0, 14.0, 8100, 7, 8, 0) };
    Ok((Box::new(screen), Setup { width: 176, height: 240, hide_inventory: false, own_labels: true }))
}

impl MekScreen for Teleporter {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 240.0);
        slot_frame(c, 152.0, 6.0);
        let (energy, capacity) = (pair(2, 3), pair(4, 5));
        vertical_power_bar(c, 158.0, 26.0, if capacity == 0 { 0.0 } else { energy as f32 / capacity as f32 });
        title(c, &host::translate("block.mekanism.teleporter", &[]), 22.0, 130.0, 4.0);
        let (text, colour) = match data(1) {
            0 => ("No frequency", 0xFFCD_0000),
            1 => ("No frame", 0xFFCD_0000),
            2 => ("No link", 0xFFCD_0000),
            3 => ("No energy", 0xFFCD_0000),
            _ => ("Ready", 0xFF00_AA00),
        };
        c.text(text, 7.0, 7.0, colour, 0, 0.8);
        self.selector.render(c, mouse);
        inventory_label(c, 8.0, 240.0 - 93.0);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => self.selector.press((p.x, p.y)),
            InputEvent::FieldSubmitted(f) => self.selector.submitted(f.id, &f.text),
            _ => false,
        }
    }
}
