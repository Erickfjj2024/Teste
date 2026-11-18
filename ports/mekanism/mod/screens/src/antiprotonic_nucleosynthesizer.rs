//! `GuiAntiprotonicNucleosynthesizer` (196 by 193): antiprotonic nucleosynthesizer machine.
//! Java reference: `mekanism.client.gui.machine.GuiAntiprotonicNucleosynthesizer` (lines 1-99).
//! Container: `mekanism.common.inventory.container.tile.MekanismTileContainer<TileEntityAntiprotonicNucleosynthesizer>`.
//! Tile: `mekanism.common.tile.machine.TileEntityAntiprotonicNucleosynthesizer`.
//!
//! Layout:
//! - Window width 196, height 193 (inventory shifted: x = 18, y = 100)
//! - Antimatter gas gauge at (5, 18), small_med 16 by 46 (18 by 48 frame), capacity 10000 mB
//! - Energy gauge at (172, 18), small_med 16 by 46 (18 by 48 frame)
//! - Dynamic horizontal rate bar at (5, 88), 183 by 8
//! - Inner screen at (45, 18, 104, 68) with process rate display
//! - Slots: input at (26, 40), output at (152, 40), antimatter gas item at (6, 69) with minus overlay, power at (173, 69)
//! - Energy tab at (-26, 137)
//! - Title centered at y = 4, inventory label at (8, 100)
//! - Upgrade window: slots 3 (in) and 4 (out), supported: muffling only (3)
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
//! - // NÃO FEITO: relâmpagos entre slots (BoltRenderer) não suportados pela API de Canvas (2D).
use crate::chemical_oxidizer::render_chemical_gauge;
use crate::chrome::{Chrome, ChromeSpec, EnergyTabSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inner_screen, inside, inventory_label, pair, title, Overlay, GREEN, WHITE};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{AreaKind, Canvas, Frame, InputEvent, OpenInfo, Setup, TEXT_CENTER, host};

pub struct AntiprotonicNucleosynthesizer {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:antiprotonic_nucleosynthesizer";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 196, 193)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 196, 193).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(20),
            energy: Some(EnergyTabSpec::for_machine(20, Some(5))),
            upgrades: Some(UpgradesSpec::new(3, 4, 20, &[3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        196.0,
    );

    Ok((
        Box::new(AntiprotonicNucleosynthesizer { chrome }),
        Setup {
            width: 196,
            height: 193,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for AntiprotonicNucleosynthesizer {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 196.0, 193.0);

        // Slots: input (26, 40), output (152, 40), extra gas (6, 69), power (173, 69)
        c.blit_sprite("mekanism:slot/input", 25.0, 39.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 151.0, 39.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/extra", 5.0, 68.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 5.0, 68.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/power", 172.0, 68.0, 18.0, 18.0);

        let mut hovered_tip: Option<Vec<(String, u32)>> = None;

        // Antimatter chemical gauge at (5, 18), small_med 16 by 46 inside 18 by 48 frame (data 16, 17, cap 10000)
        let gas_amt = data(16).max(0) as u64;
        let gas_kind = data(17).max(0) as usize;
        let gas_tip = render_chemical_gauge(
            c,
            mouse,
            5.0,
            18.0,
            16.0,
            46.0,
            "mekanism:gauge/overlay/small_med",
            gas_amt,
            10000,
            gas_kind,
        );
        if hovered_tip.is_none() {
            hovered_tip = gas_tip;
        }

        // Energy gauge at (172, 18), small_med 16 by 46 inside 18 by 48 frame
        c.blit_sprite("mekanism:gauge/normal", 172.0, 18.0, 18.0, 48.0);
        let (energy, capacity) = (pair(1, 2), pair(3, 4));
        if capacity > 0 && energy > 0 {
            let fill = (energy as f32 / capacity as f32).clamp(0.0, 1.0);
            let fill_h = (46.0 * fill).floor();
            if fill_h > 0.0 {
                c.blit("mekanism:gui/energy_gauge_fill", 173.0, 19.0 + 46.0 - fill_h, 0.0, 46.0 - fill_h, 16.0, fill_h, 16.0, 46.0);
            }
        }
        c.blit_sprite("mekanism:gauge/overlay/small_med", 173.0, 19.0, 16.0, 46.0);
        let energy_rect = [172.0, 18.0, 18.0, 48.0];
        c.area(-1, energy_rect, AreaKind::Other, true, "Energy");
        if hovered_tip.is_none() && inside(mouse, energy_rect) {
            hovered_tip = Some(vec![(format!("Energy: {} / {}", energy, capacity), WHITE)]);
        }

        // Dynamic horizontal rate bar at (5, 88), 185 by 10 (base frame), fill 183 by 8
        c.blit_sprite("mekanism:bar/base", 5.0, 88.0, 185.0, 10.0);
        let progress = data(0).max(0) as f32;
        let ticks_needed = data(23).max(1) as f32;
        let prog_frac = (progress / ticks_needed).clamp(0.0, 1.0);
        let fill_w = (183.0 * prog_frac).floor();
        if fill_w > 0.0 {
            c.blit_sprite_tinted("mekanism:bar/dynamic_rate", [6.0, 89.0, fill_w, 8.0], 0xFF64_1EAA);
        }
        let rate_rect = [5.0, 88.0, 185.0, 10.0];
        let pct = (prog_frac * 100.0).round() as u32;
        c.area(-1, rate_rect, AreaKind::Other, true, "Progress");
        if hovered_tip.is_none() && inside(mouse, rate_rect) {
            hovered_tip = Some(vec![(format!("Progress: {}%", pct), WHITE)]);
        }

        // Inner screen at (45, 18, 104, 68)
        inner_screen(c, [45.0, 18.0, 104.0, 68.0]);
        let rate_str = if data(5) == 1 { "100%" } else { "0%" };
        let rate_text = host::translate("gui.mekanism.process_rate", &[rate_str.into()]);
        c.text(&rate_text, 97.0, 75.0, GREEN, TEXT_CENTER, 1.0);

        // Title centered at y = 6 across width 196
        let title_text = host::translate("block.mekanism.antiprotonic_nucleosynthesizer", &[]);
        title(c, &title_text, 0.0, 196.0, 6.0);

        // Inventory label at (8, 100)
        inventory_label(c, 8.0, 100.0);

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
    fn test_antiprotonic_nucleosynthesizer_render_and_commands() {
        host::set_menu(host::Menu {
            data: vec![
                50, 1000, 0, 5000, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2500, 10, 0, 0, 0, 0, 0, 400,
            ],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:antiprotonic_nucleosynthesizer".into(),
            touch: false,
            gui_width: 196.0,
            gui_height: 193.0,
        })
        .expect("open antiprotonic nucleosynthesizer");

        assert_eq!(setup.width, 196);
        assert_eq!(setup.height, 193);

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
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:inner_screen")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:gauge/overlay/small_med")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, .. } if sprite == "mekanism:bar/dynamic_rate")));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Sprite { sprite, rect, .. } if sprite == "mekanism:bar/base" && *rect == [5.0, 88.0, 185.0, 10.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Area { label, rect, .. } if label == "Progress" && *rect == [5.0, 88.0, 185.0, 10.0])));
        assert!(frame.commands.iter().any(|cmd| matches!(cmd, Command::Text { y, .. } if *y == 6.0)));
    }

    #[test]
    fn test_antiprotonic_nucleosynthesizer_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:antiprotonic_nucleosynthesizer".into(),
            touch: false,
            gui_width: 196.0,
            gui_height: 193.0,
        })
        .expect("open");

        // Redstone tab is at (196, 137)
        let event = InputEvent::MousePressed(Pointer {
            x: 200.0,
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
