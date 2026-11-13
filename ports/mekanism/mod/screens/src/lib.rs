//! The screens of the Mekanism port, drawn by code (capability `screen`, API 56): each of Mekanism's `GuiXxx` classes (`GuiTeleporter`, `GuiQIODashboard`...) is a module here, written close to the
//! Java so that the positions, sprites, texts and clicks are the original's. A menu type opts in with `"code": true` in its menu JSON (the converter's `CODE_SCREENS`); without the screen, or when it
//! fails, the game shows the menu's JSON widgets, which are kept as the fallback.
//!
//! One screen is open at a time: `registry::open` makes the one for `info.menu` (`mekanism:<name>`). Add a screen with a module, a `pub fn open(info) -> Opened` in it and one line in `registry.rs`.
//! `elements.rs` has the parts every Mekanism screen is made of (`GuiElement`s: the base panel, the inner screen, a button, the power bar, the tabs...); put a part used by two screens there.
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, ScreenMod, Setup};

pub mod elements;
pub mod frequency;
/// Copies of the mod's `security_table.rs` and `transporter_table.rs` (the converter writes them; keep them equal): where the screen's data has the security mode and the transporter window's numbers.
pub mod security_table;
pub mod transporter_table;
mod registry;
mod teleporter;

/// A Mekanism screen: what `GuiMekanism` does with the menu it shows.
pub trait MekScreen {
    /// 20 times a second, after the server's data for the tick.
    fn tick(&mut self) {}
    /// One frame, from the panel's top left (the way `GuiGraphics` draws in a `Screen`'s `render`).
    fn render(&mut self, frame: &Frame, canvas: &mut Canvas);
    /// The player's input from the panel's top left: true when the screen took it (a click on a slot is the game's otherwise).
    fn input(&mut self, _event: &InputEvent) -> bool {
        false
    }
    fn close(&mut self) {}
}

/// What a screen's `open` gives back: the screen and the menu's setup (its size, whether the inventory shows, who draws the labels).
pub type Opened = Result<(Box<dyn MekScreen>, Setup), String>;

struct Mek(Box<dyn MekScreen>);

impl ScreenMod for Mek {
    fn open(info: &OpenInfo) -> Result<(Self, Setup), String> {
        let (screen, setup) = registry::open(info)?;
        Ok((Mek(screen), setup))
    }
    fn tick(&mut self) {
        self.0.tick();
    }
    fn render(&mut self, frame: &Frame, canvas: &mut Canvas) {
        self.0.render(frame, canvas);
    }
    fn input(&mut self, event: &InputEvent) -> bool {
        self.0.input(event)
    }
    fn close(&mut self) {
        self.0.close();
    }
}

horizon_mod_sdk::export_screen_mod!(Mek);
