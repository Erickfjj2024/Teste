//! Named frequencies (`Frequency`, `FrequencyIdentity`, `FrequencyManager`, `GuiFrequencySelector`): the teleporter's and the quantum entangloporter's. A frequency has a
//! name (16 characters at most), a security (public: anyone's, found by name; private: its owner's), and its owner. A block keeps the frequency it is set to in its own bytes;
//! the frequencies the screen lists are the ones that the blocks around hold (the original keeps them in the world's saved data until someone deletes them: here a frequency
//! no block holds is gone, which is the one difference). Not in: the trusted security (it needs the security system of the original), and the colour of the entangloporter's.
use super::*;

/// `FrequencyManager.MAX_FREQ_LENGTH`.
pub const MAX_NAME: usize = 16;
/// How many rows of each list a screen shows (the strings it is given).
pub const LIST_MAX: usize = 16;
/// The strings of a frequency screen: 0 the frequency, 1 its owner, 2 its security, then the public rows, then the private ones.
pub const TEXT_PUBLIC_AT: usize = 3;
pub const TEXT_PRIVATE_AT: usize = TEXT_PUBLIC_AT + LIST_MAX;

/// A frequency: its name, whether it is private, and its owner (id and name).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Freq {
    pub name: String,
    pub private: bool,
    pub owner: String,
    pub owner_name: String,
}

impl Freq {
    /// What two blocks share when they are on the same frequency: a public one by its name, a private one by its owner and its name.
    pub fn key(&self) -> String {
        if self.private { format!("o:{}:{}", self.owner, self.name) } else { format!("p:{}", self.name) }
    }
    pub fn security_label(&self) -> &'static str {
        if self.private { "Private" } else { "Public" }
    }
}

/// `InputValidator.LETTER_OR_DIGIT.or(FREQUENCY_CHARS)`: what a name may be made of.
pub fn valid_name(name: &str) -> bool {
    // `InputValidator.LETTER_OR_DIGIT.or(FREQUENCY_CHARS)`: ASCII letters and digits and the listed marks; the original does not trim (`!freq.isEmpty()`).
    !name.is_empty() && name.chars().count() <= MAX_NAME && name.chars().all(|c| c.is_ascii_alphanumeric() || "- |'\"_+:()?!/@$`~,.#".contains(c))
}

/// The frequency as bytes (appended to a block's own): a flag, the security, then the name, the owner and the owner's name, each with its length.
pub fn encode(f: &Option<Freq>, out: &mut Vec<u8>) {
    match f {
        None => out.push(0),
        Some(f) => {
            out.push(1 + u8::from(f.private));
            for s in [&f.name, &f.owner, &f.owner_name] {
                out.push(s.len().min(255) as u8);
                out.extend(&s.as_bytes()[..s.len().min(255)]);
            }
        }
    }
}

/// The frequency at `at` in `data` and where it ends; none for a flag of 0 or bytes that are not one.
pub fn decode(data: &[u8], at: usize) -> (Option<Freq>, usize) {
    let Some(&flag) = data.get(at) else { return (None, at) };
    if flag == 0 || flag > 2 {
        return (None, at + 1);
    }
    let mut i = at + 1;
    let mut parts = Vec::new();
    for _ in 0..3 {
        let Some(&n) = data.get(i) else { return (None, at + 1) };
        let Some(bytes) = data.get(i + 1..i + 1 + usize::from(n)) else { return (None, at + 1) };
        parts.push(String::from_utf8_lossy(bytes).into_owned());
        i += 1 + usize::from(n);
    }
    let owner_name = parts.pop().unwrap_or_default();
    let owner = parts.pop().unwrap_or_default();
    let name = parts.pop().unwrap_or_default();
    (Some(Freq { name, private: flag == 2, owner, owner_name }), i)
}

/// What a screen can choose from: the public frequencies and the private ones of the viewer (`getPublicFrequencies`, `getPrivateFrequencies`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Listing {
    pub public: Vec<Freq>,
    pub private: Vec<Freq>,
}

/// The listing from the frequencies the blocks hold, for the player `viewer` (an id): each frequency once, by name.
pub fn listing(known: &[Freq], viewer: &str) -> Listing {
    let mut l = Listing::default();
    for f in known {
        let list = if f.private { if f.owner != viewer { continue } else { &mut l.private } } else { &mut l.public };
        if !list.iter().any(|o: &Freq| o.key() == f.key()) {
            list.push(f.clone());
        }
    }
    l.public.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    l.private.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    l.public.truncate(LIST_MAX);
    l.private.truncate(LIST_MAX);
    l
}

/// The strings of the screen (`GuiFrequencySelector`): the frequency, its owner and security, then the rows (a public one with its owner, a private one by its name).
pub fn texts(current: &Option<Freq>, l: &Listing) -> Vec<String> {
    let mut v = vec![
        format!("Frequency: {}", current.as_ref().map_or("None", |f| f.name.as_str())),
        format!("Owner: {}", current.as_ref().map_or("None", |f| f.owner_name.as_str())),
        format!("Security: {}", current.as_ref().map_or("None", |f| f.security_label())),
    ];
    for i in 0..LIST_MAX {
        v.push(l.public.get(i).map_or(String::new(), |f| format!("{} ({})", f.name, f.owner_name)));
    }
    for i in 0..LIST_MAX {
        v.push(l.private.get(i).map_or(String::new(), |f| f.name.clone()));
    }
    v
}

/// What a press or a confirmed box on a frequency screen asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    SetPublic(usize),
    SetPrivate(usize),
    Delete,
    CreatePublic(String),
    CreatePrivate(String),
}

/// The ids a screen uses from `base`: the public rows, the private rows, the delete button, then the two boxes (public and private).
pub const fn id_public(base: i32, row: usize) -> i32 {
    base + row as i32
}
pub const fn id_private(base: i32, row: usize) -> i32 {
    base + LIST_MAX as i32 + row as i32
}
pub const fn id_delete(base: i32) -> i32 {
    base + 2 * LIST_MAX as i32
}
pub const fn id_box_public(base: i32) -> i32 {
    base + 2 * LIST_MAX as i32 + 1
}
pub const fn id_box_private(base: i32) -> i32 {
    base + 2 * LIST_MAX as i32 + 2
}

pub fn button_action(id: i32, base: i32) -> Option<Action> {
    let at = id - base;
    let n = LIST_MAX as i32;
    match at {
        0..=15 => Some(Action::SetPublic(at as usize)),
        16..=31 => Some(Action::SetPrivate((at - n) as usize)),
        32 => Some(Action::Delete),
        _ => None,
    }
}

pub fn text_action(id: i32, base: i32, text: &str) -> Option<Action> {
    let t = text;
    if id == id_box_public(base) {
        valid_name(t).then(|| Action::CreatePublic(t.to_owned()))
    } else if id == id_box_private(base) {
        valid_name(t).then(|| Action::CreatePrivate(t.to_owned()))
    } else {
        None
    }
}

/// What the block is set to after the action, by the player (`viewer`, `viewer_name`): the outer none means the action changes nothing, the inner what it is set to
/// (none for a deleted one). A frequency can only be deleted by its owner.
pub fn apply(action: &Action, current: &Option<Freq>, l: &Listing, viewer: &str, viewer_name: &str) -> Option<Option<Freq>> {
    match action {
        Action::SetPublic(i) => l.public.get(*i).filter(|f| current.as_ref() != Some(*f)).map(|f| Some(f.clone())),
        Action::SetPrivate(i) => l.private.get(*i).filter(|f| current.as_ref() != Some(*f)).map(|f| Some(f.clone())),
        Action::CreatePublic(name) | Action::CreatePrivate(name) => {
            let private = matches!(action, Action::CreatePrivate(_));
            // A name already taken by a public frequency joins it, whoever made it (`FrequencyManager.getOrCreateFrequency` finds it by identity).
            let found = if private {
                l.private.iter().find(|f| f.name == *name).cloned()
            } else {
                l.public.iter().find(|f| f.name == *name).cloned()
            };
            let f = found.unwrap_or(Freq { name: name.clone(), private, owner: viewer.to_owned(), owner_name: viewer_name.to_owned() });
            (current.as_ref() != Some(&f)).then_some(Some(f))
        }
        Action::Delete => match current {
            Some(f) if f.owner == viewer => Some(None),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(name: &str, private: bool, owner: &str) -> Freq {
        Freq { name: name.into(), private, owner: owner.into(), owner_name: format!("{owner}-name") }
    }

    #[test]
    fn a_frequency_survives_its_bytes() {
        let a = Some(f("Home base", true, "uuid-1"));
        let mut bytes = vec![9, 9];
        encode(&a, &mut bytes[2..].to_vec());
        let mut buf = Vec::new();
        encode(&a, &mut buf);
        assert_eq!(decode(&buf, 0), (a.clone(), buf.len()));
        let mut none = Vec::new();
        encode(&None, &mut none);
        assert_eq!(decode(&none, 0), (None, 1));
        assert_eq!(decode(&[], 0), (None, 0));
        assert_eq!(decode(&buf[..4], 0).0, None);
    }

    #[test]
    fn the_name_rules() {
        assert!(valid_name("Base 1"));
        // ASCII only, spaces kept (a name of spaces is a name)
        assert!(!valid_name("caf\u{e9}"));
        assert!(valid_name("  "));
        assert!(valid_name("a_b-c"));
        assert!(!valid_name(""));
        assert!(!valid_name("a name that is far too long"));
        assert!(!valid_name("tab\\there"));
    }

    #[test]
    fn two_blocks_share_a_public_name_but_not_a_private_one() {
        assert_eq!(f("x", false, "a").key(), f("x", false, "b").key());
        assert_ne!(f("x", true, "a").key(), f("x", true, "b").key());
        assert_ne!(f("x", true, "a").key(), f("x", false, "a").key());
    }

    #[test]
    fn the_listing_has_the_publics_and_the_viewers_privates() {
        let known = vec![f("b", false, "a"), f("a", false, "z"), f("b", false, "q"), f("mine", true, "me"), f("theirs", true, "you")];
        let l = listing(&known, "me");
        assert_eq!(l.public.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(l.private.len(), 1);
        let t = texts(&Some(f("a", false, "z")), &l);
        assert_eq!(t[0], "Frequency: a");
        assert_eq!(t[1], "Owner: z-name");
        assert_eq!(t[2], "Security: Public");
        assert_eq!(t[TEXT_PUBLIC_AT], "a (z-name)");
        assert_eq!(t[TEXT_PRIVATE_AT], "mine");
        assert_eq!(t.len(), TEXT_PRIVATE_AT + LIST_MAX);
    }

    #[test]
    fn the_actions_set_create_and_delete() {
        let l = listing(&[f("pub", false, "a"), f("mine", true, "me")], "me");
        assert_eq!(button_action(100 + 1, 100), Some(Action::SetPublic(1)));
        assert_eq!(button_action(100 + 16, 100), Some(Action::SetPrivate(0)));
        assert_eq!(button_action(id_delete(100), 100), Some(Action::Delete));
        assert_eq!(text_action(id_box_public(100), 100, "New"), Some(Action::CreatePublic("New".into())));
        assert_eq!(text_action(id_box_private(100), 100, ""), None);
        assert_eq!(apply(&Action::SetPublic(0), &None, &l, "me", "Me"), Some(Some(f("pub", false, "a"))));
        // setting what it already is changes nothing
        assert_eq!(apply(&Action::SetPublic(0), &Some(f("pub", false, "a")), &l, "me", "Me"), None);
        // a new name is created owned by the player; an existing public one is joined
        let made = apply(&Action::CreatePrivate("fresh".into()), &None, &l, "me", "Me").unwrap().unwrap();
        assert!(made.private && made.owner == "me");
        let joined = apply(&Action::CreatePublic("pub".into()), &None, &l, "me", "Me").unwrap().unwrap();
        assert_eq!(joined.owner, "a");
        // only the owner deletes
        assert_eq!(apply(&Action::Delete, &Some(f("mine", true, "me")), &l, "me", "Me"), Some(None));
        assert_eq!(apply(&Action::Delete, &Some(f("pub", false, "a")), &l, "me", "Me"), None);
    }
}
