//! `GuiTeleporter` (176 by 240): the title, the energy slot at (153, 7) with the power bar at (158, 26), the status line at (7, 7) and the frequency selector at (27, 14). Reference screen of the
//! port (ported from the JSON screen the converter makes, `teleporter_menu`; compare it with the Java when you touch it). Numbers (`teleporter.rs` of the mod): 0 has a frequency, 1 the status
//! (0 none, 1 no frame, 2 no destination, 3 not enough energy, 4 ready), 2-3 the energy, 4-5 its capacity, 6 how many teleporters are on the frequency, 7 and 8 how many public and private
//! frequencies the lists hold. Strings: `frequency.rs`.
use crate::elements::{base_panel, data, inside, inventory_label, pair, slot_frame, title, vertical_power_bar};
use crate::frequency::Selector;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

/// `EnumColor.DARK_RED` and `DARK_GREEN`: the colours of an error and of a ready status (`TeleporterStatus`).
const ERROR: u32 = 0xFFC9_071F;
const OK: u32 = 0xFF59_C15F;

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
        // `GuiTeleporterStatus` at (6, 6), 18 by 18: the default button background and the icon of the status (the frequency chosen decides between "no frequency" and the status).
        let status = if data(0) == 0 {
            ("no_frequency", "frequency.mekanism.none", ERROR)
        } else {
            match data(1) {
                1 => ("no_frame", "teleporter.mekanism.no_frame", ERROR),
                4 => ("ready", "teleporter.mekanism.ready", OK),
                3 => ("needs_energy", "teleporter.mekanism.needs_energy", ERROR),
                _ => ("no_link", "teleporter.mekanism.no_destination", ERROR),
            }
        };
        c.blit_sprite("mekanism:button", 6.0, 6.0, 18.0, 18.0);
        c.blit(&format!("mekanism:gui/teleporter_{}", status.0), 6.0, 6.0, 0.0, 0.0, 18.0, 18.0, 18.0, 18.0);
        if inside(mouse, [6.0, 6.0, 18.0, 18.0]) {
            c.tooltip(&[(&host::translate(status.1, &[]), status.2)]);
        }
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
