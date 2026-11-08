//! Transmitters (`TileEntityTransmitter`): cables, pipes and tubes, and the logistical, pressurized and mechanical
//! transporters. The game itself moves energy, fluids, gases and items through them (the `TransmitterSpec` their
//! registration carries); the mod only keeps what the configurator changes on one: the connection of each of its six
//! faces, whether it answers redstone, and the colour of a logistical transporter.
//!
//! The original (`ItemConfigurator.onConfigure` / `TileEntityTransmitter`): a sneak click cycles the connection of
//! the face clicked — normal, push, pull, none — and a sneak click on a disconnected side of a logistical transporter
//! cycles its colour instead; a plain click on a logistical transporter tells its colour, on any other transmitter it
//! toggles whether it answers redstone. What the port cannot do: the redstone flag has no effect on the game (the
//! port's networks move by the faces of the neighbours, and no API call reads the flag), and the colour is only a
//! number the port keeps (there is no representation of a transporter's colour in the API).
use super::*;

/// The blocks the port registers as transmitters, by their suffix (the tiers are `basic` to `ultimate`). Not here:
/// the thermodynamic conductor, the restrictive transporter and the diversion transporter — the port does not register
/// them at all, so there is no diversion mode to configure either.
pub const TRANSMITTERS: [(&str, &str); 4] = [
    ("_universal_cable", "energy"),
    ("_mechanical_pipe", "fluid"),
    ("_pressurized_tube", "gas"),
    ("_logistical_transporter", "item"),
];

/// Whether the block is a transmitter the port registers.
pub fn is_transmitter(block: &str) -> bool {
    TRANSMITTERS.iter().any(|(suffix, _)| block.ends_with(suffix))
}

/// Whether the block is a logistical transporter (the one whose colour the configurator changes).
pub fn is_logistical(block: &str) -> bool {
    block.ends_with("_logistical_transporter")
}

/// A connection of one face (`ConnectionType`): normal, push, pull, or none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Connection {
    #[default]
    Normal = 0,
    Push = 1,
    Pull = 2,
    None = 3,
}

impl Connection {
    /// The next connection when sneaking (`TransporterUtils.increment`).
    pub fn next(self) -> Connection {
        match self {
            Connection::Normal => Connection::Push,
            Connection::Push => Connection::Pull,
            Connection::Pull => Connection::None,
            Connection::None => Connection::Normal,
        }
    }

    pub fn from_byte(b: u8) -> Connection {
        match b {
            1 => Connection::Push,
            2 => Connection::Pull,
            3 => Connection::None,
            _ => Connection::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Connection::Normal => "Normal",
            Connection::Push => "Push",
            Connection::Pull => "Pull",
            Connection::None => "None",
        }
    }

    /// Whether the face connects through (everything but none): what `transmitter-faces` carries.
    pub fn connects(self) -> bool {
        self != Connection::None
    }
}

/// The colours a logistical transporter cycles through (`EnumColor`, none first).
pub const COLORS: [&str; 17] = [
    "None", "Black", "Dark Blue", "Dark Green", "Dark Aqua", "Dark Red", "Dark Purple", "Gold", "Gray",
    "Dark Gray", "Blue", "Green", "Aqua", "Red", "Light Purple", "Yellow", "White",
];

/// What a transmitter keeps for the configurator: one connection per face of the world, whether it answers redstone,
/// and (for a logistical transporter) its colour, 0 for none and 1..=16 for `COLORS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub connections: [Connection; 6],
    pub redstone: bool,
    pub color: u8,
}

impl Default for State {
    fn default() -> State {
        State { connections: [Connection::Normal; 6], redstone: false, color: 0 }
    }
}

/// The state a transmitter's data holds, or the default for one saved before the configurator kept it: six connection
/// bytes, the redstone byte and the colour byte.
pub fn decode(data: &[u8]) -> State {
    if data.len() < 8 {
        return State::default();
    }
    let mut connections = [Connection::Normal; 6];
    for (i, b) in data.iter().take(6).enumerate() {
        connections[i] = Connection::from_byte(*b);
    }
    State { connections, redstone: data[6] == 1, color: data[7].min(16) }
}

/// The bytes a transmitter's data holds for a state.
pub fn encode(state: &State) -> Vec<u8> {
    let mut out: Vec<u8> = state.connections.iter().map(|c| *c as u8).collect();
    out.push(u8::from(state.redstone));
    out.push(state.color);
    out
}

impl Mekanism {
    /// A tick of the transmitters with configurator clicks waiting: the connection of the face clicked, the colour of
    /// a logistical transporter, and whether a transmitter answers redstone (`ItemConfigurator.onConfigure`).
    pub(crate) fn run_transmitters(&mut self, transmitters: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in transmitters {
            let Some(click) = self.take_click(e) else { continue };
            if !click.mode.is_configurating() {
                continue;
            }
            let mut state = decode(&e.data);
            let logistical = is_logistical(e.state.split('[').next().unwrap_or(""));
            if click.sneak {
                if logistical && state.connections[click.face] == Connection::None {
                    // A sneak click on a disconnected side colours the transporter, as the original's onConfigure does.
                    state.color = (state.color + 1) % 17;
                    let _ = players::tell(&click.player, &format!("Color bumped to: {}", COLORS[state.color as usize]));
                } else {
                    state.connections[click.face] = state.connections[click.face].next();
                    let _ = players::tell(
                        &click.player,
                        &format!("Connection type changed to: {}", state.connections[click.face].label()),
                    );
                }
            } else if logistical {
                let _ = players::tell(&click.player, &format!("Current color: {}", COLORS[state.color as usize]));
            } else {
                // The redstone flag is kept but has no effect on the game: the port's networks move by the faces of
                // the neighbours, and no API call reads it.
                state.redstone = !state.redstone;
                let _ = players::tell(
                    &click.player,
                    &format!("Redstone sensitivity turned: {}", if state.redstone { "on" } else { "off" }),
                );
            }
            out.push(BlockEntityUpdate {
                dimension: e.dimension.clone(),
                pos: e.pos,
                data: encode(&state),
                items: None,
                energy: None,
                energy_faces: None,
                tanks: None,
                tank_faces: None,
                menu_data: None,
                transmitter_faces: Some(state.connections.iter().map(|c| c.connects()).collect()),
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_connections_cycle_and_the_state_round_trips() {
        assert_eq!(Connection::Normal.next(), Connection::Push);
        assert_eq!(Connection::Push.next(), Connection::Pull);
        assert_eq!(Connection::Pull.next(), Connection::None);
        assert_eq!(Connection::None.next(), Connection::Normal);
        assert!(!Connection::None.connects());
        assert!(Connection::Push.connects());
        let mut state = State::default();
        state.connections[0] = Connection::Pull; // down
        state.connections[5] = Connection::None; // east
        state.redstone = true;
        state.color = 11;
        assert_eq!(decode(&encode(&state)), state);
        assert_eq!(decode(&[]), State::default());
        assert!(is_transmitter("mekanism:elite_logistical_transporter"));
        assert!(is_logistical("mekanism:basic_logistical_transporter"));
        assert!(!is_logistical("mekanism:basic_universal_cable"));
        assert!(!is_transmitter("mekanism:basic_energy_cube"));
    }
}
