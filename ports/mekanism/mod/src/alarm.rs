//! The industrial alarm (`TileEntityIndustrialAlarm`): a block that turns "on" (its lit model, and light) while redstone powers it.
//!
//! Not here: its siren, and the light and the red beam that sweep around it.
use super::*;

pub const ALARM: &str = "mekanism:industrial_alarm";

impl Mekanism {
    pub(crate) fn run_alarms(&mut self, alarms: &[&BlockEntity]) -> Vec<BlockEntityUpdate> {
        let mut out = Vec::new();
        for e in alarms {
            let key = (e.dimension.clone(), e.pos.x, e.pos.y, e.pos.z);
            if !self.alarm_set.contains(&key) {
                self.alarm_set.insert(key.clone());
                // A new alarm sits on the face of the block it was put against, turned toward the player that placed it.
                if e.data.first() != Some(&1) {
                    if let Some((yaw, pitch)) = players::online()
                        .into_iter()
                        .filter(|p| p.dimension == e.dimension)
                        .min_by(|a, b| {
                            let d = |p: &Player| (p.x - f64::from(e.pos.x)).powi(2) + (p.z - f64::from(e.pos.z)).powi(2);
                            d(a).total_cmp(&d(b))
                        })
                        .and_then(|p| player_control::get_look(&p.id))
                    {
                        let name = laser::facing_name(laser::facing_for(yaw, pitch));
                        let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "facing", name);
                    }
                }
            }
            let powered = world::get_redstone(&e.dimension, e.pos.x, e.pos.y, e.pos.z) > 0;
            let lit = e.state.contains("lit=true");
            if powered != lit {
                let _ = world::set_block_property(&e.dimension, e.pos.x, e.pos.y, e.pos.z, "lit", if powered { "true" } else { "false" });
            }
            if e.data.first() != Some(&1) {
                out.push(BlockEntityUpdate {
                    dimension: e.dimension.clone(),
                    pos: e.pos,
                    data: vec![1],
                    items: None,
                    energy: None,
                    energy_faces: None,
                    tanks: None,
                    tank_faces: None,
                    menu_data: None,
                    transmitter_faces: None,
                });
            }
        }
        out
    }
}
