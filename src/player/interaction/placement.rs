use bevy::prelude::*;

use crate::{blocks::BlockRegistry, chunks::{ChunkPos, streaming::{self, ChunkStreamingState}}, player::{Player, input::PlayerInput, interaction::selection::CurrentBlockTarget}, world::WorldState};

pub fn tick(
    mut streaming_state: ResMut<ChunkStreamingState>,
    mut world: ResMut<WorldState>,
    
    input: Single<&PlayerInput, With<Player>>,
    block_reg: Res<BlockRegistry>,
    target: Res<CurrentBlockTarget>
) {
    if let Some(target) = **target {
        if !input.mouse.m1_pressed {
            return
        }
        
        let new = target.voxel + target.normal;
        
        world.set_block(new.x, new.y, new.z, block_reg.names.name_to_id("core:stone").unwrap());
        streaming::mark_dirty_chunk(&mut streaming_state, ChunkPos::from_world(new.x, new.y, new.z));
    };
}