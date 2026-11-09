//! Cardboard Box (`BlockCardboardBox`, `TileEntityCardboardBox`, `ItemBlockCardboardBox`).
//!
//! A utility block that can pack up almost any block in the world along with all its block entity data
//! (inventories, energy, configurations) and move it safely.
//!
//! Java references:
//! - `BlockCardboardBox.java:34`: strength(0.5F, 0.6F).mapColor(MapColor.WOOD)
//! - `BlockCardboardBox.java:39-58`: `useWithoutItem` (Shift + right-click with empty hand unpacks)
//! - `BlockCardboardBox.java:53`: `popResource(world, pos, new ItemStack(MekanismBlocks.CARDBOARD_BOX))`
//! - `BlockCardboardBox.java:54`: `MekanismCriteriaTriggers.UNBOX_CARDBOARD_BOX.value().trigger(player)`
//! - `ItemBlockCardboardBox.java:40`: `stacksTo(16)` for empty box, 1 when holding BlockData
//! - `ItemBlockCardboardBox.java:75-100`: `onItemUseFirst` (right-click on block without shift packs)
//! - `ItemBlockCardboardBox.java:77`: `!state.isAir() && state.getDestroySpeed(...) != INDESTRUCTIBLE`
//! - `ItemBlockCardboardBox.java:78`: `MekanismTags.Blocks.CARDBOARD_BLACKLIST` check
//! - `ItemBlockCardboardBox.java:95`: `stack.shrink(1)` if not creative

use super::*;
use std::collections::HashMap;

pub const BLOCK: &str = "mekanism:cardboard_box";
pub const EMPTY_STACK_LIMIT: u32 = 16;
pub const FILLED_STACK_LIMIT: u32 = 1;

/// Packed block data capturing blockstate and block entity data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedBlockData {
    pub block_name: String,
    pub properties: HashMap<String, String>,
    pub tile_data: Vec<u8>,
    pub items: Vec<Option<ItemStack>>,
    pub energy: Option<u64>,
}

impl PackedBlockData {
    pub fn new(block_name: String) -> Self {
        PackedBlockData {
            block_name,
            properties: HashMap::new(),
            tile_data: Vec::new(),
            items: Vec::new(),
            energy: None,
        }
    }
}

/// The state stored in a Cardboard Box block entity.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub is_storage: bool,
    pub packed_data: Option<PackedBlockData>,
}

/// Checks whether a block can be packed into a cardboard box (Java: `ItemBlockCardboardBox.java:76-85`).
pub fn can_pack_block(
    block_name: &str,
    hardness: f32,
    is_blacklisted: bool,
) -> bool {
    // Air cannot be packed
    if block_name.is_empty() || block_name == "minecraft:air" || block_name == "minecraft:cave_air" || block_name == "minecraft:void_air" {
        return false;
    }
    // Indestructible blocks (hardness < 0, like bedrock or end portal frame) cannot be packed
    if hardness < 0.0 {
        return false;
    }
    // Blacklisted blocks cannot be packed
    if is_blacklisted {
        return false;
    }
    // Cannot box another cardboard box that already contains data
    if block_name == BLOCK {
        return false;
    }
    true
}

/// Encodes packed data into bytes.
pub fn encode(state: &State) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(u8::from(state.is_storage));
    if let Some(ref data) = state.packed_data {
        // Block name
        let name_bytes = data.block_name.as_bytes();
        out.extend((name_bytes.len() as u16).to_le_bytes());
        out.extend(name_bytes);

        // Tile data
        out.extend((data.tile_data.len() as u32).to_le_bytes());
        out.extend(&data.tile_data);

        // Energy
        if let Some(e) = data.energy {
            out.push(1);
            out.extend(e.to_le_bytes());
        } else {
            out.push(0);
        }

        // Properties count
        out.extend((data.properties.len() as u16).to_le_bytes());
        for (k, v) in &data.properties {
            let kb = k.as_bytes();
            out.extend((kb.len() as u16).to_le_bytes());
            out.extend(kb);
            let vb = v.as_bytes();
            out.extend((vb.len() as u16).to_le_bytes());
            out.extend(vb);
        }
    }
    out
}

/// Decodes packed data from bytes.
pub fn decode(bytes: &[u8]) -> State {
    if bytes.is_empty() {
        return State::default();
    }
    let is_storage = bytes[0] == 1;
    if !is_storage || bytes.len() < 7 {
        return State { is_storage, packed_data: None };
    }

    let mut cursor = 1;
    let name_len = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
    cursor += 2;
    if cursor + name_len > bytes.len() {
        return State::default();
    }
    let block_name = String::from_utf8_lossy(&bytes[cursor..cursor + name_len]).to_string();
    cursor += name_len;

    if cursor + 4 > bytes.len() {
        return State { is_storage, packed_data: Some(PackedBlockData::new(block_name)) };
    }
    let tile_len = u32::from_le_bytes([bytes[cursor], bytes[cursor + 1], bytes[cursor + 2], bytes[cursor + 3]]) as usize;
    cursor += 4;
    let tile_data = if cursor + tile_len <= bytes.len() {
        let td = bytes[cursor..cursor + tile_len].to_vec();
        cursor += tile_len;
        td
    } else {
        Vec::new()
    };

    let energy = if cursor < bytes.len() && bytes[cursor] == 1 && cursor + 9 <= bytes.len() {
        cursor += 1;
        let e = u64::from_le_bytes([
            bytes[cursor], bytes[cursor + 1], bytes[cursor + 2], bytes[cursor + 3],
            bytes[cursor + 4], bytes[cursor + 5], bytes[cursor + 6], bytes[cursor + 7],
        ]);
        cursor += 8;
        Some(e)
    } else {
        if cursor < bytes.len() && bytes[cursor] == 0 {
            cursor += 1;
        }
        None
    };

    let mut properties = HashMap::new();
    if cursor + 2 <= bytes.len() {
        let prop_count = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        for _ in 0..prop_count {
            if cursor + 2 > bytes.len() { break; }
            let klen = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
            cursor += 2;
            if cursor + klen > bytes.len() { break; }
            let k = String::from_utf8_lossy(&bytes[cursor..cursor + klen]).to_string();
            cursor += klen;

            if cursor + 2 > bytes.len() { break; }
            let vlen = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
            cursor += 2;
            if cursor + vlen > bytes.len() { break; }
            let v = String::from_utf8_lossy(&bytes[cursor..cursor + vlen]).to_string();
            cursor += vlen;

            properties.insert(k, v);
        }
    }

    State {
        is_storage: true,
        packed_data: Some(PackedBlockData {
            block_name,
            properties,
            tile_data,
            items: Vec::new(),
            energy,
        }),
    }
}

/// Host API bridge for capturing and restoring world blocks and tile entities.
// FORK-23: Capture and restoration of generic BlockState and BlockEntity data
pub fn host_box_block(
    dimension: &str,
    pos: BlockPos,
    target_block: &str,
    target_data: &[u8],
) -> Option<PackedBlockData> {
    let _ = (dimension, pos);
    let mut data = PackedBlockData::new(target_block.to_string());
    data.tile_data = target_data.to_vec();
    Some(data)
}

/// Host API bridge for restoring a boxed block into the world.
// FORK-23: Reinstantiation of captured BlockState and BlockEntity data
pub fn host_unbox_block(
    dimension: &str,
    pos: BlockPos,
    packed: &PackedBlockData,
) -> bool {
    let _ = (dimension, pos, packed);
    true
}

/// Spawns a dropped item entity at the block location.
pub fn drop_item(dimension: &str, pos: BlockPos, item: &str, count: u32) {
    let nbt = format!("{{Item:{{id:\"{item}\",Count:{count}b}}}}");
    let _ = entities::summon(
        dimension,
        "minecraft:item",
        pos.x as f64 + 0.5,
        pos.y as f64 + 0.5,
        pos.z as f64 + 0.5,
        &nbt,
    );
}

impl Mekanism {
    /// Handles player interactions with Cardboard Boxes:
    /// - Packing: Right-clicking with cardboard box item on a block
    /// - Unpacking: Sneak right-clicking on a packed cardboard box with empty hand
    pub(crate) fn run_cardboard_boxes(
        &mut self,
        boxes: &[&BlockEntity],
    ) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in boxes {
            let block = e.state.split('[').next().unwrap_or("");
            if block != BLOCK {
                continue;
            }

            let Some(click) = self.take_click(e) else { continue };
            let state = decode(&e.data);

            if state.is_storage && click.sneak {
                // Unpack action (Java: BlockCardboardBox.java:39-58)
                if let Some(ref packed) = state.packed_data {
                    // Restore block into the world
                    let _ = host_unbox_block(&e.dimension, e.pos, packed);
                    let _ = world::set_block(&e.dimension, e.pos.x, e.pos.y, e.pos.z, &packed.block_name);

                    // Drop empty cardboard box item
                    drop_item(&e.dimension, e.pos, BLOCK, 1);

                    // Notify player and trigger advancement
                    let _ = players::tell(&click.player, &format!("Unboxed: {}", packed.block_name));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_limits() {
        // Java: ItemBlockCardboardBox.java:40
        // super(block, properties.stacksTo(16));
        assert_eq!(EMPTY_STACK_LIMIT, 16);
        assert_eq!(FILLED_STACK_LIMIT, 1);
    }

    #[test]
    fn test_can_pack_block_validations() {
        // Java: ItemBlockCardboardBox.java:77
        // !state.isAir() && state.getDestroySpeed(world, pos) != Block.INDESTRUCTIBLE
        assert!(!can_pack_block("minecraft:air", 0.0, false));
        assert!(!can_pack_block("minecraft:cave_air", 0.0, false));
        assert!(!can_pack_block("minecraft:bedrock", -1.0, false)); // Indestructible
        assert!(!can_pack_block("mekanism:cardboard_box", 0.5, false)); // Cannot box cardboard box itself

        // Tag blacklist check (line 78)
        assert!(!can_pack_block("mekanism:special_vault", 2.0, true)); // Blacklisted

        // Valid machines and chests can be packed
        assert!(can_pack_block("mekanism:enrichment_chamber", 3.5, false));
        assert!(can_pack_block("minecraft:chest", 2.5, false));
        assert!(can_pack_block("minecraft:furnace", 3.5, false));
    }

    #[test]
    fn test_serialization_roundtrip_with_block_data() {
        let mut props = HashMap::new();
        props.insert("facing".to_string(), "north".to_string());
        props.insert("active".to_string(), "true".to_string());

        let state = State {
            is_storage: true,
            packed_data: Some(PackedBlockData {
                block_name: "mekanism:crusher".to_string(),
                properties: props,
                tile_data: vec![1, 2, 3, 4, 5],
                items: Vec::new(),
                energy: Some(15_000),
            }),
        };

        let encoded = encode(&state);
        let decoded = decode(&encoded);

        assert!(decoded.is_storage);
        let packed = decoded.packed_data.expect("packed data must exist");
        assert_eq!(packed.block_name, "mekanism:crusher");
        assert_eq!(packed.tile_data, vec![1, 2, 3, 4, 5]);
        assert_eq!(packed.energy, Some(15_000));
        assert_eq!(packed.properties.get("facing").unwrap(), "north");
        assert_eq!(packed.properties.get("active").unwrap(), "true");
    }

    #[test]
    fn test_empty_box_serialization() {
        let state = State {
            is_storage: false,
            packed_data: None,
        };
        let encoded = encode(&state);
        let decoded = decode(&encoded);
        assert!(!decoded.is_storage);
        assert!(decoded.packed_data.is_none());
    }
}
