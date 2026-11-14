//! `TransporterWindow` (`GuiTransporterConfig`, `GuiTransporterConfigTab` at (-26, 34)):
//! The transporter configuration window (156 by 119 pixels, default at ((width - 156)/2, 15)) allowing
//! configuration of item eject color and strict input for machines that move items.
//!
//! Java reference:
//! - `mekanism.client.gui.element.window.GuiTransporterConfig`
//! - `mekanism.client.gui.element.tab.window.GuiTransporterConfigTab`
//! - `mekanism.client.gui.element.button.ColorButton`
//!
//! Server data:
//! - The machine's numbers start at the index given by `transporter_table.rs` (`TRANSPORTED`):
//!   - `data_slot`: Eject color (0 = None, 1..18 = `EnumColor` ordinals + 1).
//!   - `data_slot + 1`: Strict input flag (0 = Off, 1 = On).
//! - Button IDs:
//!   - Strict input button (exclamation at 136, 6): 9302 (`STRICT`).
//!   - Eject color button (at 112, 49): 9300 (`COLOR`, next color), right click 9301 (`PREVIOUS`, previous color).
//!
//! Differences from Java / Fidelity notes:
//! - NÃO FEITO: os seis botões de cor de entrada por lado do Java não existem no servidor
//!   (o servidor não sincroniza nem suporta cor de entrada individual por lado no PacketInputColor).
use crate::elements::{data, inside, inner_screen, slot_frame, GREEN, SUBHEADING, TITLE, WHITE, Overlay};
use crate::transporter_table::TRANSPORTED;
use horizon_mod_sdk::screen::widgets::{SideTab, Window, WindowPress};
use horizon_mod_sdk::screen::{AreaKind, Canvas, InputEvent, MouseButton, TEXT_CENTER, host};

pub const WIDTH: f32 = 156.0;
pub const HEIGHT: f32 = 119.0;

/// Button IDs sent to server.
pub const COLOR: i32 = 9300;
pub const PREVIOUS: i32 = 9301;
pub const STRICT: i32 = 9302;

/// The 18 transporter colors from `convert.py` / `EnumColor.java` (name, RGB).
pub const TRANSPORTER_COLORS: &[(&str, u32)] = &[
    ("Black", 0x404040),
    ("Blue", 0x366BD0),
    ("Green", 0x59C15F),
    ("Cyan", 0x00F3D0),
    ("Dark Red", 0xC9071F),
    ("Purple", 0xA460D9),
    ("Orange", 0xFFA160),
    ("Light Gray", 0xCFCFCF),
    ("Gray", 0x7A7A7A),
    ("Light Blue", 0x559EFF),
    ("Lime", 0x75FF89),
    ("Aqua", 0x30FFF9),
    ("Red", 0xFF383C),
    ("Magenta", 0xD55ECB),
    ("Yellow", 0xFFDD4F),
    ("White", 0xFFFFFF),
    ("Brown", 0xA17649),
    ("Pink", 0xFFBCC4),
];

/// `TransporterWindow`: the overlay window for transporter configuration.
pub struct TransporterWindow {
    pub window: Window,
    pub open: bool,
    pub data_slot: usize,
    pub panel_width: f32,
    pub panel_height: f32,
}

impl TransporterWindow {
    /// Creates a new `TransporterWindow` at `(x, y)` with the given server data slot index.
    pub fn new(x: i32, y: i32, data_slot: usize) -> Self {
        let window = Window::new(x, y, WIDTH as i32, HEIGHT as i32, "Transporter Config");
        TransporterWindow {
            window,
            open: false,
            data_slot,
            panel_width: 176.0,
            panel_height: 166.0,
        }
    }

    /// Creates a `TransporterWindow` positioned and indexed for a specific machine menu if it supports transporter config.
    pub fn for_machine(menu: &str, screen_width: i32, screen_height: i32) -> Option<Self> {
        let &(_, data_slot) = TRANSPORTED.iter().find(|(m, _)| *m == menu)?;
        let x = (screen_width - WIDTH as i32) / 2;
        let mut win = Self::new(x, 15, data_slot);
        win.panel_width = screen_width as f32;
        win.panel_height = screen_height as f32;
        win.window.place(None, [0.0, 0.0, screen_width as f32, screen_height as f32]);
        Some(win)
    }

    /// Side tab at (-26, 34) for `GuiTransporterConfigTab`.
    pub fn side_tab() -> SideTab {
        SideTab::new(-1, -26, 34, "mekanism:button/transporter_config", "Transporter Config")
    }

    /// Whether a screen slot at `(sx, sy)` is covered by the open window.
    pub fn covers(&self, sx: f32, sy: f32) -> bool {
        self.open && self.window.covers(sx, sy)
    }
}

impl Overlay for TransporterWindow {
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

        // 1. Window background (shadow, panel base, close button at 6, 6)
        self.window.draw_background(c);

        // 2. Window title: "Transporter Config" centred at y = 5
        self.window.title = "Transporter Config".to_owned();
        self.window.draw_title(c, |t| (t.len() * 6) as f32, TITLE);

        // 3. Strict Input inner screen at (38, 15, 80, 12)
        let screen_rect = [wx + 38.0, wy + 15.0, 80.0, 12.0];
        inner_screen(c, screen_rect);

        let strict = data(self.data_slot + 1) != 0;
        let strict_text = if strict {
            "Strict Input: On"
        } else {
            "Strict Input: Off"
        };
        c.draw_string(strict_text, wx + 41.0, wy + 17.0, GREEN, false);

        // 4. Strict Input button (exclamation) at (136, 6, 14, 16)
        let strict_btn_rect = [wx + 136.0, wy + 6.0, 14.0, 16.0];
        let strict_hovered = inside(mouse, strict_btn_rect);
        let strict_sprite = if strict_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(strict_sprite, strict_btn_rect[0], strict_btn_rect[1], strict_btn_rect[2], strict_btn_rect[3]);
        c.blit_sprite(
            "mekanism:button/exclamation",
            strict_btn_rect[0],
            strict_btn_rect[1],
            strict_btn_rect[2],
            strict_btn_rect[3],
        );
        c.area(STRICT, strict_btn_rect, AreaKind::Button, true, "Strict Input");
        if strict_hovered {
            c.tooltip(&[("Strict Input", WHITE)]);
        }

        // 5. Output color slot frame at (111, 48, 18, 18)
        slot_frame(c, wx + 111.0, wy + 48.0);

        // 6. Output ColorButton at (112, 49, 16, 16)
        let color_val = data(self.data_slot);
        let color_idx = usize::try_from(color_val).ok().unwrap_or(0);
        let color_rect = [wx + 112.0, wy + 49.0, 16.0, 16.0];
        let color_hovered = inside(mouse, color_rect);

        if color_idx > 0 && color_idx <= TRANSPORTER_COLORS.len() {
            let (_, rgb) = TRANSPORTER_COLORS[color_idx - 1];
            c.fill(color_rect[0], color_rect[1], color_rect[2], color_rect[3], 0xFF00_0000 | rgb);
        }
        c.area(COLOR, color_rect, AreaKind::Button, true, "Color");
        if color_hovered {
            if color_idx > 0 && color_idx <= TRANSPORTER_COLORS.len() {
                let (name, rgb) = TRANSPORTER_COLORS[color_idx - 1];
                c.tooltip(&[(name, 0xFF00_0000 | rgb)]);
            } else {
                c.tooltip(&[("None", WHITE)]);
            }
        }

        // 7. Subheadings:
        // - "Input" centred in 18..86 (width 68) at y = 105
        c.text("Input", wx + 18.0 + 34.0, wy + 105.0, SUBHEADING, TEXT_CENTER, 1.0);
        // - "Output" centred in 86..156 (width 70) at y = 68
        c.text("Output", wx + 86.0 + 35.0, wy + 68.0, SUBHEADING, TEXT_CENTER, 1.0);

        // 8. Six input color side buttons:
        // NÃO FEITO: os seis botões de cor de entrada por lado do Java não existem no servidor
        // (o servidor não sincroniza nem suporta cor de entrada individual por lado no PacketInputColor).
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if !self.open {
            return false;
        }
        let (wx, wy) = (self.window.x as f32, self.window.y as f32);
        match event {
            InputEvent::MousePressed(p) => {
                let mouse = (p.x, p.y);

                // 1. Close button at (6, 6, 8, 12)
                if inside(mouse, [wx + 6.0, wy + 6.0, 8.0, 12.0]) {
                    self.close();
                    host::play_click();
                    return true;
                }

                // 2. Strict Input button at (136, 6, 14, 16)
                if inside(mouse, [wx + 136.0, wy + 6.0, 14.0, 16.0]) {
                    host::send_button(STRICT);
                    host::play_click();
                    return true;
                }

                // 3. Color button at (112, 49, 16, 16)
                if inside(mouse, [wx + 112.0, wy + 49.0, 16.0, 16.0]) {
                    let id = if p.button == MouseButton::Right {
                        PREVIOUS
                    } else {
                        COLOR
                    };
                    host::send_button(id);
                    host::play_click();
                    return true;
                }

                // 4. Window title dragging or outside press
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
    fn test_transporter_render_and_commands() {
        host::set_menu(host::Menu {
            // data slot 38 has color=2 (Blue), slot 39 has strict=1 (On)
            data: {
                let mut d = vec![0; 40];
                d[38] = 2; // Blue
                d[39] = 1; // Strict On
                d
            },
            ..Default::default()
        });

        let mut win = TransporterWindow::new(10, 15, 38);
        win.open();

        let mut c = Canvas::new();
        win.render(&mut c, (50.0, 50.0));
        let bytes = c.finish();
        let frame = format::decode(&bytes).expect("frame decoding should succeed");
        assert!(frame.commands.len() > 8, "Frame should contain background, title, strict screen, slot, color fill");

        // Verify closed window renders nothing
        win.close();
        let mut c2 = Canvas::new();
        win.render(&mut c2, (50.0, 50.0));
        let frame2 = format::decode(&c2.finish()).unwrap();
        assert_eq!(frame2.commands.len(), 0);
    }

    #[test]
    fn test_transporter_buttons_input() {
        host::set_menu(host::Menu::default());
        let _ = host::take_sent();

        let mut win = TransporterWindow::new(10, 15, 38);
        win.open();

        // 1. Strict input button at (10 + 136, 15 + 6) = (146, 21)
        assert!(win.input(&make_pointer(147.0, 22.0, MouseButton::Left)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 9302".to_owned()));

        // 2. Color button at (10 + 112, 15 + 49) = (122, 64) - Left click -> 9300
        assert!(win.input(&make_pointer(123.0, 65.0, MouseButton::Left)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 9300".to_owned()));

        // 3. Color button - Right click -> 9301
        assert!(win.input(&make_pointer(123.0, 65.0, MouseButton::Right)));
        let sent = host::take_sent();
        assert!(sent.contains(&"button 9301".to_owned()));
    }

    #[test]
    fn test_transporter_close_and_escape() {
        let mut win = TransporterWindow::new(10, 15, 38);
        win.open();
        assert!(win.is_open());

        // Escape closes window
        assert!(win.input(&InputEvent::KeyPressed(horizon_mod_sdk::screen::Key {
            key: "key.keyboard.escape".to_owned(),
            modifiers: Modifiers::default(),
        })));
        assert!(!win.is_open());

        // Close button at (10 + 6, 15 + 6) = (16, 21)
        win.open();
        assert!(win.input(&make_pointer(18.0, 23.0, MouseButton::Left)));
        assert!(!win.is_open());
    }

    #[test]
    fn test_transporter_for_machine_lookup() {
        let win = TransporterWindow::for_machine("mekanism:crusher", 176, 166).expect("crusher is transported");
        assert_eq!(win.data_slot, 32);

        let win2 = TransporterWindow::for_machine("mekanism:teleporter", 176, 240);
        assert!(win2.is_none(), "teleporter is not transported");
    }

    #[test]
    fn test_transporter_covers_and_dragging() {
        let mut win = TransporterWindow::new(10, 15, 38);
        assert!(!win.covers(50.0, 50.0));

        win.open();
        assert!(win.covers(50.0, 50.0));
        assert!(!win.covers(200.0, 200.0));

        // Drag title bar at (50, 20)
        assert!(win.input(&make_pointer(50.0, 20.0, MouseButton::Left)));
        assert!(win.window.dragging());

        assert!(win.input(&InputEvent::MouseDragged(horizon_mod_sdk::screen::Drag {
            x: 55.0,
            y: 30.0,
            button: MouseButton::Left,
            dx: 5.0,
            dy: 10.0,
        })));
        assert_eq!(win.window.x, 15);
        assert_eq!(win.window.y, 25);

        assert!(win.input(&InputEvent::MouseReleased(Pointer {
            x: 55.0,
            y: 30.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        })));
        assert!(!win.window.dragging());

        // Click outside window on inventory (x = 5, y = 100) -> passes through
        assert!(!win.input(&make_pointer(5.0, 100.0, MouseButton::Left)));
    }
}
