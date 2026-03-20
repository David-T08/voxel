use bevy::prelude::*;

use crate::{
    blocks::{BlockId, BlockRegistry}, chunks::{
        ChunkPos,
        streaming::{self, ChunkStreamingState},
    }, player::{Player, input::PlayerInput, interaction::selection::CurrentBlockTarget}, ui::screens::hotbar::{HotbarIcon, HotbarSlot, SelectedHotbarSlot}, world::WorldState
};

pub fn tick(
    mut streaming_state: ResMut<ChunkStreamingState>,
    mut world: ResMut<WorldState>,
    
    selected: Single<&HotbarSlot, With<SelectedHotbarSlot>>,

    input: Res<PlayerInput>,
    block_reg: Res<BlockRegistry>,
    target: Res<CurrentBlockTarget>,
) {
    if let Some(target) = **target {
        if !input.mouse.m1_pressed {
            return;
        }

        let new = target.voxel + target.normal;

        world.set_block(
            new.x,
            new.y,
            new.z,
            selected.block,
        );
        streaming::mark_chunk_and_neighbors_for_light(
            &mut world,
            &mut streaming_state,
            ChunkPos::from_world(new.x, new.y, new.z),
        );
    };
}
