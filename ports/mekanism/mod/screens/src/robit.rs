//! `GuiRobit` (`mekanism:robit`, 176 by 166): the unified entity menu for the Robit, including the Main tab
//! (`GuiRobitMain`), and the window overlays for Crafting (`GuiRobitCrafting`), Inventory (`GuiRobitInventory`),
//! Smelting (`GuiRobitSmelting`), Repair (`GuiRobitRepair`), Rename (`GuiRobitRename`), and Skin selection (`GuiRobitSkinSelect`).
//! Java: `GuiRobit.java`, `GuiRobitMain.java`, `GuiRobitInventory.java`, `GuiRobitSmelting.java`, `GuiRobitCrafting.java`,
//! `GuiRobitRepair.java`, `GuiRobitRename.java`, `GuiRobitSkinSelect.java`.
//! Server data (`robit_world.rs`, `robit.rs`):
//! - `data[0]`: following (1 yes, 0 no)
//! - `data[1]`: drop pickup (1 yes, 0 no)
//! - `data[2]`: energy in FE (at most 40,000)
//! - `data[3]`: furnace smelting progress percentage (0..100)
//! Strings: 0 greeting ("Hi, <name>!"), 1 energy, 2 following, 3 pickup, 4 owner, 5 custom name.
//!
//! Slots:
//! - 0..26: Robit inventory (3x9)
//! - 27: energy slot (153, 17)
//! - 28..29: furnace input (51, 35) & output (116, 35)
//! - 30..38: crafting 3x3 grid
//! - 39: crafting result
//! - 40..41: repair inputs
//! - 42: repair result
//! - 43..78: player inventory (always active at inventory_y = 84)
//!
//! Differences from Java:
//! - In Java, each tab switch sends a network packet opening a separate Gui screen / container.
//!   In the port, all tabs and overlays operate within the single `mekanism:robit` menu, as designed in `convert.py::robit_menu`.
// NÃO FEITO: a aba de segurança (Security tab) do Robit não faz parte do menu do port.
// NÃO FEITO: a textura estilizada da bigorna vanilla (usado frame padrão de input/output).
use crate::elements::{GREEN, SUBHEADING, TITLE, WHITE, base_panel, data, inner_screen, inside, inventory_label, slot_frame, string, title};
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, FieldFilter, Frame, InputEvent, MouseButton, OpenInfo, Setup, host};

pub const BUTTON_FOLLOW: i32 = 1;
pub const BUTTON_DROP_PICKUP: i32 = 2;
pub const BUTTON_GO_HOME: i32 = 3;
pub const FIELD_RENAME: i32 = 10;
pub const FIELD_REPAIR: i32 = 11;
pub const SKIN_BASE: i32 = 20;

pub const SKINS: [&str; 13] = [
    "Robit", "Allay", "Pride", "Lesbian", "Trans", "Aro", "Ace", "Bi", "Enby", "Pan", "Gay",
    "Agender", "Genderfluid",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobitTab {
    Main,
    Crafting,
    Inventory,
    Smelting,
    Repair,
    Rename,
    Skin,
}

pub struct Robit {
    tab: RobitTab,
}

pub fn open(_info: &OpenInfo) -> Opened {
    Ok((
        Box::new(Robit { tab: RobitTab::Main }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for Robit {
    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Inventory label at (8, 74)
        inventory_label(c, 8.0, 74.0);

        // Side tab holder at (176, 6, 26, 106)
        c.blit_sprite("mekanism:holder_right", 176.0, 6.0, 26.0, 106.0);

        // 5 side tabs at x = 179, y = 10 + 20 * k
        let tabs = [
            (RobitTab::Main, "mekanism:gui/button/main", "Robit"),
            (RobitTab::Crafting, "mekanism:gui/button/crafting", "Crafting"),
            (RobitTab::Inventory, "mekanism:gui/button/main", "Inventory"),
            (RobitTab::Smelting, "mekanism:gui/button/smelting", "Smelting"),
            (RobitTab::Repair, "mekanism:gui/button/repair", "Repair"),
        ];
        for (k, (t, icon, label)) in tabs.into_iter().enumerate() {
            let y = 10.0 + 20.0 * k as f32;
            let is_cur = self.tab == t;
            let rect = [179.0, y, 18.0, 18.0];
            let hovered = inside(mouse, rect);
            let sprite = if is_cur {
                "mekanism:button_focus"
            } else if hovered {
                "mekanism:button_focus"
            } else {
                "mekanism:button"
            };
            c.blit_sprite(sprite, rect[0], rect[1], rect[2], rect[3]);
            c.blit(icon, rect[0] + 1.0, rect[1] + 1.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
            c.area(300 + k as i32, rect, AreaKind::Tab, !is_cur, label);
            if hovered {
                c.tooltip(&[(label, WHITE)]);
            }
        }

        // Draw active tab / window contents and configure slot visibilities
        match self.tab {
            RobitTab::Main => self.render_main(c, mouse),
            RobitTab::Crafting => self.render_crafting(c),
            RobitTab::Inventory => self.render_inventory(c),
            RobitTab::Smelting => self.render_smelting(c),
            RobitTab::Repair => self.render_repair(c),
            RobitTab::Rename => self.render_rename(c, mouse),
            RobitTab::Skin => self.render_skin(c, mouse),
        }

        // Configure slots visibility based on current active tab
        for i in 0..43 {
            let (x, y, active) = match self.tab {
                RobitTab::Main => {
                    if i == 27 {
                        (153.0, 17.0, true)
                    } else {
                        (0.0, 0.0, false)
                    }
                }
                RobitTab::Inventory => {
                    if i < 27 {
                        let col = (i % 9) as f32;
                        let row = (i / 9) as f32;
                        (8.0 + 18.0 * col, 18.0 + 18.0 * row, true)
                    } else {
                        (0.0, 0.0, false)
                    }
                }
                RobitTab::Smelting => {
                    if i == 28 {
                        (51.0, 35.0, true)
                    } else if i == 29 {
                        (116.0, 35.0, true)
                    } else {
                        (0.0, 0.0, false)
                    }
                }
                RobitTab::Crafting => {
                    if (30..=38).contains(&i) {
                        let c_idx = i - 30;
                        let col = (c_idx % 3) as f32;
                        let row = (c_idx / 3) as f32;
                        (30.0 + 18.0 * col, 17.0 + 18.0 * row, true)
                    } else if i == 39 {
                        (124.0, 35.0, true)
                    } else {
                        (0.0, 0.0, false)
                    }
                }
                RobitTab::Repair => {
                    if i == 40 {
                        (27.0, 47.0, true)
                    } else if i == 41 {
                        (76.0, 47.0, true)
                    } else if i == 42 {
                        (134.0, 47.0, true)
                    } else {
                        (0.0, 0.0, false)
                    }
                }
                RobitTab::Rename | RobitTab::Skin => (0.0, 0.0, false),
            };
            c.slot(i as u32, x, y, active);
        }

        // Render slots
        c.slots();
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::MousePressed(p) => {
                if p.button == MouseButton::Left {
                    let mouse = (p.x, p.y);

                    // Side tab clicks
                    for (k, (t, _, _)) in [
                        (RobitTab::Main, "main", "Robit"),
                        (RobitTab::Crafting, "crafting", "Crafting"),
                        (RobitTab::Inventory, "inventory", "Inventory"),
                        (RobitTab::Smelting, "smelting", "Smelting"),
                        (RobitTab::Repair, "repair", "Repair"),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let y = 10.0 + 20.0 * k as f32;
                        if inside(mouse, [179.0, y, 18.0, 18.0]) {
                            self.tab = t;
                            host::play_click();
                            return true;
                        }
                    }

                    // Main tab action buttons
                    if self.tab == RobitTab::Main {
                        if inside(mouse, [6.0, 16.0, 18.0, 18.0]) {
                            host::send_button(BUTTON_GO_HOME);
                            host::play_click();
                            return true;
                        }
                        if inside(mouse, [6.0, 35.0, 18.0, 18.0]) {
                            self.tab = RobitTab::Rename;
                            host::play_click();
                            return true;
                        }
                        if inside(mouse, [6.0, 54.0, 18.0, 18.0]) {
                            self.tab = RobitTab::Skin;
                            host::play_click();
                            return true;
                        }
                        if inside(mouse, [152.0, 35.0, 18.0, 18.0]) {
                            host::send_button(BUTTON_DROP_PICKUP);
                            host::play_click();
                            return true;
                        }
                        if inside(mouse, [152.0, 54.0, 18.0, 18.0]) {
                            host::send_button(BUTTON_FOLLOW);
                            host::play_click();
                            return true;
                        }
                    }

                    // Rename Back button
                    if self.tab == RobitTab::Rename && inside(mouse, [6.0, 6.0, 12.0, 12.0]) {
                        self.tab = RobitTab::Main;
                        host::play_click();
                        return true;
                    }

                    // Skin Back and select buttons
                    if self.tab == RobitTab::Skin {
                        if inside(mouse, [6.0, 6.0, 12.0, 12.0]) {
                            self.tab = RobitTab::Main;
                            host::play_click();
                            return true;
                        }
                        for (i, _) in SKINS.iter().enumerate() {
                            let col = (i % 3) as f32;
                            let row = (i / 3) as f32;
                            let bx = 8.0 + 56.0 * col;
                            let by = 18.0 + 12.0 * row;
                            if inside(mouse, [bx, by, 54.0, 11.0]) {
                                host::send_button(SKIN_BASE + i as i32);
                                host::play_click();
                                return true;
                            }
                        }
                    }
                }
                false
            }
            InputEvent::FieldSubmitted(f) => {
                if f.id == FIELD_RENAME && !f.text.trim().is_empty() {
                    host::send_text(FIELD_RENAME, f.text.trim());
                    self.tab = RobitTab::Main;
                    return true;
                }
                if f.id == FIELD_REPAIR {
                    host::send_text(FIELD_REPAIR, &f.text);
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

impl Robit {
    fn render_main(&self, c: &mut Canvas, mouse: (f32, f32)) {
        title(c, "Robit", 0.0, 176.0, 6.0);

        // Inner screen at (27, 16, 122, 56)
        inner_screen(c, [27.0, 16.0, 122.0, 56.0]);
        for i in 0..5 {
            let t = string(i);
            if !t.is_empty() {
                let y = if i == 0 { 20.0 } else { 20.0 + 9.0 * (i as f32 + 1.0) };
                c.text(&t, 29.0, y, GREEN, 0, 0.8);
            }
        }

        // Horizontal power bar at (26, 73, 122, 6)
        c.blit_sprite("mekanism:bar/base", 26.0, 73.0, 122.0, 6.0);
        let energy = data(2).clamp(0, 40_000) as f32;
        let frac = energy / 40_000.0;
        if frac > 0.0 {
            c.blit_sprite("mekanism:bar/horizontal_power", 27.0, 74.0, 120.0 * frac, 4.0);
        }

        // Energy slot at (153, 17)
        c.blit_sprite("mekanism:slot/power", 152.0, 16.0, 18.0, 18.0);

        // Action buttons
        let actions = [
            (6.0, 16.0, BUTTON_GO_HOME, "mekanism:gui/button/home", "Go home"),
            (6.0, 35.0, -1, "mekanism:gui/button/rename", "Rename"),
            (6.0, 54.0, -1, "mekanism:gui/button/skin", "Skin"),
            (152.0, 35.0, BUTTON_DROP_PICKUP, "mekanism:gui/button/drop", "Pick up items"),
            (152.0, 54.0, BUTTON_FOLLOW, "mekanism:gui/button/follow", "Follow"),
        ];
        for (x, y, bid, icon, tip) in actions {
            let rect = [x, y, 18.0, 18.0];
            let hovered = inside(mouse, rect);
            let sprite = if hovered {
                "mekanism:button_focus"
            } else {
                "mekanism:button"
            };
            c.blit_sprite(sprite, rect[0], rect[1], rect[2], rect[3]);
            c.blit(icon, rect[0] + 1.0, rect[1] + 1.0, 0.0, 0.0, 16.0, 16.0, 16.0, 16.0);
            c.area(bid, rect, AreaKind::Button, true, tip);
            if hovered {
                c.tooltip(&[(tip, WHITE)]);
            }
        }
    }

    fn render_crafting(&self, c: &mut Canvas) {
        c.blit_sprite("mekanism:base", 0.0, 0.0, 176.0, 84.0);
        c.text("Robit Crafting", 8.0, 6.0, TITLE, 0, 1.0);

        // 3x3 grid slot frames at (30 + 18*c, 17 + 18*r)
        for r in 0..3 {
            for col in 0..3 {
                slot_frame(c, 29.0 + 18.0 * col as f32, 16.0 + 18.0 * r as f32);
            }
        }

        // Progress arrow at (90, 35)
        c.blit_sprite("mekanism:progress/right/empty", 90.0, 35.0, 24.0, 16.0);

        // Result slot frame at (124, 35)
        c.blit_sprite("mekanism:slot/output", 123.0, 34.0, 18.0, 18.0);
    }

    fn render_inventory(&self, c: &mut Canvas) {
        c.blit_sprite("mekanism:base", 0.0, 0.0, 176.0, 84.0);
        c.text("Robit Inventory", 8.0, 6.0, TITLE, 0, 1.0);

        // 27 slot frames at (8 + 18*c, 18 + 18*r)
        for r in 0..3 {
            for col in 0..9 {
                slot_frame(c, 7.0 + 18.0 * col as f32, 17.0 + 18.0 * r as f32);
            }
        }
    }

    fn render_smelting(&self, c: &mut Canvas) {
        c.blit_sprite("mekanism:base", 0.0, 0.0, 176.0, 84.0);
        c.text("Robit Smelting", 8.0, 6.0, TITLE, 0, 1.0);

        // Input slot frame at (51, 35)
        c.blit_sprite("mekanism:slot/input", 50.0, 34.0, 18.0, 18.0);

        // Progress bar at (78, 38, 25, 9)
        c.blit_sprite("mekanism:progress/bar/empty", 78.0, 38.0, 25.0, 9.0);
        let progress = data(3).clamp(0, 100) as f32;
        if progress > 0.0 {
            c.blit_sprite("mekanism:progress/bar/full", 78.0, 38.0, 25.0 * (progress / 100.0), 9.0);
        }

        // Output slot frame at (116, 35)
        c.blit_sprite("mekanism:slot/output", 115.0, 34.0, 18.0, 18.0);
    }

    fn render_repair(&self, c: &mut Canvas) {
        c.blit_sprite("mekanism:base", 0.0, 0.0, 176.0, 84.0);
        c.text("Robit Repair", 8.0, 6.0, TITLE, 0, 1.0);

        c.text("Name:", 8.0, 24.0, TITLE, 0, 0.9);
        inner_screen(c, [60.0, 21.0, 103.0, 12.0]);
        c.field(FIELD_REPAIR, [61.0, 22.0, 101.0, 10.0], 50, FieldFilter::Any, "", 1, WHITE);

        // Input slots frames at (27, 47) and (76, 47)
        c.blit_sprite("mekanism:slot/input", 26.0, 46.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/input", 75.0, 46.0, 18.0, 18.0);

        // Output slot frame at (134, 47)
        c.blit_sprite("mekanism:slot/output", 133.0, 46.0, 18.0, 18.0);

        c.text("Only joining two damaged tools works here.", 8.0, 68.0, SUBHEADING, 0, 0.7);
    }

    fn render_rename(&self, c: &mut Canvas, mouse: (f32, f32)) {
        c.blit_sprite("mekanism:base", 0.0, 0.0, 176.0, 84.0);
        c.text("Rename the Robit", 24.0, 6.0, TITLE, 0, 1.0);

        // Back button at (6, 6, 12, 12)
        let back_rect = [6.0, 6.0, 12.0, 12.0];
        let hovered = inside(mouse, back_rect);
        let sprite = if hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(sprite, back_rect[0], back_rect[1], back_rect[2], back_rect[3]);
        c.blit("mekanism:gui/button/back", back_rect[0] + 1.0, back_rect[1] + 1.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0);
        c.area(-1, back_rect, AreaKind::Button, true, "Back");

        // Textfield at (29, 32, 118, 10)
        inner_screen(c, [27.0, 30.0, 122.0, 14.0]);
        let cur_name = string(5);
        c.field(FIELD_RENAME, [29.0, 32.0, 118.0, 10.0], 32, FieldFilter::Any, &cur_name, 1, WHITE);

        c.text("Type a name and confirm", 29.0, 50.0, SUBHEADING, 0, 0.8);
    }

    fn render_skin(&self, c: &mut Canvas, mouse: (f32, f32)) {
        c.blit_sprite("mekanism:base", 0.0, 0.0, 176.0, 84.0);
        c.text("Skin", 24.0, 6.0, TITLE, 0, 1.0);

        // Back button at (6, 6, 12, 12)
        let back_rect = [6.0, 6.0, 12.0, 12.0];
        let hovered = inside(mouse, back_rect);
        let sprite = if hovered {
            "mekanism:button_focus"
        } else {
            "mekanism:button"
        };
        c.blit_sprite(sprite, back_rect[0], back_rect[1], back_rect[2], back_rect[3]);
        c.blit("mekanism:gui/button/back", back_rect[0] + 1.0, back_rect[1] + 1.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0);
        c.area(-1, back_rect, AreaKind::Button, true, "Back");

        for (i, name) in SKINS.iter().enumerate() {
            let col = (i % 3) as f32;
            let row = (i / 3) as f32;
            let bx = 8.0 + 56.0 * col;
            let by = 18.0 + 12.0 * row;
            let rect = [bx, by, 54.0, 11.0];
            let b_hover = inside(mouse, rect);
            let b_sprite = if b_hover {
                "mekanism:button_focus"
            } else {
                "mekanism:button"
            };
            c.blit_sprite(b_sprite, rect[0], rect[1], rect[2], rect[3]);
            c.text(name, bx + 3.0, by + 2.0, TITLE, 0, 0.65);
            c.area(SKIN_BASE + i as i32, rect, AreaKind::Button, true, name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::format::Command;
    use horizon_mod_sdk::screen::{FieldText, Modifiers, Pointer};

    #[test]
    fn robit_renders_tabs_and_handles_buttons() {
        host::set_menu(host::Menu {
            data: vec![
                1,      // following
                1,      // pickup
                40_000, // energy
                50,     // smelting progress
            ],
            strings: vec![
                "Hi, Robit!".into(),
                "Energy: 40.00 kFE".into(),
                "Following: Yes".into(),
                "Drop Pickup: Yes".into(),
                "Owner: Steve".into(),
                "Robit".into(),
            ],
            ..Default::default()
        });

        let info = OpenInfo {
            menu: "mekanism:robit".into(),
            gui_width: 176.0,
            gui_height: 166.0,
            touch: false,
        };
        let (mut screen, setup) = open(&info).expect("open robit");
        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);
        assert!(!setup.hide_inventory);
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
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Hi, Robit!")));
        assert!(decoded.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Owner: Steve")));

        // Test click Go Home button (6, 16)
        let home_click = InputEvent::MousePressed(Pointer {
            x: 10.0,
            y: 20.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&home_click));
        let sent = host::take_sent();
        assert_eq!(sent, vec![format!("button {BUTTON_GO_HOME}"), "click".to_string()]);

        // Test click Rename button (6, 35) to switch tab
        let rename_tab_click = InputEvent::MousePressed(Pointer {
            x: 10.0,
            y: 40.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&rename_tab_click));

        // Submit rename textfield
        let rename_submit = InputEvent::FieldSubmitted(FieldText {
            id: FIELD_RENAME,
            text: "Bolt".into(),
        });
        assert!(screen.input(&rename_submit));
        let sent = host::take_sent();
        assert_eq!(sent, vec!["click".to_string(), format!("text {FIELD_RENAME} \"Bolt\"")]);

        // Test click Side Tab Crafting (179, 30)
        let crafting_tab_click = InputEvent::MousePressed(Pointer {
            x: 185.0,
            y: 35.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&crafting_tab_click));

        let mut canvas2 = Canvas::new();
        screen.render(&frame, &mut canvas2);
        let bytes2 = canvas2.finish();
        let decoded2 = horizon_mod_sdk::screen::format::decode(&bytes2).expect("decode canvas");
        assert!(decoded2.commands.iter().any(|cmd| matches!(cmd, Command::Text { text, .. } if text == "Robit Crafting")));
    }
}
