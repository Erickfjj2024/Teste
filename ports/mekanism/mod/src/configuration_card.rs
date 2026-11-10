//! Configuration Card (`ItemConfigurationCard`, `IConfigCardAccess`).
//!
//! An item used to copy and paste machine configuration data (side configurations,
//! ejector settings, redstone controls, machine specific settings) between compatible machines.
//!
//! Interactions:
//! 1. Copy Configuration (Shift + Right-click on a machine):
//!    - Saves configuration data into the card's `mekanism:configuration_data` component.
//!    - Sends action bar message: "Retrieved configuration data from %s" (EnumColor.INDIGO).
//!    - Triggers advancement trigger `CONFIGURATION_CARD` with `copied = true`.
//! 2. Paste Configuration (Right-click on a machine without sneaking):
//!    - Checks compatibility (`isConfigurationDataCompatible`).
//!    - If compatible: applies configuration, sends "Injected configuration data of type %s" (EnumColor.INDIGO),
//!      and triggers advancement with `copied = false`.
//!    - If incompatible: sends "Unequal configuration data formats." (EnumColor.RED).
//! 3. Clear Card (Shift + Right-click in the air):
//!    - Clears the card's data component.
//!    - Sends action bar message: "Cleared Configuration Card".
//!
//! Java source of truth:
//! - `mekanism.common.item.ItemConfigurationCard`
//! - `mekanism.api.IConfigCardAccess`
//! - `mekanism.common.tile.base.TileEntityMekanism`
//! - `mekanism.common.tile.prefab.TileEntityConfigurableMachine`
//! - `mekanism.common.tile.component.TileComponentConfig`
//! - `mekanism.common.tile.component.TileComponentEjector`
//! - `mekanism.common.advancements.triggers.ConfigurationCardTrigger`

use super::*;
use std::collections::HashMap;

pub const ITEM: &str = "mekanism:configuration_card";
pub const CONFIG_DATA_COMPONENT: &str = "mekanism:configuration_data";

// SNBT / NBT serialization constants matching Java `SerializationConstants`
pub const DATA_NAME: &str = "data_name";
pub const DATA_TYPE: &str = "data_type";
pub const CONTROL_TYPE: &str = "control_type";
pub const CONFIG_PREFIX: &str = "config"; // config0 .. config4
pub const EJECT_PREFIX: &str = "eject";   // eject0 .. eject4
pub const STRICT_INPUT: &str = "strict_input";
pub const COLOR: &str = "color";
pub const INPUT_COLOR: &str = "input_color";

// Language translation keys and fallback strings (matching en_us.json)
pub const MSG_CLEARED: &str = "Cleared Configuration Card";
pub const MSG_GOT: &str = "Retrieved configuration data from %s";
pub const MSG_SET: &str = "Injected configuration data of type %s";
pub const MSG_UNEQUAL: &str = "Unequal configuration data formats.";
pub const MSG_HAS_DATA: &str = "Data: %s";
pub const MSG_NONE: &str = "None";

// ============================================================================
// 1. DATA STRUCTURES & SERIALIZATION
// ============================================================================

/// The configuration data payload stored on a Configuration Card (`ItemConfigurationCard.java`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigurationData {
    /// Machine translation key / display name (e.g. "block.mekanism.enrichment_chamber").
    pub data_name: String,
    /// Machine block ID (e.g. "mekanism:enrichment_chamber").
    pub data_type: String,
    /// Redstone control ordinal (0=Disabled, 1=High, 2=Low, 3=Pulse).
    pub control_type: Option<u8>,
    /// Side configuration for up to 5 transmissions (Items: 0, Energy: 1, Fluids: 2, Chemicals: 3, Heat: 4).
    /// Each transmission has 6 relative sides (FRONT=0, LEFT=1, RIGHT=2, BACK=3, TOP=4, BOTTOM=5).
    pub side_config: Option<[[u8; 6]; 5]>,
    /// Auto-eject flags for up to 5 transmissions.
    pub eject: Option<[bool; 5]>,
    /// Ejector strict input setting.
    pub strict_input: Option<bool>,
    /// Ejector output transporter color index (-1 for none).
    pub output_color: Option<i32>,
    /// Ejector input transporter color indices per side.
    pub input_colors: Option<[i32; 6]>,
    /// Machine-specific extra parameters (e.g. energy_usage, miner radius/min/max, etc.).
    pub extra: HashMap<String, String>,
}

impl ConfigurationData {
    pub fn is_empty(&self) -> bool {
        self.data_type.is_empty()
    }

    pub fn has_data(&self) -> bool {
        !self.is_empty()
    }

    /// Serializes this configuration data into an SNBT string for `mekanism:configuration_data`.
    pub fn encode_snbt(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("\"{DATA_NAME}\":\"{}\"", self.data_name));
        parts.push(format!("\"{DATA_TYPE}\":\"{}\"", self.data_type));

        if let Some(ctrl) = self.control_type {
            parts.push(format!("\"{CONTROL_TYPE}\":{}b", ctrl));
        }

        if let Some(sides) = &self.side_config {
            for (t, side_arr) in sides.iter().enumerate() {
                let ints: Vec<String> = side_arr.iter().map(|d| d.to_string()).collect();
                parts.push(format!("\"{CONFIG_PREFIX}{t}\":[{}]", ints.join(",")));
            }
        }

        if let Some(ejects) = &self.eject {
            for (t, ej) in ejects.iter().enumerate() {
                parts.push(format!("\"{EJECT_PREFIX}{t}\":{}b", if *ej { 1 } else { 0 }));
            }
        }

        if let Some(strict) = self.strict_input {
            parts.push(format!("\"{STRICT_INPUT}\":{}b", if strict { 1 } else { 0 }));
        }

        if let Some(col) = self.output_color {
            parts.push(format!("\"{COLOR}\":{}", col));
        }

        if let Some(in_cols) = &self.input_colors {
            let ints: Vec<String> = in_cols.iter().map(|c| c.to_string()).collect();
            parts.push(format!("\"{INPUT_COLOR}\":[{}]", ints.join(",")));
        }

        // Extra machine-specific keys sorted for deterministic output
        let mut extra_keys: Vec<&String> = self.extra.keys().collect();
        extra_keys.sort();
        for k in extra_keys {
            if let Some(val) = self.extra.get(k) {
                parts.push(format!("\"{k}\":\"{val}\""));
            }
        }

        format!("{{{}}}", parts.join(","))
    }

    /// Decodes configuration data from an SNBT/JSON component string.
    pub fn decode_snbt(snbt: &str) -> Option<Self> {
        let text = snbt.trim();
        if text.is_empty() || text == "{}" {
            return None;
        }

        let mut data = ConfigurationData::default();
        let entries = config::split_components(text)?;

        let mut side_config = [[0u8; 6]; 5];
        let mut has_side_config = false;
        let mut eject = [false; 5];
        let mut has_eject = false;

        for (key, range) in entries {
            let val_str = text[range].trim();
            match key.as_str() {
                DATA_NAME => {
                    data.data_name = val_str.trim_matches('"').to_string();
                }
                DATA_TYPE => {
                    data.data_type = val_str.trim_matches('"').to_string();
                }
                CONTROL_TYPE => {
                    data.control_type = val_str.trim_end_matches('b').parse().ok();
                }
                STRICT_INPUT => {
                    data.strict_input = Some(val_str == "1b" || val_str == "true" || val_str == "1");
                }
                COLOR => {
                    data.output_color = val_str.parse().ok();
                }
                INPUT_COLOR => {
                    if val_str.starts_with('[') && val_str.ends_with(']') {
                        let inner = &val_str[1..val_str.len() - 1];
                        let mut arr = [-1i32; 6];
                        for (i, num_str) in inner.split(',').enumerate() {
                            if i < 6 {
                                arr[i] = num_str.trim().parse().unwrap_or(-1);
                            }
                        }
                        data.input_colors = Some(arr);
                    }
                }
                k if k.starts_with(CONFIG_PREFIX) => {
                    if let Ok(t_idx) = k[CONFIG_PREFIX.len()..].parse::<usize>() {
                        if t_idx < 5 && val_str.starts_with('[') && val_str.ends_with(']') {
                            let inner = &val_str[1..val_str.len() - 1];
                            for (i, num_str) in inner.split(',').enumerate() {
                                if i < 6 {
                                    side_config[t_idx][i] = num_str.trim().parse().unwrap_or(0);
                                }
                            }
                            has_side_config = true;
                        }
                    }
                }
                k if k.starts_with(EJECT_PREFIX) => {
                    if let Ok(t_idx) = k[EJECT_PREFIX.len()..].parse::<usize>() {
                        if t_idx < 5 {
                            eject[t_idx] = val_str == "1b" || val_str == "true" || val_str == "1";
                            has_eject = true;
                        }
                    }
                }
                other => {
                    data.extra.insert(other.to_string(), val_str.trim_matches('"').to_string());
                }
            }
        }

        if has_side_config {
            data.side_config = Some(side_config);
        }
        if has_eject {
            data.eject = Some(eject);
        }

        if data.is_empty() {
            None
        } else {
            Some(data)
        }
    }

    /// Java: `ItemConfigurationCard.java:45-47`: Tooltip format.
    /// Emits "Data: <name>" or "Data: None".
    pub fn card_tooltip(&self) -> String {
        if self.has_data() {
            let name = display_name(if self.data_name.is_empty() { &self.data_type } else { &self.data_name });
            format!("Data: {name}")
        } else {
            "Data: None".to_string()
        }
    }
}

// ============================================================================
// 2. MACHINE COMPATIBILITY (IS_CONFIGURATION_DATA_COMPATIBLE)
// ============================================================================

/// Checks whether configuration data saved from `source_type` can be applied to `target_type`.
/// Java: `IConfigCardAccess.java:40-42`, `TileEntityElectricMachine.java:132-134`,
/// `TileEntityFactory.java:444-446`, `MekanismUtils.java:650-657`.
pub fn is_compatible(source_type: &str, target_type: &str) -> bool {
    // 1. Exact block ID match
    if source_type == target_type {
        return true;
    }

    // 2. Factory families: an electric machine and all 4 factory tiers of the same process
    const FACTORY_FAMILIES: &[&[&str]] = &[
        // Smelting
        &[
            "mekanism:energized_smelter",
            "mekanism:basic_smelting_factory",
            "mekanism:advanced_smelting_factory",
            "mekanism:elite_smelting_factory",
            "mekanism:ultimate_smelting_factory",
        ],
        // Enriching
        &[
            "mekanism:enrichment_chamber",
            "mekanism:basic_enriching_factory",
            "mekanism:advanced_enriching_factory",
            "mekanism:elite_enriching_factory",
            "mekanism:ultimate_enriching_factory",
        ],
        // Crushing
        &[
            "mekanism:crusher",
            "mekanism:basic_crushing_factory",
            "mekanism:advanced_crushing_factory",
            "mekanism:elite_crushing_factory",
            "mekanism:ultimate_crushing_factory",
        ],
        // Compressing
        &[
            "mekanism:osmium_compressor",
            "mekanism:basic_compressing_factory",
            "mekanism:advanced_compressing_factory",
            "mekanism:elite_compressing_factory",
            "mekanism:ultimate_compressing_factory",
        ],
        // Combining
        &[
            "mekanism:combiner",
            "mekanism:basic_combining_factory",
            "mekanism:advanced_combining_factory",
            "mekanism:elite_combining_factory",
            "mekanism:ultimate_combining_factory",
        ],
        // Purifying
        &[
            "mekanism:purification_chamber",
            "mekanism:basic_purifying_factory",
            "mekanism:advanced_purifying_factory",
            "mekanism:elite_purifying_factory",
            "mekanism:ultimate_purifying_factory",
        ],
        // Injecting
        &[
            "mekanism:chemical_injection_chamber",
            "mekanism:basic_injecting_factory",
            "mekanism:advanced_injecting_factory",
            "mekanism:elite_injecting_factory",
            "mekanism:ultimate_injecting_factory",
        ],
        // Infusing
        &[
            "mekanism:metallurgic_infuser",
            "mekanism:basic_infusing_factory",
            "mekanism:advanced_infusing_factory",
            "mekanism:elite_infusing_factory",
            "mekanism:ultimate_infusing_factory",
        ],
        // Sawing
        &[
            "mekanism:precision_sawmill",
            "mekanism:basic_sawing_factory",
            "mekanism:advanced_sawing_factory",
            "mekanism:elite_sawing_factory",
            "mekanism:ultimate_sawing_factory",
        ],
    ];

    for fam in FACTORY_FAMILIES {
        if fam.contains(&source_type) && fam.contains(&target_type) {
            return true;
        }
    }

    // Nothing else: the original's `isConfigurationDataCompatible` is the same block, plus `isSameTypeFactory` in the factories and the machines that have
    // one (electric machines, the combiner, the infuser, the sawmill). Energy cubes, tanks, bins and transmitters of other tiers are not compatible.
    false
}

// ============================================================================
// 3. MACHINE CONFIGURATION EXTRACTION & INJECTION
// ============================================================================

/// The machines the card works on besides the assemblicator and the resistive heater: those whose data starts with the configuration (kinds with the
/// sides at byte 2 and the control where `redstone::offset` says).
pub fn card_machine(block: &str) -> Option<&'static crate::Def> {
    crate::MACHINES.iter().find(|d| {
        d.block == block && matches!(d.kind, crate::Kind::Electric | crate::Kind::Process | crate::Kind::Sawmill | crate::Kind::Factory | crate::Kind::Continuous | crate::Kind::Reaction | crate::Kind::Pump)
    })
}

/// Whether the card does anything on this block (`Capabilities.CONFIG_CARD`).
pub fn supports(block: &str) -> bool {
    block == assemblicator::BLOCK || block == crate::resistive_heater::BLOCK || card_machine(block).is_some()
}

/// The English name a card shows for a stored translation key (`block.mekanism.enrichment_chamber` -> `Enrichment Chamber`): the port has no language
/// files at hand, but the original's block names are the id in title case. A name already written out is kept.
pub fn display_name(key_or_name: &str) -> String {
    let path = key_or_name.strip_prefix("block.mekanism.").or_else(|| key_or_name.strip_prefix("mekanism:"));
    match path {
        Some(p) if !p.contains(' ') => p
            .split('_')
            .map(|w| {
                let mut c = w.chars();
                c.next().map_or_else(String::new, |f| f.to_uppercase().collect::<String>() + c.as_str())
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => key_or_name.to_owned(),
    }
}

/// Translates a block id to its default translation key / title.
pub fn block_translation_key(block_id: &str) -> String {
    let name = block_id.strip_prefix("mekanism:").unwrap_or(block_id);
    format!("block.mekanism.{name}")
}

/// Extracts configuration data from a block entity according to Java's `IConfigCardAccess`.
/// Java: `TileEntityMekanism.java:1505-1510`, `TileEntityConfigurableMachine.java:38-43`.
pub fn extract_configuration(block_name: &str, e: &BlockEntity) -> ConfigurationData {
    let mut data = ConfigurationData {
        data_name: block_translation_key(block_name),
        data_type: block_name.to_string(),
        control_type: None,
        side_config: None,
        eject: None,
        strict_input: None,
        output_color: None,
        input_colors: None,
        extra: HashMap::new(),
    };

    // If it's a Formulaic Assemblicator
    if block_name == assemblicator::BLOCK {
        if let Some(state) = assemblicator::AssemblicatorState::decode(&e.data) {
            data.control_type = Some(state.control as u8);
            let mut sides = [[0u8; 6]; 5];
            for (t, s_arr) in state.config.sides.iter().enumerate() {
                for (s, d) in s_arr.iter().enumerate() {
                    sides[t][s] = *d as u8;
                }
            }
            data.side_config = Some(sides);
            data.eject = Some(state.config.eject);
            data.extra.insert("auto".into(), if state.auto_mode { "1" } else { "0" }.into());
            data.extra.insert("stock_control".into(), if state.stock_control { "1" } else { "0" }.into());
            return data;
        }
    }

    // The resistive heater: the redstone control and the energy it uses (`TileEntityResistiveHeater.getConfigurationData`).
    if block_name == crate::resistive_heater::BLOCK {
        if let Some(h) = crate::resistive_heater::ResistiveHeater::decode(&e.data, 300.0) {
            data.control_type = Some(h.redstone as u8);
            data.extra.insert("energy_usage".into(), h.energy_usage_joules.to_string());
        }
        return data;
    }

    // A configurable machine: the control, then the sides and eject flags of its configuration (`configComponent.write`).
    if let Some(def) = card_machine(block_name) {
        let cfg = Config::decode(e.data.get(2..).unwrap_or(&[]), def.layout).unwrap_or_else(|| config::default_config(def.layout));
        let mut sides = [[0u8; 6]; 5];
        for (t, s_arr) in cfg.sides.iter().enumerate() {
            for (s, d) in s_arr.iter().enumerate() {
                sides[t][s] = *d as u8;
            }
        }
        data.side_config = Some(sides);
        data.eject = Some(cfg.eject);
        data.control_type = e.data.get(crate::redstone::offset(def.kind)).copied();
    }

    data
}

/// Applies configuration data to a block entity.
/// Java: `TileEntityMekanism.java:1513-1516`, `TileEntityConfigurableMachine.java:46-50`.
pub fn apply_configuration(target_block: &str, e: &mut BlockEntity, data: &ConfigurationData) -> bool {
    if !is_compatible(&data.data_type, target_block) {
        return false;
    }

    // Target is a Formulaic Assemblicator
    if target_block == assemblicator::BLOCK {
        let mut state = assemblicator::AssemblicatorState::decode(&e.data).unwrap_or_default();
        if let Some(ctrl) = data.control_type {
            state.control = Control::from_byte(ctrl);
        }
        if let Some(sides) = &data.side_config {
            for (t, s_arr) in sides.iter().enumerate() {
                for (s, d) in s_arr.iter().enumerate() {
                    state.config.sides[t][s] = Data::from_byte(*d);
                }
            }
        }
        if let Some(ejects) = &data.eject {
            state.config.eject = *ejects;
        }
        if let Some(auto_val) = data.extra.get("auto") {
            state.auto_mode = auto_val == "1" || auto_val == "true";
        }
        if let Some(sc_val) = data.extra.get("stock_control") {
            state.stock_control = sc_val == "1" || sc_val == "true";
        }
        e.data = state.encode();
        return true;
    }

    // The resistive heater: the control and the energy usage (`setLegacyEnergyIfPresent` -> `updateEnergyUsage`).
    if target_block == crate::resistive_heater::BLOCK {
        if let Some(mut h) = crate::resistive_heater::ResistiveHeater::decode(&e.data, 300.0) {
            if let Some(ctrl) = data.control_type {
                h.redstone = crate::resistive_heater::RedstoneMode::from_u8(ctrl);
            }
            if let Some(usage) = data.extra.get("energy_usage").and_then(|u| u.parse::<u64>().ok()) {
                h.set_energy_usage(usage);
            }
            e.data = h.encode();
        }
        return true;
    }

    // A configurable machine: the sides and eject flags, then the control at the place its kind keeps it.
    let Some(def) = card_machine(target_block) else { return false };
    if let Some(sides) = &data.side_config {
        let mut cfg = Config::decode(e.data.get(2..).unwrap_or(&[]), def.layout).unwrap_or_else(|| config::default_config(def.layout));
        for (t, s_arr) in sides.iter().enumerate() {
            for (s, d) in s_arr.iter().enumerate() {
                cfg.sides[t][s] = Data::from_byte(*d);
            }
        }
        if let Some(ejects) = &data.eject {
            cfg.eject = *ejects;
        }
        let encoded = cfg.encode();
        if e.data.len() < 2 + encoded.len() {
            e.data.resize(2 + encoded.len(), 0);
        }
        e.data[2..2 + encoded.len()].copy_from_slice(&encoded);
    }
    if let Some(ctrl) = data.control_type {
        let at = crate::redstone::offset(def.kind);
        if e.data.len() <= at {
            e.data.resize(at + 1, 0);
        }
        e.data[at] = Control::from_byte(ctrl) as u8;
    }

    true
}

// ============================================================================
// 4. ITEM INTERACTIONS & ADVANCEMENTS
// ============================================================================

pub enum CardInteractionResult {
    /// Configuration was successfully copied from the machine to the card.
    Copied { message: String },
    /// Configuration was successfully pasted from the card to the machine.
    Pasted { message: String },
    /// Configuration formats were incompatible between card and machine.
    Incompatible { message: String },
    /// Card has no data to paste.
    Pass,
}

pub struct ConfigurationCardItem;

impl ConfigurationCardItem {
    /// Reads configuration data from the item's components.
    pub fn get_data(stack: &ItemStack) -> Option<ConfigurationData> {
        if let Some(entries) = config::split_components(&stack.components) {
            for (key, range) in entries {
                if key == CONFIG_DATA_COMPONENT {
                    return ConfigurationData::decode_snbt(&stack.components[range]);
                }
            }
        }
        None
    }

    /// Sets configuration data on the item's components.
    pub fn set_data(stack: &mut ItemStack, data: &ConfigurationData) {
        let snbt = data.encode_snbt();
        let pair = format!("\"{CONFIG_DATA_COMPONENT}\":{snbt}");
        if stack.components.trim().is_empty() || stack.components.trim() == "{}" {
            stack.components = format!("{{{pair}}}");
            return;
        }
        if let Some(entries) = config::split_components(&stack.components) {
            for (key, range) in &entries {
                if key == CONFIG_DATA_COMPONENT {
                    stack.components.replace_range(range.clone(), &snbt);
                    return;
                }
            }
            let at = stack.components.rfind('}').unwrap_or(stack.components.len());
            stack.components.insert_str(at, &format!(",{pair}"));
        } else {
            stack.components = format!("{{{pair}}}");
        }
    }

    /// Clears configuration data from the item.
    /// Java: `ItemConfigurationCard.java:104`.
    pub fn clear_data(stack: &mut ItemStack) {
        if let Some(entries) = config::split_components(&stack.components) {
            let mut kept = Vec::new();
            for (key, range) in entries {
                if key != CONFIG_DATA_COMPONENT {
                    kept.push(format!("\"{key}\":{}", &stack.components[range]));
                }
            }
            if kept.is_empty() {
                stack.components.clear();
            } else {
                stack.components = format!("{{{}}}", kept.join(","));
            }
        }
    }

    /// Handles using the Configuration Card on a block entity (`useOn`).
    /// Java: `ItemConfigurationCard.java:51-96`.
    pub fn use_on(
        player_id: &str,
        sneaking: bool,
        block_name: &str,
        block_entity: &mut BlockEntity,
        card_stack: &mut ItemStack,
    ) -> CardInteractionResult {
        if sneaking {
            // Shift + Right-Click: COPY
            let data = extract_configuration(block_name, block_entity);
            Self::set_data(card_stack, &data);
            let display_name = display_name(if data.data_name.is_empty() { block_name } else { &data.data_name });
            let msg = format!("Retrieved configuration data from {display_name}");
            let _ = players::tell(player_id, &msg);

            // FORK-34: Server advancement criteria trigger CONFIGURATION_CARD (copied = true)
            // horizon_advancements::trigger_configuration_card(player_id, true);

            CardInteractionResult::Copied { message: msg }
        } else {
            // Right-Click without sneaking: PASTE
            let Some(card_data) = Self::get_data(card_stack) else {
                return CardInteractionResult::Pass;
            };

            if is_compatible(&card_data.data_type, block_name) {
                apply_configuration(block_name, block_entity, &card_data);
                let display_name = display_name(if card_data.data_name.is_empty() { &card_data.data_type } else { &card_data.data_name });
                let msg = format!("Injected configuration data of type {display_name}");
                let _ = players::tell(player_id, &msg);

                // FORK-34: Server advancement criteria trigger CONFIGURATION_CARD (copied = false)
                // horizon_advancements::trigger_configuration_card(player_id, false);

                CardInteractionResult::Pasted { message: msg }
            } else {
                let msg = MSG_UNEQUAL.to_string();
                let _ = players::tell(player_id, &msg);
                CardInteractionResult::Incompatible { message: msg }
            }
        }
    }

    /// Handles using the Configuration Card in the air (`use`).
    /// Java: `ItemConfigurationCard.java:100-110`.
    pub fn use_air(player_id: &str, sneaking: bool, card_stack: &mut ItemStack) -> bool {
        if sneaking {
            Self::clear_data(card_stack);
            let _ = players::tell(player_id, MSG_CLEARED);
            true
        } else {
            false
        }
    }
}

// ============================================================================
// 5. UNIT TESTS (AUDITED AGAINST JAVA 1.21.1)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Configuration data SNBT roundtrip matches Java serialization format.
    /// Java source: `TileComponentConfig.java:340-353`, `ItemConfigurationCard.java:68-71`.
    #[test]
    fn test_configuration_data_snbt_roundtrip() {
        let mut sides = [[0u8; 6]; 5];
        // Transmission 0 (Items): Input on left, Output on right
        sides[0][1] = 1; // Left = Input
        sides[0][2] = 4; // Right = Output
        sides[0][3] = 8; // Back = Energy

        let mut eject = [false; 5];
        eject[0] = true;

        let mut extra = HashMap::new();
        extra.insert("auto".into(), "1".into());

        let data = ConfigurationData {
            data_name: "block.mekanism.enrichment_chamber".into(),
            data_type: "mekanism:enrichment_chamber".into(),
            control_type: Some(1), // Control::High
            side_config: Some(sides),
            eject: Some(eject),
            strict_input: Some(true),
            output_color: Some(3),
            input_colors: Some([0, 1, 2, 3, 4, 5]),
            extra,
        };

        let encoded = data.encode_snbt();
        assert!(encoded.contains("\"data_name\":\"block.mekanism.enrichment_chamber\""));
        assert!(encoded.contains("\"data_type\":\"mekanism:enrichment_chamber\""));
        assert!(encoded.contains("\"config0\":[0,1,4,8,0,0]"));
        assert!(encoded.contains("\"eject0\":1b"));
        assert!(encoded.contains("\"strict_input\":1b"));

        let decoded = ConfigurationData::decode_snbt(&encoded).expect("failed to decode SNBT");
        assert_eq!(decoded.data_name, "block.mekanism.enrichment_chamber");
        assert_eq!(decoded.data_type, "mekanism:enrichment_chamber");
        assert_eq!(decoded.control_type, Some(1));
        assert_eq!(decoded.strict_input, Some(true));
        assert_eq!(decoded.output_color, Some(3));
        assert_eq!(decoded.input_colors, Some([0, 1, 2, 3, 4, 5]));
        assert_eq!(decoded.side_config.unwrap()[0], [0, 1, 4, 8, 0, 0]);
        assert_eq!(decoded.eject.unwrap()[0], true);
        assert_eq!(decoded.extra.get("auto"), Some(&"1".to_string()));
    }

    /// Test: Factory compatibility allows machines and factory tiers to share configuration.
    /// Java source: `TileEntityElectricMachine.java:132-134`, `MekanismUtils.java:650-657`.
    #[test]
    fn test_factory_compatibility() {
        // Crusher is compatible with its factory tiers
        assert!(is_compatible("mekanism:crusher", "mekanism:basic_crushing_factory"));
        assert!(is_compatible("mekanism:basic_crushing_factory", "mekanism:ultimate_crushing_factory"));
        assert!(is_compatible("mekanism:ultimate_crushing_factory", "mekanism:crusher"));

        // Smelter is compatible with smelting factories
        assert!(is_compatible("mekanism:energized_smelter", "mekanism:elite_smelting_factory"));

        // Crusher is NOT compatible with Smelter or Enriching Factory
        assert!(!is_compatible("mekanism:crusher", "mekanism:energized_smelter"));
        assert!(!is_compatible("mekanism:crusher", "mekanism:basic_enriching_factory"));
    }

    /// Cubes, tanks, bins and transmitters of other tiers are NOT compatible: the original only compares the block (and the factory type).
    /// Java: `IConfigCardAccess.isConfigurationDataCompatible` default and the six overrides.
    #[test]
    fn test_other_tiers_are_not_compatible() {
        assert!(is_compatible("mekanism:basic_energy_cube", "mekanism:basic_energy_cube"));
        assert!(!is_compatible("mekanism:basic_energy_cube", "mekanism:ultimate_energy_cube"));
        assert!(!is_compatible("mekanism:basic_chemical_tank", "mekanism:creative_chemical_tank"));
        assert!(!is_compatible("mekanism:basic_fluid_tank", "mekanism:elite_fluid_tank"));
        assert!(!is_compatible("mekanism:basic_bin", "mekanism:ultimate_bin"));
        assert!(!is_compatible("mekanism:basic_universal_cable", "mekanism:elite_universal_cable"));
        assert!(!is_compatible("mekanism:basic_energy_cube", "mekanism:basic_chemical_tank"));
    }

    fn machine_entity(block: &str, data: Vec<u8>) -> BlockEntity {
        BlockEntity {
            dimension: "minecraft:overworld".into(),
            pos: BlockPos { x: 1, y: 64, z: 1 },
            state: block.into(),
            data,
            items: Vec::new(),
            energy: 0,
            energy_faces: Vec::new(),
            tanks: Vec::new(),
            tank_faces: Vec::new(),
            menu_data: Vec::new(),
            transmitter_faces: Vec::new(),
        }
    }

    /// The card copies and pastes the sides, the eject flags and the control of a machine at the offsets its kind keeps them in.
    #[test]
    fn test_card_roundtrip_on_two_kinds_of_machines() {
        for (block, control_at) in [("mekanism:enrichment_chamber", 37usize), ("mekanism:chemical_infuser", 39usize)] {
            let def = card_machine(block).expect("a configurable machine");
            assert_eq!(crate::redstone::offset(def.kind), control_at);
            let mut cfg = config::default_config(def.layout);
            cfg.eject[0] = true;
            cfg.sides[0][1] = Data::Output;
            let mut data = vec![0u8; control_at + 1];
            data[2..37].copy_from_slice(&cfg.encode());
            data[control_at] = Control::Low as u8;
            let source = machine_entity(block, data);
            let card = extract_configuration(block, &source);
            assert_eq!(card.control_type, Some(Control::Low as u8));
            assert_eq!(card.eject.map(|e| e[0]), Some(true));

            let mut target = machine_entity(block, vec![0u8; control_at + 1]);
            assert!(apply_configuration(block, &mut target, &card));
            assert_eq!(target.data[control_at], Control::Low as u8);
            let back = Config::decode(&target.data[2..], def.layout).unwrap();
            assert!(back.eject[0]);
            assert_eq!(back.sides[0][1], Data::Output);
        }
    }

    /// The card does something only on blocks that have the capability.
    #[test]
    fn test_card_supports_only_configurable_blocks() {
        assert!(supports("mekanism:enrichment_chamber"));
        assert!(supports(assemblicator::BLOCK));
        assert!(supports("mekanism:resistive_heater"));
        assert!(!supports("mekanism:basic_energy_cube"));
        assert!(!supports("minecraft:stone"));
    }

    /// The resistive heater keeps its control and its energy usage on the card.
    #[test]
    fn test_card_resistive_heater() {
        let mut h = crate::resistive_heater::ResistiveHeater::new(300.0);
        h.set_energy_usage(5_000);
        h.redstone = crate::resistive_heater::RedstoneMode::from_u8(2);
        let source = machine_entity("mekanism:resistive_heater", h.encode());
        let card = extract_configuration("mekanism:resistive_heater", &source);
        assert_eq!(card.extra.get("energy_usage").map(String::as_str), Some("5000"));
        let mut target = machine_entity("mekanism:resistive_heater", crate::resistive_heater::ResistiveHeater::new(300.0).encode());
        assert!(apply_configuration("mekanism:resistive_heater", &mut target, &card));
        let got = crate::resistive_heater::ResistiveHeater::decode(&target.data, 300.0).unwrap();
        assert_eq!(got.energy_usage_joules, 5_000);
        assert_eq!(got.redstone as u8, 2);
    }

    /// Test: Card tooltip presentation.
    /// Java source: `ItemConfigurationCard.java:45-47`.
    #[test]
    fn test_card_tooltip() {
        let empty_data = ConfigurationData::default();
        assert_eq!(empty_data.card_tooltip(), "Data: None");

        let configured = ConfigurationData {
            data_name: "Enrichment Chamber".into(),
            data_type: "mekanism:enrichment_chamber".into(),
            ..Default::default()
        };
        assert_eq!(configured.card_tooltip(), "Data: Enrichment Chamber");
    }

    /// Test: Card clearing on shift + air right click.
    /// Java source: `ItemConfigurationCard.java:100-110`.
    #[test]
    fn test_card_clear() {
        let mut card = ItemStack {
            item: ITEM.into(),
            count: 1,
            components: String::new(),
        };

        let data = ConfigurationData {
            data_name: "Enrichment Chamber".into(),
            data_type: "mekanism:enrichment_chamber".into(),
            ..Default::default()
        };
        ConfigurationCardItem::set_data(&mut card, &data);
        assert!(ConfigurationCardItem::get_data(&card).is_some());

        ConfigurationCardItem::clear_data(&mut card);
        assert!(ConfigurationCardItem::get_data(&card).is_none());
    }

    /// Test: Card copy and paste mechanics between block entities.
    /// Java source: `ItemConfigurationCard.java:51-96`.
    #[test]
    fn test_card_use_on_copy_and_paste() {
        let mut card = ItemStack {
            item: ITEM.into(),
            count: 1,
            components: String::new(),
        };

        // Source machine: Crusher with custom side config
        let mut crusher_entity = BlockEntity {
            dimension: "minecraft:overworld".into(),
            pos: BlockPos { x: 10, y: 64, z: 10 },
            state: "mekanism:crusher[facing=north]".into(),
            data: vec![0; 40],
            items: vec![None; 2],
            energy: 1000,
            tanks: Vec::new(),
            energy_faces: Vec::new(),
            tank_faces: Vec::new(),
            menu_data: Vec::new(),
            transmitter_faces: Vec::new(),
        };
        // Set redstone control to 2 (Low)
        crusher_entity.data[37] = 2;

        // Shift + right-click to copy
        let copy_res = ConfigurationCardItem::use_on("player1", true, "mekanism:crusher", &mut crusher_entity, &mut card);
        assert!(matches!(copy_res, CardInteractionResult::Copied { .. }));

        let data = ConfigurationCardItem::get_data(&card).expect("card should have data");
        assert_eq!(data.data_type, "mekanism:crusher");
        assert_eq!(data.control_type, Some(2));

        // Target 1: Compatible Basic Crushing Factory
        let mut factory_entity = BlockEntity {
            dimension: "minecraft:overworld".into(),
            pos: BlockPos { x: 12, y: 64, z: 10 },
            state: "mekanism:basic_crushing_factory[facing=north]".into(),
            data: vec![0; 40],
            items: vec![None; 7],
            energy: 1000,
            tanks: Vec::new(),
            energy_faces: Vec::new(),
            tank_faces: Vec::new(),
            menu_data: Vec::new(),
            transmitter_faces: Vec::new(),
        };
        let paste_res = ConfigurationCardItem::use_on("player1", false, "mekanism:basic_crushing_factory", &mut factory_entity, &mut card);
        assert!(matches!(paste_res, CardInteractionResult::Pasted { .. }));
        assert_eq!(factory_entity.data[37], 2); // Redstone control transferred!

        // Target 2: Incompatible Smelter
        let mut smelter_entity = BlockEntity {
            dimension: "minecraft:overworld".into(),
            pos: BlockPos { x: 14, y: 64, z: 10 },
            state: "mekanism:energized_smelter[facing=north]".into(),
            data: vec![0; 40],
            items: vec![None; 2],
            energy: 1000,
            tanks: Vec::new(),
            energy_faces: Vec::new(),
            tank_faces: Vec::new(),
            menu_data: Vec::new(),
            transmitter_faces: Vec::new(),
        };
        let bad_paste = ConfigurationCardItem::use_on("player1", false, "mekanism:energized_smelter", &mut smelter_entity, &mut card);
        assert!(matches!(bad_paste, CardInteractionResult::Incompatible { .. }));
    }

    #[test]
    fn test_display_names_are_the_english_ones() {
        assert_eq!(display_name("block.mekanism.enrichment_chamber"), "Enrichment Chamber");
        assert_eq!(display_name("mekanism:formulaic_assemblicator"), "Formulaic Assemblicator");
        assert_eq!(display_name("Enrichment Chamber"), "Enrichment Chamber");
    }
}
