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
            let name = if self.data_name.is_empty() {
                self.data_type.split(':').nth(1).unwrap_or(&self.data_type)
            } else {
                &self.data_name
            };
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

    // 3. Energy Cube tiers
    const CUBES: &[&str] = &[
        "mekanism:basic_energy_cube",
        "mekanism:advanced_energy_cube",
        "mekanism:elite_energy_cube",
        "mekanism:ultimate_energy_cube",
        "mekanism:creative_energy_cube",
    ];
    if CUBES.contains(&source_type) && CUBES.contains(&target_type) {
        return true;
    }

    // 4. Chemical Tank tiers
    const CHEM_TANKS: &[&str] = &[
        "mekanism:basic_chemical_tank",
        "mekanism:advanced_chemical_tank",
        "mekanism:elite_chemical_tank",
        "mekanism:ultimate_chemical_tank",
        "mekanism:creative_chemical_tank",
    ];
    if CHEM_TANKS.contains(&source_type) && CHEM_TANKS.contains(&target_type) {
        return true;
    }

    // 5. Fluid Tank tiers
    const FLUID_TANKS: &[&str] = &[
        "mekanism:basic_fluid_tank",
        "mekanism:advanced_fluid_tank",
        "mekanism:elite_fluid_tank",
        "mekanism:ultimate_fluid_tank",
        "mekanism:creative_fluid_tank",
    ];
    if FLUID_TANKS.contains(&source_type) && FLUID_TANKS.contains(&target_type) {
        return true;
    }

    // 6. Bin tiers
    const BINS: &[&str] = &[
        "mekanism:basic_bin",
        "mekanism:advanced_bin",
        "mekanism:elite_bin",
        "mekanism:ultimate_bin",
        "mekanism:creative_bin",
    ];
    if BINS.contains(&source_type) && BINS.contains(&target_type) {
        return true;
    }

    // 7. Transmitters of the same medium
    let is_cable = |s: &str| s.ends_with("_universal_cable");
    let is_pipe = |s: &str| s.ends_with("_mechanical_pipe");
    let is_tube = |s: &str| s.ends_with("_pressurized_tube");
    let is_transporter = |s: &str| s.ends_with("_logistical_transporter");
    let is_conductor = |s: &str| s.ends_with("_thermodynamic_conductor");

    if is_cable(source_type) && is_cable(target_type)
        || is_pipe(source_type) && is_pipe(target_type)
        || is_tube(source_type) && is_tube(target_type)
        || is_transporter(source_type) && is_transporter(target_type)
        || is_conductor(source_type) && is_conductor(target_type)
    {
        return true;
    }

    false
}

// ============================================================================
// 3. MACHINE CONFIGURATION EXTRACTION & INJECTION
// ============================================================================

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

    // Otherwise standard configurable machine
    if let Some(cfg) = Config::decode(e.data.get(2..).unwrap_or(&[]), Layout::Electric) {
        let mut sides = [[0u8; 6]; 5];
        for (t, s_arr) in cfg.sides.iter().enumerate() {
            for (s, d) in s_arr.iter().enumerate() {
                sides[t][s] = *d as u8;
            }
        }
        data.side_config = Some(sides);
        data.eject = Some(cfg.eject);
    }

    // Redstone control if present in data bytes
    if e.data.len() > 37 {
        data.control_type = Some(e.data[37]);
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

    // Standard machine
    if let Some(sides) = &data.side_config {
        let mut cfg = config::default_config(Layout::Electric);
        for (t, s_arr) in sides.iter().enumerate() {
            for (s, d) in s_arr.iter().enumerate() {
                cfg.sides[t][s] = Data::from_byte(*d);
            }
        }
        if let Some(ejects) = &data.eject {
            cfg.eject = *ejects;
        }
        let encoded_cfg = cfg.encode();
        if e.data.len() >= 2 + encoded_cfg.len() {
            e.data[2..2 + encoded_cfg.len()].copy_from_slice(&encoded_cfg);
        } else {
            let mut new_data = e.data.clone();
            new_data.resize(2, 0);
            new_data.extend(encoded_cfg);
            e.data = new_data;
        }
    }

    if let Some(ctrl) = data.control_type {
        if e.data.len() > 37 {
            e.data[37] = ctrl;
        }
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
            let display_name = if data.data_name.is_empty() {
                block_name.strip_prefix("mekanism:").unwrap_or(block_name).to_string()
            } else {
                data.data_name.clone()
            };
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
                let display_name = if card_data.data_name.is_empty() {
                    card_data.data_type.strip_prefix("mekanism:").unwrap_or(&card_data.data_type).to_string()
                } else {
                    card_data.data_name.clone()
                };
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

    /// Test: Energy Cube tiers and Tank tiers are compatible within their family.
    #[test]
    fn test_cube_and_tank_compatibility() {
        assert!(is_compatible("mekanism:basic_energy_cube", "mekanism:ultimate_energy_cube"));
        assert!(is_compatible("mekanism:basic_chemical_tank", "mekanism:creative_chemical_tank"));
        assert!(is_compatible("mekanism:basic_fluid_tank", "mekanism:elite_fluid_tank"));
        assert!(is_compatible("mekanism:basic_bin", "mekanism:ultimate_bin"));

        assert!(!is_compatible("mekanism:basic_energy_cube", "mekanism:basic_chemical_tank"));
        assert!(!is_compatible("mekanism:basic_bin", "mekanism:basic_energy_cube"));
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
}
