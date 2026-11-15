//! `GuiLogisticalSorter` (276 by 254): left inner screen at (9, 17, 85 by 140) displaying
//! filter count and sorter settings, single item toggle at (12, 46), auto eject toggle at (12, 106),
//! element holder at (95, 17, 158 by 118) for four filter rows, new filter (96, 136) and delete (200, 136)
//! buttons, and security tab on the right.
//!
//! Java source of truth:
//! - `mekanism.client.gui.GuiLogisticalSorter` (`GuiLogisticalSorter.java:36-104`)
//! - `mekanism.client.gui.GuiFilterHolder` (`GuiFilterHolder.java:28-75`)
//! - `mekanism.common.tile.TileEntityLogisticalSorter`
//!
//! Numbers (`sorter.rs`): 0 auto eject (1 yes, 0 no), 1 single item (1 yes, 0 no), 2 filter count,
//! 3 selected row plus one, 4 security mode. Strings: 0 header/stats, 1..4 filter row descriptions.

use crate::chrome::{Chrome, ChromeSpec};
use crate::elements::{base_panel, button, data, element_holder, inner_screen, inside, string, title, GREEN, TITLE, WHITE};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub struct LogisticalSorter {
    chrome: Chrome,
    selected_row: Option<usize>,
}

pub fn open(info: &OpenInfo) -> Opened {
    let chrome_spec = ChromeSpec {
        menu: "mekanism:logistical_sorter",
        redstone: None,
        energy: None,
        upgrades: None,
        warning: None,
        side_config: None,
        transporter: None,
    };
    let screen = LogisticalSorter {
        chrome: Chrome::new(chrome_spec, 276.0),
        selected_row: None,
    };
    Ok((Box::new(screen), Setup { width: 276, height: 254, hide_inventory: false, own_labels: true }))
}

impl MekScreen for LogisticalSorter {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 276.0, 254.0);

        // Left inner screen at (9, 17, 85, 140)
        inner_screen(c, [9.0, 17.0, 85.0, 140.0]);
        let filters_trans = host::translate("filter.mekanism.filter.count", &[data(2).to_string()]);
        let filters_str = if filters_trans.is_empty() { format!("Filters: {}", data(2)) } else { filters_trans };
        c.text(&filters_str, 12.0, 21.0, GREEN, 0, 0.80);

        let single_title = host::translate("logistical_sorter.mekanism.single_item", &[]);
        c.text(if single_title.is_empty() { "Single Item" } else { &single_title }, 11.0, 36.0, GREEN, 0, 0.80);
        let single_on = data(1) == 1;
        c.text(if single_on { "On" } else { "Off" }, 23.0, 49.0, if single_on { GREEN } else { 0xFFFF_5555 }, 0, 0.80);

        let rr_title = host::translate("logistical_sorter.mekanism.round_robin", &[]);
        c.text(if rr_title.is_empty() { "Round Robin" } else { &rr_title }, 11.0, 66.0, GREEN, 0, 0.80);
        c.text("Off", 23.0, 79.0, 0xFF80_8080, 0, 0.80);

        let eject_title = host::translate("logistical_sorter.mekanism.auto_eject", &[]);
        c.text(if eject_title.is_empty() { "Auto Eject" } else { &eject_title }, 11.0, 96.0, GREEN, 0, 0.80);
        let eject_on = data(0) == 1;
        c.text(if eject_on { "On" } else { "Off" }, 23.0, 109.0, if eject_on { GREEN } else { 0xFFFF_5555 }, 0, 0.80);

        let def_title = host::translate("logistical_sorter.mekanism.default", &[]);
        c.text(if def_title.is_empty() { "Default" } else { &def_title }, 11.0, 126.0, GREEN, 0, 0.80);

        // Toggle buttons: single item at (12, 46, 14, 14), round robin at (12, 76, 14, 14), auto eject at (12, 106, 14, 14)
        c.blit_sprite("mekanism:button", 12.0, 46.0, 14.0, 14.0);
        c.blit("mekanism:gui/button/single", 12.0, 46.0, 0.0, 0.0, 14.0, 14.0, 14.0, 14.0);
        c.area(7401, [12.0, 46.0, 14.0, 14.0], AreaKind::Button, true, "Single Item");
        if inside(mouse, [12.0, 46.0, 14.0, 14.0]) {
            let tip = host::translate("logistical_sorter.mekanism.single_item.description", &[]);
            c.tooltip(&[(if tip.is_empty() { "Single item: sends one item at a time" } else { &tip }, WHITE)]);
        }

        // NÃO FEITO: round robin toggle no servidor/menu_data (o port não tem round robin ativo no servidor)
        c.blit_sprite("mekanism:button", 12.0, 76.0, 14.0, 14.0);
        c.blit("mekanism:gui/button/round_robin", 12.0, 76.0, 0.0, 0.0, 14.0, 14.0, 14.0, 14.0);
        c.area(7403, [12.0, 76.0, 14.0, 14.0], AreaKind::Button, true, "Round Robin");
        if inside(mouse, [12.0, 76.0, 14.0, 14.0]) {
            let tip = host::translate("logistical_sorter.mekanism.round_robin.description", &[]);
            c.tooltip(&[(if tip.is_empty() { "Cycles between all connected inventories when sending items." } else { &tip }, WHITE)]);
        }

        c.blit_sprite("mekanism:button", 12.0, 106.0, 14.0, 14.0);
        c.blit("mekanism:gui/button/auto_eject", 12.0, 106.0, 0.0, 0.0, 14.0, 14.0, 14.0, 14.0);
        c.area(7402, [12.0, 106.0, 14.0, 14.0], AreaKind::Button, true, "Auto Eject");
        if inside(mouse, [12.0, 106.0, 14.0, 14.0]) {
            let tip = host::translate("logistical_sorter.mekanism.auto_eject.description", &[]);
            c.tooltip(&[(if tip.is_empty() { "Auto eject: automatically extracts items" } else { &tip }, WHITE)]);
        }

        // Filter holder at (95, 17, 158, 118)
        element_holder(c, [95.0, 17.0, 158.0, 118.0]);
        let total_filters = (data(2).max(0) as usize).min(4);
        for i in 0..4 {
            let row_rect = [96.0, 18.0 + 29.0 * i as f32, 156.0, 27.0];
            let is_sel = self.selected_row == Some(i);
            let sprite = if is_sel { "mekanism:button_focus" } else { "mekanism:button" };
            c.blit_sprite(sprite, row_rect[0], row_rect[1], row_rect[2], row_rect[3]);

            if i < total_filters {
                let desc = string(1 + i);
                c.text(if desc.is_empty() { "Filter" } else { &desc }, 100.0, 24.0 + 29.0 * i as f32, TITLE, 0, 0.85);
                c.area(7410 + i as i32, row_rect, AreaKind::Row, true, "Select filter");
            }
        }

        // New filter button holder at (95, 135, 158, 22)
        element_holder(c, [95.0, 135.0, 158.0, 22.0]);
        button(c, 7400, [96.0, 136.0, 100.0, 20.0], "New Filter", true, mouse, None);
        button(c, 7404, [200.0, 136.0, 52.0, 20.0], "Delete", self.selected_row.is_some(), mouse, None);

        // Title and inventory label (inv at x=58, y=172)
        title(c, &host::translate("block.mekanism.logistical_sorter", &[]), 0.0, 276.0, 6.0);
        c.draw_string(&host::translate("container.inventory", &[]), 58.0, 160.0, TITLE, false);

        // Chrome (security tab at x=276)
        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }
        if let InputEvent::MousePressed(p) = event {
            if p.button == MouseButton::Left {
                let mouse = (p.x, p.y);
                if inside(mouse, [12.0, 46.0, 14.0, 14.0]) {
                    host::send_button(7401);
                    host::play_click();
                    return true;
                }
                if inside(mouse, [12.0, 76.0, 14.0, 14.0]) {
                    host::send_button(7403);
                    host::play_click();
                    return true;
                }
                if inside(mouse, [12.0, 106.0, 14.0, 14.0]) {
                    host::send_button(7402);
                    host::play_click();
                    return true;
                }
                if inside(mouse, [96.0, 136.0, 100.0, 20.0]) {
                    host::send_button(7400);
                    host::play_click();
                    return true;
                }
                if self.selected_row.is_some() && inside(mouse, [200.0, 136.0, 52.0, 20.0]) {
                    host::send_button(7404);
                    host::play_click();
                    self.selected_row = None;
                    return true;
                }
                let total_filters = (data(2).max(0) as usize).min(4);
                for i in 0..total_filters {
                    let row_rect = [96.0, 18.0 + 29.0 * i as f32, 156.0, 27.0];
                    if inside(mouse, row_rect) {
                        self.selected_row = Some(i);
                        host::send_button(7410 + i as i32);
                        host::play_click();
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};

    #[test]
    fn logistical_sorter_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                1, // 0: auto eject
                0, // 1: single item
                2, // 2: filter count
                0, // 3: selected row
                0, // 4: security
            ],
            strings: vec![
                "Header".into(),
                "Filter: #c:ingots".into(),
                "Filter: minecraft:diamond".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:logistical_sorter".into(),
            gui_width: 276.0,
            gui_height: 254.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open logistical sorter");
        assert_eq!(setup.width, 276);
        assert_eq!(setup.height, 254);

        let frame = Frame { mouse_x: 10.0, mouse_y: 10.0, partial_tick: 0.0, time_ms: 1000 };
        let mut canvas = Canvas::new();
        screen.render(&frame, &mut canvas);
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text.contains("filter.mekanism.filter.count") || text.contains("Filters: 2"))));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Blit { texture, .. } if texture == "mekanism:gui/button/round_robin")));

        // Click Single Item button at (14, 48)
        let click_event = InputEvent::MousePressed(Pointer {
            x: 14.0,
            y: 48.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 7401".to_string(), "click".to_string()]);

        // Click Round Robin button at (14, 78)
        let click_event_rr = InputEvent::MousePressed(Pointer {
            x: 14.0,
            y: 78.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click_event_rr));
        let sent_rr = host::take_sent();
        assert_eq!(sent_rr, vec!["button 7403".to_string(), "click".to_string()]);
    }
}
