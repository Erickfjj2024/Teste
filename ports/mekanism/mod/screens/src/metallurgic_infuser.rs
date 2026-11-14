//! `GuiMetallurgicInfuser` (176 by 166): metallurgic infuser.
//! Java reference: `mekanism.client.gui.machine.GuiMetallurgicInfuser` (lines 19-45).
//!
//! Layout:
//! - Vertical power bar at (164, 15), 6 by 54 frame, 4 by 52 fill
//! - Chemical bar at (7, 15), 6 by 54 frame, 4 by 52 fill
//! - Dump button at (16, 59), 21 by 10 (id 4000)
//! - Progress bar at (72, 47), 32 by 8 (ProgressType.RIGHT)
//! - Energy tab at (-26, 137)
//! - Title centered at y = 6, inventory label at (8, 72)
//! - Slots: input at (51, 43), output at (109, 43), extra at (17, 35), power at (143, 35)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chemical_oxidizer::chemical_meta;
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, vertical_power_bar, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub const DUMP_BUTTON_ID: i32 = 4000;

pub struct MetallurgicInfuser {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:metallurgic_infuser";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(20),
            energy: Some(EnergyTabSpec::for_machine(20, Some(5))),
            upgrades: Some(UpgradesSpec::new(3, 4, 20, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(MetallurgicInfuser { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for MetallurgicInfuser {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input (51, 43), output (109, 43), extra (17, 35), power (143, 35)
        c.blit_sprite("mekanism:slot/input", 50.0, 42.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 108.0, 42.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/extra", 16.0, 34.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 142.0, 34.0, 18.0, 18.0);

        // Vertical power bar at (164, 15)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        vertical_power_bar(c, 165.0, 16.0, power_frac);

        let power_rect = [164.0, 15.0, 6.0, 54.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Chemical bar at (7, 15), frame 6 by 54, fill 4 by 52
        c.blit_sprite("mekanism:bar/base", 7.0, 15.0, 6.0, 54.0);
        let chem_amount = data(16).max(0) as u64;
        let chem_kind = data(17).max(0) as usize;
        let chem_frac = (chem_amount as f32 / 1000.0).clamp(0.0, 1.0);
        if let Some(m) = chemical_meta(chem_kind) {
            if chem_amount > 0 {
                let filled_h = (52.0 * chem_frac).floor();
                if filled_h > 0.0 {
                    c.fill(8.0, 16.0 + 52.0 - filled_h, 4.0, filled_h, m.tint);
                }
            }
        }
        let chem_rect = [7.0, 15.0, 6.0, 54.0];
        c.area(-1, chem_rect, AreaKind::Other, true, "Chemical");
        if inside(mouse, chem_rect) && hovered_tip.is_none() {
            if let Some(m) = chemical_meta(chem_kind) {
                if chem_amount > 0 {
                    hovered_tip = Some(vec![(format!("{}: {} mB", m.label, chem_amount), WHITE)]);
                } else {
                    hovered_tip = Some(vec![("Empty".to_string(), WHITE)]);
                }
            } else {
                hovered_tip = Some(vec![("Empty".to_string(), WHITE)]);
            }
        }

        // Dump button at (16, 59), 21 by 10
        c.blit_sprite("mekanism:button/dump", 16.0, 59.0, 21.0, 10.0);
        let dump_rect = [16.0, 59.0, 21.0, 10.0];
        c.area(DUMP_BUTTON_ID, dump_rect, AreaKind::Button, true, "Dump");
        if inside(mouse, dump_rect) {
            c.fill(16.0, 59.0, 21.0, 10.0, 0x33FF_FFFF);
            if hovered_tip.is_none() {
                hovered_tip = Some(vec![("Dump".to_string(), WHITE)]);
            }
        }

        // Progress bar at (72, 47), 32 by 8 (RIGHT)
        c.blit_sprite("mekanism:progress/right/empty", 72.0, 47.0, 32.0, 8.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(23).max(1) as f32;
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        if prog_frac > 0.0 {
            let filled_w = (32.0 * prog_frac).floor();
            c.enable_scissor(72.0, 47.0, filled_w, 8.0);
            c.blit_sprite("mekanism:progress/right/full", 72.0, 47.0, 32.0, 8.0);
            c.disable_scissor();
        }

        // Title centered at y = 6, Inventory at (8, 72)
        let title_text = host::translate("block.mekanism.metallurgic_infuser", &[]);
        title(c, &title_text, 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 72.0);

        if let Some(lines) = hovered_tip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }

        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        if self.chrome.input(event) {
            return true;
        }

        if let InputEvent::MousePressed(p) = event {
            if inside((p.x, p.y), [16.0, 59.0, 21.0, 10.0]) {
                host::send_button(DUMP_BUTTON_ID);
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
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};
    use horizon_mod_sdk::screen_format;

    #[test]
    fn test_metallurgic_infuser_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                50, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 500, 1, 0, 0, 0, 4000, 0, 200,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:metallurgic_infuser".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open metallurgic infuser");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let mut c = Canvas::new();
        screen.render(
            &Frame {
                mouse_x: 20.0,
                mouse_y: 62.0,
                partial_tick: 0.0,
                time_ms: 0,
            },
            &mut c,
        );
        let bytes = c.finish();
        let frame = screen_format::decode(&bytes).expect("decode frame");
        assert!(!frame.commands.is_empty());
    }

    #[test]
    fn test_metallurgic_infuser_dump_click() {
        host::set_menu(host::Menu {
            data: vec![0; 40],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:metallurgic_infuser".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open metallurgic infuser");

        let click = InputEvent::MousePressed(Pointer {
            x: 20.0,
            y: 62.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click));
        assert_eq!(host::take_sent(), vec!["button 4000", "click"]);
    }
}
