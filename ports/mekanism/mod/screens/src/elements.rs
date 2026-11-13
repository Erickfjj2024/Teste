//! The parts Mekanism's screens are made of (`GuiElement` and its subclasses), drawn with the screen `Canvas`. Positions are from the panel's top left, in GUI pixels, as in the Java.
//! Sprites are `mekanism:<path>` under `textures/gui/sprites/` (nine-sliced by their `.mcmeta`), textures `mekanism:gui/<path>`.
use horizon_mod_sdk::screen::{AreaKind, Canvas, TEXT_CENTER, host};

/// `SpecialColors.TEXT_TITLE` (0xFF404040) and the green of an inner screen's text (0xFF3CFE9A).
pub const TITLE: u32 = 0xFF40_4040;
pub const GREEN: u32 = 0xFF3C_FE9A;
pub const WHITE: u32 = 0xFFFF_FFFF;
pub const SUBHEADING: u32 = 0xFF78_7878;

pub type Rect = [f32; 4];

pub fn inside(mouse: (f32, f32), r: Rect) -> bool {
    mouse.0 >= r[0] && mouse.0 < r[0] + r[2] && mouse.1 >= r[1] && mouse.1 < r[1] + r[3]
}

/// The menu's data number `i` (0 when the server has not sent it yet).
pub fn data(i: usize) -> i32 {
    host::menu_data().get(i).copied().unwrap_or(0)
}

/// A 32-bit number the server sends as two 16-bit halves (data `lo` and `hi`).
pub fn pair(lo: usize, hi: usize) -> u64 {
    let d = host::menu_data();
    let half = |i: usize| (d.get(i).copied().unwrap_or(0) as u64) & 0xFFFF;
    half(lo) | (half(hi) << 16)
}

/// The menu's string `i` (empty before the server sends it).
pub fn string(i: usize) -> String {
    host::menu_strings().get(i).cloned().unwrap_or_default()
}

/// `GuiMekanism`'s background: the base sprite over the whole panel.
pub fn base_panel(c: &mut Canvas, w: f32, h: f32) {
    c.blit_sprite("mekanism:base", 0.0, 0.0, w, h);
}

/// `GuiInnerScreen`: the dark well that holds green text.
pub fn inner_screen(c: &mut Canvas, rect: Rect) {
    c.blit_sprite("mekanism:inner_screen", rect[0], rect[1], rect[2], rect[3]);
}

/// `GuiElementHolder` (`BackgroundType.ELEMENT_HOLDER`).
pub fn element_holder(c: &mut Canvas, rect: Rect) {
    c.blit_sprite("mekanism:element_holder", rect[0], rect[1], rect[2], rect[3]);
}

/// `GuiMekanism.renderTitleText`: the title, centred in `[x, x + width)` at `y` (Mekanism's `titleLabelY` is 6, many screens use 4 or 5).
pub fn title(c: &mut Canvas, text: &str, x: f32, width: f32, y: f32) {
    c.text(text, x + width / 2.0, y, TITLE, TEXT_CENTER, 1.0);
}

/// The "Inventory" label above the player's inventory (`inventoryLabelY`).
pub fn inventory_label(c: &mut Canvas, x: f32, y: f32) {
    c.draw_string(&host::translate("container.inventory", &[]), x, y, TITLE, false);
}

/// A text button (`TranslationButton` / `MekanismButton`): the button sprite (the inactive one when not `active`), its label centred, a clickable area with `id`, and the label as tooltip when `tip` is given.
pub fn button(c: &mut Canvas, id: i32, rect: Rect, label: &str, active: bool, mouse: (f32, f32), tip: Option<&str>) {
    let hovered = active && inside(mouse, rect);
    let sprite = if !active {
        "mekanism:button_inactive"
    } else if hovered {
        "mekanism:button_focus"
    } else {
        "mekanism:button"
    };
    c.blit_sprite(sprite, rect[0], rect[1], rect[2], rect[3]);
    let colour = if active { WHITE } else { 0xFFA0_A0A0 };
    c.text(label, rect[0] + rect[2] / 2.0, rect[1] + (rect[3] - 8.0) / 2.0, colour, TEXT_CENTER, 1.0);
    c.area(id, rect, AreaKind::Button, active, label);
    if hovered {
        if let Some(t) = tip {
            c.tooltip(&[(t, WHITE)]);
        }
    }
}

/// `GuiVerticalPowerBar`: the 6 by 54 frame at (`x - 1`, `y - 1`) and the 4 by 52 fill rising from the bottom by `fraction` (0 to 1).
pub fn vertical_power_bar(c: &mut Canvas, x: f32, y: f32, fraction: f32) {
    c.blit_sprite("mekanism:bar/base", x - 1.0, y - 1.0, 6.0, 54.0);
    let filled = (52.0 * fraction.clamp(0.0, 1.0)).floor();
    if filled > 0.0 {
        c.blit("mekanism:gui/bar/vertical_power", x, y + 52.0 - filled, 0.0, 52.0 - filled, 4.0, filled, 4.0, 52.0);
    }
}

/// A slot's frame (`GuiSlot`, `SlotType.NORMAL`) with its top left at (`x`, `y`) (the slot's item is drawn one pixel in).
pub fn slot_frame(c: &mut Canvas, x: f32, y: f32) {
    c.blit_sprite("mekanism:slot/normal", x, y, 18.0, 18.0);
}
