//! `GuiPRC` (176 by 171): pressurized reaction chamber.
//! Java reference: `mekanism.client.gui.machine.GuiPRC` (lines 20-45).
//!
//! Layout:
//! - Height 171 (imageHeight += 5), inventory label at y = 77 (inventoryLabelY += 5)
//! - Vertical power bar at (163, 21), 6 by 54 frame, 4 by 52 fill at (164, 22)
//! - Standard fluid gauge at (5, 15), 16 by 58 (input fluid tank, cap 10,000 mB)
//! - Standard chemical gauge at (28, 15), 16 by 58 (input gas tank, cap 10,000 mB)
//! - Small chemical gauge at (140, 45), 16 by 28 (output gas tank, cap 10,000 mB)
//! - Progress bar at (77, 43), 32 by 8 (ProgressType.RIGHT)
//! - Energy tab at (-26, 137)
//! - Title centered at y = 6, inventory label at (8, 77)
//! - Slots: input at (54, 40), output at (116, 40), power at (141, 22)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chemical_oxidizer::render_chemical_gauge;
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, vertical_power_bar, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct PRC {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:pressurized_reaction_chamber";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 171)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 171).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(24),
            energy: Some(EnergyTabSpec::for_machine(24, Some(5))),
            upgrades: Some(UpgradesSpec::new(3, 4, 24, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(PRC { chrome }),
        Setup {
            width: 176,
            height: 171,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for PRC {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 171.0);

        // Slots: input (54, 40), output (116, 40), power (141, 22)
        c.blit_sprite("mekanism:slot/input", 53.0, 39.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 115.0, 39.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 140.0, 21.0, 18.0, 18.0);

        // Vertical power bar at (163, 21)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        vertical_power_bar(c, 164.0, 22.0, power_frac);

        let power_rect = [163.0, 21.0, 6.0, 54.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Standard fluid gauge at (5, 15), 16 by 58 (t0, data 16, 17)
        let fluid_amt = data(16).max(0) as u64;
        let fluid_kind = data(17).max(0) as usize;
        let fluid_tip = render_chemical_gauge(c, mouse, 5.0, 15.0, 16.0, 58.0, "mekanism:gauge/overlay/standard", fluid_amt, 10000, fluid_kind);
        if hovered_tip.is_none() {
            hovered_tip = fluid_tip;
        }

        // Standard chemical gauge at (28, 15), 16 by 58 (t1, data 18, 19)
        let chem_in_amt = data(18).max(0) as u64;
        let chem_in_kind = data(19).max(0) as usize;
        let chem_in_tip = render_chemical_gauge(c, mouse, 28.0, 15.0, 16.0, 58.0, "mekanism:gauge/overlay/standard", chem_in_amt, 10000, chem_in_kind);
        if hovered_tip.is_none() {
            hovered_tip = chem_in_tip;
        }

        // Small chemical gauge at (140, 45), 16 by 28 (t2, data 20, 21)
        let chem_out_amt = data(20).max(0) as u64;
        let chem_out_kind = data(21).max(0) as usize;
        let chem_out_tip = render_chemical_gauge(c, mouse, 140.0, 45.0, 16.0, 28.0, "mekanism:gauge/overlay/small", chem_out_amt, 10000, chem_out_kind);
        if hovered_tip.is_none() {
            hovered_tip = chem_out_tip;
        }

        // Progress bar at (77, 43), 32 by 8 (RIGHT)
        c.blit_sprite("mekanism:progress/right/empty", 77.0, 43.0, 32.0, 8.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(27).max(1) as f32; // ctl0 + 3 = 24 + 3 = 27
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        if prog_frac > 0.0 {
            let filled_w = (32.0 * prog_frac).floor();
            c.enable_scissor(77.0, 43.0, filled_w, 8.0);
            c.blit_sprite("mekanism:progress/right/full", 77.0, 43.0, 32.0, 8.0);
            c.disable_scissor();
        }

        // Title centered at y = 6, Inventory at (8, 77)
        let title_text = host::translate("block.mekanism.pressurized_reaction_chamber", &[]);
        title(c, &title_text, 0.0, 176.0, 6.0);
        inventory_label(c, 8.0, 77.0);

        if let Some(lines) = hovered_tip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }

        self.chrome.render(c, mouse);
    }

    fn input(&mut self, event: &InputEvent) -> bool {
        self.chrome.input(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};
    use horizon_mod_sdk::screen_format;

    #[test]
    fn test_prc_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                25, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2000, 94, 1500, 4, 800, 13, 0, 0,
                0, 4000, 0, 100,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:pressurized_reaction_chamber".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 171.0,
        })
        .expect("open prc");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 171);

        let mut c = Canvas::new();
        screen.render(
            &Frame {
                mouse_x: 10.0,
                mouse_y: 20.0,
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
    fn test_prc_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 45],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:pressurized_reaction_chamber".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 171.0,
        })
        .expect("open prc");

        let click = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 142.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        assert!(screen.input(&click));
        assert_eq!(host::take_sent(), vec!["button 6000", "click"]);
    }
}
