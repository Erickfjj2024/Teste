//! `GuiModuleTweaker` (`mekanism:module_tweaker`, 266 by 186): 14 virtual gear slots (4 armor at (8, 8), 9 hotbar from
//! (58, 161), and off-hand at (8, 88)), the scrollable list of installed modules at (30, 20, 116 by 116), the module
//! settings display at (150, 20, 108 by 134), and the element holder at (30, 136, 120 by 18) for helmet options.
//! Java: `GuiModuleTweaker.java` (lines 42-235), `ModuleTweakerContainer.java`.
//! Server data (`module_tweaker.rs`):
//! - `data[0]`: picked slot (0..13)
//! - `data[1]`: picked module row (-1 none)
//! - `data[2]`: how many modules the picked item holds
//! - `data[3..17]`: whether each of the 14 slots holds a tweakable item (1 or 0)
//! Strings: 0..15 module rows; 16 + 2*i and 17 + 2*i are label and value for setting row `i` (0..7).
//!
//! Differences and omissions from Java:
// NÃO FEITO: a janela de cor ARGB (Color Modulation Unit) requer ColorPicker que não existe na API do SDK; o servidor alterna entre 16 cores de corantes no clique.
// NÃO FEITO: o botão de opções do capacete MekaSuit (GuiMekaSuitHelmetOptions) não tem suporte no servidor nem janela no SDK.
// NÃO FEITO: navegação por setas do teclado (GLFW_KEY_UP/DOWN/LEFT/RIGHT) para alternar slots selecionados.
// NÃO FEITO: pré-visualização da armadura 3D (ArmorPreview / ArmorStand) requer renderizador de entidade 3D inexistente na API do Canvas.
use crate::elements::{GREEN, WHITE, base_panel, data, element_holder, inner_screen, inside, slot_frame, string, title};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const SLOT_BASE: i32 = 9500;
pub const LIST_BASE: i32 = 9520;
pub const CONFIG_BASE: i32 = 9540;

pub const GEAR_SLOTS: [(f32, f32); 14] = [
    (8.0, 8.0),
    (8.0, 26.0),
    (8.0, 44.0),
    (8.0, 62.0),
    (58.0, 161.0),
    (76.0, 161.0),
    (94.0, 161.0),
    (112.0, 161.0),
    (130.0, 161.0),
    (148.0, 161.0),
    (166.0, 161.0),
    (184.0, 161.0),
    (202.0, 161.0),
    (8.0, 88.0),
];

pub struct ModuleTweaker;

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(ModuleTweaker),
        Setup {
            width: 266,
            height: 186,
            hide_inventory: true,
            own_labels: true,
        },
    ))
}

impl MekScreen for ModuleTweaker {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 266.0, 186.0);

        // Title at y = 4
        title(c, &host::translate("gui.mekanism.module_tweaker", &[]), 24.0, 266.0 - 24.0, 4.0);

        // 14 Gear Slots
        let picked_slot = data(0);
        for (i, &(x, y)) in GEAR_SLOTS.iter().enumerate() {
            slot_frame(c, x - 1.0, y - 1.0);
            let tweakable = data(3 + i) != 0;
            if !tweakable {
                // Dim overlay for non-tweakable items
                c.fill(x, y, 16.0, 16.0, 0xCC33_3333);
            }
            if picked_slot == i as i32 {
                // White selection border around the selected slot
                c.blit_sprite("mekanism:slot/overlay/select", x - 1.0, y - 1.0, 18.0, 18.0);
            }
            c.area(
                SLOT_BASE + i as i32,
                [x - 1.0, y - 1.0, 18.0, 18.0],
                AreaKind::Button,
                tweakable,
                "Slot",
            );
        }

        // Module list in holder at (30, 20, 116, 116)
        inner_screen(c, [30.0, 20.0, 116.0, 116.0]);
        let module_count = data(2).clamp(0, 16) as usize;
        let picked_module = data(1);
        for row in 0..module_count {
            let y = 22.0 + 11.0 * row as f32;
            let text = string(row);
            if picked_module == row as i32 {
                c.fill(31.0, y - 1.0, 114.0, 11.0, 0x403C_FE9A);
            }
            c.text(&text, 33.0, y, GREEN, 0, 0.85);
            c.area(
                LIST_BASE + row as i32,
                [31.0, y - 1.0, 114.0, 11.0],
                AreaKind::Row,
                true,
                &text,
            );
        }

        // Module settings display at (150, 20, 108, 134)
        inner_screen(c, [150.0, 20.0, 108.0, 134.0]);
        for i in 0..8 {
            let label = string(16 + 2 * i);
            if !label.is_empty() {
                let value = string(17 + 2 * i);
                let y = 22.0 + 18.0 * i as f32;
                c.text(&label, 154.0, y, WHITE, 0, 0.8);
                c.text(&value, 154.0, y + 8.0, GREEN, 0, 0.8);
                c.area(
                    CONFIG_BASE + i as i32,
                    [152.0, y - 1.0, 104.0, 17.0],
                    AreaKind::Button,
                    true,
                    &label,
                );
            }
        }

        // Element holder and Options button at (30, 136, 120, 18)
        element_holder(c, [30.0, 136.0, 120.0, 18.0]);
        let opt_rect = [31.0, 137.0, 118.0, 16.0];
        crate::elements::button(c, -1, opt_rect, "Options", false, mouse, None);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left {
                    // Clicks on gear slots
                    for (i, &(x, y)) in GEAR_SLOTS.iter().enumerate() {
                        if inside((p.x, p.y), [x - 1.0, y - 1.0, 18.0, 18.0]) {
                            if data(3 + i) != 0 {
                                host::send_button(SLOT_BASE + i as i32);
                                host::play_click();
                                return true;
                            }
                        }
                    }
                    // Clicks on module rows
                    let module_count = data(2).clamp(0, 16) as usize;
                    for row in 0..module_count {
                        let y = 22.0 + 11.0 * row as f32;
                        if inside((p.x, p.y), [31.0, y - 1.0, 114.0, 11.0]) {
                            host::send_button(LIST_BASE + row as i32);
                            host::play_click();
                            return true;
                        }
                    }
                    // Clicks on config setting rows
                    for i in 0..8 {
                        let label = string(16 + 2 * i);
                        if !label.is_empty() {
                            let y = 22.0 + 18.0 * i as f32;
                            if inside((p.x, p.y), [152.0, y - 1.0, 104.0, 17.0]) {
                                host::send_button(CONFIG_BASE + i as i32);
                                host::play_click();
                                return true;
                            }
                        }
                    }
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
    use horizon_mod_sdk::screen::{Modifiers, Pointer};

    #[test]
    fn module_tweaker_renders_and_handles_buttons() {
        let mut d = vec![0; 17];
        d[0] = 0; // picked slot 0
        d[1] = 0; // picked module 0
        d[2] = 1; // 1 module
        d[3] = 1; // slot 0 is tweakable
        let mut s = vec![String::new(); 32];
        s[0] = "Jetpack Unit (1/1)".into();
        s[16] = "Enabled".into();
        s[17] = "On".into();
        host::set_menu(host::Menu {
            data: d,
            strings: s,
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:module_tweaker".into(),
            gui_width: 266.0,
            gui_height: 186.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open module_tweaker");
        assert_eq!(setup.width, 266);
        assert_eq!(setup.height, 186);
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
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Jetpack Unit (1/1)")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Enabled")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "On")));

        // Click slot 0
        let slot_click = InputEvent::MousePressed(Pointer {
            x: 10.0,
            y: 10.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&slot_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {SLOT_BASE}"), "click".to_string()]);

        // Click module 0
        let mod_click = InputEvent::MousePressed(Pointer {
            x: 50.0,
            y: 25.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&mod_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {LIST_BASE}"), "click".to_string()]);

        // Click config row 0
        let cfg_click = InputEvent::MousePressed(Pointer {
            x: 160.0,
            y: 25.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&cfg_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {CONFIG_BASE}"), "click".to_string()]);
    }
}
