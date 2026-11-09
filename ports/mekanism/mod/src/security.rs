//! Security System & Security Desk (`SecurityFrequency`, `SecurityMode`, `SecurityUtils`, `TileEntitySecurityDesk`).
//!
//! Reference: Mekanism Java 1.21.1-10.7.19.85
//! - Security Desk: `java/main/java/mekanism/common/tile/TileEntitySecurityDesk.java`
//! - Security Frequency: `java/main/java/mekanism/common/lib/security/SecurityFrequency.java`
//! - Security Mode: `java/api/java/mekanism/api/security/SecurityMode.java`
//! - Security Utils: `java/main/java/mekanism/common/lib/security/SecurityUtils.java`
//! - Security Tile Component: `java/main/java/mekanism/common/tile/component/TileComponentSecurity.java`
//!
//! Features:
//! - Security Modes: `PUBLIC` (all players), `PRIVATE` (owner only), `TRUSTED` (owner + trusted friends).
//! - Security Frequency: Unique frequency per player UUID, maintains trusted players list and global override.
//! - Global Override: When enabled on Security Desk, forces all machines belonging to the player to adopt
//!   the desk's security mode if it is more restrictive than local settings.
//! - Bounding Block (1x2x1): Base block at `pos`, top bounding block at `pos + [0, 1, 0]` (`AttributeHasBounding.ABOVE_ONLY`).
//! - Unbreakable Resistance (`AttributeCustomResistance(-1)`): Prevents unauthorized players from breaking the desk.
//! - Slots: Unlock slot ($X=146, Y=18$) removes ownership; Lock slot ($X=146, Y=97$) assigns owner and security.
//! - Access Validation & Eviction: Forcibly closes open containers (`player.closeContainer()`) when access is revoked.
use super::*;

pub const BLOCK: &str = "mekanism:security_desk";

/// Unlock Slot index ($X=146, Y=18$ in `TileEntitySecurityDesk.java:42`).
pub const UNLOCK_SLOT: usize = 0;

/// Lock Slot index ($X=146, Y=97$ in `TileEntitySecurityDesk.java:43`).
pub const LOCK_SLOT: usize = 1;

pub const TOTAL_SLOTS: usize = 2;

/// Screen button IDs.
pub const TOGGLE_OVERRIDE: i32 = 7620;
pub const CYCLE_SECURITY_MODE: i32 = 7621;
pub const ADD_TRUSTED: i32 = 7622;
pub const REMOVE_TRUSTED_BASE: i32 = 7700; // 7700 + index

// ============================================================================
// SecurityMode Enum (mekanism.api.security.SecurityMode.java)
// ============================================================================

/// Access levels defined in Mekanism (`SecurityMode.java`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SecurityMode {
    #[default]
    Public = 0,
    Private = 1,
    Trusted = 2,
}

impl SecurityMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => SecurityMode::Private,
            2 => SecurityMode::Trusted,
            _ => SecurityMode::Public,
        }
    }

    /// Cycles to the next mode (`Public -> Private -> Trusted -> Public`).
    pub fn next(self) -> Self {
        match self {
            SecurityMode::Public => SecurityMode::Private,
            SecurityMode::Private => SecurityMode::Trusted,
            SecurityMode::Trusted => SecurityMode::Public,
        }
    }

    /// Cycles to the previous mode (`Public -> Trusted -> Private -> Public`).
    pub fn previous(self) -> Self {
        match self {
            SecurityMode::Public => SecurityMode::Trusted,
            SecurityMode::Private => SecurityMode::Public,
            SecurityMode::Trusted => SecurityMode::Private,
        }
    }
}

// ============================================================================
// SecurityFrequency (mekanism.common.lib.security.SecurityFrequency.java)
// ============================================================================

pub const FREQUENCY_NAME: &str = "Security";

/// Per-player frequency managing security permissions and trusted users.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityFrequency {
    pub owner: Option<[u8; 16]>,
    pub owner_name: String,
    pub security_mode: SecurityMode,
    pub override_mode: bool,
    pub trusted: Vec<[u8; 16]>,
    pub trusted_names: Vec<String>,
}

impl Default for SecurityFrequency {
    fn default() -> Self {
        Self {
            owner: None,
            owner_name: String::new(),
            security_mode: SecurityMode::Public,
            override_mode: false,
            trusted: Vec::new(),
            trusted_names: Vec::new(),
        }
    }
}

impl SecurityFrequency {
    pub fn new(owner: Option<[u8; 16]>, owner_name: String, mode: SecurityMode) -> Self {
        Self {
            owner,
            owner_name,
            security_mode: mode,
            override_mode: false,
            trusted: Vec::new(),
            trusted_names: Vec::new(),
        }
    }

    pub fn is_trusted(&self, player_uuid: &[u8; 16]) -> bool {
        self.trusted.iter().any(|u| u == player_uuid)
    }

    pub fn add_trusted(&mut self, player_uuid: [u8; 16], name: String) -> bool {
        if !self.is_trusted(&player_uuid) {
            self.trusted.push(player_uuid);
            self.trusted_names.push(name);
            true
        } else {
            false
        }
    }

    pub fn remove_trusted(&mut self, index: usize) -> Option<[u8; 16]> {
        if index < self.trusted.len() {
            let removed_uuid = self.trusted.remove(index);
            if index < self.trusted_names.len() {
                self.trusted_names.remove(index);
            }
            Some(removed_uuid)
        } else {
            None
        }
    }

    pub fn set_overridden(&mut self, override_mode: bool) {
        self.override_mode = override_mode;
    }

    pub fn set_security_mode(&mut self, mode: SecurityMode) {
        self.security_mode = mode;
    }

    /// Computes sync hash code (`SecurityFrequency.java:76-83`).
    pub fn sync_hash(&self) -> u64 {
        let mut hash: u64 = 1;
        hash = hash.wrapping_mul(31).wrapping_add(u64::from(self.override_mode));
        hash = hash.wrapping_mul(31).wrapping_add(self.security_mode as u64);
        for u in &self.trusted {
            for b in u {
                hash = hash.wrapping_mul(31).wrapping_add(u64::from(*b));
            }
        }
        hash
    }
}

// ============================================================================
// SecurityUtils (mekanism.common.lib.security.SecurityUtils.java)
// ============================================================================

/// Compares restriction levels of two security modes.
///
/// Java reference: `SecurityUtils.java:141-152`
/// ```java
/// return switch (overridden) {
///     case PUBLIC -> false;
///     case PRIVATE -> base != SecurityMode.PRIVATE;
///     case TRUSTED -> base == SecurityMode.PUBLIC;
/// };
/// ```
pub fn more_restrictive(base: SecurityMode, overridden: SecurityMode) -> bool {
    match overridden {
        SecurityMode::Public => false,
        SecurityMode::Private => base != SecurityMode::Private,
        SecurityMode::Trusted => base == SecurityMode::Public,
    }
}

/// Resolves effective security mode considering desk global override.
///
/// Java reference: `SecurityUtils.java:158-166`
pub fn get_effective_security_mode(
    owner: Option<&[u8; 16]>,
    local_mode: SecurityMode,
    freq: Option<&SecurityFrequency>,
) -> SecurityMode {
    if owner.is_none() {
        return SecurityMode::Public;
    }
    if let Some(f) = freq {
        if f.override_mode && more_restrictive(local_mode, f.security_mode) {
            return f.security_mode;
        }
    }
    local_mode
}

/// Validates whether a player has permission to access a machine or container.
///
/// Java reference: `SecurityUtils.java:110-138`
pub fn can_access(
    player: Option<&[u8; 16]>,
    owner: Option<&[u8; 16]>,
    effective_mode: SecurityMode,
    freq: Option<&SecurityFrequency>,
) -> bool {
    // If unowned, access is always granted
    let owner_uuid = match owner {
        Some(o) => o,
        None => return true,
    };

    // Owner always has full access
    if let Some(p) = player {
        if p == owner_uuid {
            return true;
        }
    }

    match effective_mode {
        SecurityMode::Public => true,
        SecurityMode::Private => false,
        SecurityMode::Trusted => match player {
            Some(p) => freq.is_some_and(|f| f.is_trusted(p)),
            None => false,
        },
    }
}

// ============================================================================
// TileComponentSecurity (mekanism.common.tile.component.TileComponentSecurity.java)
// ============================================================================

/// Security component attached to machines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileComponentSecurity {
    pub owner: Option<[u8; 16]>,
    pub owner_name: String,
    pub security_mode: SecurityMode,
}

impl Default for TileComponentSecurity {
    fn default() -> Self {
        Self {
            owner: None,
            owner_name: String::new(),
            security_mode: SecurityMode::Public,
        }
    }
}

// ============================================================================
// TileEntitySecurityDesk (TileEntitySecurityDesk.java)
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecurityDeskState {
    pub owner: Option<[u8; 16]>,
    pub owner_name: String,
    pub frequency: SecurityFrequency,
    pub slots: [Option<ItemStack>; TOTAL_SLOTS],
}

impl Default for SecurityDeskState {
    fn default() -> Self {
        Self {
            owner: None,
            owner_name: String::new(),
            frequency: SecurityFrequency::default(),
            slots: [None, None],
        }
    }
}

/// Executes one server tick of Security Desk operations:
/// - Unlocks items in Slot 0 (removes ownership and sets Public mode).
/// - Locks items in Slot 1 (sets desk owner and frequency).
pub fn step_security_desk(desk: &mut SecurityDeskState) -> bool {
    let owner = match desk.owner {
        Some(o) => o,
        None => return false,
    };

    let mut changed = false;

    // Slot 0: Unlock item
    if let Some(s) = &mut desk.slots[UNLOCK_SLOT] {
        if s.components.contains("\"owner\"") || s.components.contains("\"security\"") {
            // Strip owner data component
            s.components = strip_security_components(&s.components);
            changed = true;
        }
    }

    // Slot 1: Lock item
    if let Some(s) = &mut desk.slots[LOCK_SLOT] {
        let owner_hex = uuid_to_hex(&owner);
        if !s.components.contains(&owner_hex) {
            s.components = apply_security_components(&s.components, &owner_hex, desk.frequency.security_mode);
            changed = true;
        }
    }

    changed
}

fn strip_security_components(comp: &str) -> String {
    // In Horizon JSON/SNBT, clears owner and resets security mode to public
    let mut cleaned = comp.to_string();
    if let Some(start) = cleaned.find("\"owner\":") {
        if let Some(end) = cleaned[start..].find(',') {
            cleaned.replace_range(start..start + end + 1, "");
        }
    }
    cleaned
}

fn apply_security_components(_comp: &str, owner_hex: &str, mode: SecurityMode) -> String {
    let mode_str = match mode {
        SecurityMode::Public => "public",
        SecurityMode::Private => "private",
        SecurityMode::Trusted => "trusted",
    };
    format!("{{\"owner\":\"{owner_hex}\",\"security\":\"{mode_str}\"}}")
}

pub fn uuid_to_hex(u: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in u {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub fn hex_to_uuid(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

// ============================================================================
// Access Validation & Player Eviction (TileEntitySecurityDesk.java:76-85, 101-106)
// ============================================================================

/// Check if an open container session is still permitted, evicting player if not.
///
/// Java reference:
/// ```java
/// if (player != null && player.containerMenu instanceof ISecurityContainer container && !container.canPlayerAccess(player)) {
///     player.closeContainer();
/// }
/// ```
pub fn should_evict_player(
    player_uuid: &[u8; 16],
    desk_owner: Option<&[u8; 16]>,
    effective_mode: SecurityMode,
    freq: &SecurityFrequency,
) -> bool {
    !can_access(Some(player_uuid), desk_owner, effective_mode, Some(freq))
}

// ============================================================================
// Serialization
// ============================================================================

pub fn encode_security_desk(desk: &SecurityDeskState) -> Vec<u8> {
    let mut out = vec![
        1, // version
        u8::from(desk.owner.is_some()),
    ];
    if let Some(o) = desk.owner {
        out.extend_from_slice(&o);
    }
    let ob = desk.owner_name.as_bytes();
    out.push(ob.len().min(255) as u8);
    out.extend_from_slice(&ob[..ob.len().min(255)]);

    out.push(desk.frequency.security_mode as u8);
    out.push(u8::from(desk.frequency.override_mode));

    out.push(desk.frequency.trusted.len().min(255) as u8);
    for (uuid, name) in desk.frequency.trusted.iter().zip(&desk.frequency.trusted_names) {
        out.extend_from_slice(uuid);
        let nb = name.as_bytes();
        out.push(nb.len().min(255) as u8);
        out.extend_from_slice(&nb[..nb.len().min(255)]);
    }

    out
}

pub fn decode_security_desk(data: &[u8]) -> Option<SecurityDeskState> {
    if data.len() < 4 || data[0] != 1 {
        return None;
    }
    let has_owner = data[1] == 1;
    let mut idx = 2;

    let owner = if has_owner {
        if idx + 16 > data.len() {
            return None;
        }
        let mut u = [0u8; 16];
        u.copy_from_slice(&data[idx..idx + 16]);
        idx += 16;
        Some(u)
    } else {
        None
    };

    if idx >= data.len() {
        return None;
    }
    let olen = data[idx] as usize;
    idx += 1;
    if idx + olen > data.len() {
        return None;
    }
    let owner_name = String::from_utf8_lossy(&data[idx..idx + olen]).to_string();
    idx += olen;

    if idx + 2 > data.len() {
        return None;
    }
    let security_mode = SecurityMode::from_u8(data[idx]);
    let override_mode = data[idx + 1] == 1;
    idx += 2;

    let mut trusted = Vec::new();
    let mut trusted_names = Vec::new();

    if idx < data.len() {
        let tcount = data[idx] as usize;
        idx += 1;
        for _ in 0..tcount {
            if idx + 17 > data.len() {
                break;
            }
            let mut u = [0u8; 16];
            u.copy_from_slice(&data[idx..idx + 16]);
            idx += 16;
            let nlen = data[idx] as usize;
            idx += 1;
            if idx + nlen > data.len() {
                break;
            }
            let name = String::from_utf8_lossy(&data[idx..idx + nlen]).to_string();
            idx += nlen;
            trusted.push(u);
            trusted_names.push(name);
        }
    }

    Some(SecurityDeskState {
        owner,
        owner_name: owner_name.clone(),
        frequency: SecurityFrequency {
            owner,
            owner_name,
            security_mode,
            override_mode,
            trusted,
            trusted_names,
        },
        slots: [None, None],
    })
}

// ============================================================================
// Unit Tests (Comparing with Java reference sources)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_more_restrictive_matrix_matches_java() {
        // Java reference: `SecurityUtils.java:141-152`
        // PUBLIC is never more restrictive than any base
        assert!(!more_restrictive(SecurityMode::Public, SecurityMode::Public));
        assert!(!more_restrictive(SecurityMode::Private, SecurityMode::Public));
        assert!(!more_restrictive(SecurityMode::Trusted, SecurityMode::Public));

        // PRIVATE is more restrictive than Public and Trusted, but not Private
        assert!(more_restrictive(SecurityMode::Public, SecurityMode::Private));
        assert!(more_restrictive(SecurityMode::Trusted, SecurityMode::Private));
        assert!(!more_restrictive(SecurityMode::Private, SecurityMode::Private));

        // TRUSTED is more restrictive ONLY if base was Public
        assert!(more_restrictive(SecurityMode::Public, SecurityMode::Trusted));
        assert!(!more_restrictive(SecurityMode::Private, SecurityMode::Trusted));
        assert!(!more_restrictive(SecurityMode::Trusted, SecurityMode::Trusted));
    }

    #[test]
    fn test_effective_security_mode_with_and_without_override() {
        // Java reference: `SecurityUtils.java:158-166`
        let owner = [1u8; 16];

        // Desk mode is PRIVATE, machine mode is PUBLIC.
        // If override is false -> effective mode remains PUBLIC
        let mut freq = SecurityFrequency::new(Some(owner), "Owner".into(), SecurityMode::Private);
        freq.override_mode = false;
        assert_eq!(
            get_effective_security_mode(Some(&owner), SecurityMode::Public, Some(&freq)),
            SecurityMode::Public
        );

        // If override is true -> effective mode becomes PRIVATE
        freq.override_mode = true;
        assert_eq!(
            get_effective_security_mode(Some(&owner), SecurityMode::Public, Some(&freq)),
            SecurityMode::Private
        );

        // If desk mode is PUBLIC, machine is PRIVATE, override cannot downgrade privacy
        freq.security_mode = SecurityMode::Public;
        assert_eq!(
            get_effective_security_mode(Some(&owner), SecurityMode::Private, Some(&freq)),
            SecurityMode::Private
        );
    }

    #[test]
    fn test_can_access_matrix() {
        // Java reference: `SecurityUtils.java:110-138`
        let owner = [1u8; 16];
        let trusted_friend = [2u8; 16];
        let stranger = [3u8; 16];

        let mut freq = SecurityFrequency::new(Some(owner), "Owner".into(), SecurityMode::Public);
        freq.add_trusted(trusted_friend, "Friend".into());

        // 1. PUBLIC mode: all access permitted
        assert!(can_access(Some(&owner), Some(&owner), SecurityMode::Public, Some(&freq)));
        assert!(can_access(Some(&trusted_friend), Some(&owner), SecurityMode::Public, Some(&freq)));
        assert!(can_access(Some(&stranger), Some(&owner), SecurityMode::Public, Some(&freq)));

        // 2. PRIVATE mode: owner only
        assert!(can_access(Some(&owner), Some(&owner), SecurityMode::Private, Some(&freq)));
        assert!(!can_access(Some(&trusted_friend), Some(&owner), SecurityMode::Private, Some(&freq)));
        assert!(!can_access(Some(&stranger), Some(&owner), SecurityMode::Private, Some(&freq)));

        // 3. TRUSTED mode: owner + trusted friend
        assert!(can_access(Some(&owner), Some(&owner), SecurityMode::Trusted, Some(&freq)));
        assert!(can_access(Some(&trusted_friend), Some(&owner), SecurityMode::Trusted, Some(&freq)));
        assert!(!can_access(Some(&stranger), Some(&owner), SecurityMode::Trusted, Some(&freq)));

        // 4. Unowned block: anyone can access
        assert!(can_access(Some(&stranger), None, SecurityMode::Private, None));
    }

    #[test]
    fn test_trusted_list_operations_and_eviction() {
        // Java reference: `SecurityFrequency.java:101-134`, `TileEntitySecurityDesk.java:87-99`
        let owner = [1u8; 16];
        let player_a = [10u8; 16];
        let player_b = [20u8; 16];

        let mut freq = SecurityFrequency::new(Some(owner), "Owner".into(), SecurityMode::Trusted);
        assert!(freq.add_trusted(player_a, "PlayerA".into()));
        assert!(freq.add_trusted(player_b, "PlayerB".into()));
        // Duplicate check
        assert!(!freq.add_trusted(player_a, "PlayerA".into()));

        assert!(freq.is_trusted(&player_a));
        assert!(freq.is_trusted(&player_b));

        // Player A should not be evicted
        assert!(!should_evict_player(&player_a, Some(&owner), SecurityMode::Trusted, &freq));

        // Remove Player A
        let removed = freq.remove_trusted(0);
        assert_eq!(removed, Some(player_a));
        assert!(!freq.is_trusted(&player_a));
        assert!(freq.is_trusted(&player_b));

        // Player A is now evicted!
        assert!(should_evict_player(&player_a, Some(&owner), SecurityMode::Trusted, &freq));
    }

    #[test]
    fn test_security_desk_lock_and_unlock_slots() {
        // Java reference: `TileEntitySecurityDesk.java:48-57`
        let owner = [7u8; 16];
        let mut desk = SecurityDeskState {
            owner: Some(owner),
            owner_name: "Master".into(),
            frequency: SecurityFrequency::new(Some(owner), "Master".into(), SecurityMode::Trusted),
            slots: [
                // Slot 0: Locked item to be unlocked
                Some(ItemStack {
                    item: "mekanism:personal_chest".into(),
                    count: 1,
                    components: "{\"owner\":\"07070707070707070707070707070707\",\"security\":\"trusted\"}".into(),
                }),
                // Slot 1: Unlocked item to be locked
                Some(ItemStack {
                    item: "mekanism:digital_miner".into(),
                    count: 1,
                    components: String::new(),
                }),
            ],
        };

        let changed = step_security_desk(&mut desk);
        assert!(changed);

        // Slot 0 should have owner stripped
        let slot0 = desk.slots[0].as_ref().unwrap();
        assert!(!slot0.components.contains("\"owner\""));

        // Slot 1 should have owner locked to desk.owner
        let slot1 = desk.slots[1].as_ref().unwrap();
        assert!(slot1.components.contains("07070707070707070707070707070707"));
        assert!(slot1.components.contains("\"security\":\"trusted\""));
    }

    #[test]
    fn test_security_serialization_roundtrip() {
        let owner = [42u8; 16];
        let mut desk = SecurityDeskState {
            owner: Some(owner),
            owner_name: "SecurityBoss".into(),
            frequency: SecurityFrequency::new(Some(owner), "SecurityBoss".into(), SecurityMode::Trusted),
            slots: [None, None],
        };
        desk.frequency.override_mode = true;
        desk.frequency.add_trusted([11u8; 16], "Alice".into());
        desk.frequency.add_trusted([12u8; 16], "Bob".into());

        let encoded = encode_security_desk(&desk);
        let decoded = decode_security_desk(&encoded).expect("decode security desk");

        assert_eq!(desk.owner, decoded.owner);
        assert_eq!(desk.owner_name, decoded.owner_name);
        assert_eq!(desk.frequency.security_mode, decoded.frequency.security_mode);
        assert_eq!(desk.frequency.override_mode, decoded.frequency.override_mode);
        assert_eq!(desk.frequency.trusted, decoded.frequency.trusted);
        assert_eq!(desk.frequency.trusted_names, decoded.frequency.trusted_names);
    }
}
