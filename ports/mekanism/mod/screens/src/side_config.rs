//! `SideConfigWindow` (`GuiSideConfiguration`, `GuiSideConfigurationTab` at (-26, 6)):
//! The side configuration window (156 by 135 pixels, default at ((width - 156)/2, 15)) allowing
//! configuration of each machine side's DataType for each transmission (Items, Energy, Fluids,
//! Chemicals, Heat).
//!
//! Java reference:
//! - `mekanism.client.gui.element.window.GuiSideConfiguration`
//! - `mekanism.client.gui.element.button.SideDataButton`
//! - `mekanism.client.gui.element.tab.GuiConfigTypeTab`
//! - `mekanism.client.gui.element.tab.window.GuiSideConfigurationTab`
//!
//! Server data:
//! - Transmission `t` has its numbers in data slots `6 + 2*t` and `7 + 2*t` (nibble-packed).
//!   - Slot `6 + 2*t`: low 4 nibbles are sides 0..3 (Front, Left, Right, Back).
//!   - Slot `7 + 2*t`: nibble 0 is Top (side 4), nibble 1 is Bottom (side 5), nibble 2 is eject flag (bit 8).
//! - Button IDs:
//!   - Side buttons: `1000 + t * 100 + side * 10` (left click next), `+ 1` (right click previous).
//!   - Auto-eject toggle: `2000 + t`.
//!   - Clear sides button: `3000 + t * 10` (left click next), `+ 1` (right click previous).
//!
//! Differences from Java / Fidelity notes:
//! - NÃO FEITO: desenhar o bloco vizinho sobre a cor (neighbor no JSON / otherBlockItem no Java SideDataButton),
//!   pois o SDK/host não expõe blocos vizinhos do mundo (Level) para screens.wasm.
use crate::elements::{data, inside, inner_screen, GREEN, SUBHEADING, TITLE, WHITE, Overlay, Rect};
use horizon_mod_sdk::screen::widgets::{SideTab, Window, WindowPress};
use horizon_mod_sdk::screen::{AreaKind, Canvas, InputEvent, MouseButton, TEXT_CENTER, host};

pub const WIDTH: f32 = 156.0;
pub const HEIGHT: f32 = 135.0;

pub const TRANSMISSION_ITEMS: usize = 0;
pub const TRANSMISSION_ENERGY: usize = 1;
pub const TRANSMISSION_FLUIDS: usize = 2;
pub const TRANSMISSION_CHEMICALS: usize = 3;
pub const TRANSMISSION_HEAT: usize = 4;

pub const TRANSMISSION_NAMES: [&str; 5] = ["Items", "Energy", "Fluids", "Chemicals", "Heat"];
pub const TRANSMISSION_ICONS: [&str; 5] = [
    "mekanism:transmission/items",
    "mekanism:transmission/energy",
    "mekanism:transmission/fluids",
    "mekanism:transmission/chemicals",
    "mekanism:transmission/heat",
];
pub const TRANSMISSION_TINTS: [u32; 5] = [
    0xFFCF_CFCF, // Items
    0xFF59_C15F, // Energy
    0xFF36_6BD0, // Fluids
    0xFFFF_DD4F, // Chemicals
    0xFFFA_A160, // Heat (0xFFA160)
];

/// RelativeSide index (Front, Left, Right, Back, Top, Bottom) -> (x, y) relative to window top-left.
pub const SIDE_BUTTONS: [(usize, (f32, f32)); 6] = [
    (0, (67.0, 69.0)), // Front
    (1, (44.0, 69.0)), // Left
    (2, (90.0, 69.0)), // Right
    (3, (44.0, 92.0)), // Back
    (4, (67.0, 46.0)), // Top
    (5, (67.0, 92.0)), // Bottom
];

pub const SIDE_NAMES: [&str; 6] = ["Front", "Left", "Right", "Back", "Top", "Bottom"];

pub const DATA_LABELS: [&str; 10] = [
    "None",
    "Input",
    "Input (1)",
    "Input (2)",
    "Output",
    "Output (1)",
    "Output (2)",
    "Input/Output",
    "Energy",
    "Extra",
];

/// Lightened button background colors matching Java `BasicColorButton` / `convert.py` SWATCH_COLORS.
pub const DATA_COLORS: [u32; 10] = [
    0xFFE8_E8E8, // None
    0xFFE2_1F37, // Input
    0xFFFF_B07A, // Input (1)
    0xFFFF_B07A, // Input (2)
    0xFF54_87EA, // Output
    0xFF54_87EA, // Output (1)
    0xFF19_FFDE, // Output (2)
    0xFFC2_84F2, // Input/Output
    0xFF7B_DA80, // Energy
    0xFFFF_E268, // Extra
];

/// Base EnumColor ARGB values for tooltip text.
pub const DATA_BASE_COLORS: [u32; 10] = [
    0xFFCF_CFCF, // None
    0xFFC9_071F, // Input
    0xFFFA_A160, // Input (1)
    0xFFFA_A160, // Input (2)
    0xFF36_6BD0, // Output
    0xFF36_6BD0, // Output (1)
    0xFF00_F3D0, // Output (2)
    0xFFA4_60D9, // Input/Output
    0xFF59_C15F, // Energy
    0xFFFF_DD4F, // Extra
];

/// Returns default transmissions configured for a given machine menu ID.
pub fn transmissions_for(machine: &str) -> &'static [usize] {
    let base = machine.strip_prefix("mekanism:").unwrap_or(machine);
    match base {
        "electric_pump" | "fluidic_plenisher" => &[TRANSMISSION_FLUIDS, TRANSMISSION_ITEMS, TRANSMISSION_ENERGY],
        "chemical_infuser" | "pigment_mixer" | "isotopic_centrifuge" | "solar_neutron_activator" => {
            &[TRANSMISSION_CHEMICALS, TRANSMISSION_ITEMS, TRANSMISSION_ENERGY]
        }
        "chemical_washer" | "rotary_condensentrator" => {
            &[TRANSMISSION_CHEMICALS, TRANSMISSION_FLUIDS, TRANSMISSION_ITEMS, TRANSMISSION_ENERGY]
        }
        "electrolytic_separator" => {
            &[TRANSMISSION_FLUIDS, TRANSMISSION_CHEMICALS, TRANSMISSION_ITEMS, TRANSMISSION_ENERGY]
        }
        "pressurized_reaction_chamber" => {
            &[TRANSMISSION_ITEMS, TRANSMISSION_CHEMICALS, TRANSMISSION_FLUIDS, TRANSMISSION_ENERGY]
        }
        b if b.ends_with("_energy_cube") => &[TRANSMISSION_ENERGY],
        b if b.ends_with("_chemical_tank") => &[TRANSMISSION_CHEMICALS, TRANSMISSION_ITEMS],
        b if b.ends_with("_fluid_tank") => &[TRANSMISSION_FLUIDS, TRANSMISSION_ITEMS],
        // Factories and process machines
        b if b.contains("factory") => {
            if b.contains("infusing") || b.contains("purifying") || b.contains("injecting") || b.contains("compressing") {
                &[TRANSMISSION_ITEMS, TRANSMISSION_CHEMICALS, TRANSMISSION_ENERGY]
            } else {
                &[TRANSMISSION_ITEMS, TRANSMISSION_ENERGY]
            }
        }
        "metallurgic_infuser"
        | "purification_chamber"
        | "chemical_injection_chamber"
        | "osmium_compressor"
        | "chemical_crystallizer"
        | "painting_machine"
        | "chemical_dissolution_chamber"
        | "chemical_oxidizer"
        | "nutritional_liquifier"
        | "pigment_extractor"
        | "antiprotonic_nucleosynthesizer" => &[TRANSMISSION_ITEMS, TRANSMISSION_CHEMICALS, TRANSMISSION_ENERGY],
        _ => &[TRANSMISSION_ITEMS, TRANSMISSION_ENERGY],
    }
}

/// Checks whether transmission `t` can auto-eject on `machine`.
pub fn can_eject_for(machine: &str, t: usize) -> bool {
    let base = machine.strip_prefix("mekanism:").unwrap_or(machine);
    if t == TRANSMISSION_ENERGY || t == TRANSMISSION_HEAT {
        return false;
    }
    if t == TRANSMISSION_CHEMICALS {
        let no_eject_chem = matches!(
            base,
            "metallurgic_infuser"
                | "purification_chamber"
                | "chemical_injection_chamber"
                | "osmium_compressor"
                | "chemical_crystallizer"
                | "painting_machine"
        ) || (base.contains("factory")
            && (base.contains("infusing")
                || base.contains("purifying")
                || base.contains("injecting")
                || base.contains("compressing")));
        if no_eject_chem {
            return false;
        }
    }
    if t == TRANSMISSION_FLUIDS && (base == "chemical_washer" || base == "electrolytic_separator") {
        return false;
    }
    if t == TRANSMISSION_ITEMS && base.ends_with("_chemical_tank") {
        return false;
    }
    true
}

/// `SideConfigWindow`: the overlay window for configuring machine sides.
pub struct SideConfigWindow {
    pub window: Window,
    pub open: bool,
    pub current_tab: usize,
    pub transmissions: Vec<usize>,
    pub can_eject: Vec<bool>,
    pub panel_width: f32,
    pub panel_height: f32,
}

impl SideConfigWindow {
    /// Creates a new `SideConfigWindow` with explicit transmissions and eject capabilities.
    pub fn new(x: i32, y: i32, transmissions: Vec<usize>, can_eject: Vec<bool>) -> Self {
        let t_list = if transmissions.is_empty() {
            vec![TRANSMISSION_ITEMS, TRANSMISSION_ENERGY]
        } else {
            transmissions
        };
        let eject_list = if can_eject.len() == t_list.len() {
            can_eject
        } else {
            t_list.iter().map(|&t| t == TRANSMISSION_ITEMS || t == TRANSMISSION_FLUIDS || t == TRANSMISSION_CHEMICALS).collect()
        };
        let initial_title = format!("{} Config", TRANSMISSION_NAMES[t_list[0]]);
        let window = Window::new(x, y, WIDTH as i32, HEIGHT as i32, &initial_title);
        SideConfigWindow {
            window,
            open: false,
            current_tab: 0,
            transmissions: t_list,
            can_eject: eject_list,
            panel_width: 176.0,
            panel_height: 166.0,
        }
    }

    /// Creates a `SideConfigWindow` sized and configured for a specific machine menu.
    pub fn for_machine(menu: &str, screen_width: i32, screen_height: i32) -> Self {
        let trans = transmissions_for(menu);
        let can_ej: Vec<bool> = trans.iter().map(|&t| can_eject_for(menu, t)).collect();
        let x = (screen_width - WIDTH as i32) / 2;
        let mut win = Self::new(x, 15, trans.to_vec(), can_ej);
        win.panel_width = screen_width as f32;
        win.panel_height = screen_height as f32;
        win
    }

    /// Side tab at (-26, 6) for `GuiSideConfigurationTab`.
    pub fn side_tab() -> SideTab {
        SideTab::new(-1, -26, 6, "mekanism:button/configuration", "Configuration")
    }

    /// Whether a screen slot at `(sx, sy)` is covered by the open window.
    pub fn covers(&self, sx: f32, sy: f32) -> bool {
        self.open && self.window.covers(sx, sy)
    }

    /// Current active transmission type index (0..4).
    pub fn current_transmission(&self) -> usize {
        self.transmissions.get(self.current_tab).copied().unwrap_or(0)
    }
}

impl Overlay for SideConfigWindow {
    fn is_open(&self) -> bool {
        self.open
    }

    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
        self.window.release();
    }

    fn render(&mut self, c: &mut Canvas, mouse: (f32, f32)) {
        if !self.open {
            return;
        }
        let (wx, wy) = (self.window.x as f32, self.window.y as f32);
        let ty = self.current_transmission();
        let can_eject = self.can_eject.get(self.current_tab).copied().unwrap_or(false);

        // 1. Window background (shadow, panel base, close button at 6, 6)
        self.window.draw_background(c);

        // 2. Window title: "%s Config", centred between start (12) and end (138)
        let title_text = format!("{} Config", TRANSMISSION_NAMES[ty]);
        self.window.title = title_text;
        self.window.draw_title(c, |t| (t.len() * 6) as f32, TITLE);

        // 3. Eject status inner screen at (38, 25, 80, 12)
        let screen_rect = [wx + 38.0, wy + 25.0, 80.0, 12.0];
        inner_screen(c, screen_rect);

        // Data words for current transmission: slot0 = 6 + 2*ty, slot1 = 7 + 2*ty
        let slot0 = 6 + 2 * ty;
        let slot1 = 7 + 2 * ty;
        let w0 = data(slot0);
        let w1 = data(slot1);
        let is_ejecting = ((w1 >> 8) & 1) == 1;

        let eject_text = if !can_eject {
            "Can't Eject"
        } else if is_ejecting {
            "Eject: On"
        } else {
            "Eject: Off"
        };
        // Centred text vertically in the 12px inner screen: y + 27
        c.draw_string(eject_text, wx + 41.0, wy + 27.0, GREEN, false);
        c.area(-1, screen_rect, AreaKind::Other, true, "Eject State");
        if inside(mouse, screen_rect) && !can_eject {
            c.tooltip(&[("This machine cannot auto-eject this type", WHITE)]);
        }

        // 4. Auto-eject button at (136, 6, 14, 14)
        let eject_rect = [wx + 136.0, wy + 6.0, 14.0, 14.0];
        let eject_hovered = can_eject && inside(mouse, eject_rect);
        let eject_sprite = if !can_eject {
            "mekanism:button_inactive"
        } else if eject_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(eject_sprite, eject_rect[0], eject_rect[1], eject_rect[2], eject_rect[3]);
        c.blit_sprite("mekanism:button/auto_eject", eject_rect[0], eject_rect[1], eject_rect[2], eject_rect[3]);
        c.area(2000 + ty as i32, eject_rect, AreaKind::Button, can_eject, "Auto-eject");
        if eject_hovered {
            c.tooltip(&[("Auto-eject", WHITE)]);
        }

        // 5. Clear sides button at (136, 95, 14, 14)
        let clear_rect = [wx + 136.0, wy + 95.0, 14.0, 14.0];
        let clear_hovered = inside(mouse, clear_rect);
        let clear_sprite = if clear_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(clear_sprite, clear_rect[0], clear_rect[1], clear_rect[2], clear_rect[3]);
        c.blit_sprite("mekanism:button/clear_sides", clear_rect[0], clear_rect[1], clear_rect[2], clear_rect[3]);
        c.area(3000 + ty as i32 * 10, clear_rect, AreaKind::Button, true, "Clear sides");
        if clear_hovered {
            c.tooltip(&[
                ("Clear sides", WHITE),
                ("Click to cycle every side, right click to go back", SUBHEADING),
            ]);
        }

        // 6. Six side buttons (22 by 22), coloured by DataType
        for &(side, (bx, by)) in SIDE_BUTTONS.iter() {
            let rect: Rect = [wx + bx, wy + by, 22.0, 22.0];
            let hovered = inside(mouse, rect);
            let data_ord = match side {
                0 => (w0 >> 0) & 0x0F,
                1 => (w0 >> 4) & 0x0F,
                2 => (w0 >> 8) & 0x0F,
                3 => (w0 >> 12) & 0x0F,
                4 => (w1 >> 0) & 0x0F,
                _ => (w1 >> 4) & 0x0F,
            } as usize;
            let color = DATA_COLORS.get(data_ord).copied().unwrap_or(0xFFE8_E8E8);
            let button_sprite = if hovered {
                "mekanism:button_focus"
            } else {
                "mekanism:button"
            };
            c.blit_sprite_tinted(button_sprite, rect, color);

            // NÃO FEITO: desenhar o bloco vizinho sobre a cor (neighbor no JSON / otherBlockItem no Java SideDataButton),
            // pois o SDK/host não expõe blocos vizinhos do mundo (Level) para screens.wasm

            let side_name = SIDE_NAMES.get(side).copied().unwrap_or("");
            let data_label = DATA_LABELS.get(data_ord).copied().unwrap_or("None");
            let btn_id = 1000 + ty as i32 * 100 + side as i32 * 10;
            c.area(btn_id, rect, AreaKind::Button, true, side_name);
            if hovered {
                let text_color = DATA_BASE_COLORS.get(data_ord).copied().unwrap_or(WHITE);
                c.tooltip(&[(side_name, WHITE), (data_label, text_color)]);
            }
        }

        // 7. Subheading "Slots" at y = 120, centred
        c.text("Slots", wx + WIDTH / 2.0, wy + 120.0, SUBHEADING, TEXT_CENTER, 1.0);

        // 8. Transmission tabs outside the window (4 on left, rest on right)
        for (i, &t) in self.transmissions.iter().enumerate() {
            if i == self.current_tab {
                continue; // Java: tab.visible = currentType != tab.getTransmissionType()
            }
            let tab_x = if i < 4 { wx - 26.0 } else { wx + WIDTH };
            let tab_y = wy + 2.0 + 28.0 * (i % 4) as f32;
            let holder = if i < 4 { "mekanism:holder_left" } else { "mekanism:holder_right" };
            let tint = TRANSMISSION_TINTS[t];
            c.blit_sprite_tinted(holder, [tab_x, tab_y, 26.0, 18.0], tint);

            let icon = TRANSMISSION_ICONS[t];
            let icon_x = tab_x + if i < 4 { 5.0 } else { 3.0 };
            c.blit_sprite(icon, icon_x, tab_y, 18.0, 18.0);

            let label = TRANSMISSION_NAMES[t];
            c.area(-1, [tab_x, tab_y, 26.0, 18.0], AreaKind::Tab, true, label);
            if inside(mouse, [tab_x, tab_y, 26.0, 18.0]) {
                c.tooltip(&[(label, WHITE)]);
            }
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if !self.open {
            return false;
        }
        let (wx, wy) = (self.window.x as f32, self.window.y as f32);
        let ty = self.current_transmission();
        match event {
            InputEvent::MousePressed(p) => {
                let mouse = (p.x, p.y);

                // 1. Transmission tabs outside window
                for (i, _) in self.transmissions.iter().enumerate() {
                    if i == self.current_tab {
                        continue;
                    }
                    let tab_x = if i < 4 { wx - 26.0 } else { wx + WIDTH };
                    let tab_y = wy + 2.0 + 28.0 * (i % 4) as f32;
                    if inside(mouse, [tab_x, tab_y, 26.0, 18.0]) {
                        self.current_tab = i;
                        host::play_click();
                        return true;
                    }
                }

                // 2. Window close button at (6, 6, 8, 12)
                if inside(mouse, [wx + 6.0, wy + 6.0, 8.0, 12.0]) {
                    self.close();
                    host::play_click();
                    return true;
                }

                // 3. Auto-eject button at (136, 6, 14, 14)
                let can_eject = self.can_eject.get(self.current_tab).copied().unwrap_or(false);
                if can_eject && inside(mouse, [wx + 136.0, wy + 6.0, 14.0, 14.0]) {
                    host::send_button(2000 + ty as i32);
                    host::play_click();
                    return true;
                }

                // 4. Clear sides button at (136, 95, 14, 14)
                if inside(mouse, [wx + 136.0, wy + 95.0, 14.0, 14.0]) {
                    let back = p.button == MouseButton::Right;
                    let id = 3000 + ty as i32 * 10 + if back { 1 } else { 0 };
                    host::send_button(id);
                    host::play_click();
                    return true;
                }

                // 5. Six side buttons (22 by 22)
                for &(side, (bx, by)) in SIDE_BUTTONS.iter() {
                    if inside(mouse, [wx + bx, wy + by, 22.0, 22.0]) {
                        let back = p.button == MouseButton::Right;
                        let id = 1000 + ty as i32 * 100 + side as i32 * 10 + if back { 1 } else { 0 };
                        host::send_button(id);
                        host::play_click();
                        return true;
                    }
                }

                // 6. Window dragging or outside press
                let inventory_top = Some((self.panel_height - 90.0) as f64);
                let press = self.window.press(p.x as f64, p.y as f64, inventory_top, self.panel_width as f64);
                match press {
                    WindowPress::Closed => {
                        self.close();
                        host::play_click();
                        true
                    }
                    WindowPress::Dragging => true,
                    WindowPress::Inside => true,
                    WindowPress::Inventory => false,
                    WindowPress::Blocked => false,
                }
            }
            InputEvent::MouseReleased(_) => {
                if self.window.dragging() {
                    self.window.release();
                    true
                } else {
                    false
                }
            }
            InputEvent::MouseDragged(d) => {
                if self.window.dragging() {
                    self.window.drag(d.x as f64, d.y as f64, [0.0, 0.0, self.panel_width, self.panel_height]);
                    true
                } else {
                    false
                }
            }
            InputEvent::KeyPressed(k) => {
                if k.key == "key.keyboard.escape" {
                    self.close();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format;
    use horizon_mod_sdk::screen::{Modifiers, Pointer};

    fn make_pointer(x: f32, y: f32, button: MouseButton) -> InputEvent {
        InputEvent::MousePressed(Pointer {
            x,
            y,
            button,
            modifiers: Modifiers::default(),
        })
    }

    #[test]
    fn test_side_config_render_and_commands() {
        host::set_menu(host::Menu {
            // slots 0..5 dummy, 6 and 7 for Items (transmission 0)
            // slot 6: front=1 (Input), left=2, right=4 (Output), back=7
            // slot 7: top=8 (Energy), bottom=9 (Extra), eject=1 (bit 8)
            data: vec![0, 0, 0, 0, 0, 0, 0x7421, 0x0198],
            ..Default::default()
        });

        let mut win = SideConfigWindow::new(10, 15, vec![TRANSMISSION_ITEMS, TRANSMISSION_ENERGY], vec![true, false]);
        win.open();

        let mut c = Canvas::new();
        win.render(&mut c, (50.0, 50.0));
        let bytes = c.finish();
        let frame = format::decode(&bytes).expect("frame decoding should succeed");
        assert!(frame.commands.len() > 10, "Frame should contain background, title, buttons, tabs");

        // Verify that closing hides window render
        win.close();
        let mut c2 = Canvas::new();
        win.render(&mut c2, (50.0, 50.0));
        let frame2 = format::decode(&c2.finish()).unwrap();
        assert_eq!(frame2.commands.len(), 0, "Closed window should render nothing");
    }

    #[test]
    fn test_side_config_side_buttons_clicks() {
        host::set_menu(host::Menu::default());
        let _ = host::take_sent();

        let mut win = SideConfigWindow::new(10, 15, vec![TRANSMISSION_ITEMS, TRANSMISSION_ENERGY], vec![true, false]);
        win.open();

        // Click front side button at (10 + 67, 15 + 69) = (77, 84)
        assert!(win.input(&make_pointer(78.0, 85.0, MouseButton::Left)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 1000".to_owned()));

        // Right click on front side button: button 1001
        assert!(win.input(&make_pointer(78.0, 85.0, MouseButton::Right)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 1001".to_owned()));

        // Left click on top button at (10 + 67, 15 + 46) = (77, 61): side 4 -> 1040
        assert!(win.input(&make_pointer(78.0, 62.0, MouseButton::Left)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 1040".to_owned()));
    }

    #[test]
    fn test_side_config_eject_and_clear_buttons() {
        host::set_menu(host::Menu::default());
        let _ = host::take_sent();

        let mut win = SideConfigWindow::new(10, 15, vec![TRANSMISSION_ITEMS, TRANSMISSION_ENERGY], vec![true, false]);
        win.open();

        // Click eject button at (10 + 136, 15 + 6) = (146, 21)
        assert!(win.input(&make_pointer(147.0, 22.0, MouseButton::Left)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 2000".to_owned()));

        // Click clear sides button at (10 + 136, 15 + 95) = (146, 110)
        assert!(win.input(&make_pointer(147.0, 111.0, MouseButton::Left)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 3000".to_owned()));

        // Right click clear sides button
        assert!(win.input(&make_pointer(147.0, 111.0, MouseButton::Right)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 3001".to_owned()));
    }

    #[test]
    fn test_side_config_tab_switching() {
        host::set_menu(host::Menu::default());
        let _ = host::take_sent();

        let mut win = SideConfigWindow::new(10, 15, vec![TRANSMISSION_ITEMS, TRANSMISSION_ENERGY], vec![true, false]);
        win.open();
        assert_eq!(win.current_tab, 0);

        // Tab 1 (Energy) is at index 1: x = 10 - 26 = -16, y = 15 + 2 + 28 = 45
        assert!(win.input(&make_pointer(-10.0, 50.0, MouseButton::Left)));
        assert_eq!(win.current_tab, 1);
        assert_eq!(win.current_transmission(), TRANSMISSION_ENERGY);

        // Now transmission is Energy (ty = 1). Clicking eject should do nothing (no button sent) because can_eject is false
        let _ = host::take_sent();
        let taken = win.input(&make_pointer(147.0, 22.0, MouseButton::Left));
        assert!(taken, "window should consume click inside its bounds");
        let sent = host::take_sent();
        assert!(!sent.contains(&"button 2001".to_owned()));

        // Clicking clear sides on Energy transmission sends 3010
        assert!(win.input(&make_pointer(147.0, 111.0, MouseButton::Left)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 3010".to_owned()));
    }

    #[test]
    fn test_side_config_close_and_escape() {
        let mut win = SideConfigWindow::new(10, 15, vec![TRANSMISSION_ITEMS, TRANSMISSION_ENERGY], vec![true, false]);
        win.open();
        assert!(win.is_open());

        // Escape closes window
        assert!(win.input(&InputEvent::KeyPressed(horizon_mod_sdk::screen::Key {
            key: "key.keyboard.escape".to_owned(),
            modifiers: Modifiers::default(),
        })));
        assert!(!win.is_open());

        // Re-open and close via close button at (10 + 6, 15 + 6) = (16, 21)
        win.open();
        assert!(win.input(&make_pointer(18.0, 23.0, MouseButton::Left)));
        assert!(!win.is_open());
    }

    #[test]
    fn test_side_config_covers_and_dragging() {
        let mut win = SideConfigWindow::new(10, 15, vec![TRANSMISSION_ITEMS, TRANSMISSION_ENERGY], vec![true, false]);
        // When closed, nothing is covered
        assert!(!win.covers(50.0, 50.0));

        win.open();
        // Slot at (50, 50) is covered by window (10, 15, 156, 135)
        assert!(win.covers(50.0, 50.0));
        // Slot at (200, 200) is outside window
        assert!(!win.covers(200.0, 200.0));

        // Dragging by title bar (10..166, 15..33)
        // Click title bar at (50, 20)
        assert!(win.input(&make_pointer(50.0, 20.0, MouseButton::Left)));
        assert!(win.window.dragging());

        // Drag by dx = 5, dy = 10
        assert!(win.input(&InputEvent::MouseDragged(horizon_mod_sdk::screen::Drag {
            x: 55.0,
            y: 30.0,
            button: MouseButton::Left,
            dx: 5.0,
            dy: 10.0,
        })));
        assert_eq!(win.window.x, 15);
        assert_eq!(win.window.y, 25);

        // Mouse released ends drag
        assert!(win.input(&InputEvent::MouseReleased(Pointer {
            x: 55.0,
            y: 30.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        })));
        assert!(!win.window.dragging());

        // Click on inventory area (y >= 166 - 90 = 76, outside window e.g. at x = 5, y = 100)
        // Window x is 15..171, so x = 5 is outside window
        assert!(!win.input(&make_pointer(5.0, 100.0, MouseButton::Left)));
    }
}
