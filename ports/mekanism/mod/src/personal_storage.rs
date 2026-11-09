//! Personal Storage: Personal Chest and Personal Barrel (`TileEntityPersonalStorage`, `TileEntityPersonalChest`, `TileEntityPersonalBarrel`).
//!
//! 54-slot secure storage containers that can be placed in the world or accessed portably from the player's hand.
//!
//! Java references:
//! - `PersonalStorageManager.java:150-154`: 54 slots (6 rows × 9 columns)
//! - `TileEntityPersonalStorage.java:72-74`: `AutomationType.MANUAL || SecurityMode == PUBLIC`
//! - `TileEntityPersonalStorage.java:110`: `PiglinAi.angerNearbyPiglins(player, true)`
//! - `TileEntityPersonalStorage.java:136`: `persistsToItem = true` (retains inventory when broken)
//! - `TileEntityPersonalBarrel.java:24-31`: `SoundEvents.BARREL_OPEN`, `SoundEvents.BARREL_CLOSE`
//! - `TileEntityPersonalChest.java:28-35`: `SoundEvents.CHEST_OPEN`, `SoundEvents.CHEST_CLOSE`
//! - `TileEntityPersonalChest.java:66-69`: block above being solid blocks opening
//! - `ItemBlockPersonalStorage.java:43-49`: portable use (right-click in hand opens GUI without placing)
//! - `ItemBlockPersonalStorage.java:67`: sneak right-click places the block in the world

use super::*;

pub const CHEST_BLOCK: &str = "mekanism:personal_chest";
pub const BARREL_BLOCK: &str = "mekanism:personal_barrel";

/// Total storage capacity: 54 slots (6 rows × 9 columns) (Java: `PersonalStorageManager.java:150-154`).
pub const TOTAL_SLOTS: usize = 54;
pub const ROWS: usize = 6;
pub const COLUMNS: usize = 9;

/// Sound events.
pub const SOUND_CHEST_OPEN: &str = "minecraft:block.chest.open";
pub const SOUND_CHEST_CLOSE: &str = "minecraft:block.chest.close";
pub const SOUND_BARREL_OPEN: &str = "minecraft:block.barrel.open";
pub const SOUND_BARREL_CLOSE: &str = "minecraft:block.barrel.close";

/// Security access modes (Java: `SecurityMode.java`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SecurityMode {
    #[default]
    Public = 0,
    Private = 1,
    Trusted = 2,
}

impl SecurityMode {
    pub fn from_byte(b: u8) -> Self {
        match b {
            1 => SecurityMode::Private,
            2 => SecurityMode::Trusted,
            _ => SecurityMode::Public,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SecurityMode::Public => "Public",
            SecurityMode::Private => "Private",
            SecurityMode::Trusted => "Trusted",
        }
    }
}

/// State of a Personal Storage block entity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub owner_uuid: Option<String>,
    pub security_mode: SecurityMode,
    pub open_count: u32,
    pub is_barrel: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            owner_uuid: None,
            security_mode: SecurityMode::Public,
            open_count: 0,
            is_barrel: false,
        }
    }
}

pub fn decode(data: &[u8]) -> State {
    if data.is_empty() {
        return State::default();
    }
    let security_mode = SecurityMode::from_byte(data[0]);
    let is_barrel = data.get(1).copied().unwrap_or(0) == 1;
    let open_count = if data.len() >= 6 {
        u32::from_le_bytes([data[2], data[3], data[4], data[5]])
    } else {
        0
    };

    let owner_uuid = if data.len() >= 8 {
        let len = u16::from_le_bytes([data[6], data[7]]) as usize;
        if data.len() >= 8 + len {
            Some(String::from_utf8_lossy(&data[8..8 + len]).to_string())
        } else {
            None
        }
    } else {
        None
    };

    State {
        owner_uuid,
        security_mode,
        open_count,
        is_barrel,
    }
}

pub fn encode(state: &State) -> Vec<u8> {
    let mut out = vec![
        state.security_mode as u8,
        u8::from(state.is_barrel),
    ];
    out.extend(state.open_count.to_le_bytes());
    if let Some(ref owner) = state.owner_uuid {
        let b = owner.as_bytes();
        out.extend((b.len() as u16).to_le_bytes());
        out.extend(b);
    } else {
        out.extend(0u16.to_le_bytes());
    }
    out
}

/// Evaluates player access permission (Java: `TileEntityPersonalStorage.java:72-74`).
pub fn can_player_access(
    player_id: &str,
    owner_uuid: Option<&str>,
    mode: SecurityMode,
    trusted_list: &[String],
) -> bool {
    // If no owner is assigned, anyone can claim/access
    let Some(owner) = owner_uuid else {
        return true;
    };

    // Owner always has full access
    if player_id == owner {
        return true;
    }

    match mode {
        SecurityMode::Public => true,
        SecurityMode::Private => false,
        SecurityMode::Trusted => trusted_list.iter().any(|t| t == player_id),
    }
}

/// Evaluates external automation (hopper, logistical transporter) access permission.
/// Automation is ONLY permitted if the storage is set to PUBLIC (Java: `TileEntityPersonalStorage.java:73`).
pub fn can_automation_access(mode: SecurityMode) -> bool {
    mode == SecurityMode::Public
}

/// Checks whether a Personal Chest can open (Java: `TileEntityPersonalChest.java:66-69`).
/// If the block above the chest is solid/opaque, opening is blocked.
pub fn is_chest_blocked_by_ceiling(solid_block_above: bool) -> bool {
    solid_block_above
}

/// Returns the sound to play when opening or closing.
pub fn sound_for(is_barrel: bool, open: bool) -> &'static str {
    match (is_barrel, open) {
        (true, true) => SOUND_BARREL_OPEN,
        (true, false) => SOUND_BARREL_CLOSE,
        (false, true) => SOUND_CHEST_OPEN,
        (false, false) => SOUND_CHEST_CLOSE,
    }
}

/// Host API bridge for portable inventory opening (opening GUI directly from hand).
// FORK-7 / FORK-23: Hand-held 54-slot container GUI opening API
pub fn open_portable_storage_gui(player_id: &str, is_barrel: bool, slots: &[Option<ItemStack>]) {
    let _ = (player_id, is_barrel, slots);
}

/// Host API bridge for angering piglins on opening personal storage (Java: `PiglinAi.angerNearbyPiglins`).
pub fn anger_nearby_piglins(dimension: &str, pos: BlockPos) {
    let _ = (dimension, pos);
}

impl Mekanism {
    /// Server tick handler for Personal Chests and Personal Barrels.
    pub(crate) fn run_personal_storages(
        &mut self,
        entities: &[&BlockEntity],
    ) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in entities {
            let block = e.state.split('[').next().unwrap_or("");
            let is_chest = block == CHEST_BLOCK;
            let is_barrel = block == BARREL_BLOCK;
            if !is_chest && !is_barrel {
                continue;
            }

            let mut state = decode(&e.data);
            state.is_barrel = is_barrel;

            let Some(click) = self.take_click(e) else { continue };

            // Check if owner is already set; if not, claim ownership
            if state.owner_uuid.is_none() {
                state.owner_uuid = Some(click.player.clone());
            }

            // Check player permission
            let allowed = can_player_access(
                &click.player,
                state.owner_uuid.as_deref(),
                state.security_mode,
                &[], // Trusted list (from player friends/teams)
            );

            if !allowed {
                let _ = players::tell(&click.player, "Access denied: Personal storage is private.");
                continue;
            }

            // For personal chest, check if block above is solid
            if is_chest {
                // In full engine, query world block above (e.pos.x, e.pos.y + 1, e.pos.z)
                // If solid, chest cannot open
            }

            // Trigger angering nearby piglins (Java: `TileEntityPersonalStorage.java:111`)
            anger_nearby_piglins(&e.dimension, e.pos);

            // Open container menu
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: encode(&state),
                items: None,
                energy: None,
                energy_faces: None,
                tanks: None,
                tank_faces: None,
                menu_data: Some(vec![
                    state.security_mode as i32,
                    state.open_count as i32,
                ]),
                transmitter_faces: None,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_count() {
        // Java: PersonalStorageManager.java:150-154
        // 6 rows * 9 cols = 54 slots
        assert_eq!(TOTAL_SLOTS, 54);
        assert_eq!(ROWS * COLUMNS, 54);
    }

    #[test]
    fn test_security_rules_for_players() {
        // Java: TileEntityPersonalStorage.java:72-74
        let owner = "Alice";
        let friend = "Bob";
        let stranger = "Eve";
        let trusted = vec![friend.to_string()];

        // Public mode: everyone can open
        assert!(can_player_access(owner, Some(owner), SecurityMode::Public, &trusted));
        assert!(can_player_access(friend, Some(owner), SecurityMode::Public, &trusted));
        assert!(can_player_access(stranger, Some(owner), SecurityMode::Public, &trusted));

        // Private mode: only owner can open
        assert!(can_player_access(owner, Some(owner), SecurityMode::Private, &trusted));
        assert!(!can_player_access(friend, Some(owner), SecurityMode::Private, &trusted));
        assert!(!can_player_access(stranger, Some(owner), SecurityMode::Private, &trusted));

        // Trusted mode: owner and friends can open
        assert!(can_player_access(owner, Some(owner), SecurityMode::Trusted, &trusted));
        assert!(can_player_access(friend, Some(owner), SecurityMode::Trusted, &trusted));
        assert!(!can_player_access(stranger, Some(owner), SecurityMode::Trusted, &trusted));
    }

    #[test]
    fn test_automation_access_rules() {
        // Java: TileEntityPersonalStorage.java:73
        // Automation (hoppers/transporters) only allowed if PUBLIC
        assert!(can_automation_access(SecurityMode::Public));
        assert!(!can_automation_access(SecurityMode::Private));
        assert!(!can_automation_access(SecurityMode::Trusted));
    }

    #[test]
    fn test_chest_blocked_by_ceiling() {
        // Java: TileEntityPersonalChest.java:66-69
        assert!(is_chest_blocked_by_ceiling(true));
        assert!(!is_chest_blocked_by_ceiling(false));
    }

    #[test]
    fn test_sound_selection() {
        // Java: TileEntityPersonalBarrel.java:24, TileEntityPersonalChest.java:28
        assert_eq!(sound_for(false, true), SOUND_CHEST_OPEN);
        assert_eq!(sound_for(false, false), SOUND_CHEST_CLOSE);
        assert_eq!(sound_for(true, true), SOUND_BARREL_OPEN);
        assert_eq!(sound_for(true, false), SOUND_BARREL_CLOSE);
    }

    #[test]
    fn test_state_serialization() {
        let state = State {
            owner_uuid: Some("player-uuid-1234".to_string()),
            security_mode: SecurityMode::Trusted,
            open_count: 2,
            is_barrel: true,
        };
        let encoded = encode(&state);
        let decoded = decode(&encoded);

        assert_eq!(decoded.owner_uuid.as_deref(), Some("player-uuid-1234"));
        assert_eq!(decoded.security_mode, SecurityMode::Trusted);
        assert_eq!(decoded.open_count, 2);
        assert!(decoded.is_barrel);
    }
}
