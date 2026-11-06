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

/// The sides relative to the cube's facing, in the order of the data bytes and of `RelativeSide`
/// as the model names them.
pub const SIDES: [&str; 6] = ["front", "back", "left", "right", "top", "bottom"];

/// The configuration a cube starts with (Mekanism's `FRONT_OUT_EJECT`): the front outputs, the rest take.
pub fn default_modes() -> [Mode; 6] {
    let mut m = [Mode::Input; 6];
    m[0] = Mode::Output;
    m
}

/// The six modes a cube's data holds; a new cube's (no data) has the default.
pub fn decode(data: &[u8]) -> [Mode; 6] {
    if data.len() < 6 {
        return default_modes();
    }
    let mut m = [Mode::None; 6];
    for (i, b) in data.iter().take(6).enumerate() {
        m[i] = Mode::from_byte(*b);
    }
    m
}

pub fn encode(modes: &[Mode; 6]) -> Vec<u8> {
    modes.iter().map(|m| *m as u8).collect()
}

/// The faces, in the order the game lists them: down, up, north, south, west, east.
pub const DOWN: usize = 0;
pub const UP: usize = 1;
pub const NORTH: usize = 2;
pub const SOUTH: usize = 3;
pub const WEST: usize = 4;
pub const EAST: usize = 5;

/// A horizontal direction turned a quarter clockwise seen from above: north, east, south, west.
fn clockwise(face: usize) -> usize {
    match face {
        NORTH => EAST,
        EAST => SOUTH,
        SOUTH => WEST,
        _ => NORTH,
    }
}

/// The face of the world a relative side is, for a cube facing `facing` (`north`...). The model faces
/// north: its right is the west face; turning it clockwise by the facing turns every side with it.
pub fn face_of(facing: &str, side: usize) -> usize {
    let turns = match facing {
        "east" => 1,
        "south" => 2,
        "west" => 3,
        _ => 0,
    };
    let mut face = match SIDES[side] {
        "front" => NORTH,
        "back" => SOUTH,
        "right" => WEST,
        "left" => EAST,
        "top" => UP,
        _ => DOWN,
    };
    if face != UP && face != DOWN {
        for _ in 0..turns {
            face = clockwise(face);
        }
    }
    face
}

/// The relative side a world face is of a cube facing `facing`.
pub fn side_of(facing: &str, face: usize) -> usize {
    (0..6).find(|s| face_of(facing, *s) == face).unwrap_or(0)
}

/// The sides of the six faces for the game, in its order of faces.
pub fn flow_faces(facing: &str, modes: &[Mode; 6]) -> [u8; 6] {
    let mut out = [0u8; 6];
    for (s, m) in modes.iter().enumerate() {
        out[face_of(facing, s)] = m.flow();
    }
    out
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
        let m = decode(&[]);
        assert_eq!(m[0], Mode::Output);
        assert!(m[1..].iter().all(|m| *m == Mode::Input));
        assert_eq!(decode(&encode(&m)), m);
    }

    #[test]
    fn sides_turn_with_the_facing() {
        assert_eq!(face_of("north", 0), NORTH);
        assert_eq!(face_of("north", 3), WEST); // right
        assert_eq!(face_of("east", 0), EAST);
        assert_eq!(face_of("east", 3), NORTH); // right of an east-facing cube
        assert_eq!(face_of("south", 2), WEST); // left
        assert_eq!(face_of("west", 1), EAST); // back
        for f in ["north", "east", "south", "west"] {
            let mut seen: Vec<usize> = (0..6).map(|s| face_of(f, s)).collect();
            seen.sort();
            assert_eq!(seen, vec![0, 1, 2, 3, 4, 5]);
            for s in 0..6 {
                assert_eq!(side_of(f, face_of(f, s)), s);
            }
        }
    }

    #[test]
    fn faces_of_the_game_get_their_modes() {
        let flow = flow_faces("east", &default_modes());
        assert_eq!(flow[EAST], 2);
        assert_eq!(flow[WEST], 1);
        assert_eq!(flow[UP], 1);
    }

    #[test]
    fn a_state_string_gives_its_properties() {
        let s = "mekanism:basic_energy_cube[facing=east,side_front=out,waterlogged=false]";
        assert_eq!(property(s, "facing"), Some("east"));
        assert_eq!(property(s, "side_front"), Some("out"));
        assert_eq!(property(s, "side_top"), None);
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
