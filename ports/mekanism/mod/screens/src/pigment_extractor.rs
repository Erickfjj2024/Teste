//! `GuiPigmentExtractor` (176 by 166): pigment extractor.
//! Java reference: `mekanism.client.gui.machine.GuiPigmentExtractor` (lines 25-108).
//! Container: `mekanism.common.inventory.container.tile.MekanismTileContainer<TileEntityPigmentExtractor>`.
//! Tile: `mekanism.common.tile.machine.TileEntityPigmentExtractor`.
//!
//! Layout:
//! - Horizontal power bar at (115, 75), 54 by 6 frame, 52 by 4 fill
//! - Output pigment chemical gauge at (131, 13), standard 16 by 58 (18 by 60 frame), capacity 20000 mB
//! - Progress bar at (64, 40), 48 by 8 (ProgressType.LARGE_RIGHT), colored with pigment output color
//! - Energy tab at (-26, 137)
//! - Title centered at y = 5, inventory label at (8, 72)
//! - Slots: input at (26, 36), extra at (152, 55) (plus), power at (152, 14)
//! - Upgrade window: slots 2 (in) and 3 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
//! - Progress arrow tint uses the output pigment color representation from chemical metadata.
use crate::chemical_oxidizer::{chemical_meta, horizontal_power_bar, render_chemical_gauge};
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct PigmentExtractor {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:pigment_extractor";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(20),
            energy: Some(EnergyTabSpec::for_machine(20, Some(5))),
            upgrades: Some(UpgradesSpec::new(2, 3, 20, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(PigmentExtractor { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for PigmentExtractor {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input at (26, 36), extra at (152, 55), power at (152, 14)
        c.blit_sprite("mekanism:slot/input", 25.0, 35.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/extra", 151.0, 54.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 151.0, 54.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 151.0, 13.0, 18.0, 18.0);

        // Horizontal power bar at (115, 75)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        horizontal_power_bar(c, 115.0, 75.0, power_frac);

        let power_rect = [114.0, 74.0, 54.0, 6.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Chemical output gauge at (131, 13), 16 by 58 inside 18 by 60
        let chem_amount = data(18).max(0) as u64;
        let chem_kind = data(19).max(0) as usize;
        let gauge_tip = render_chemical_gauge(c, mouse, 131.0, 13.0, 16.0, 58.0, "mekanism:gauge/overlay/standard", chem_amount, 20000, chem_kind);
        if hovered_tip.is_none() {
            hovered_tip = gauge_tip;
        }

        // Progress bar at (64, 40): 48 by 8 (LARGE_RIGHT), colored with pigment
        c.blit_sprite("mekanism:progress/large_right/empty", 64.0, 40.0, 48.0, 8.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(23).max(1) as f32;
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        if prog_frac > 0.0 {
            let filled_w = (48.0 * prog_frac).floor();
            let tint = chemical_meta(chem_kind).map_or(WHITE, |m| m.tint);
            c.enable_scissor(64.0, 40.0, filled_w, 8.0);
            c.blit_sprite_tinted("mekanism:progress/large_right/full", [64.0, 40.0, 48.0, 8.0], tint);
            c.disable_scissor();
        }

        // Title centered at y = 5
        let title_text = host::translate("block.mekanism.pigment_extractor", &[]);
        title(c, &title_text, 0.0, 176.0, 5.0);

        // Inventory label at (8, 72)
        inventory_label(c, 8.0, 72.0);

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
    fn test_pigment_extractor_render() {
        host::set_menu(host::Menu {
            data: vec![50, 500, 0, 1000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5000, 25, 0, 0, 0, 100],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:pigment_extractor".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open pigment extractor");

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
    fn test_pigment_extractor_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:pigment_extractor".into(),
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
