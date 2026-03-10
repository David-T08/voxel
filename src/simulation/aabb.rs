use bevy::prelude::*;

const EPSILON: f32 = 0.001;
use crate::{blocks::BlockId, world::WorldState};

#[derive(Debug)]
pub struct AABBWorldCollision {
    pub position: IVec3,
    pub block: BlockId
}

pub fn collides_with_world(min: Vec3, max: Vec3, world: &WorldState) -> Option<AABBWorldCollision> {
    let v_min = min.floor().as_ivec3();
    let v_max = (max - Vec3::splat(EPSILON)).floor().as_ivec3();
    
    for x in v_min.x..=v_max.x {
        for y in v_min.y..=v_max.y {
            for z in v_min.z..=v_max.z {
                if world.is_solid(x, y, z) {
                    return Some(AABBWorldCollision {
                        block: world.get_block(x, y, z),
                        position: IVec3::new(x, y, z),
                    });
                }
            }
        }
    }
    
    None
}