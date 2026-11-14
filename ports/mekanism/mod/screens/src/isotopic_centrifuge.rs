//! `GuiIsotopicCentrifuge` (176 by 166): isotopic centrifuge.
//! Java reference: `mekanism.client.gui.machine.GuiIsotopicCentrifuge` (lines 20-52).
//! Container: `mekanism.common.inventory.container.tile.MekanismTileContainer<TileEntityIsotopicCentrifuge>`.
//! Tile: `mekanism.common.tile.machine.TileEntityIsotopicCentrifuge`.
//!
//! Layout:
//! - Horizontal power bar at (115, 75), 54 by 6 frame, 52 by 4 fill
//! - Input chemical gauge at (25, 13), standard 16 by 58 (18 by 60 frame), capacity 10000 mB
//! - Output chemical gauge at (133, 13), standard 16 by 58 (18 by 60 frame), capacity 10000 mB
//! - Progress bar at (64, 39), 48 by 8 (ProgressType.LARGE_RIGHT)
//! - Energy tab at (-26, 137)
//! - Title centered at y = 4, inventory label at (8, 76)
//! - Slots: input at (5, 56) (minus), output at (155, 56) (plus), power at (155, 14)
//! - Upgrade window: slots 2 (in) and 3 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
use crate::chemical_oxidizer::{horizontal_power_bar, render_chemical_gauge};
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, inventory_label, pair, title, Overlay, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct IsotopicCentrifuge {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:isotopic_centrifuge";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(24),
            energy: Some(EnergyTabSpec::for_machine(24, Some(5))),
            upgrades: Some(UpgradesSpec::new(2, 3, 24, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(IsotopicCentrifuge { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for IsotopicCentrifuge {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input at (5, 56), output at (155, 56), power at (155, 14)
        c.blit_sprite("mekanism:slot/input", 4.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 4.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 154.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 154.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 154.0, 13.0, 18.0, 18.0);

        // Horizontal power bar at (115, 75)
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        let power_frac = if capacity == 0 { 0.0 } else { (energy as f32 / capacity as f32).clamp(0.0, 1.0) };
        horizontal_power_bar(c, 115.0, 75.0, power_frac);

        let power_rect = [115.0, 75.0, 54.0, 6.0];
        c.area(-1, power_rect, AreaKind::Other, true, "Energy");
        let mut hovered_tip: Option<Vec<(String, u32)>> = None;
        if inside(mouse, power_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Chemical input gauge at (25, 13), 16 by 58 inside 18 by 60
        let in_amount = data(16).max(0) as u64;
        let in_kind = data(17).max(0) as usize;
        let in_tip = render_chemical_gauge(c, mouse, 25.0, 13.0, 16.0, 58.0, "mekanism:gauge/overlay/standard", in_amount, 10000, in_kind);
        if hovered_tip.is_none() {
            hovered_tip = in_tip;
        }

        // Chemical output gauge at (133, 13), 16 by 58 inside 18 by 60
        let out_amount = data(18).max(0) as u64;
        let out_kind = data(19).max(0) as usize;
        let out_tip = render_chemical_gauge(c, mouse, 133.0, 13.0, 16.0, 58.0, "mekanism:gauge/overlay/standard", out_amount, 10000, out_kind);
        if hovered_tip.is_none() {
            hovered_tip = out_tip;
        }

        // Progress bar at (64, 39): 48 by 8 (LARGE_RIGHT)
        c.blit_sprite("mekanism:progress/large_right/empty", 64.0, 39.0, 48.0, 8.0);
        let active = data(0) > 0;
        if active {
            c.blit_sprite("mekanism:progress/large_right/full", 64.0, 39.0, 48.0, 8.0);
        }

        // Title centered at y = 4
        let title_text = host::translate("block.mekanism.isotopic_centrifuge", &[]);
        title(c, &title_text, 0.0, 176.0, 4.0);

        // Inventory label at (8, 74)
        inventory_label(c, 8.0, 74.0);

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
    use horizon_mod_sdk::screen_format::Command;
    use horizon_mod_sdk::screen::{Modifiers, MouseButton, Pointer};

    #[test]
    fn test_isotopic_centrifuge_render() {
        host::set_menu(host::Menu {
            data: vec![1, 500, 0, 1000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2000, 56, 1000, 75, 0, 0, 0, 0, 0],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:isotopic_centrifuge".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open isotopic centrifuge");

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
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && *rect == [115.0, 75.0, 54.0, 6.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Area { label, rect, .. } if label == "Energy" && *rect == [115.0, 75.0, 54.0, 6.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Text { y, .. } if *y == 74.0)));
    }

    #[test]
    fn test_isotopic_centrifuge_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:isotopic_centrifuge".into(),
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
