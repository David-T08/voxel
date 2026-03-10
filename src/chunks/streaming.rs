use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

const MAX_TOTAL_GENERATE_QUEUE: usize = 1024;
const MAX_GENERATE_QUEUE_PER_FRAME: usize = 256;
const MAX_TOTAL_MESH_QUEUE: usize = 256;
const MAX_UNLOAD_QUEUE_PER_FRAME: usize = 512;

use crate::{chunks::ChunkPos, debugging::DebugRenderStats};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnPos {
    pub x: i32,
    pub z: i32,
}

impl ColumnPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

pub fn chunk_priority(center: ColumnPos, pos: ColumnPos) -> i32 {
    let dx = pos.x - center.x;
    let dz = pos.z - center.z;

    dx * dx + dz * dz
}

#[derive(Resource, Default, Debug)]
pub struct ChunkStreamingState {
    pub desired: HashSet<ColumnPos>,
    pub active: HashSet<ColumnPos>,
    pub generating: HashSet<ColumnPos>,

    pub to_generate: VecDeque<ColumnPos>,
    pub queued_generate: HashSet<ColumnPos>,

    pub to_mesh: VecDeque<ChunkPos>,
    pub queued_mesh: HashSet<ChunkPos>,
    pub meshing: HashSet<ChunkPos>,

    pub to_unload: VecDeque<ColumnPos>,
    pub queued_unload: HashSet<ColumnPos>,
}

#[derive(Component)]
pub struct ChunkViewer {
    pub horizontal_radius: i32,
}

fn prune_generate_queue(streaming: &mut ChunkStreamingState, center: ColumnPos) {
    let mut retained: Vec<_> = streaming
        .to_generate
        .drain(..)
        .filter(|pos| streaming.desired.contains(pos))
        .collect();

    retained.sort_by_key(|pos| chunk_priority(center, *pos));
    retained.truncate(MAX_TOTAL_GENERATE_QUEUE);

    streaming.to_generate = retained.into_iter().collect();
    streaming.queued_generate = streaming.to_generate.iter().copied().collect();
}

fn prune_mesh_queue(streaming: &mut ChunkStreamingState) {
    let retained: Vec<_> = streaming
        .to_mesh
        .drain(..)
        .filter(|pos| {
            let column = ColumnPos::new(pos.x, pos.z);
            streaming.active.contains(&column) && streaming.desired.contains(&column)
        })
        .collect();

    streaming.to_mesh = retained.into_iter().collect();
    streaming.queued_mesh = streaming.to_mesh.iter().copied().collect();
}

pub fn request_columns_for_viewers(
    viewers: Query<(&Transform, &ChunkViewer)>,
    mut streaming: ResMut<ChunkStreamingState>,
) {
    streaming.desired.clear();

    for (transform, viewer) in viewers {
        let center_chunk = ChunkPos::from_translation(transform.translation);
        let center = ColumnPos::new(center_chunk.x, center_chunk.z);

        for dx in -viewer.horizontal_radius..=viewer.horizontal_radius {
            for dz in -viewer.horizontal_radius..=viewer.horizontal_radius {
                streaming
                    .desired
                    .insert(ColumnPos::new(center.x + dx, center.z + dz));
            }
        }
    }
}

pub fn update_chunk_queues(
    viewers: Query<&Transform, With<ChunkViewer>>,

    mut streaming: ResMut<ChunkStreamingState>,
    mut stats: ResMut<DebugRenderStats>,
) {
    let Some(center_transform) = viewers.iter().next() else {
        return;
    };

    let center_chunk = ChunkPos::from_translation(center_transform.translation);
    let center = ColumnPos::new(center_chunk.x, center_chunk.z);

    if streaming.to_generate.len() > MAX_TOTAL_GENERATE_QUEUE {
        prune_generate_queue(&mut streaming, center);
    }

    if streaming.to_mesh.len() > MAX_TOTAL_MESH_QUEUE {
        prune_mesh_queue(&mut streaming);
    }

    let mut missing: Vec<ColumnPos> = streaming
        .desired
        .iter()
        .copied()
        .filter(|pos| {
            !streaming.active.contains(pos)
                && !streaming.generating.contains(pos)
                && !streaming.queued_generate.contains(pos)
        })
        .collect();

    missing.sort_by_key(|pos| chunk_priority(center, *pos));

    let free_slots = MAX_TOTAL_GENERATE_QUEUE.saturating_sub(streaming.to_generate.len());
    let budget = free_slots.min(MAX_GENERATE_QUEUE_PER_FRAME);

    for pos in missing.into_iter().take(budget) {
        streaming.to_generate.push_back(pos);
        streaming.queued_generate.insert(pos);
    }

    let mut stale_active: Vec<ColumnPos> = streaming
        .active
        .iter()
        .copied()
        .filter(|pos| !streaming.desired.contains(pos) && !streaming.queued_unload.contains(pos))
        .collect();

    stale_active.sort_by_key(|pos| -chunk_priority(center, *pos));

    for pos in stale_active.into_iter().take(MAX_UNLOAD_QUEUE_PER_FRAME) {
        streaming.to_unload.push_back(pos);
        streaming.queued_unload.insert(pos);
    }

    stats.chunks_to_generate = streaming.to_generate.len() as u64;
    stats.chunks_to_unload = streaming.to_unload.len() as u64;
    stats.chunks_to_mesh = streaming.to_mesh.len() as u64;
}
