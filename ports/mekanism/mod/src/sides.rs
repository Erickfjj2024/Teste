//! The side configuration of an energy cube, as Mekanism's configurator changes it: each of the six
//! sides of the cube (relative to the way it faces) has a mode, and the block shows it.
//! Pure functions, tested on the host.

/// What a side does with energy (Mekanism's `DataType`, the four an energy cube supports).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    None = 0,
    Input = 1,
    Output = 2,
    Both = 3,
}

impl Mode {
    pub fn from_byte(b: u8) -> Mode {
        match b {
            1 => Mode::Input,
            2 => Mode::Output,
            3 => Mode::Both,
            _ => Mode::None,
        }
    }
    /// The next mode in Mekanism's order, or the one before.
    pub fn step(self, back: bool) -> Mode {
        let n = (self as u8 + if back { 3 } else { 1 }) % 4;
        Mode::from_byte(n)
    }
    pub fn name(self) -> &'static str {
        match self {
            Mode::None => "off",
            Mode::Input => "input",
            Mode::Output => "output",
            Mode::Both => "input/output",
        }
    }
    /// What the block shows (`side_*` states): the original lights a side that can output, and shows the
    /// port of one that can only take.
    pub fn look(self) -> &'static str {
        match self {
            Mode::None => "off",
            Mode::Input => "in",
            Mode::Output | Mode::Both => "out",
        }
    }
    /// The game's flow side, as a string index into `FlowSide`: none, input, output, both.
    pub fn flow(self) -> u8 {
        self as u8
    }
}

/// The six sides are the faces of the world, in the order the game lists them (down, up, north, south,
/// west, east): the block shows them as `side_<name>` states, and a cube's data holds one byte for each.
pub const SIDES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
pub const DOWN: usize = 0;
pub const UP: usize = 1;
pub const NORTH: usize = 2;
pub const SOUTH: usize = 3;
pub const WEST: usize = 4;
pub const EAST: usize = 5;

/// The configuration a cube starts with (Mekanism's `FRONT_OUT_EJECT`): the front outputs, the rest take.
/// The front is the face the cube was placed to look at the player with.
pub fn default_modes(front: usize) -> [Mode; 6] {
    let mut m = [Mode::Input; 6];
    m[front] = Mode::Output;
    m
}

/// The six modes a cube's data holds; None when it has no data yet (a new cube).
pub fn decode(data: &[u8]) -> Option<[Mode; 6]> {
    if data.len() < 6 {
        return None;
    }
    let mut m = [Mode::None; 6];
    for (i, b) in data.iter().take(6).enumerate() {
        m[i] = Mode::from_byte(*b);
    }
    Some(m)
}

pub fn encode(modes: &[Mode; 6]) -> Vec<u8> {
    modes.iter().map(|m| *m as u8).collect()
}

/// The horizontal face a placed block's front takes so that it looks at a player who looks `yaw`
/// (0 faces south): the opposite of where they look.
pub fn front_toward(yaw: f32) -> usize {
    let y = f64::from(yaw).to_radians();
    let (dx, dz) = (-y.sin(), y.cos());
    if dx.abs() > dz.abs() {
        if dx > 0.0 { WEST } else { EAST }
    } else if dz > 0.0 {
        NORTH
    } else {
        SOUTH
    }
}

/// The sides of the six faces for the game, as `Mode::flow` numbers.
pub fn flow_faces(modes: &[Mode; 6]) -> [u8; 6] {
    modes.map(Mode::flow)
}

/// The value of a property in a state string, `mekanism:basic_energy_cube[facing=north,...]`.
pub fn property<'a>(state: &'a str, name: &str) -> Option<&'a str> {
    let inside = state.split_once('[')?.1.trim_end_matches(']');
    inside.split(',').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == name).then_some(v)
    })
}

/// The face of the block at `pos` that a player's look enters first: the eye is `eye`, the look `dir`.
/// None when the look misses the block.
pub fn face_hit(eye: [f64; 3], dir: [f64; 3], pos: [i32; 3]) -> Option<usize> {
    let mut near = f64::NEG_INFINITY;
    let mut far = f64::INFINITY;
    let mut entered = None;
    for axis in 0..3 {
        let lo = f64::from(pos[axis]);
        let hi = lo + 1.0;
        if dir[axis].abs() < 1e-12 {
            if eye[axis] < lo || eye[axis] > hi {
                return None;
            }
            continue;
        }
        let (mut t0, mut t1) = ((lo - eye[axis]) / dir[axis], (hi - eye[axis]) / dir[axis]);
        // Seen from the min side the face entered is the min one (west, down, north).
        let mut face_min = true;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
            face_min = false;
        }
        if t0 > near {
            near = t0;
            entered = Some(match (axis, face_min) {
                (0, true) => WEST,
                (0, false) => EAST,
                (1, true) => DOWN,
                (1, false) => UP,
                (_, true) => NORTH,
                (_, false) => SOUTH,
            });
        }
        far = far.min(t1);
        if near > far || far < 0.0 {
            return None;
        }
    }
    entered
}

/// The look of a player (`yaw` 0 faces south, `pitch` -90 looks up), as a direction.
pub fn look(yaw: f32, pitch: f32) -> [f64; 3] {
    let (y, p) = (f64::from(yaw).to_radians(), f64::from(pitch).to_radians());
    [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_cycle_both_ways() {
        let mut m = Mode::None;
        for _ in 0..4 {
            m = m.step(false);
        }
        assert_eq!(m, Mode::None);
        assert_eq!(Mode::None.step(false), Mode::Input);
        assert_eq!(Mode::Input.step(false), Mode::Output);
        assert_eq!(Mode::Both.step(false), Mode::None);
        assert_eq!(Mode::None.step(true), Mode::Both);
    }

    #[test]
    fn a_new_cube_outputs_in_front_and_takes_elsewhere() {
        assert_eq!(decode(&[]), None);
        let m = default_modes(SOUTH);
        assert_eq!(m[SOUTH], Mode::Output);
        assert_eq!(m.iter().filter(|m| **m == Mode::Input).count(), 5);
        assert_eq!(decode(&encode(&m)), Some(m));
    }

    #[test]
    fn the_front_looks_at_the_player() {
        // A player looking south (yaw 0) has the cube's front toward them, on its north face.
        assert_eq!(front_toward(0.0), NORTH);
        assert_eq!(front_toward(180.0), SOUTH);
        assert_eq!(front_toward(90.0), EAST); // looking west
        assert_eq!(front_toward(-90.0), WEST); // looking east
    }

    #[test]
    fn faces_of_the_game_get_their_modes() {
        let flow = flow_faces(&default_modes(EAST));
        assert_eq!(flow[EAST], 2);
        assert_eq!(flow[WEST], 1);
        assert_eq!(flow[UP], 1);
    }

    #[test]
    fn a_state_string_gives_its_properties() {
        let s = "mekanism:basic_energy_cube[side_north=out,side_up=in]";
        assert_eq!(property(s, "side_north"), Some("out"));
        assert_eq!(property(s, "side_up"), Some("in"));
        assert_eq!(property(s, "side_down"), None);
        assert_eq!(property("mekanism:crusher", "facing"), None);
    }

    #[test]
    fn a_look_enters_the_face_it_meets() {
        // Standing west of the block at (10, 64, 10) looking east (yaw -90) enters its west face.
        let d = look(-90.0, 0.0);
        assert!((d[0] - 1.0).abs() < 1e-9);
        assert_eq!(face_hit([8.5, 64.5, 10.5], d, [10, 64, 10]), Some(WEST));
        // South of it looking north (yaw 180) enters the south face.
        let d = look(180.0, 0.0);
        assert_eq!(face_hit([10.5, 64.5, 13.0], d, [10, 64, 10]), Some(SOUTH));
        // Above looking down enters the top.
        let d = look(0.0, 90.0);
        assert_eq!(face_hit([10.5, 67.0, 10.5], d, [10, 64, 10]), Some(UP));
        // Looking away misses.
        assert_eq!(face_hit([8.5, 64.5, 10.5], look(90.0, 0.0), [10, 64, 10]), None);
        // From the side and above, a shallow look reaches the side first.
        let d = look(-90.0, 20.0);
        assert_eq!(face_hit([8.0, 65.0, 10.5], d, [10, 64, 10]), Some(WEST));
    }
}
