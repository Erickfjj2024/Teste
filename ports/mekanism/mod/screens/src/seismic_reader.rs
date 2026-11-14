//! `GuiSeismicReader` (`mekanism:seismic_reader`, 150 by 182): the title at (0, 3), the inner screen at (5, 11, 69 by 50)
//! displaying the inspected block's name and abundancy in the column, the column's inner screen at (77, 11, 51 by 160)
//! with the selection arrow at (79, 81) and height display beside it, nine virtual slots going up from (95, 146),
//! the scroll bar track at (129, 25, 14 by 132), and the up/down buttons at (129, 11) and (129, 157).
//! Java: `GuiSeismicReader.java` (lines 53-206), `SeismicReaderContainer.java`.
//! Server data (`seismic_reader.rs`): `data[0]` is can_up (scroll > 0), `data[1]` is can_down (scroll + 1 < blocks).
//! Strings: 0 block name, 1 abundancy, 2 layer height.
//!
//! Differences from Java:
//! - In Java, the client receives the entire vertical column and renders fluid sprites / 3D block projections.
//!   In the port, the server sends the 9 visible layer items to virtual slots 0..8.
//! - The scroll bar track is displayed between the buttons, but continuous thumb dragging is not supported by the
//!   server protocol, which steps layer-by-layer via button IDs 8401 (up) and 8402 (down).
// NÃO FEITO: a barra de rolagem contínua com arraste do polegar (GuiScrollBar) não é suportada pelo servidor, que só aceita botões 8401 e 8402.
// NÃO FEITO: texturas e animações de fluidos dinâmicos diretamente do nível (renderizadas como baldes pelo servidor).
use crate::elements::{GREEN, base_panel, data, element_holder, inner_screen, inside, string, title};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const BUTTON_UP: i32 = 8401;
pub const BUTTON_DOWN: i32 = 8402;

pub struct SeismicReader;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(SeismicReader),
        Setup {
            width: 150,
            height: 182,
            hide_inventory: true,
            own_labels: true,
        },
    ))
}

impl MekScreen for SeismicReader {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 150.0, 182.0);

        // Title centered at y = 3
        title(c, &host::translate("item.mekanism.seismic_reader", &[]), 0.0, 150.0, 3.0);

        // Left inner screen (69 by 50 at 5, 11): block name and abundancy
        inner_screen(c, [5.0, 11.0, 69.0, 50.0]);
        let block_name = string(0);
        if !block_name.is_empty() {
            c.text(&block_name, 8.0, 14.0, GREEN, 0, 0.85);
        }
        let abundancy = string(1);
        if !abundancy.is_empty() {
            c.text(&abundancy, 8.0, 25.0, GREEN, 0, 0.85);
        }

        // Column inner screen (51 by 160 at 77, 11)
        inner_screen(c, [77.0, 11.0, 51.0, 160.0]);

        // Selection arrow at (79, 81)
        c.blit_sprite("mekanism:arrow_selection", 79.0, 81.0, 14.0, 14.0);

        // Layer height text beside the arrow at (112, 87)
        let height_str = string(2);
        if !height_str.is_empty() {
            c.text(&height_str, 112.0, 84.0, GREEN, 0, 0.85);
        }

        // Scrollbar track (element holder) at (129, 25, 14 by 132)
        element_holder(c, [129.0, 25.0, 14.0, 132.0]);

        // Up button at (129, 11, 14, 14)
        let can_up = data(0) != 0;
        let up_rect = [129.0, 11.0, 14.0, 14.0];
        let up_hovered = can_up && inside(mouse, up_rect);
        let up_sprite = if !can_up {
            "mekanism:button_inactive"
        } else if up_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(up_sprite, up_rect[0], up_rect[1], up_rect[2], up_rect[3]);
        c.blit("mekanism:gui/button/up", up_rect[0] + 1.0, up_rect[1] + 1.0, 0.0, 0.0, 12.0, 12.0, 12.0, 12.0);
        c.area(BUTTON_UP, up_rect, AreaKind::Button, can_up, "Up");
        if up_hovered {
            c.tooltip(&[("Up", crate::elements::WHITE)]);
        }

        // Down button at (129, 157, 14, 14)
        let can_down = data(1) != 0;
        let down_rect = [129.0, 157.0, 14.0, 14.0];
        let down_hovered = can_down && inside(mouse, down_rect);
        let down_sprite = if !can_down {
            "mekanism:button_inactive"
        } else if down_hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(down_sprite, down_rect[0], down_rect[1], down_rect[2], down_rect[3]);
        c.blit("mekanism:gui/button/down", down_rect[0] + 1.0, down_rect[1] + 1.0, 0.0, 0.0, 12.0, 12.0, 12.0, 12.0);
        c.area(BUTTON_DOWN, down_rect, AreaKind::Button, can_down, "Down");
        if down_hovered {
            c.tooltip(&[("Down", crate::elements::WHITE)]);
        }
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left {
                    if inside((p.x, p.y), [129.0, 11.0, 14.0, 14.0]) && data(0) != 0 {
                        host::send_button(BUTTON_UP);
                        host::play_click();
                        return true;
                    }
                    if inside((p.x, p.y), [129.0, 157.0, 14.0, 14.0]) && data(1) != 0 {
                        host::send_button(BUTTON_DOWN);
                        host::play_click();
                        return true;
                    }
                }
                false
            }
            InputEvent::MouseScrolled(w) => {
                if w.delta > 0.0 && data(0) != 0 {
                    host::send_button(BUTTON_UP);
                    host::play_click();
                    return true;
                }
                if w.delta < 0.0 && data(1) != 0 {
                    host::send_button(BUTTON_DOWN);
                    host::play_click();
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{Modifiers, Pointer, Wheel};

    #[test]
    fn seismic_reader_renders_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![1, 1], // can_up = true, can_down = true
            strings: vec![
                "Stone".into(),
                "Abundancy: 12".into(),
                "-60".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:seismic_reader".into(),
            gui_width: 150.0,
            gui_height: 182.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open seismic_reader");
        assert_eq!(setup.width, 150);
        assert_eq!(setup.height, 182);
        assert!(setup.hide_inventory);
        assert!(setup.own_labels);

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
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Stone")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Abundancy: 12")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "-60")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == BUTTON_UP)));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Area { id, .. } if *id == BUTTON_DOWN)));

        // Test click on Up button
        let up_click = InputEvent::MousePressed(Pointer {
            x: 130.0,
            y: 15.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&up_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {BUTTON_UP}"), "click".to_string()]);

        // Test mouse scroll down
        let scroll_event = InputEvent::MouseScrolled(Wheel {
            x: 100.0,
            y: 100.0,
            delta: -1.0,
        });
        assert!(screen.input(&scroll_event));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {BUTTON_DOWN}"), "click".to_string()]);
    }
}
