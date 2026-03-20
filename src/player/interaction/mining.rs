use bevy::prelude::*;

use crate::{
    blocks::BlockId,
    chunks::{
        ChunkPos,
        streaming::{self, ChunkStreamingState},
    },
    player::{Player, input::PlayerInput, interaction::selection::CurrentBlockTarget},
    world::WorldState,
};

pub fn tick(
    mut streaming_state: ResMut<ChunkStreamingState>,
    mut world: ResMut<WorldState>,

    input: Res<PlayerInput>,
    target: Res<CurrentBlockTarget>,
) {
    if let Some(target) = **target {
        if !input.mouse.m2_pressed {
            return;
        }

        world.set_block(target.voxel.x, target.voxel.y, target.voxel.z, BlockId(0));
        debug!(
            "[interaction/mining]: Broke {} at {}",
            target.voxel,
            ChunkPos::from_world(target.voxel.x, target.voxel.y, target.voxel.z)
        );

        let center = ChunkPos::from_world(target.voxel.x, target.voxel.y, target.voxel.z);

        streaming::clear_light_state_radius_1(&mut streaming_state, center);
        streaming::mark_chunk_and_neighbors_for_light(&mut world, &mut streaming_state, center);
        // TODO: this causes block breaks to not be re-rendered sometimes since mesh version gets bumped and cancels
        // look into it later
        // streaming::mark_chunk_and_neighbors_for_mesh(&mut world, &mut streaming_state, center);
    };
}
