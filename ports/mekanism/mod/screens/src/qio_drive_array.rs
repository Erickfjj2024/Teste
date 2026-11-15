//! `GuiQIODriveArray`: The QIO Drive Array screen displaying the frequency status and 12 drive slots.
//!
//! Java reference:
//! - `mekanism.client.gui.qio.GuiQIODriveArray` (lines 1-35)
//! - `mekanism.client.gui.element.custom.GuiQIOFrequencyDataScreen` (lines 1-63)
//! - `mekanism.client.gui.element.bar.GuiDigitalBar` (lines 1-31)
//! - `mekanism.client.gui.element.tab.GuiQIOFrequencyTab` (lines 1-52)
//! - `mekanism.client.gui.element.tab.GuiSecurityTab` (lines 1-55)
//! - `mekanism.common.tile.qio.TileEntityQIODriveArray`
//!
//! Layout and dimensions:
//! - Main panel: 176 by 206 pixels (`imageWidth = 176`, `imageHeight = 206`, `inventoryLabelY = 112`)
//! - Title: at (0, 6, 176, 6)
//! - Frequency data screen: (15, 19, 144, 46) with items bar (26, 39, 52, 8) and types bar (98, 39, 52, 8)
//! - 12 drive slots: 2 rows of 6 slots at (34 + 18*x, 70 + 18*y)
//! - Inventory label: at (8, 112)
//! - Frequency tab: left at (-26, 6), opens `QIOFrequencyWindow` (160 by 154)
//! - Security tab: right at (176, 34), buttons 9100 (`SECURITY_NEXT`) / 9101 (`SECURITY_PREV`)
//!
//! Differences from Java / Fidelity notes:
//! - The frequency selector is accessed via a modal window overlay opened by the frequency tab,
//!   consistent with `convert.py:2037-2068` and `qio_dashboard`.
//! - Digital bars use standard Mekanism bar frames (`mekanism:bar/base`) with green fill matching `convert.py`.
//! - Security mode is read from data slot 6 (`security_table::SECURED` entry for `qio_drive_array`).
use crate::elements::{
    base_panel, data, inner_screen, inside, inventory_label, slot_frame, string, title, Overlay,
    GREEN, WHITE,
};
use crate::qio_frequency_select::{
    draw_frequency_tab, is_frequency_tab_hovered, QIOFrequencyWindow,
};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{
    AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, TEXT_CENTER, host,
};

pub const SECURITY_NEXT: i32 = 9100;
pub const SECURITY_PREV: i32 = 9101;
pub const TAB_SECURITY_COLOR: u32 = 0xFFFF_A160;

/// The QIO Drive Array screen.
pub struct QIODriveArray {
    pub freq_window: QIOFrequencyWindow,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let screen = QIODriveArray {
        freq_window: QIOFrequencyWindow::new(176.0, 206.0, 8500, 4, 5, 3),
    };
    Ok((
        Box::new(screen),
        Setup {
            width: 176,
            height: 206,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for QIODriveArray {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 206.0);

        // Title at (0, 6, 176, 6)
        let title_text = host::translate("block.mekanism.qio_drive_array", &[]);
        title(c, &title_text, 0.0, 176.0, 6.0);

        // Frequency Data Screen (15, 19, 144, 46)
        inner_screen(c, [15.0, 19.0, 144.0, 46.0]);
        let has_freq = data(3) != 0;
        let freq_name = string(0);
        let freq_display = if has_freq && !freq_name.is_empty() {
            freq_name
        } else {
            host::translate("frequency.mekanism.none", &[])
        };
        c.text(&freq_display, 17.0, 24.0, GREEN, 0, 0.8);

        // Items Bar at (26, 39, 50, 8)
        c.blit_sprite("mekanism:bar/base", 26.0, 39.0, 50.0, 8.0);
        let item_fraction = (data(1) as f32 / 1000.0).clamp(0.0, 1.0);
        let item_fill_w = (48.0 * item_fraction).round();
        if item_fill_w > 0.0 {
            c.fill(27.0, 40.0, item_fill_w, 6.0, 0xFF00_CD00);
        }
        let items_label = host::translate("qio.mekanism.items", &[]);
        c.text(&items_label, 51.0, 51.0, GREEN, TEXT_CENTER, 0.8);
        let items_detail = string(1);
        if !items_detail.is_empty() {
            c.text(&items_detail, 22.0, 58.0, GREEN, 0, 0.6);
        }
        if inside(mouse, [26.0, 39.0, 50.0, 8.0]) {
            let tip = if items_detail.is_empty() { "Items" } else { &items_detail };
            c.tooltip(&[(tip, WHITE)]);
        }

        // Types Bar at (98, 39, 50, 8)
        c.blit_sprite("mekanism:bar/base", 98.0, 39.0, 50.0, 8.0);
        let type_fraction = (data(2) as f32 / 1000.0).clamp(0.0, 1.0);
        let type_fill_w = (48.0 * type_fraction).round();
        if type_fill_w > 0.0 {
            c.fill(99.0, 40.0, type_fill_w, 6.0, 0xFF00_CD00);
        }
        let types_label = host::translate("qio.mekanism.types", &[]);
        c.text(&types_label, 123.0, 51.0, GREEN, TEXT_CENTER, 0.8);
        let types_detail = string(2);
        if !types_detail.is_empty() {
            c.text(&types_detail, 94.0, 58.0, GREEN, 0, 0.6);
        }
        if inside(mouse, [98.0, 39.0, 50.0, 8.0]) {
            let tip = if types_detail.is_empty() { "Types" } else { &types_detail };
            c.tooltip(&[(tip, WHITE)]);
        }

        // 12 Drive slots: 2 rows of 6 at (34 + 18*x, 70 + 18*y)
        for y in 0..2 {
            for x in 0..6 {
                slot_frame(c, 33.0 + 18.0 * x as f32, 69.0 + 18.0 * y as f32);
            }
        }

        // Player inventory label at (8, 112)
        inventory_label(c, 8.0, 112.0);

        // Frequency tab at (-26, 6)
        draw_frequency_tab(c, 6.0, self.freq_window.is_open(), mouse);

        // Security tab at (176, 34)
        let sec_r = [176.0, 34.0, 26.0, 18.0];
        c.blit_sprite_tinted("mekanism:holder_right", sec_r, TAB_SECURITY_COLOR);
        let sec_mode = (data(6).max(0) as usize) % 3;
        let icon_sprite = match sec_mode {
            1 => "mekanism:security/private",
            2 => "mekanism:security/protected",
            _ => "mekanism:security/public",
        };
        c.blit_sprite(icon_sprite, 176.0 + 4.0, 34.0, 18.0, 18.0);
        c.area(SECURITY_NEXT, sec_r, AreaKind::Tab, true, "Security");
        if inside(mouse, sec_r) {
            let mode_key = match sec_mode {
                1 => "security.mekanism.private",
                2 => "security.mekanism.trusted",
                _ => "security.mekanism.public",
            };
            let sec_tip = host::translate("tooltip.mekanism.security", &[host::translate(mode_key, &[])]);
            c.tooltip(&[(&sec_tip, WHITE)]);
        }

        // Frequency window overlay (when open)
        self.freq_window.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        // 1. Frequency window takes precedence if open
        if self.freq_window.is_open() {
            if self.freq_window.input(event) {
                return true;
            }
        }

        // 2. Tab clicks
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

            // Security tab at (176, 34)
            if inside((p.x, p.y), [176.0, 34.0, 26.0, 18.0]) {
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

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, Pointer};

    #[test]
    fn qio_drive_array_renders_bars_and_handles_tabs() {
        host::set_menu(host::Menu {
            data: vec![
                1,   // active
                500, // items (50%)
                250, // types (25%)
                1,   // has_freq
                2,   // public count
                0,   // private count
                1,   // security mode: Private
            ],
            strings: vec![
                "Frequency: Main".into(),
                "Items: 500 / 1000".into(),
                "Types: 2 / 8".into(),
                "Frequency: Main".into(),
                "Owner: Steve".into(),
                "Security: Public".into(),
                "Main".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:qio_drive_array".into(),
            gui_width: 176.0,
            gui_height: 206.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open screen");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 206);
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

        // Verify frequency text, items detail, and security tab area
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Frequency: Main")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Items: 500 / 1000")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == SECURITY_NEXT)));

        // Click security tab at (180, 40)
        let click_sec = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 40.0,
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_sec));
        assert_eq!(host::take_sent(), vec!["button 9101".to_string(), "click".to_string()]);

        // Click frequency tab at (-10, 10)
        let click_freq_tab = InputEvent::MousePressed(Pointer {
            x: -10.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_freq_tab));
        let sent_freq = host::take_sent();
        assert_eq!(sent_freq, vec!["click".to_string()]);

        // Re-render: frequency window overlay should be drawn with row button 8500
        let mut canvas2 = Canvas::new();
        screen.render(&frame, &mut canvas2);
        let bytes2 = canvas2.finish();
        let decoded2 = horizon_mod_sdk::screen::format::decode(&bytes2).expect("decode canvas");
        assert!(decoded2.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == 8500)));
    }

    #[test]
    fn test_qio_drive_array_digital_bars_and_security_tooltip() {
        let mut d = vec![0; 10];
        // Security mode: 2 (trusted -> protected) at data[6]
        d[6] = 2;

        host::set_menu(host::Menu {
            data: d,
            strings: vec![],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:qio_drive_array".into(),
            gui_width: 176.0,
            gui_height: 206.0,
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

        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Tooltip { lines } if lines.iter().any(|(t, _)| t == "tooltip.mekanism.security"))));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:security/protected")));

        // Digital bars base width 50, height 8 at x=26 and x=98
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && rect[0] == 26.0 && rect[2] == 50.0 && rect[3] == 8.0)));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && rect[0] == 98.0 && rect[2] == 50.0 && rect[3] == 8.0)));
    }
}
