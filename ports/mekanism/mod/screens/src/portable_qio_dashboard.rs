//! `GuiPortableQIODashboard`: The handheld Portable QIO Dashboard GUI.
//!
//! Java reference:
//! - `mekanism.client.gui.qio.GuiPortableQIODashboard` (lines 1-37)
//! - `mekanism.client.gui.qio.GuiQIOItemViewer` (lines 1-229)
//! - `mekanism.common.inventory.container.item.PortableQIODashboardContainer`
//! - Server logic: `port/mod/src/portable_qio.rs`
//!
//! Layout and dimensions:
//! - Main panel: 178 by 211 pixels (`width = 178, height = 211`, `inventory_y = 129`)
//! - Frequency tab: left at (-26, 6), opens `QIOFrequencyWindow` (160 by 154)
//! - Title: "Portable QIO Dashboard" at (0, 5, 178, 5)
//! - Core item catalogue: 8x4 item grid, search box (8562), sort button (8560), direction button (8561)
//!
//! Differences from Java / Fidelity notes:
//! - The handheld dashboard does not include crafting windows or tabs (only present on the block-based QIO Dashboard).
//! - Security tab is not present on the handheld item screen (in Java `GuiPortableQIODashboard` extends `GuiMekanism`, not `GuiMekanismTile`).
use crate::elements::{base_panel, title, Overlay};
use crate::qio_frequency_select::{
    draw_frequency_tab, is_frequency_tab_hovered, QIOFrequencyWindow,
};
use crate::qio_item_viewer::QIOItemViewer;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

/// The Portable QIO Dashboard screen.
pub struct PortableQIODashboard {
    pub viewer: QIOItemViewer,
    pub freq_window: QIOFrequencyWindow,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let screen = PortableQIODashboard {
        viewer: QIOItemViewer::new(178.0, 211.0),
        freq_window: QIOFrequencyWindow::new(178.0, 211.0, 8500, 6, 7, 1),
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

impl MekScreen for PortableQIODashboard {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 178.0, 211.0);

        // Title at (0, 5, 178, 5)
        let title_text = host::translate("item.mekanism.portable_qio_dashboard", &[]);
        title(c, &title_text, 0.0, 178.0, 5.0);

        // Core item catalogue viewer (frequency text, search field, 8x4 grid, sort controls)
        self.viewer.render_viewer(c, mouse);

        // Frequency tab at (-26, 6)
        draw_frequency_tab(c, 6.0, self.freq_window.is_open(), mouse);

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

        // 2. Frequency tab click at (-26, 6)
        if let InputEvent::MousePressed(p) = event {
            if is_frequency_tab_hovered((p.x, p.y), 6.0) {
                if self.freq_window.is_open() {
                    self.freq_window.close();
                } else {
                    self.freq_window.open();
                }
                host::play_click();
                return true;
            }
        }

        // 3. Core viewer input
        self.viewer.input_viewer(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{MouseButton, Modifiers, Pointer};

    #[test]
    fn portable_qio_dashboard_renders_and_toggles_frequency() {
        host::set_menu(host::Menu {
            data: vec![1, 0, 0, 1, 0, 1, 1, 0], // has_freq = 1, sort = 1 (Count), asc = 0
            strings: vec![
                "Frequency: Pocket (10/100)".into(),
                "Frequency: Pocket".into(),
                "Owner: Steve".into(),
                "Security: Public".into(),
                "Pocket".into(),
            ],
            grid: vec![host::GridEntry {
                item: "minecraft:redstone".into(),
                components: "{}".into(),
                count: 320,
            }],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:portable_qio_dashboard".into(),
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

        // Verify title, item in grid, and frequency tab area
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "item.mekanism.portable_qio_dashboard")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Item { item, .. } if item == "minecraft:redstone")));

        // Test clicking frequency tab at (-10, 10)
        let click_freq_tab = InputEvent::MousePressed(Pointer {
            x: -10.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_freq_tab));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["click".to_string()]);

        // Re-render: frequency window should now be drawn
        let mut canvas2 = Canvas::new();
        screen.render(&frame, &mut canvas2);
        let bytes2 = canvas2.finish();
        let decoded2 = horizon_mod_sdk::screen::format::decode(&bytes2).expect("decode canvas");
        assert!(decoded2.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == 8500)));
    }
}
