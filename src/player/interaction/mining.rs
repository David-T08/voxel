use bevy::prelude::*;

use crate::{blocks::BlockId, chunks::{ChunkPos, streaming::{self, ChunkStreamingState}}, player::{Player, input::PlayerInput, interaction::selection::CurrentBlockTarget}, world::WorldState};

pub fn tick(
    mut streaming_state: ResMut<ChunkStreamingState>,
    mut world: ResMut<WorldState>,
    
    input: Single<&PlayerInput, With<Player>>,
    target: Res<CurrentBlockTarget>
) {
    if let Some(target) = **target {
        if !input.mouse.m2_pressed {
            return
        }
        
        world.set_block(target.voxel.x, target.voxel.y, target.voxel.z, BlockId(0));
        streaming::mark_chunk_and_neighbors_for_light(&mut streaming_state, ChunkPos::from_world(target.voxel.x, target.voxel.y, target.voxel.z));
    };
}