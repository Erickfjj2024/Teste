//! Filters of the logistical sorter and the digital miner (`SorterFilter`, `MinerFilter`: `IItemStackFilter`, `ITagFilter`), the two kinds the port can honour: an item (or block) by
//! its id, and a tag (every item of it). A filter is made on the screen from a typed id or tag and kept in the block's bytes; the list is what the screen shows.
//!
//! Not here: the mod-id filter (the host cannot list the items of a mod), a colour for each filter (it routes in the logistical transporters), minimum and maximum counts and the
//! size mode, a tag with a wildcard, and the fuzzy mode of the item filter (the original compares items' components).
use super::*;

/// How many filters a block keeps and how many rows its screen shows.
pub const MAX_FILTERS: usize = 32;
pub const ROWS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    /// A tag (`c:ingots/iron`) rather than an item or block (`minecraft:iron_ingot`).
    pub tag: bool,
    pub target: String,
}

impl Filter {
    /// The line of the list: the kind and what it names.
    pub fn row(&self) -> String {
        format!("{}: {}", if self.tag { "Tag" } else { "Item" }, self.target)
    }
}

/// What a filter may name: `namespace:path` of lower-case letters, digits and `_ . - /` (a tag's path may have slashes).
pub fn valid_target(tag: bool, text: &str) -> bool {
    let t = text.trim();
    let Some((ns, path)) = t.split_once(':') else { return false };
    let ok = |s: &str, slash: bool| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_.-".contains(c) || (slash && c == '/'));
    t.len() <= 128 && ok(ns, false) && ok(path, tag)
}

/// Adds a filter (`addFilter`): refused for a target that is not valid, one already in the list, or a full list.
pub fn add(list: &mut Vec<Filter>, tag: bool, text: &str) -> bool {
    let t = text.trim();
    if list.len() >= MAX_FILTERS || !valid_target(tag, t) || list.iter().any(|f| f.tag == tag && f.target == t) {
        return false;
    }
    list.push(Filter { tag, target: t.to_owned() });
    true
}

pub fn encode(list: &[Filter], out: &mut Vec<u8>) {
    out.push(list.len().min(MAX_FILTERS) as u8);
    for f in list.iter().take(MAX_FILTERS) {
        out.push(u8::from(f.tag));
        out.push(f.target.len().min(255) as u8);
        out.extend(&f.target.as_bytes()[..f.target.len().min(255)]);
    }
}

/// The list at `at` in `data` and where it ends (an empty list for bytes that are not one).
pub fn decode(data: &[u8], at: usize) -> (Vec<Filter>, usize) {
    let Some(&n) = data.get(at) else { return (Vec::new(), at) };
    let mut i = at + 1;
    let mut list = Vec::new();
    for _ in 0..n {
        let (Some(&tag), Some(&len)) = (data.get(i), data.get(i + 1)) else { return (list, i) };
        let Some(bytes) = data.get(i + 2..i + 2 + usize::from(len)) else { return (list, i) };
        list.push(Filter { tag: tag == 1, target: String::from_utf8_lossy(bytes).into_owned() });
        i += 2 + usize::from(len);
    }
    (list, i)
}

/// The item ids the filters name: an item filter its item, a tag filter every item of the tag (`members` lists a tag), each once.
pub fn items(list: &[Filter], members: &dyn Fn(&str) -> Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for f in list {
        let found = if f.tag { members(&f.target) } else { vec![f.target.clone()] };
        for id in found {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

/// Whether a filter takes a block or item id: an item filter by id, a tag filter when the id is in the tag (`tags` lists an id's tags).
pub fn matches(f: &Filter, id: &str, tags: &dyn Fn(&str) -> Vec<String>) -> bool {
    if f.tag { tags(id).iter().any(|t| *t == f.target) } else { f.target == id }
}

/// The strings of a screen's list (`GuiFilterHolder`): the count, then a row for each filter up to `ROWS`.
pub fn texts(list: &[Filter], selected: Option<usize>) -> Vec<String> {
    let mut v = vec![format!("Filters: {}", list.len())];
    for i in 0..ROWS {
        v.push(list.get(i).map_or(String::new(), |f| format!("{}{}", if selected == Some(i) { "> " } else { "" }, f.row())));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filter_needs_a_valid_target() {
        assert!(valid_target(false, "minecraft:iron_ingot"));
        assert!(valid_target(true, "c:ingots/iron"));
        assert!(!valid_target(false, "c:ingots/iron"));
        assert!(!valid_target(false, "iron"));
        assert!(!valid_target(true, "C:Ingots"));
        assert!(!valid_target(true, ""));
    }

    #[test]
    fn filters_are_added_once_and_kept_in_bytes() {
        let mut list = Vec::new();
        assert!(add(&mut list, false, " minecraft:stone "));
        assert!(!add(&mut list, false, "minecraft:stone"));
        assert!(add(&mut list, true, "c:ores/iron"));
        assert!(!add(&mut list, true, "nope"));
        let mut bytes = Vec::new();
        encode(&list, &mut bytes);
        assert_eq!(decode(&bytes, 0), (list.clone(), bytes.len()));
        assert_eq!(decode(&[], 0).0, Vec::new());
        assert_eq!(decode(&bytes[..4], 0).0.len(), 0);
        assert_eq!(decode(&bytes[..18], 0).0.len(), 1);
        for i in 0..MAX_FILTERS {
            add(&mut list, false, &format!("minecraft:item_{i}"));
        }
        assert_eq!(list.len(), MAX_FILTERS);
    }

    #[test]
    fn the_filters_name_items_and_match_them() {
        let list = vec![Filter { tag: false, target: "minecraft:stone".into() }, Filter { tag: true, target: "c:ingots/iron".into() }];
        let members = |t: &str| if t == "c:ingots/iron" { vec!["minecraft:iron_ingot".to_owned(), "minecraft:stone".to_owned()] } else { Vec::new() };
        assert_eq!(items(&list, &members), vec!["minecraft:stone".to_owned(), "minecraft:iron_ingot".to_owned()]);
        let tags = |id: &str| if id == "minecraft:iron_ingot" { vec!["c:ingots/iron".to_owned()] } else { Vec::new() };
        assert!(matches(&list[0], "minecraft:stone", &tags));
        assert!(matches(&list[1], "minecraft:iron_ingot", &tags));
        assert!(!matches(&list[1], "minecraft:stone", &tags));
    }

    #[test]
    fn the_list_texts() {
        let list = vec![Filter { tag: true, target: "c:ores/tin".into() }];
        let t = texts(&list, Some(0));
        assert_eq!(t[0], "Filters: 1");
        assert_eq!(t[1], "> Tag: c:ores/tin");
        assert_eq!(t[2], "");
        assert_eq!(t.len(), 1 + ROWS);
    }
}
