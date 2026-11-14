//! `GuiQIOItemViewer`: The base view for QIO dashboards displaying the frequency's item catalogue.
//!
//! Java reference:
//! - `mekanism.client.gui.qio.GuiQIOItemViewer` (lines 1-229)
//! - `mekanism.client.gui.element.scroll.GuiSlotScroll` (lines 1-214)
//! - `mekanism.client.gui.element.scroll.GuiScrollBar` (lines 1-100)
//! - `mekanism.common.inventory.container.QIOItemViewerContainer`
//!
//! Layout and dimensions:
//! - Main panel: 178 by 211 pixels (`imageWidth = 16 + 8 * 18 + 18 = 178`, `imageHeight = 43 + 4 * 18 + 96 = 211`)
//! - Frequency inner screen: (7, 15, 162, 12)
//! - Search label: x = 46, y = 31 (right aligned)
//! - Search field: (50, 28, 118, 12), text field ID 8562 (`SEARCH_BOX`), background `element_holder`
//! - Item grid: (7, 43, 144, 72), 8 columns by 4 rows of 18x18 slots
//! - Scrollbar: (155, 43, 14, 72)
//! - Sort controls: at y = 118, sort label "Sort:" at x = 110 (right-aligned),
//!   type button at (115, 116, 41, 12) (ID 8560: Name, Count, Mod, Registry),
//!   direction button at (157, 116, 12, 12) (ID 8561: Ascending / Descending)
//! - Inventory label: at (8, 118) (`imageHeight - 93 = 118`)
//!
//! Differences from Java / Fidelity notes:
//! - // NÃO FEITO: controles de redimensionamento dinâmico (GuiResizeControls) - o tamanho da grade é fixo em 8x4 pelo protocolo do servidor.
//! - // NÃO FEITO: abas de configuração do cliente para auto-focus da barra de busca e destino de rejeitos do JEI/EMI - são opções puramente locais do cliente Java sem suporte no protocolo.
//! - // NÃO FEITO: aba de direção de transferência de receitas (GuiTargetDirectionTab) - visualizador de receitas não integrado na tela WebAssembly.
//! - // NÃO FEITO: pausa de ordenação ao segurar Shift - a ordenação é feita pelo servidor em qio_world.rs.
use crate::elements::{
    button, data, element_holder, inner_screen, inside, inventory_label, slot_frame, string,
    GREEN, TITLE, WHITE,
};
use horizon_mod_sdk::screen::widgets::ScrollBar;
use horizon_mod_sdk::screen::{
    AreaKind, Canvas, FieldFilter, InputEvent, MouseButton, host,
};

pub const SEARCH_BOX: i32 = 8562;
pub const SORT_TYPE: i32 = 8560;
pub const SORT_DIRECTION: i32 = 8561;

pub const GRID_COLUMNS: usize = 8;
pub const GRID_ROWS: usize = 4;
pub const GRID_X: f32 = 7.0;
pub const GRID_Y: f32 = 43.0;
pub const SCROLLBAR_X: f32 = 155.0;
pub const SCROLLBAR_Y: f32 = 43.0;
pub const SCROLLBAR_HEIGHT: f32 = 72.0;

/// Formats large item counts matching Java `UnitDisplayUtils.getDisplay(count, 1)`.
pub fn format_count(count: u64) -> String {
    if count < 10_000 {
        count.to_string()
    } else if count < 1_000_000 {
        let val = count as f64 / 1_000.0;
        format!("{val:.1}K")
    } else if count < 1_000_000_000 {
        let val = count as f64 / 1_000_000.0;
        format!("{val:.1}M")
    } else {
        let val = count as f64 / 1_000_000_000.0;
        format!("{val:.1}B")
    }
}

/// The reusable core item catalogue viewer of `GuiQIOItemViewer`.
pub struct QIOItemViewer {
    pub search_text: String,
    pub scroll_bar: ScrollBar,
    #[allow(dead_code)]
    pub gui_width: f32,
    #[allow(dead_code)]
    pub gui_height: f32,
}

impl QIOItemViewer {
    pub fn new(gui_width: f32, gui_height: f32) -> Self {
        QIOItemViewer {
            search_text: String::new(),
            scroll_bar: ScrollBar::new(SCROLLBAR_X as i32, SCROLLBAR_Y as i32, SCROLLBAR_HEIGHT as i32),
            gui_width,
            gui_height,
        }
    }

    /// Renders the viewer elements: frequency screen, search box, item grid, scrollbar, sort controls, and inventory label.
    pub fn render_viewer(&mut self, c: &mut Canvas, mouse: (f32, f32)) {
        // 1. Frequency inner screen (7, 15, 162, 12)
        inner_screen(c, [7.0, 15.0, 162.0, 12.0]);
        let has_freq = data(0) != 0;
        let freq_str = string(0);
        let header_text = if !has_freq || freq_str.is_empty() {
            host::translate("frequency.mekanism.none", &[])
        } else {
            freq_str
        };
        c.text(&header_text, 10.0, 17.0, GREEN, 0, 0.9);
        if inside(mouse, [7.0, 15.0, 162.0, 12.0]) {
            c.tooltip(&[(&header_text, WHITE)]);
        }

        // 2. Search label at (46, 31) right-aligned and field at (50, 28, 118, 12)
        let search_label = host::translate("qio.mekanism.list_search", &[]);
        let label_w = host::text_width(&search_label);
        c.draw_string(&search_label, 46.0 - label_w, 31.0, TITLE, false);

        element_holder(c, [50.0, 28.0, 118.0, 12.0]);
        c.field(SEARCH_BOX, [51.0, 29.0, 116.0, 10.0], 50, FieldFilter::Any, &self.search_text, 1, WHITE);

        // 3. Item grid (8 columns x 4 rows at (7, 43))
        let grid_items = host::menu_grid();
        let total_rows = (grid_items.len() + GRID_COLUMNS - 1) / GRID_COLUMNS;
        let start_row = self.scroll_bar.first(total_rows as i32, GRID_ROWS as i32) as usize;

        for r in 0..GRID_ROWS {
            for col in 0..GRID_COLUMNS {
                let cell_x = GRID_X + col as f32 * 18.0;
                let cell_y = GRID_Y + r as f32 * 18.0;
                slot_frame(c, cell_x, cell_y);

                let idx = (start_row + r) * GRID_COLUMNS + col;
                let cell_rect = [cell_x + 1.0, cell_y + 1.0, 16.0, 16.0];
                let hovered = inside(mouse, cell_rect);

                if hovered {
                    // Hover highlight overlay matching GuiSlot.DEFAULT_HOVER_COLOR (0x80FFFFFF)
                    c.fill(cell_rect[0], cell_rect[1], cell_rect[2], cell_rect[3], 0x80FF_FFFF);
                }

                if idx < grid_items.len() {
                    let entry = &grid_items[idx];
                    c.item(&entry.item, &entry.components, 1, cell_rect[0], cell_rect[1], 0, 1.0);

                    // Render count text
                    if entry.count == 0 {
                        c.text("0", cell_rect[0] + 16.0, cell_rect[1] + 9.0, 0xFFFF_D700, 4, 0.6);
                    } else if entry.count > 1 {
                        let count_str = format_count(entry.count);
                        c.text(&count_str, cell_rect[0] + 16.0, cell_rect[1] + 9.0, WHITE, 4, 0.6);
                    }

                    if hovered {
                        if entry.count >= 10_000 {
                            let detail = format!("Stored: {}", entry.count);
                            c.tooltip(&[(&entry.item, WHITE), (&detail, 0xFF78_7878)]);
                        } else {
                            c.tooltip(&[(&entry.item, WHITE)]);
                        }
                    }
                }
            }
        }

        // 4. Scrollbar at (155, 43, 14, 72)
        self.scroll_bar.draw(c, total_rows as i32, GRID_ROWS as i32);

        // 5. Sort controls at y = 118
        let sort_label = host::translate("qio.mekanism.list_sort", &[]);
        let sort_w = host::text_width(&sort_label);
        let sort_x = 115.0;
        let sort_y = 116.0;
        c.draw_string(&sort_label, sort_x - 5.0 - sort_w, 118.0, TITLE, false);

        // Sort type button (41 by 12)
        let sort_mode = data(3).clamp(0, 3) as usize;
        let sort_names = ["Name", "Count", "Mod", "Registry"];
        let sort_display = sort_names[sort_mode];
        button(
            c,
            SORT_TYPE,
            [sort_x, sort_y, 41.0, 12.0],
            sort_display,
            true,
            mouse,
            Some("Sort by name, size, mod or registry name"),
        );

        // Sort direction button (12 by 12 at (157, 116))
        let ascending = data(4) != 0;
        let dir_rect = [157.0, sort_y, 12.0, 12.0];
        let dir_hovered = inside(mouse, dir_rect);
        let dir_sprite = if dir_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(dir_sprite, dir_rect[0], dir_rect[1], dir_rect[2], dir_rect[3]);
        let icon_sprite = if ascending {
            "mekanism:button/arrow_up"
        } else {
            "mekanism:button/arrow_down"
        };
        c.blit_sprite(icon_sprite, dir_rect[0], dir_rect[1], dir_rect[2], dir_rect[3]);
        let dir_tip = if ascending { "Ascending" } else { "Descending" };
        c.area(SORT_DIRECTION, dir_rect, AreaKind::Button, true, dir_tip);
        if dir_hovered {
            c.tooltip(&[(dir_tip, WHITE)]);
        }

        // 6. Inventory label at (8, 118)
        inventory_label(c, 8.0, 118.0);
    }

    /// Handles input events destined for the core item viewer.
    pub fn input_viewer(&mut self, event: &InputEvent) -> bool {
        let grid_items = host::menu_grid();
        let total_rows = (grid_items.len() + GRID_COLUMNS - 1) / GRID_COLUMNS;
        let start_row = self.scroll_bar.first(total_rows as i32, GRID_ROWS as i32) as usize;

        match event {
            InputEvent::MousePressed(p) => {
                // Check sort buttons
                if inside((p.x, p.y), [115.0, 116.0, 41.0, 12.0]) {
                    host::send_button(SORT_TYPE);
                    host::play_click();
                    return true;
                }
                if inside((p.x, p.y), [157.0, 116.0, 12.0, 12.0]) {
                    host::send_button(SORT_DIRECTION);
                    host::play_click();
                    return true;
                }

                // Check scrollbar
                if inside((p.x, p.y), [SCROLLBAR_X, SCROLLBAR_Y, 14.0, SCROLLBAR_HEIGHT]) {
                    if self.scroll_bar.press(p.x as f64, p.y as f64, total_rows as i32, GRID_ROWS as i32) {
                        return true;
                    }
                }

                // Check item grid
                if inside((p.x, p.y), [GRID_X, GRID_Y, GRID_COLUMNS as f32 * 18.0, GRID_ROWS as f32 * 18.0]) {
                    let col = ((p.x - GRID_X) / 18.0).floor() as usize;
                    let row = ((p.y - GRID_Y) / 18.0).floor() as usize;
                    if col < GRID_COLUMNS && row < GRID_ROWS {
                        let idx = (start_row + row) * GRID_COLUMNS + col;
                        let button_code: u8 = match p.button {
                            MouseButton::Left => 0,
                            MouseButton::Right => 1,
                            MouseButton::Middle => 2,
                        };
                        if idx < grid_items.len() {
                            host::send_grid_click(idx as u32, button_code, p.modifiers.shift);
                        } else {
                            host::send_grid_click(u32::MAX, button_code, p.modifiers.shift);
                        }
                        host::play_click();
                        return true;
                    }
                }
                false
            }
            InputEvent::MouseDragged(d) => {
                self.scroll_bar.drag(d.y as f64, total_rows as i32, GRID_ROWS as i32);
                false
            }
            InputEvent::MouseReleased(_) => {
                self.scroll_bar.release();
                false
            }
            InputEvent::MouseScrolled(s) => {
                if inside((s.x, s.y), [GRID_X, GRID_Y, (GRID_COLUMNS as f32 * 18.0) + 14.0, GRID_ROWS as f32 * 18.0]) {
                    if self.scroll_bar.wheel(s.delta as f64, total_rows as i32, GRID_ROWS as i32) {
                        return true;
                    }
                }
                false
            }
            InputEvent::FieldChanged(f) if f.id == SEARCH_BOX => {
                self.search_text = f.text.clone();
                true
            }
            InputEvent::FieldSubmitted(f) if f.id == SEARCH_BOX => {
                self.search_text = f.text.clone();
                host::send_text(SEARCH_BOX, &f.text);
                true
            }
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
    fn item_viewer_renders_grid_and_handles_clicks() {
        host::set_menu(host::Menu {
            data: vec![1, 0, 0, 0, 1], // has_freq = 1, sort = 0 (Name), ascending = 1
            strings: vec!["Frequency: Main (500/2000)".into()],
            grid: vec![
                host::GridEntry {
                    item: "minecraft:iron_ingot".into(),
                    components: "{}".into(),
                    count: 128,
                },
                host::GridEntry {
                    item: "minecraft:gold_ingot".into(),
                    components: "{}".into(),
                    count: 15_400,
                },
            ],
            ..Default::default()
        });

        let mut viewer = QIOItemViewer::new(178.0, 211.0);
        let mut canvas = Canvas::new();
        viewer.render_viewer(&mut canvas, (50.0, 50.0));
        let bytes = canvas.finish();
        let decoded = horizon_mod_sdk::screen::format::decode(&bytes).expect("decode canvas");

        // Verify elements rendered
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text.contains("Frequency: Main"))));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Item { item, .. } if item == "minecraft:iron_ingot")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == SORT_TYPE)));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == SORT_DIRECTION)));

        // Click on first item in grid at (7 + 5, 43 + 5)
        let click_item = InputEvent::MousePressed(Pointer {
            x: 12.0,
            y: 48.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(viewer.input_viewer(&click_item));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["grid 0 0".to_string(), "click".to_string()]);

        // Click on sort type button at (120, 120)
        let click_sort = InputEvent::MousePressed(Pointer {
            x: 120.0,
            y: 120.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(viewer.input_viewer(&click_sort));
        let sent2 = host::take_sent();
        assert_eq!(sent2, vec!["button 8560".to_string(), "click".to_string()]);

        // Click on empty cell in grid at (7 + 18 * 4 + 5, 43 + 5)
        let click_empty = InputEvent::MousePressed(Pointer {
            x: 7.0 + 18.0 * 4.0 + 5.0,
            y: 48.0,
            button: MouseButton::Right,
            modifiers: Modifiers { shift: true, ..Default::default() },
        });
        assert!(viewer.input_viewer(&click_empty));
        let sent3 = host::take_sent();
        assert_eq!(sent3, vec![format!("grid {} 1 shift", u32::MAX), "click".to_string()]);
    }
}
