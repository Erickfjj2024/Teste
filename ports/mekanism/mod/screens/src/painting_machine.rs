//! `GuiPaintingMachine` (176 by 166): painting machine.
//! Java reference: `mekanism.client.gui.machine.GuiPaintingMachine` (lines 20-68).
//! Container: `mekanism.common.inventory.container.tile.MekanismTileContainer<TileEntityPaintingMachine>`.
//! Tile: `mekanism.common.tile.machine.TileEntityPaintingMachine`.
//!
//! Layout:
//! - Vertical power bar at (164, 15), 6 by 54 frame, 4 by 52 fill
//! - Input pigment chemical gauge at (25, 13), standard 16 by 58 (18 by 60 frame), capacity 15000 mB
//! - Progress bar at (64, 39), 48 by 8 (ProgressType.LARGE_RIGHT), colored with pigment color
//! - Energy tab at (-26, 137)
//! - Title centered at y = 4, inventory label at (8, 76)
//! - Slots: input at (45, 35), output at (116, 35), pigment input at (6, 56) (minus), power at (144, 35)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
//! - Progress arrow tint uses the input pigment color representation from chemical metadata.
use crate::chemical_oxidizer::{chemical_meta, render_chemical_gauge};
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, vertical_power_bar, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct PaintingMachine {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:painting_machine";
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
        Box::new(PaintingMachine { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for PaintingMachine {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input at (45, 35), output at (116, 35), extra at (6, 56), power at (144, 35)
        c.blit_sprite("mekanism:slot/input", 44.0, 34.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 115.0, 34.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/extra", 5.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 5.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 143.0, 34.0, 18.0, 18.0);

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

        // Chemical input gauge at (25, 13), 16 by 58 inside 18 by 60
        let chem_amount = data(16).max(0) as u64;
        let chem_kind = data(17).max(0) as usize;
        let gauge_tip = render_chemical_gauge(c, mouse, 25.0, 13.0, 16.0, 58.0, "mekanism:gauge/overlay/standard", chem_amount, 15000, chem_kind);
        if hovered_tip.is_none() {
            hovered_tip = gauge_tip;
        }

        // Progress bar at (64, 39): 48 by 8 (LARGE_RIGHT), colored with pigment
        c.blit_sprite("mekanism:progress/large_right/empty", 64.0, 39.0, 48.0, 8.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(23).max(1) as f32;
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        if prog_frac > 0.0 {
            let filled_w = (48.0 * prog_frac).floor();
            let tint = chemical_meta(chem_kind).map_or(WHITE, |m| m.tint);
            c.enable_scissor(64.0, 39.0, filled_w, 8.0);
            c.blit_sprite_tinted("mekanism:progress/large_right/full", [64.0, 39.0, 48.0, 8.0], tint);
            c.disable_scissor();
        }

        // Title centered at y = 4
        let title_text = host::translate("block.mekanism.painting_machine", &[]);
        title(c, &title_text, 0.0, 176.0, 4.0);

        // Inventory label at (8, 76)
        inventory_label(c, 8.0, 76.0);

        // Tooltip if hovered
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
    use horizon_mod_sdk::screen::format as screen_format;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};

    #[test]
    fn test_painting_machine_render() {
        host::set_menu(host::Menu {
            data: vec![10, 500, 0, 1000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3000, 20, 0, 0, 0, 0, 0, 100],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:painting_machine".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open painting machine");

        assert_eq!(setup.width, 176);
        assert_eq!(setup.height, 166);

        let mut c = Canvas::new();
        screen.render(
            &Frame {
                mouse_x: 10.0,
                mouse_y: 10.0,
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
    fn test_painting_machine_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:painting_machine".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open");

        let event = InputEvent::MousePressed(Pointer {
            x: 180.0,
            y: 140.0,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
        });
        let handled = screen.input(&event);
        assert!(handled);
        let sent = host::take_sent();
        assert_eq!(sent, vec!["button 6000", "click"]);
    }
}
