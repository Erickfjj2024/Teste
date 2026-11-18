//! The screens of the Mekanism port, drawn by code (capability `screen`, API 56): each of Mekanism's `GuiXxx` classes (`GuiTeleporter`, `GuiQIODashboard`...) is a module here, written close to the
//! Java so that the positions, sprites, texts and clicks are the original's. A menu type opts in with `"code": true` in its menu JSON (the converter's `CODE_SCREENS`); without the screen, or when it
//! fails, the game shows the menu's JSON widgets, which are kept as the fallback.
//!
//! One screen is open at a time: `registry::open` makes the one for `info.menu` (`mekanism:<name>`). Add a screen with a module, a `pub fn open(info) -> Opened` in it and one line in `registry.rs`.
//! `elements.rs` has the parts every Mekanism screen is made of (`GuiElement`s: the base panel, the inner screen, a button, the power bar, the tabs...); put a part used by two screens there.
use horizon_mod_sdk::screen::{Canvas, Frame, InputEvent, OpenInfo, ScreenMod, Setup};

pub mod elements;
pub mod frequency;
/// Copies of the mod's `security_table.rs` and `transporter_table.rs` (the converter writes them; keep them equal): where the screen's data has the security mode and the transporter window's numbers.
pub mod security_table;
pub mod transporter_table;
pub mod chrome;
pub mod side_config;
pub mod transporter_window;
mod dictionary;
mod module_tweaker;
mod portable_teleporter;
mod robit;
mod security_desk;
mod seismic_reader;
mod factory;
mod bin;
mod chemical_tank;
mod dimensional_stabilizer;
mod energy_cube;
mod fluid_tank;
mod personal_storage;
mod portable_qio_dashboard;
mod qio_dashboard;
mod qio_drive_array;
pub mod qio_frequency_select;
pub mod qio_item_viewer;
mod metallurgic_infuser;
mod precision_sawmill;
mod chemical_oxidizer;
mod advanced_electric_machine;
mod electric_machine;
mod combiner;
mod prc;
mod bio_generator;
mod laser_tractor_beam;
mod formulaic_assemblicator;
mod fluidic_plenisher;
mod resistive_heater;
mod seismic_vibrator;
mod gas_generator;
mod solar_generator;
mod oredictionificator;
mod digital_miner;
mod logistical_sorter;
mod fuelwood_heater;
mod electric_pump;
mod modification_station;
mod heat_generator;
mod laser_amplifier;
mod wind_generator;
mod nutritional_liquifier;
mod sps;
mod industrial_turbine;
mod fusion_reactor_controller;
mod dynamic_tank;
mod thermoelectric_boiler;
mod fission_reactor;
mod thermal_evaporation_controller;
mod induction_matrix;
mod rotary_condensentrator;
mod chemical_dissolution_chamber;
mod chemical_crystallizer;
mod chemical_washer;
mod chemical_infuser;
mod electrolytic_separator;
mod isotopic_centrifuge;
mod painting_machine;
mod pigment_extractor;
mod solar_neutron_activator;
mod antiprotonic_nucleosynthesizer;
mod quantum_entangloporter;
mod pigment_mixer;
mod registry;
mod teleporter;

/// A Mekanism screen: what `GuiMekanism` does with the menu it shows.
pub trait MekScreen {
    /// 20 times a second, after the server's data for the tick.
    fn tick(&mut self) {}
    /// One frame, from the panel's top left (the way `GuiGraphics` draws in a `Screen`'s `render`).
    fn render(&mut self, frame: &Frame, canvas: &mut Canvas);
    /// The player's input from the panel's top left: true when the screen took it (a click on a slot is the game's otherwise).
    fn input(&mut self, _event: &InputEvent) -> bool {
        false
    }
    fn close(&mut self) {}
}

/// What a screen's `open` gives back: the screen and the menu's setup (its size, whether the inventory shows, who draws the labels).
pub type Opened = Result<(Box<dyn MekScreen>, Setup), String>;

struct Mek(Box<dyn MekScreen>);

impl ScreenMod for Mek {
    fn open(info: &OpenInfo) -> Result<(Self, Setup), String> {
        let (screen, setup) = registry::open(info)?;
        Ok((Mek(screen), setup))
    }
    fn tick(&mut self) {
        self.0.tick();
    }
    fn render(&mut self, frame: &Frame, canvas: &mut Canvas) {
        self.0.render(frame, canvas);
    }
    fn input(&mut self, event: &InputEvent) -> bool {
        self.0.input(event)
    }
    fn close(&mut self) {
        self.0.close();
    }
}

horizon_mod_sdk::export_screen_mod!(Mek);
