//! The menu types with a code screen: `mekanism:<menu>` to the module that draws it. One arm a screen (keep them sorted).
use crate::{Opened, dictionary, module_tweaker, portable_teleporter, robit, security_desk, seismic_reader, teleporter};
use horizon_mod_sdk::screen::OpenInfo;

pub fn open(info: &OpenInfo) -> Opened {
    match info.menu.as_str() {
        "mekanism:dictionary" => dictionary::open(info),
        "mekanism:module_tweaker" => module_tweaker::open(info),
        "mekanism:portable_teleporter" => portable_teleporter::open(info),
        "mekanism:robit" => robit::open(info),
        "mekanism:security_desk" => security_desk::open(info),
        "mekanism:seismic_reader" => seismic_reader::open(info),
        "mekanism:teleporter" => teleporter::open(info),
        other => Err(format!("no code screen for {other}")),
    }
}
