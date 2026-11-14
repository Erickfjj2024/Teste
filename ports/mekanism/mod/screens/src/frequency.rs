//! `GuiFrequencySelector` (132 by 121 at (27, y) in the original; here at (`x`, `y`)): the public, trusted and private image buttons (38 by 20, 4 apart: `MekanismImageButton` with the icon centred),
//! the list of the frequencies of the chosen security (`GuiTextScrollList`), the frequency, owner and security lines under it, the name box and the Delete button. The server side is `frequency.rs`
//! of the mod: strings 0 to 2 are the three lines, then `LIST_MAX` public rows and `LIST_MAX` private rows; the click ids are `base + row` (public), `base + 16 + row` (private), `base + 32` (delete)
//! and the name boxes' fields `base + 33` (public) and `base + 34` (private).
use crate::elements::{GREEN, TITLE, WHITE, data, element_holder, inner_screen, inside, string};
use horizon_mod_sdk::screen::{AreaKind, Canvas, FieldFilter, host};

pub const LIST_MAX: usize = 16;

pub struct Selector {
    pub x: f32,
    pub y: f32,
    /// The ids the server knows the selector by (`frequency.rs::id_public(base, ..)`).
    pub base: i32,
    /// The menu's numbers for how many public and private frequencies the lists hold, and the first string of the three lines.
    pub public_count: usize,
    pub private_count: usize,
    pub first_string: usize,
    /// 0 the public list, 1 the private one (`GuiFrequencySelector.securityMode`).
    pub security: u8,
}

impl Selector {
    pub fn new(x: f32, y: f32, base: i32, public_count: usize, private_count: usize, first_string: usize) -> Self {
        Selector { x, y, base, public_count, private_count, first_string, security: 0 }
    }

    fn row_string(&self, private: bool, row: usize) -> String {
        string(self.first_string + 3 + if private { LIST_MAX } else { 0 } + row)
    }

    /// Draws the selector; `mouse` is the cursor in panel coordinates.
    pub fn render(&self, c: &mut Canvas, mouse: (f32, f32)) {
        let (x, y) = (self.x, self.y);
        // The three security buttons: the chosen one is the inactive one.
        for (k, (icon, label, bx)) in [("public", "Public", 0.0), ("trusted", "Trusted", 42.0), ("private", "Private", 84.0)].into_iter().enumerate() {
            let active = (k == 0 && self.security != 0) || (k == 2 && self.security != 1) && k != 1;
            let rect = [x + bx, y, 38.0, 20.0];
            let hovered = active && inside(mouse, rect);
            let sprite = if !active { "mekanism:button_inactive" } else if hovered { "mekanism:button_focus" } else { "mekanism:button" };
            c.blit_sprite(sprite, rect[0], rect[1], rect[2], rect[3]);
            c.blit(&format!("mekanism:gui/button/{icon}"), rect[0] - 1.0, rect[1] + 2.0, 0.0, 0.0, 40.0, 16.0, 40.0, 16.0);
            c.area(100 + k as i32, rect, AreaKind::Button, active, label);
            if hovered {
                c.tooltip(&[(label, WHITE)]);
            }
        }
        // The list (122 by 42, 10 pixel rows).
        inner_screen(c, [x, y + 22.0, 122.0, 42.0]);
        let private = self.security == 1;
        let count = if private { data(self.private_count) } else { data(self.public_count) }.clamp(0, 4) as usize;
        for row in 0..count.min(4) {
            let text = self.row_string(private, row);
            c.draw_string(&text, x + 3.0, y + 24.0 + 10.0 * row as f32, GREEN, false);
            let id = self.base + if private { LIST_MAX as i32 } else { 0 } + row as i32;
            c.area(id, [x + 1.0, y + 23.0 + 10.0 * row as f32, 120.0, 10.0], AreaKind::Row, true, &text);
        }
        // The frequency, owner and security lines.
        for k in 0..3 {
            c.text(&string(self.first_string + k), x, y + 67.0 + 10.0 * k as f32, TITLE, 0, 0.85);
        }
        // The name box (98 by 11 at 23, 99) and Delete (60 by 18 at 62, 113).
        inner_screen(c, [x + 23.0, y + 99.0, 98.0, 11.0]);
        let field = self.base + if private { 34 } else { 33 };
        c.field(field, [x + 24.0, y + 100.0, 96.0, 9.0], 16, FieldFilter::Any, "", 1, WHITE);
        let delete = [x + 62.0, y + 113.0, 60.0, 18.0];
        crate::elements::button(c, self.base + 32, delete, &host::translate("button.mekanism.delete", &[]), true, mouse, None);
        let _ = element_holder;
    }

    /// A press inside the selector: true when it took it (a button, a row or Delete).
    pub fn press(&mut self, mouse: (f32, f32)) -> bool {
        let (x, y) = (self.x, self.y);
        if inside(mouse, [x, y, 38.0, 20.0]) && self.security != 0 {
            self.security = 0;
            host::play_click();
            return true;
        }
        if inside(mouse, [x + 84.0, y, 38.0, 20.0]) && self.security != 1 {
            self.security = 1;
            host::play_click();
            return true;
        }
        let private = self.security == 1;
        let count = if private { data(self.private_count) } else { data(self.public_count) }.clamp(0, 4) as usize;
        for row in 0..count {
            if inside(mouse, [x + 1.0, y + 23.0 + 10.0 * row as f32, 120.0, 10.0]) {
                host::send_button(self.base + if private { LIST_MAX as i32 } else { 0 } + row as i32);
                host::play_click();
                return true;
            }
        }
        if inside(mouse, [x + 62.0, y + 113.0, 60.0, 18.0]) {
            host::send_button(self.base + 32);
            host::play_click();
            return true;
        }
        false
    }

    /// The name the player confirmed in the box: sent to the server as the box's own text message.
    pub fn submitted(&self, field: i32, text: &str) -> bool {
        if field == self.base + 33 || field == self.base + 34 {
            host::send_text(field, text);
            return true;
        }
        false
    }
}
