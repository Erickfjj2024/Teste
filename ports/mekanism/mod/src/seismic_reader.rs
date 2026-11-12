//! The Seismic Reader (`ItemSeismicReader`, `GuiSeismicReader`): a use opens a screen that lists the blocks of the column under the player, from the world's bottom up to the feet. The
//! screen shows nine of them at a time as icons (the original's `renderItem`; here they are the menu's virtual slots, which cannot be taken), the name and abundancy (how many of that block are in the
//! column) of the one at the arrow, and its height; the up and down buttons move the arrow.
//!
//! Not here: the scroll bar, fluids drawn from their texture (a fluid shows as its bucket), the abundancy of a fluid in a waterlogged block, and the reader's energy use (the item has none in
//! the original either: it is a plain item).
use super::*;
use horizon_mod_sdk::server::{ItemMenu, ItemMenuUpdate};
use std::collections::HashMap;

pub const ITEM: &str = "mekanism:seismic_reader";
pub const MENU: &str = "mekanism:seismic_reader";
pub const BUTTON_UP: i32 = 8401;
pub const BUTTON_DOWN: i32 = 8402;
/// The icons the screen shows at once, and the menu's slots.
pub const ICONS: usize = 9;

/// What is read for an open screen: the column's blocks from the bottom up, and the layer the arrow is at (counted from the top, as the scroll bar is).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Column {
    pub blocks: Vec<String>,
    pub min_height: i32,
    pub scroll: usize,
}

/// The lowest block of a dimension (`getMinBuildHeight`).
pub fn min_height(dimension: &str) -> i32 {
    if dimension == "minecraft:overworld" { -64 } else { 0 }
}

/// The index of the layer the arrow is at (`getCurrentLayer`).
pub fn current_layer(c: &Column) -> Option<usize> {
    (c.blocks.len()).checked_sub(c.scroll + 1)
}

/// The block id with the name air for any kind of air.
fn plain(block: &str) -> String {
    let id = block.split('[').next().unwrap_or(block);
    if matches!(id, "minecraft:cave_air" | "minecraft:void_air") { "minecraft:air".to_owned() } else { id.to_owned() }
}

/// The item that stands for a block in the screen (its own, or the bucket of a fluid); none for air.
pub fn icon_of(block: &str) -> Option<String> {
    match block {
        "minecraft:air" => None,
        "minecraft:water" => Some("minecraft:water_bucket".to_owned()),
        "minecraft:lava" => Some("minecraft:lava_bucket".to_owned()),
        other => Some(other.to_owned()),
    }
}

/// The screen's strings: the block's name and abundancy, then the height of the layer.
pub fn texts(c: &Column) -> Vec<String> {
    let Some(layer) = current_layer(c) else { return vec![String::new(), String::new(), String::new()] };
    let block = &c.blocks[layer];
    let count = c.blocks.iter().filter(|b| *b == block).count();
    vec![dynamic::display_name(block), format!("Abundancy: {count}"), format!("{}", c.min_height + layer as i32)]
}

/// The nine virtual slots: the layers around the arrow, the lowest at the bottom (`layer = current + (i - 4)`).
pub fn icons(c: &Column) -> Vec<Option<ItemStack>> {
    let current = current_layer(c).map_or(-1, |l| l as i32);
    (0..ICONS)
        .map(|i| {
            let layer = current + (i as i32 - 4);
            let block = usize::try_from(layer).ok().and_then(|l| c.blocks.get(l))?;
            icon_of(block).map(|item| ItemStack { item, count: 1, components: String::new() })
        })
        .collect()
}

#[derive(Default)]
pub struct Readers {
    pub columns: HashMap<String, Column>,
}

impl Mekanism {
    /// A use of the Seismic Reader reads the column under the player and opens its screen.
    pub(crate) fn seismic_reader_used(&mut self, player: &Player, stack: &ItemStack) -> bool {
        if stack.item != ITEM {
            return false;
        }
        let Some((slot, _)) = Self::find_hotbar_slot(&player.id, stack) else { return false };
        let (x, z) = (player.x.floor() as i32, player.z.floor() as i32);
        let top = player.y.floor() as i32;
        let bottom = min_height(&player.dimension);
        let blocks: Vec<String> = (bottom..=top).map(|y| world::get_block(&player.dimension, x, y, z).map_or_else(|| "minecraft:air".to_owned(), |b| plain(&b))).collect();
        self.readers.columns.insert(player.id.clone(), Column { blocks, min_height: bottom, scroll: 0 });
        self.item_menu_sent.remove(&(player.id.clone(), MENU.to_owned()));
        players::open_item_menu(&player.id, slot, MENU)
    }

    /// One tick of the open screen: the icons, the strings and the arrow's layer.
    pub(crate) fn seismic_reader_tick(&mut self, player: &Player, menu: &ItemMenu) -> Option<ItemMenuUpdate> {
        let column = self.readers.columns.get(&player.id)?.clone();
        let strings = texts(&column);
        let slots = icons(&column);
        let can_up = column.scroll > 0;
        let can_down = column.scroll + 1 < column.blocks.len();
        let data = vec![i32::from(can_up), i32::from(can_down)];
        let same_slots = menu.items.len() == slots.len() && menu.items.iter().zip(&slots).all(|(a, b)| a.as_ref().map(|s| &s.item) == b.as_ref().map(|s| &s.item));
        let key = (player.id.clone(), menu.menu.clone());
        if same_slots && menu.menu_data == data && self.item_menu_sent.get(&key) == Some(&strings) {
            return None;
        }
        self.item_menu_sent.insert(key, strings.clone());
        Some(ItemMenuUpdate { items: Some(slots), menu_data: Some(data), menu_text: Some(strings), close: false })
    }

    /// The up and down buttons move the arrow a layer (`scrollBar.adjustScroll`).
    pub(crate) fn seismic_reader_button(&mut self, player: &Player, id: i32) -> Option<ItemMenuUpdate> {
        let column = self.readers.columns.get_mut(&player.id)?;
        match id {
            BUTTON_UP => column.scroll = column.scroll.saturating_sub(1),
            BUTTON_DOWN => column.scroll = (column.scroll + 1).min(column.blocks.len().saturating_sub(1)),
            _ => return None,
        }
        None
    }

    /// The icons cannot be taken: the screen puts them back.
    pub(crate) fn seismic_reader_taken(&mut self, player: &Player) -> Option<ItemMenuUpdate> {
        let column = self.readers.columns.get(&player.id)?;
        Some(ItemMenuUpdate { items: Some(icons(column)), menu_data: None, menu_text: None, close: false })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn column(blocks: &[&str], scroll: usize) -> Column {
        Column { blocks: blocks.iter().map(|b| (*b).to_owned()).collect(), min_height: -64, scroll }
    }

    #[test]
    fn the_arrow_starts_at_the_top_and_the_height_counts_from_the_bottom() {
        let c = column(&["minecraft:bedrock", "minecraft:stone", "minecraft:stone", "minecraft:air"], 0);
        assert_eq!(current_layer(&c), Some(3));
        let t = texts(&column(&["minecraft:bedrock", "minecraft:stone", "minecraft:stone", "minecraft:air"], 1));
        assert_eq!(t, ["Stone", "Abundancy: 2", "-62"]);
    }

    #[test]
    fn nine_icons_around_the_arrow_with_the_lowest_first() {
        let blocks: Vec<String> = (0..6).map(|i| format!("minecraft:block_{i}")).collect();
        let c = Column { blocks, min_height: 0, scroll: 2 };
        // The arrow is at layer 3, so slot i shows layer 3 + (i - 4).
        let shown = icons(&c);
        assert_eq!(shown.len(), ICONS);
        assert_eq!(shown[0], None);
        assert_eq!(shown[1].as_ref().map(|s| s.item.as_str()), Some("minecraft:block_0"));
        assert_eq!(shown[5].as_ref().map(|s| s.item.as_str()), Some("minecraft:block_4"));
        assert_eq!(shown[6].as_ref().map(|s| s.item.as_str()), Some("minecraft:block_5"));
        assert_eq!(shown[7], None);
    }

    #[test]
    fn air_has_no_icon_and_a_fluid_shows_its_bucket() {
        assert_eq!(icon_of("minecraft:air"), None);
        assert_eq!(icon_of("minecraft:lava").as_deref(), Some("minecraft:lava_bucket"));
        assert_eq!(plain("minecraft:cave_air"), "minecraft:air");
        assert_eq!(plain("minecraft:oak_stairs[facing=north]"), "minecraft:oak_stairs");
    }
}
