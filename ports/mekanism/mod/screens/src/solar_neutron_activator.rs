//! `GuiSolarNeutronActivator` (176 by 166): solar neutron activator.
//! Java reference: `mekanism.client.gui.machine.GuiSolarNeutronActivator` (lines 17-43).
//! Container: `mekanism.common.inventory.container.tile.MekanismTileContainer<TileEntitySolarNeutronActivator>`.
//! Tile: `mekanism.common.tile.machine.TileEntitySolarNeutronActivator`.
//!
//! Layout:
//! - Input chemical gauge at (25, 13), standard 16 by 58 (18 by 60 frame), capacity 10000 mB
//! - Output chemical gauge at (133, 13), standard 16 by 58 (18 by 60 frame), capacity 10000 mB
//! - Progress bar at (64, 39), 48 by 8 (ProgressType.LARGE_RIGHT)
//! - Title centered at y = 4, inventory label at (8, 76)
//! - Slots: input at (5, 56) (minus), output at (155, 56) (plus)
//! - Upgrade window: slots 2 (in) and 3 (out), supported: speed (0), energy (1), muffling (3)
//! - No energy: no power bar, no energy tab, no charge slot
//!
//! Differences from Java / Fidelity notes:
//! - Follows Java layout pixel-by-pixel. Warning tracking on widgets is reflected via the warning tab.
//! - Faithful to Java in omitting energy bar, energy tab and energy slots (the machine operates purely on solar radiation).
use crate::chemical_oxidizer::render_chemical_gauge;
use crate::chrome::{Chrome, ChromeSpec, UpgradesSpec};
use crate::elements::{base_panel, data, inventory_label, title, Overlay};
use crate::side_config::SideConfigWindow;
use crate::transporter_window::TransporterWindow;
use crate::{MekScreen, Opened};
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, Setup, host};

pub struct SolarNeutronActivator {
    pub chrome: Chrome,
}

pub fn open(_info: &OpenInfo) -> Opened {
    let menu = "mekanism:solar_neutron_activator";
    let side_config = Some(Box::new(SideConfigWindow::for_machine(menu, 176, 166)) as Box<dyn Overlay>);
    let transporter = TransporterWindow::for_machine(menu, 176, 166).map(|w| Box::new(w) as Box<dyn Overlay>);

    let chrome = Chrome::new(
        ChromeSpec {
            menu,
            redstone: Some(24),
            energy: None,
            upgrades: Some(UpgradesSpec::new(2, 3, 24, &[0, 1, 3])),
            warning: Some(5),
            side_config,
            transporter,
        },
        176.0,
    );

    Ok((
        Box::new(SolarNeutronActivator { chrome }),
        Setup {
            width: 176,
            height: 166,
            hide_inventory: false,
            own_labels: true,
        },
    ))
}

impl MekScreen for SolarNeutronActivator {
    fn tick(&mut self) {
        self.chrome.tick();
    }

    fn render(&mut self, frame: &Frame, c: &mut Canvas) {
        let mouse = (frame.mouse_x, frame.mouse_y);
        base_panel(c, 176.0, 166.0);

        // Slots: input at (5, 56), output at (155, 56)
        c.blit_sprite("mekanism:slot/input", 4.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/minus", 4.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/output", 154.0, 55.0, 18.0, 18.0);
        c.blit_sprite("mekanism:slot/overlay/plus", 154.0, 55.0, 18.0, 18.0);

        let mut hovered_tip: Option<Vec<(String, u32)>> = None;

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
        let title_text = host::translate("block.mekanism.solar_neutron_activator", &[]);
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
    fn test_solar_neutron_activator_render() {
        host::set_menu(host::Menu {
            data: vec![1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2000, 99, 1000, 77, 0, 0, 0, 0, 0],
            strings: vec![],
            ..Default::default()
        });

        let (mut screen, setup) = open(&OpenInfo {
            menu: "mekanism:solar_neutron_activator".into(),
            touch: false,
            gui_width: 176.0,
            gui_height: 166.0,
        })
        .expect("open solar neutron activator");

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
    fn test_solar_neutron_activator_redstone_click() {
        host::set_menu(host::Menu {
            data: vec![0; 50],
            strings: vec![],
            ..Default::default()
        });
        let _ = host::take_sent();

        let (mut screen, _) = open(&OpenInfo {
            menu: "mekanism:solar_neutron_activator".into(),
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
