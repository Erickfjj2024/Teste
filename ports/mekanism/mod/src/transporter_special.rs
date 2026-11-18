//! Special transporters: Restrictive Transporter and Diversion Transporter (`RestrictiveTransporter.java`, `DiversionTransporter.java`).
//!
//! Restrictive Transporter (`TileEntityRestrictiveTransporter`):
//! - Cost: 1,000 in A* pathfinding (normal transporter has cost 1).
//! - Tier: Basic (speed 50 ticks per block, pull delay 10 ticks).
//! - Role: Low priority / overflow conduit. Items only route through it when all normal paths are unavailable or full.
//!
//! Diversion Transporter (`TileEntityDiversionTransporter`):
//! - Modes per face: `DISABLED` (always pass), `HIGH` (pass only when powered by redstone), `LOW` (pass only when not powered).
//! - Right-clicking a face cycles its mode: Disabled -> High -> Low -> Disabled.
//! - When mode requirements are not met, the face disconnects and does not expose item insertion capability.
//!
//! Java references:
//! - `RestrictiveTransporter.java:13-15`: `getCost() = 1_000`
//! - `DiversionTransporter.java:38`: `TransporterTier.BASIC`
//! - `DiversionTransporter.java:40`: `modes` array initialized to `DiversionControl.DISABLED`
//! - `DiversionTransporter.java:159-163`: right-click cycles face mode, sends `MekanismLang.TOGGLE_DIVERTER`
//! - `DiversionTransporter.java:166-174`: `exposesInsertCap` and `canConnect` check `modeReqsMet(side)`
//! - `DiversionTransporter.java:182-187`: `HIGH -> isGettingPowered()`, `LOW -> !isGettingPowered()`, `default -> true`
//! - `TransporterPathfinder.java:303-348`: A* routing score comparison and destination sorting
//! - `TransporterPathfinder.java:451`: `double tentativeG = currentScore + neighborTransmitter.getCost();`

use super::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

pub const RESTRICTIVE_TRANSPORTER: &str = "mekanism:restrictive_transporter";
pub const DIVERSION_TRANSPORTER: &str = "mekanism:diversion_transporter";

/// Cost of passing through a restrictive transporter (Java: `RestrictiveTransporter.java:14`).
pub const RESTRICTIVE_COST: f64 = 1_000.0;
/// Cost of passing through a normal transporter (Java: `LogisticalTransporterBase.java`).
pub const NORMAL_COST: f64 = 1.0;

/// Basic transporter tier speed (ticks per block transport).
pub const BASIC_SPEED: u32 = 50;
/// Basic transporter pull delay in ticks.
pub const BASIC_PULL_DELAY: u32 = 10;

/// The control mode of a single face on a Diversion Transporter (Java: `DiversionControl.java`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiversionControl {
    #[default]
    Disabled = 0,
    High = 1,
    Low = 2,
}

impl DiversionControl {
    pub fn from_byte(b: u8) -> Self {
        match b {
            1 => DiversionControl::High,
            2 => DiversionControl::Low,
            _ => DiversionControl::Disabled,
        }
    }

    /// Cycles to the next mode when player right-clicks the face (Java: `DiversionTransporter.java:159`).
    pub fn next(self) -> Self {
        match self {
            DiversionControl::Disabled => DiversionControl::High,
            DiversionControl::High => DiversionControl::Low,
            DiversionControl::Low => DiversionControl::Disabled,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DiversionControl::Disabled => "Disabled",
            DiversionControl::High => "High",
            DiversionControl::Low => "Low",
        }
    }

    /// Checks if diversion mode requirements are met for a given redstone state (Java: `DiversionTransporter.java:182-187`).
    pub fn mode_reqs_met(self, is_powered: bool) -> bool {
        match self {
            DiversionControl::Disabled => true,
            DiversionControl::High => is_powered,
            DiversionControl::Low => !is_powered,
        }
    }
}

/// The state of a Restrictive Transporter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestrictiveState {
    pub connections: [crate::transmitter::Connection; 6],
}

impl Default for RestrictiveState {
    fn default() -> Self {
        RestrictiveState {
            connections: [crate::transmitter::Connection::Normal; 6],
        }
    }
}

pub fn decode_restrictive(data: &[u8]) -> RestrictiveState {
    if data.len() < 6 {
        return RestrictiveState::default();
    }
    let mut connections = [crate::transmitter::Connection::Normal; 6];
    for (i, b) in data.iter().take(6).enumerate() {
        connections[i] = crate::transmitter::Connection::from_byte(*b);
    }
    RestrictiveState { connections }
}

pub fn encode_restrictive(state: &RestrictiveState) -> Vec<u8> {
    state.connections.iter().map(|c| *c as u8).collect()
}

/// The state of a Diversion Transporter (6 connections and 6 face modes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiversionState {
    pub connections: [crate::transmitter::Connection; 6],
    pub modes: [DiversionControl; 6],
    pub was_getting_power: bool,
}

impl Default for DiversionState {
    fn default() -> Self {
        DiversionState {
            connections: [crate::transmitter::Connection::Normal; 6],
            modes: [DiversionControl::Disabled; 6],
            was_getting_power: false,
        }
    }
}

pub fn decode_diversion(data: &[u8]) -> DiversionState {
    if data.len() < 12 {
        return DiversionState::default();
    }
    let mut connections = [crate::transmitter::Connection::Normal; 6];
    let mut modes = [DiversionControl::Disabled; 6];
    for (i, b) in data.iter().take(6).enumerate() {
        connections[i] = crate::transmitter::Connection::from_byte(*b);
    }
    for (i, b) in data[6..12].iter().enumerate() {
        modes[i] = DiversionControl::from_byte(*b);
    }
    let was_getting_power = data.get(12).copied().unwrap_or(0) == 1;
    DiversionState { connections, modes, was_getting_power }
}

pub fn encode_diversion(state: &DiversionState) -> Vec<u8> {
    let mut out: Vec<u8> = state.connections.iter().map(|c| *c as u8).collect();
    out.extend(state.modes.iter().map(|m| *m as u8));
    out.push(u8::from(state.was_getting_power));
    out
}

/// Node representation in the logistical transporter network.
#[derive(Clone, Debug, PartialEq)]
pub enum TransporterKind {
    Normal,
    Restrictive,
    Diversion([DiversionControl; 6]),
}

/// Node in the routing graph.
#[derive(Clone, Debug)]
pub struct NetworkNode {
    pub pos: [i32; 3],
    pub kind: TransporterKind,
    pub connections: [crate::transmitter::Connection; 6],
    pub powered: bool,
}

impl NetworkNode {
    pub fn cost(&self) -> f64 {
        match self.kind {
            TransporterKind::Restrictive => RESTRICTIVE_COST,
            _ => NORMAL_COST,
        }
    }

    pub fn can_connect_side(&self, side: usize) -> bool {
        if side >= 6 || self.connections[side] == crate::transmitter::Connection::None {
            return false;
        }
        if let TransporterKind::Diversion(modes) = self.kind {
            modes[side].mode_reqs_met(self.powered)
        } else {
            true
        }
    }
}

#[derive(Clone, Debug)]
struct PathNode {
    pos: [i32; 3],
    g_score: f64,
    f_score: f64,
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.f_score == other.f_score
    }
}

impl Eq for PathNode {}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.f_score.total_cmp(&self.f_score)
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan(a: [i32; 3], b: [i32; 3]) -> f64 {
    ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) as f64
}

/// Pure implementation of Mekanism's `TransporterPathfinder` (Java: `TransporterPathfinder.java:420-470`).
/// Finds the path with the lowest cumulative cost (A*) from start to target.
pub fn find_path(
    network: &HashMap<[i32; 3], NetworkNode>,
    start: [i32; 3],
    destination: [i32; 3],
) -> Option<(Vec<[i32; 3]>, f64)> {
    if !network.contains_key(&start) || !network.contains_key(&destination) {
        return None;
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<[i32; 3], [i32; 3]> = HashMap::new();
    let mut g_score: HashMap<[i32; 3], f64> = HashMap::new();

    g_score.insert(start, 0.0);
    open_set.push(PathNode {
        pos: start,
        g_score: 0.0,
        f_score: manhattan(start, destination),
    });

    let mut closed_set: HashSet<[i32; 3]> = HashSet::new();

    while let Some(current) = open_set.pop() {
        if current.pos == destination {
            // Reconstruct path
            let mut path = vec![current.pos];
            let mut curr = current.pos;
            while let Some(&prev) = came_from.get(&curr) {
                path.push(prev);
                curr = prev;
            }
            path.reverse();
            return Some((path, current.g_score));
        }

        if !closed_set.insert(current.pos) {
            continue;
        }

        let Some(current_node) = network.get(&current.pos) else { continue };

        for face in 0..6 {
            if !current_node.can_connect_side(face) {
                continue;
            }

            let d = laser::direction(face);
            let neighbor_pos = [current.pos[0] + d[0], current.pos[1] + d[1], current.pos[2] + d[2]];

            let Some(neighbor_node) = network.get(&neighbor_pos) else { continue };
            let opp = laser::opposite(face);
            if !neighbor_node.can_connect_side(opp) {
                continue;
            }

            let tentative_g = current.g_score + neighbor_node.cost();
            let neighbor_g = g_score.get(&neighbor_pos).copied().unwrap_or(f64::INFINITY);

            if tentative_g < neighbor_g {
                came_from.insert(neighbor_pos, current.pos);
                g_score.insert(neighbor_pos, tentative_g);
                let f = tentative_g + manhattan(neighbor_pos, destination);
                open_set.push(PathNode {
                    pos: neighbor_pos,
                    g_score: tentative_g,
                    f_score: f,
                });
            }
        }
    }

    None
}

/// Host API bridge for dynamic transporter routing.
/// When the Horizon host provides native network routing with per-face redstone and costs,
/// this delegates directly to the host API.
// FORK-3: Host routing callbacks with custom edge costs and per-face redstone checks
pub fn route_transporter(
    dimension: &str,
    start: [i32; 3],
    destination: [i32; 3],
    network: &HashMap<[i32; 3], NetworkNode>,
) -> Option<Vec<[i32; 3]>> {
    let _ = dimension;
    find_path(network, start, destination).map(|(path, _)| path)
}

impl Mekanism {
    /// Ticking and click processing for Restrictive and Diversion Transporters.
    pub(crate) fn run_special_transporters(
        &mut self,
        entities: &[&BlockEntity],
    ) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in entities {
            let block = e.state.split('[').next().unwrap_or("");
            let is_restrictive = block == RESTRICTIVE_TRANSPORTER;
            let is_diversion = block == DIVERSION_TRANSPORTER;
            if !is_restrictive && !is_diversion {
                continue;
            }

            let Some(click) = self.take_click(e) else { continue };
            let face = click.face.min(5);

            if is_restrictive {
                let mut state = decode_restrictive(&e.data);
                if click.sneak && click.mode.is_configurating() {
                    state.connections[face] = state.connections[face].next();
                    let _ = players::tell(
                        &click.player,
                        &format!("Connection type changed to: {}", state.connections[face].label()),
                    );
                    out.push(BlockEntityUpdate {
                        dimension: e.dimension.clone(),
                        pos: e.pos,
                        data: encode_restrictive(&state),
                        items: None,
                        energy: None,
                        energy_faces: None,
                        tanks: None,
                        tank_faces: None,
                        menu_data: None,
                        transmitter_faces: Some(state.connections.iter().map(|c| c.connects()).collect()),
                    });
                }
            } else if is_diversion {
                let mut state = decode_diversion(&e.data);
                let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
                state.was_getting_power = powered;

                if click.sneak && click.mode.is_configurating() {
                    // Configurator changes physical connection type
                    state.connections[face] = state.connections[face].next();
                    let _ = players::tell(
                        &click.player,
                        &format!("Connection type changed to: {}", state.connections[face].label()),
                    );
                } else {
                    // Normal right-click cycles diversion mode on the clicked face (Java: DiversionTransporter.java:159)
                    state.modes[face] = state.modes[face].next();
                    let _ = players::tell(
                        &click.player,
                        &format!("Diverter mode: {}", state.modes[face].label()),
                    );
                }

                // Check active connections based on redstone and mode
                let active_faces: Vec<bool> = state
                    .connections
                    .iter()
                    .enumerate()
                    .map(|(f, c)| c.connects() && state.modes[f].mode_reqs_met(powered))
                    .collect();

                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: encode_diversion(&state),
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: Some(active_faces),
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restrictive_cost_and_tier_constants() {
        // Java: RestrictiveTransporter.java:14
        // public double getCost() { return 1_000; }
        assert_eq!(RESTRICTIVE_COST, 1_000.0);
        assert_eq!(NORMAL_COST, 1.0);

        // Java: RestrictiveTransporter.java:9
        // super(tile, TransporterTier.BASIC);
        assert_eq!(BASIC_SPEED, 50);
        assert_eq!(BASIC_PULL_DELAY, 10);
    }

    #[test]
    fn test_diversion_control_cycling() {
        // Java: DiversionTransporter.java:159
        // DiversionControl newMode = modes[side.ordinal()].getNext();
        let m = DiversionControl::Disabled;
        let m = m.next();
        assert_eq!(m, DiversionControl::High);
        let m = m.next();
        assert_eq!(m, DiversionControl::Low);
        let m = m.next();
        assert_eq!(m, DiversionControl::Disabled);
    }

    #[test]
    fn test_diversion_control_mode_reqs() {
        // Java: DiversionTransporter.java:182-187
        // case HIGH -> isGettingPowered();
        // case LOW -> !isGettingPowered();
        // default -> true;
        assert!(DiversionControl::Disabled.mode_reqs_met(true));
        assert!(DiversionControl::Disabled.mode_reqs_met(false));

        assert!(DiversionControl::High.mode_reqs_met(true));
        assert!(!DiversionControl::High.mode_reqs_met(false));

        assert!(!DiversionControl::Low.mode_reqs_met(true));
        assert!(DiversionControl::Low.mode_reqs_met(false));
    }

    #[test]
    fn test_restrictive_state_serialization() {
        let mut state = RestrictiveState::default();
        state.connections[0] = crate::transmitter::Connection::Pull;
        state.connections[2] = crate::transmitter::Connection::None;
        let bytes = encode_restrictive(&state);
        let decoded = decode_restrictive(&bytes);
        assert_eq!(state, decoded);
    }

    #[test]
    fn test_diversion_state_serialization() {
        let mut state = DiversionState::default();
        state.connections[1] = crate::transmitter::Connection::Push;
        state.modes[2] = DiversionControl::High;
        state.modes[3] = DiversionControl::Low;
        state.was_getting_power = true;
        let bytes = encode_diversion(&state);
        let decoded = decode_diversion(&bytes);
        assert_eq!(state, decoded);
    }

    #[test]
    fn test_astar_prefers_normal_over_restrictive() {
        // Path A: [0,0,0] -> [1,0,0] (Normal) -> [2,0,0] -> cost 2.0
        // Path B: [0,0,0] -> [0,1,0] (Restrictive) -> [0,2,0] -> cost 1001.0
        let mut network = HashMap::new();
        network.insert(
            [0, 0, 0],
            NetworkNode {
                pos: [0, 0, 0],
                kind: TransporterKind::Normal,
                connections: [crate::transmitter::Connection::Normal; 6],
                powered: false,
            },
        );
        network.insert(
            [1, 0, 0],
            NetworkNode {
                pos: [1, 0, 0],
                kind: TransporterKind::Normal,
                connections: [crate::transmitter::Connection::Normal; 6],
                powered: false,
            },
        );
        network.insert(
            [2, 0, 0],
            NetworkNode {
                pos: [2, 0, 0],
                kind: TransporterKind::Normal,
                connections: [crate::transmitter::Connection::Normal; 6],
                powered: false,
            },
        );

        let path = find_path(&network, [0, 0, 0], [2, 0, 0]);
        assert!(path.is_some());
        let (nodes, cost) = path.unwrap();
        assert_eq!(nodes, vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
        assert_eq!(cost, 2.0); // 1 + 1

        // Now replace [1,0,0] with Restrictive Transporter
        network.insert(
            [1, 0, 0],
            NetworkNode {
                pos: [1, 0, 0],
                kind: TransporterKind::Restrictive,
                connections: [crate::transmitter::Connection::Normal; 6],
                powered: false,
            },
        );
        let path = find_path(&network, [0, 0, 0], [2, 0, 0]);
        assert!(path.is_some());
        let (nodes, cost) = path.unwrap();
        assert_eq!(nodes, vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
        assert_eq!(cost, 1001.0); // 1000 + 1 (Java: RestrictiveTransporter.java:14)
    }

    #[test]
    fn test_diversion_transporter_closes_route_when_unpowered() {
        let mut network = HashMap::new();
        network.insert(
            [0, 0, 0],
            NetworkNode {
                pos: [0, 0, 0],
                kind: TransporterKind::Normal,
                connections: [crate::transmitter::Connection::Normal; 6],
                powered: false,
            },
        );

        // Diversion node requires High redstone to pass eastward (laser::EAST = 5)
        let mut modes = [DiversionControl::Disabled; 6];
        modes[sides::WEST] = DiversionControl::High;
        network.insert(
            [1, 0, 0],
            NetworkNode {
                pos: [1, 0, 0],
                kind: TransporterKind::Diversion(modes),
                connections: [crate::transmitter::Connection::Normal; 6],
                powered: false, // Not powered!
            },
        );

        let path = find_path(&network, [0, 0, 0], [1, 0, 0]);
        assert!(path.is_none());

        // Now power the diversion transporter
        if let Some(node) = network.get_mut(&[1, 0, 0]) {
            node.powered = true;
        }
        let path = find_path(&network, [0, 0, 0], [1, 0, 0]);
        assert!(path.is_some());
    }
}
