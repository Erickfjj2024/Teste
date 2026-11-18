//! `GuiQIOFrequencySelect` / `GuiQIOTileFrequencySelect` / `GuiQIOItemFrequencySelect` / `GuiQIOFrequencyTab`:
//! Frequency selection tab and modal window for QIO blocks and items.
//!
//! Java reference:
//! - `mekanism.client.gui.qio.GuiQIOTileFrequencySelect` (lines 1-59)
//! - `mekanism.client.gui.qio.GuiQIOItemFrequencySelect` (lines 1-52)
//! - `mekanism.client.gui.element.tab.GuiQIOFrequencyTab` (lines 1-52)
//! - `mekanism.client.gui.element.custom.GuiFrequencySelector` (lines 1-150)
//!
//! In Java, clicking `GuiQIOFrequencyTab` sends a packet to open a separate frequency screen
//! (`GuiQIOTileFrequencySelect` / `GuiQIOItemFrequencySelect`) with a "back" button at (6, 6).
//! In the port (`convert.py:1943-1962`, `qio_world.rs:23`), the frequency selector is presented
//! as a modal overlay window (`width = 160, height = 154`) opened directly by the tab at (-26, 6),
//! retaining the player's view of the dashboard/drive array underneath.
//!
//! Differences from Java / Fidelity notes:
//! - Rendered as an overlay window instead of a full screen switch, matching `convert.py:1943`
//!   and other Mekanism modal windows (Upgrades, Side Config).
//! - Text field for new frequency name is managed by `frequency::Selector` using `Canvas::field`.
//!
//! Server data mapping:
//! - Selector base ID: 8500 (`qio_world.rs::FREQ_BASE`)
//! - Buttons: 8500..8515 public rows, 8516..8531 private rows, 8532 delete
//! - Fields: 8533 public name, 8534 private name
use crate::elements::{inside, Overlay, TITLE, WHITE};
use crate::frequency::Selector;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::widgets::{Window, WindowPress};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const TAB_WIDTH: f32 = 26.0;
pub const TAB_HEIGHT: f32 = 18.0;
pub const WINDOW_WIDTH: i32 = 160;
pub const WINDOW_HEIGHT: i32 = 154;

/// Draws the QIO frequency tab on the left of the main panel at `(-26, y)`.
pub fn draw_frequency_tab(c: &mut Canvas, y: f32, open: bool, mouse: (f32, f32)) {
    let r = [-TAB_WIDTH, y, TAB_WIDTH, TAB_HEIGHT];
    c.blit_sprite("mekanism:holder_left", r[0], r[1], r[2], r[3]);
    let tint = if open { 0xFF80_8080 } else { WHITE };
    c.blit_sprite_tinted("mekanism:button/frequency", [-22.0, y, 18.0, 18.0], tint);
    let label = host::translate("qio.mekanism.set_frequency", &[]);
    c.area(-1, r, AreaKind::Tab, !open, &label);
    if inside(mouse, r) {
        c.tooltip(&[(&label, WHITE)]);
    }
}

/// Checks if the cursor is over the frequency tab at `(-26, y)`.
pub fn is_frequency_tab_hovered(mouse: (f32, f32), y: f32) -> bool {
    inside(mouse, [-TAB_WIDTH, y, TAB_WIDTH, TAB_HEIGHT])
}

/// Modal window overlay containing the QIO frequency selector.
pub struct QIOFrequencyWindow {
    pub window: Window,
    pub selector: Selector,
    pub open: bool,
    pub panel_width: f32,
    pub panel_height: f32,
}

impl QIOFrequencyWindow {
    /// Creates a new `QIOFrequencyWindow` centered horizontally for the given panel dimensions.
    pub fn new(gui_width: f32, gui_height: f32, base: i32, public_count: usize, private_count: usize, first_string: usize) -> Self {
        let wx = ((gui_width - WINDOW_WIDTH as f32) / 2.0).floor() as i32;
        let wy = 26;
        let title = host::translate("qio.mekanism.set_frequency", &[]);
        let window = Window::new(wx, wy, WINDOW_WIDTH, WINDOW_HEIGHT, &title);
        let selector = Selector::new(wx as f32 + 14.0, wy as f32 + 18.0, base, public_count, private_count, first_string);
        QIOFrequencyWindow {
            window,
            selector,
            open: false,
            panel_width: gui_width,
            panel_height: gui_height,
        }
    }

    /// Whether the open window covers a slot at GUI coordinates `(sx, sy)`.
    #[allow(dead_code)]
    pub fn covers(&self, sx: f32, sy: f32) -> bool {
        self.open && self.window.covers(sx, sy)
    }
}

impl Overlay for QIOFrequencyWindow {
    fn render(&mut self, c: &mut Canvas, mouse: (f32, f32)) {
        if !self.open {
            return;
        }
        self.window.draw_background(c);
        self.window.draw_title(c, host::text_width, TITLE);
        self.selector.x = self.window.x as f32 + 14.0;
        self.selector.y = self.window.y as f32 + 18.0;
        self.selector.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if !self.open {
            return false;
        }

        match event {
            InputEvent::MousePressed(p) => {
                let mx = p.x as f64;
                let my = p.y as f64;
                let inv_top = Some((self.panel_height - 93.0) as f64);
                let press = self.window.press(mx, my, inv_top, self.panel_width as f64);
                match press {
                    WindowPress::Closed => {
                        self.open = false;
                        host::play_click();
                        true
                    }
                    WindowPress::Dragging => true,
                    WindowPress::Inside => {
                        if p.button == MouseButton::Left {
                            if self.selector.press((p.x, p.y)) {
                                return true;
                            }
                        }
                        true
                    }
                    WindowPress::Inventory => false,
                    WindowPress::Blocked => true,
                }
            }
            InputEvent::MouseDragged(d) => {
                if self.window.dragging() {
                    let bounds = [0.0, 0.0, self.panel_width, self.panel_height];
                    self.window.drag(d.x as f64, d.y as f64, bounds);
                    self.selector.x = self.window.x as f32 + 14.0;
                    self.selector.y = self.window.y as f32 + 18.0;
                    true
                } else {
                    false
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
            InputEvent::FieldSubmitted(f) => self.selector.submitted(f.id, &f.text),
            InputEvent::KeyPressed(k) if k.key == "key.keyboard.escape" => {
                self.open = false;
                true
            }
            _ => false,
        }
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
    }
}

/// Standalone screen for `mekanism:qio_frequency_select`.
pub struct StandaloneFrequencySelect {
    pub selector: Selector,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let screen = StandaloneFrequencySelect {
        selector: Selector::new(14.0, 18.0, 8500, 0, 1, 0),
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 160,
            height: 154,
            hide_inventory: true,
            own_labels: true,
        },
    ))
}

impl MekScreen for StandaloneFrequencySelect {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        crate::elements::base_panel(c, 160.0, 154.0);
        let title = host::translate("qio.mekanism.set_frequency", &[]);
        crate::elements::title(c, &title, 0.0, 160.0, 5.0);
        self.selector.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => self.selector.press((p.x, p.y)),
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
    fn qio_frequency_tab_and_window_toggle_and_send_buttons() {
        let mut strings = vec![
            "Frequency: Main".into(),
            "Owner: Steve".into(),
            "Security: Public".into(),
        ];
        strings.extend((0..16).map(|i| format!("Freq{i}")));
        strings.extend((0..16).map(|i| format!("Priv{i}")));

        host::set_menu(host::Menu {
            data: vec![1, 0, 0, 0, 2, 1], // public_count = 2, private_count = 1
            strings,
            ..Default::default()
        });

        let mut win = QIOFrequencyWindow::new(178.0, 211.0, 8500, 4, 5, 0);
        assert!(!win.is_open());

        win.open();
        assert!(win.is_open());

        let frame = Frame {
            mouse_x: 50.0,
            mouse_y: 50.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        win.render(&mut canvas, (frame.mouse_x, frame.mouse_y));
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");

        // Should contain area for close button (-1) and first row (8500)
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == -1)));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == 8500)));

        // Click on first row at (wx + 14 + 1, wy + 18 + 23)
        let row_x = win.window.x as f32 + 15.0;
        let row_y = win.window.y as f32 + 42.0;
        let click_row = InputEvent::MousePressed(Pointer {
            x: row_x,
            y: row_y,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(win.input(&click_row));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 8500".to_string(), "click".to_string()]);

        // Escape closes window
        let esc = InputEvent::KeyPressed(horizon_mod_sdk::screen::Key {
            key: "key.keyboard.escape".into(),
            modifiers: Modifiers::default(),
        });
        assert!(win.input(&esc));
        assert!(!win.is_open());
    }

    #[test]
    fn standalone_frequency_select_renders_without_panic() {
        host::set_menu(host::Menu {
            data: vec![1, 1],
            strings: vec!["Frequency: Main".into(), "Owner: Steve".into(), "Security: Public".into(), "Main".into()],
            ..Default::default()
        });
        let info = OpenInfo {
            menu: "mekanism:qio_frequency_select".into(),
            gui_width: 160.0,
            gui_height: 154.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open screen");
        assert_eq!(setup.width, 160);
        assert_eq!(setup.height, 154);
        let frame = Frame {
            mouse_x: 10.0,
            mouse_y: 10.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.len() > 5);
    }
}
