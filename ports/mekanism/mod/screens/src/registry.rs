//! The menu types with a code screen: `mekanism:<menu>` to the module that draws it. One arm a screen (keep them sorted).
use crate::{Opened, teleporter};
use horizon_mod_sdk::screen::OpenInfo;

pub fn open(info: &OpenInfo) -> Opened {
    match info.menu.as_str() {
        "mekanism:teleporter" => teleporter::open(info),
        other => Err(format!("no code screen for {other}")),
    }
}
