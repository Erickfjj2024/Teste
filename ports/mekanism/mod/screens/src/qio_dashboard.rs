//! `GuiQIODashboard`: The block-based QIO Dashboard GUI with 3 crafting windows, frequency selector,
//! and security configuration.
//!
//! Java reference:
//! - `mekanism.client.gui.qio.GuiQIODashboard` (lines 1-38)
//! - `mekanism.client.gui.qio.GuiQIOItemViewer` (lines 1-229)
//! - `mekanism.client.gui.element.window.GuiCraftingWindow` (lines 1-70)
//! - `mekanism.client.gui.element.tab.window.GuiCraftingWindowTab` (lines 1-113)
//! - `mekanism.client.gui.element.tab.GuiSecurityTab` (lines 1-55)
//! - `mekanism.common.inventory.container.tile.QIODashboardContainer`
//!
//! Layout and dimensions:
//! - Main panel: 178 by 211 pixels (`width = 178, height = 211`)
//! - Frequency tab: left at (-26, 6), opens `QIOFrequencyWindow` (160 by 154)
//! - Crafting tabs: left at (-26, 34), (-26, 62), (-26, 90), opening crafting windows 1, 2, and 3
//! - Crafting windows: 124 by 80 pixels each, containing 3x3 ingredient slots (8 + 18*c, 18 + 18*r),
//!   crafting arrow (70, 38), result slot (100, 36), and clear button (104, 60, ID 7611 + w)
//! - Security tab: right at (178, 34), buttons 9100 (`SECURITY_NEXT`) / 9101 (`SECURITY_PREV`)
//!
//! Differences from Java / Fidelity notes:
//! - In Java, a single `GuiCraftingWindowTab` at (-26, 34) spawns up to 3 crafting windows sequentially.
//!   Matching `convert.py:2005-2035`, 3 dedicated tabs at y = 34, 62, 90 allow opening/closing each window directly.
//! - Virtual crafting slots are positioned using `Canvas::slot(slot_idx, x, y, active)`. When a crafting window
//!   is closed, its slots are set inactive so they do not intercept clicks or draw over the panel.
//! - Security mode is read from data slot 8 (`security_table::SECURED` entry for `qio_dashboard`).
use crate::elements::{
    base_panel, data, inside, slot_frame, title, Overlay, TITLE, WHITE,
};
use crate::qio_frequency_select::{
    draw_frequency_tab, is_frequency_tab_hovered, QIOFrequencyWindow,
};
use crate::qio_item_viewer::QIOItemViewer;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::widgets::{Window, WindowPress};
use horizon_mod_sdk::screen::{
    AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host,
};

pub const CLEAR_CRAFTING_WINDOW_BASE: i32 = 7611;
pub const SECURITY_NEXT: i32 = 9100;
pub const SECURITY_PREV: i32 = 9101;
pub const TAB_SECURITY_COLOR: u32 = 0xFFFF_A160;

pub const CRAFTING_WINDOW_WIDTH: i32 = 124;
pub const CRAFTING_WINDOW_HEIGHT: i32 = 80;

/// A single crafting window overlay (124 by 80).
pub struct CraftingWindow {
    pub index: usize,
    pub window: Window,
    pub open: bool,
    pub panel_width: f32,
    pub panel_height: f32,
}

impl CraftingWindow {
    pub fn new(index: usize, panel_width: f32, panel_height: f32) -> Self {
        let wx = ((panel_width - CRAFTING_WINDOW_WIDTH as f32) / 2.0).floor() as i32 + (index as i32 * 10);
        let wy = 30 + (index as i32 * 10);
        let title_str = host::translate("gui.mekanism.crafting.window", &[(index + 1).to_string()]);
        CraftingWindow {
            index,
            window: Window::new(wx, wy, CRAFTING_WINDOW_WIDTH, CRAFTING_WINDOW_HEIGHT, &title_str),
            open: false,
            panel_width,
            panel_height,
        }
    }

    #[allow(dead_code)]
    pub fn covers(&self, sx: f32, sy: f32) -> bool {
        self.open && self.window.covers(sx, sy)
    }

    pub fn render(&mut self, c: &mut Canvas, mouse: (f32, f32)) {
        if !self.open {
            return;
        }
        let wx = self.window.x as f32;
        let wy = self.window.y as f32;

        self.window.draw_background(c);
        self.window.draw_title(c, host::text_width, TITLE);

        // 3x3 crafting grid slot frames at (8 + 18*c, 18 + 18*r)
        for r in 0..3 {
            for col in 0..3 {
                slot_frame(c, wx + 8.0 + col as f32 * 18.0, wy + 18.0 + r as f32 * 18.0);
            }
        }

        // Crafting arrow at (70, 38)
        c.blit_sprite("mekanism:arrow/right", wx + 70.0, wy + 38.0, 22.0, 15.0);

        // Output slot at (100, 36)
        slot_frame(c, wx + 100.0, wy + 36.0);

        // Clear button at (104, 60, 14, 14)
        let clear_rect = [wx + 104.0, wy + 60.0, 14.0, 14.0];
        let clear_hovered = inside(mouse, clear_rect);
        let clear_sprite = if clear_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(clear_sprite, clear_rect[0], clear_rect[1], clear_rect[2], clear_rect[3]);
        c.blit_sprite("mekanism:button/clear_sides", clear_rect[0], clear_rect[1], clear_rect[2], clear_rect[3]);
        let clear_tip = host::translate("gui.mekanism.crafting.window.clear", &[]);
        c.area(
            CLEAR_CRAFTING_WINDOW_BASE + self.index as i32,
            clear_rect,
            AreaKind::Button,
            true,
            &clear_tip,
        );
        if clear_hovered {
            c.tooltip(&[(&clear_tip, WHITE)]);
        }
    }

    pub fn input(&mut self, event: &InputEvent) -> bool {
        if !self.open {
            return false;
        }

        match event {
            InputEvent::MousePressed(p) => {
                let mx = p.x as f64;
                let my = p.y as f64;
                let wx = self.window.x as f32;
                let wy = self.window.y as f32;

                // Check clear button click
                let clear_rect = [wx + 104.0, wy + 60.0, 14.0, 14.0];
                if inside((p.x, p.y), clear_rect) {
                    host::send_button(CLEAR_CRAFTING_WINDOW_BASE + self.index as i32);
                    host::play_click();
                    return true;
                }

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
                        // Check if click was on one of the slots (game owns slot interactions)
                        for r in 0..3 {
                            for col in 0..3 {
                                let slot_r = [wx + 8.0 + col as f32 * 18.0, wy + 18.0 + r as f32 * 18.0, 18.0, 18.0];
                                if inside((p.x, p.y), slot_r) {
                                    return false; // let game process slot click
                                }
                            }
                        }
                        let out_r = [wx + 100.0, wy + 36.0, 18.0, 18.0];
                        if inside((p.x, p.y), out_r) {
                            return false; // let game process slot click
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
            InputEvent::KeyPressed(k) if k.key == "key.keyboard.escape" => {
                self.open = false;
                true
            }
            _ => false,
        }
    }
}

/// The QIO Dashboard screen.
pub struct QIODashboard {
    pub viewer: QIOItemViewer,
    pub freq_window: QIOFrequencyWindow,
    pub crafting_windows: [CraftingWindow; 3],
}

pub fn open(_info: &OpenInfo) -> Opened {
    let screen = QIODashboard {
        viewer: QIOItemViewer::new(178.0, 211.0),
        freq_window: QIOFrequencyWindow::new(178.0, 211.0, 8500, 6, 7, 1),
        crafting_windows: [
            CraftingWindow::new(0, 178.0, 211.0),
            CraftingWindow::new(1, 178.0, 211.0),
            CraftingWindow::new(2, 178.0, 211.0),
        ],
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 178,
            height: 211,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for QIODashboard {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 178.0, 211.0);

        // Title at (0, 5, 178, 5)
        let title_text = host::translate("block.mekanism.qio_dashboard", &[]);
        title(c, &title_text, 0.0, 178.0, 5.0);

        // Core item catalogue viewer
        self.viewer.render_viewer(c, mouse);

        // Frequency tab at (-26, 6)
        draw_frequency_tab(c, 6.0, self.freq_window.is_open(), mouse);

        // Crafting tabs on left at y = 34, 62, 90
        for w in 0..3 {
            let tab_y = 34.0 + 28.0 * w as f32;
            let r = [-26.0, tab_y, 26.0, 18.0];
            let open = self.crafting_windows[w].open;
            c.blit_sprite("mekanism:holder_left", r[0], r[1], r[2], r[3]);
            let tint = if open { 0xFF80_8080 } else { WHITE };
            c.blit_sprite_tinted("mekanism:button/crafting", [-22.0, tab_y, 18.0, 18.0], tint);
            let tab_label = host::translate("gui.mekanism.crafting.window", &[(w + 1).to_string()]);
            c.area(-1, r, AreaKind::Tab, !open, &tab_label);
            if inside(mouse, r) {
                c.tooltip(&[(&tab_label, WHITE)]);
            }
        }

        // Security tab on right at (178, 34)
        let sec_r = [178.0, 34.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", sec_r, TAB_SECURITY_COLOR);
        let sec_mode = (data(8).max(0) as usize) % 3;
        let icon_sprite = match sec_mode {
            1 => "mekanism:security/private",
            2 => "mekanism:security/protected",
            _ => "mekanism:security/public",
        };
        c.blit_sprite(icon_sprite, 178.0 + 4.0, 34.0, 18.0, 18.0);
        c.area(SECURITY_NEXT, sec_r, AreaKind::Tab, true, "Security");
        if inside(mouse, sec_r) {
            let mode_key = match sec_mode {
                1 => "security.mekanism.private",
                2 => "security.mekanism.trusted",
                _ => "security.mekanism.public",
            };
            let sec_tip = host::translate("gui.mekanism.security", &[host::translate(mode_key, &[])]);
            c.tooltip(&[(&sec_tip, WHITE)]);
        }

        // Emit slot positions for the 3 crafting windows
        for w in 0..3 {
            let win = &self.crafting_windows[w];
            if win.open {
                let wx = win.window.x as f32;
                let wy = win.window.y as f32;
                for r in 0..3 {
                    for col in 0..3 {
                        let slot_idx = 10 * w + 3 * r + col;
                        let sx = wx + 8.0 + col as f32 * 18.0 + 1.0;
                        let sy = wy + 18.0 + r as f32 * 18.0 + 1.0;
                        c.slot(slot_idx as u32, sx, sy, true);
                    }
                }
                let res_idx = 10 * w + 9;
                let rx = wx + 100.0 + 1.0;
                let ry = wy + 36.0 + 1.0;
                c.slot(res_idx as u32, rx, ry, true);
            } else {
                for i in 0..10 {
                    let slot_idx = 10 * w + i;
                    c.slot(slot_idx as u32, 0.0, 0.0, false);
                }
            }
        }

        // Render crafting windows overlays
        for w in 0..3 {
            self.crafting_windows[w].render(c, mouse);
        }

        // Frequency window overlay (on top of everything)
        self.freq_window.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        // 1. Frequency window takes precedence if open
        if self.freq_window.is_open() {
            if self.freq_window.input(event) {
                return true;
            }
        }

        // 2. Crafting windows in reverse order (topmost first)
        for w in (0..3).rev() {
            if self.crafting_windows[w].open {
                if self.crafting_windows[w].input(event) {
                    return true;
                }
            }
        }

        // 3. Tab clicks
        if let InputEvent::MousePressed(p) = event {
            // Frequency tab at (-26, 6)
            if is_frequency_tab_hovered((p.x, p.y), 6.0) {
                if self.freq_window.is_open() {
                    self.freq_window.close();
                } else {
                    self.freq_window.open();
                }
                host::play_click();
                return true;
            }

            // Crafting tabs at (-26, 34 + 28*w)
            for w in 0..3 {
                let tab_y = 34.0 + 28.0 * w as f32;
                if inside((p.x, p.y), [-26.0, tab_y, 26.0, 18.0]) {
                    self.crafting_windows[w].open = !self.crafting_windows[w].open;
                    host::play_click();
                    return true;
                }
            }

            // Security tab at (178, 34)
            if inside((p.x, p.y), [178.0, 34.0, 26.0, 18.0]) {
                let btn = if p.button == MouseButton::Left {
                    SECURITY_NEXT
                } else {
                    SECURITY_PREV
                };
                host::send_button(btn);
                host::play_click();
                return true;
            }
        }

        // 4. Core viewer input (item grid, search, sort buttons)
        self.viewer.input_viewer(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, Pointer};

    #[test]
    fn qio_dashboard_renders_and_handles_crafting_and_security_tabs() {
        host::set_menu(host::Menu {
            data: vec![1, 0, 0, 0, 1, 0, 1, 0, 0], // data(8) = security 0 (Public)
            strings: vec![
                "Frequency: Main (100/500)".into(),
                "Frequency: Main".into(),
                "Owner: Steve".into(),
                "Security: Public".into(),
                "Main".into(),
            ],
            grid: vec![host::GridEntry {
                item: "minecraft:diamond".into(),
                components: "{}".into(),
                count: 64,
            }],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:qio_dashboard".into(),
            gui_width: 178.0,
            gui_height: 211.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open screen");
        assert_eq!(setup.width, 178);
        assert_eq!(setup.height, 211);
        assert!(!setup.hide_inventory);
        assert!(setup.own_labels);

        let frame = Frame {
            mouse_x: 20.0,
            mouse_y: 20.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");

        // Verify title and security tab area present
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == SECURITY_NEXT)));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Item { item, .. } if item == "minecraft:diamond")));

        // Test clicking security tab at (180, 40)
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        assert_eq!(host::take_sent(), vec!["button 9100".to_string(), "click".to_string()]);

        // Test clicking crafting tab 1 at (-10, 36) to open crafting window 0
        let click_craft_tab = InputEvent::MousePressed(Pointer {
            x: -10.0,
            y: 36.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_craft_tab));
        let sent_craft = host::take_sent();
        assert_eq!(sent_craft, vec!["click".to_string()]);

        // Re-render: crafting window 0 should be visible with its clear button (7611)
        let mut canvas2 = Canvas::new();
        screen.render(&frame, &mut canvas2);
        let bytes2 = canvas2.finish();
        let decoded2 = horizon_mod_sdk::screen::format::decode(&bytes2).expect("decode canvas");
        assert!(decoded2.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == CLEAR_CRAFTING_WINDOW_BASE)));

        // Test clicking the clear button inside crafting window 0
        // Window 0 is at wx = ((178 - 124) / 2) = 27, wy = 30. Clear button is at (27 + 104, 30 + 60) = (131, 90)
        let click_clear = InputEvent::MousePressed(Pointer {
            x: 135.0,
            y: 95.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_clear));
        let sent_clear = host::take_sent();
        assert_eq!(sent_clear, vec!["button 7611".to_string(), "click".to_string()]);
    }

    #[test]
    fn test_qio_dashboard_crafting_window_title_and_security_tooltip() {
        let mut d = vec![0; 10];
        // Security mode: 2 (trusted -> protected) at data[8]
        d[8] = 2;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:qio_dashboard".into(),
            gui_width: 178.0,
            gui_height: 211.0,
            touch: false,
        };
        let (mut screen, _) = open(&info).expect("open screen");

        // Hover over security tab at (180, 40)
        let frame_sec = Frame {
            mouse_x: 180.0,
            mouse_y: 40.0,
            partial_tick: 0.0,
            time_ms: 1000,
        };
        let mut canvas = Canvas::new();
        screen.render(&frame_sec, &mut canvas);
        let decoded = horizon_mod_sdk::screen::format::decode(&canvas.finish()).expect("decode");

        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "gui.mekanism.security"))));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:security/protected")));

        // Open crafting window 1 (click tab at -10, 36)
        let click_tab = InputEvent::MousePressed(Pointer {
            x: -10.0,
            y: 36.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_tab));

        // Render and check translated crafting window title
        let mut canvas2 = Canvas::new();
        screen.render(&frame_sec, &mut canvas2);
        let decoded2 = horizon_mod_sdk::screen::format::decode(&canvas2.finish()).expect("decode");
        assert!(decoded2.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "gui.mekanism.crafting.window")));
    }
}
