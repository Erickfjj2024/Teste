//! `GuiPigmentMixer` (176 by 166): pigment mixer machine.
//! Java reference: `mekanism.client.gui.machine.GuiPigmentMixer` (lines 1-135).
//! Container: `mekanism.common.inventory.container.tile.MekanismTileContainer<TileEntityPigmentMixer>`.
//! Tile: `mekanism.common.tile.machine.TileEntityPigmentMixer`.
//!
//! Layout:
//! - Horizontal power bar at (115, 75), 54 by 6 frame, 52 by 4 fill
//! - Left chemical input gauge at (25, 13), standard 16 by 58 (18 by 60 frame), capacity 1000 mB (MAX_INPUT_PIGMENT)
//! - Center chemical output gauge at (79, 4), standard 16 by 58 (18 by 60 frame), capacity 2000 mB (MAX_OUTPUT_PIGMENT)
//! - Right chemical input gauge at (133, 13), standard 16 by 58 (18 by 60 frame), capacity 1000 mB (MAX_INPUT_PIGMENT)
//! - Two progress arrows at (47, 39) (ProgressType.SMALL_RIGHT) and (101, 39) (ProgressType.SMALL_LEFT), colored with pigment colors
//! - Energy tab at (-26, 137)
//! - Title left-aligned at (4, 5) with offset for center gauge; inventory label at (8, 74)
//! - Slots: left input at (6, 56) with minus overlay, right input at (154, 56) with minus overlay,
//!   output at (80, 65) with plus overlay, power at (154, 14)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: speed (0), energy (1), muffling (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
//! - Progress arrows tint with left and right input pigment color representation from chemical metadata.
use crate::chemical_oxidizer::{chemical_meta, horizontal_power_bar, render_chemical_gauge};
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inside, pair, Overlay, TITLE, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct PigmentMixer {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:pigment_mixer";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

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
        Box::new(PigmentMixer { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for PigmentMixer {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: left input (6, 56), right input (154, 56), center output (80, 65), power (154, 14)
        c.blit_sprite("mekanism:slot/input", 5.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 5.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/input", 153.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 153.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 79.0, 64.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 79.0, 64.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 153.0, 13.0, 18.0, 18.0);

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

        // Left Chemical gauge at (25, 13), 16 by 58 inside 18 by 60 (t0, data 16, 17, cap 1000)
        let left_amt = data(16).max(0) as u64;
        let left_kind = data(17).max(0) as usize;
        let left_tip = render_chemical_gauge(
            c,
            mouse,
            25.0,
            13.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            left_amt,
            1000,
            left_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = left_tip;
        }

        // Center Output Chemical gauge at (79, 4), 16 by 58 inside 18 by 60 (t2, data 20, 21, cap 2000)
        let center_amt = data(20).max(0) as u64;
        let center_kind = data(21).max(0) as usize;
        let center_tip = render_chemical_gauge(
            c,
            mouse,
            79.0,
            4.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            center_amt,
            2000,
            center_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = center_tip;
        }

        // Right Chemical gauge at (133, 13), 16 by 58 inside 18 by 60 (t1, data 18, 19, cap 1000)
        let right_amt = data(18).max(0) as u64;
        let right_kind = data(19).max(0) as usize;
        let right_tip = render_chemical_gauge(
            c,
            mouse,
            133.0,
            13.0,
            16.0,
            58.0,
            "mekanism:gauge/overlay/standard",
            right_amt,
            1000,
            right_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = right_tip;
        }

        // Two small progress arrows at (47, 39) and (101, 39)
        c.blit_sprite("mekanism:progress/small_right/empty", 47.0, 39.0, 28.0, 8.0);
        c.blit_sprite("mekanism:progress/small_left/empty", 101.0, 39.0, 28.0, 8.0);
        c.area(-1, [47.0, 39.0, 28.0, 8.0], AreaKind::Other, true, "Progress");
        c.area(-1, [101.0, 39.0, 28.0, 8.0], AreaKind::Other, true, "Progress");
        let active = data(0) != 0;
        if active {
            let left_tint = chemical_meta(left_kind).map_or(WHITE, |m| m.tint);
            let right_tint = chemical_meta(right_kind).map_or(WHITE, |m| m.tint);
            c.blit_sprite_tinted("mekanism:progress/small_right/full", [47.0, 39.0, 28.0, 8.0], left_tint);
            c.blit_sprite_tinted("mekanism:progress/small_left/full", [101.0, 39.0, 28.0, 8.0], right_tint);
        }

        // Title left-aligned at (1, 5) to avoid center gauge
        let title_text = host::translate("block.mekanism.pigment_mixer", &[]);
        c.draw_string(&title_text, 1.0, 5.0, TITLE, false);

        // Inventory label at (8, 74)
        c.draw_string(&host::translate("container.inventory", &[]), 8.0, 74.0, TITLE, false);

        // Tooltip if hovered
        if let Some(lines) = hovered_tip {
            let refs: Vec<(&str, u32)> = lines.iter().map(|(s, col)| (s.as_str(), *col)).collect();
            c.tooltip(&refs);
        }

        // Chrome (tabs and overlays)
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
    use horizon_mod_sdk::screen_format::{self, Command};

    #[test]
    fn test_pigment_mixer_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                1, 500, 0, 2000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 500, 1, 800, 2, 1000, 3, 0, 0, 0, 100, 0, 50,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:pigment_mixer".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open pigment mixer");

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
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:base")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:progress/small_right/full")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:progress/small_left/full")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && *rect == [115.0, 75.0, 54.0, 6.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Area { label, rect, .. } if label == "Energy" && *rect == [115.0, 75.0, 54.0, 6.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Text { x, y, .. } if *x == 1.0 && *y == 5.0)));
    }

    #[test]
    fn test_pigment_mixer_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:pigment_mixer".into(),
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
