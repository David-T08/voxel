use bevy::prelude::*;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::Instant,
};

const MAX_TOTAL_GENERATE_QUEUE: usize = 1024;
const MAX_GENERATE_QUEUE_PER_FRAME: usize = 256;
const MAX_TOTAL_MESH_QUEUE: usize = 256;
const MAX_TOTAL_LIGHT_QUEUE: usize = 2048;
const MAX_UNLOAD_QUEUE_PER_FRAME: usize = 512;

use crate::{
    async_util::pipeline::PipelineQueue,
    chunks::ChunkPos,
    debugging::{DebugRenderStats, DebugSystemTimes},
    lighting::LightSeed,
    world::WorldState,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub generate: PipelineQueue<ColumnPos>,
    pub light: PipelineQueue<ChunkPos>,
    pub mesh: PipelineQueue<ChunkPos>,

    pub to_unload: VecDeque<ColumnPos>,
    pub queued_unload: HashSet<ColumnPos>,

    pub relight_again: HashSet<ChunkPos>,
    pub pending_light_seeds: HashMap<ChunkPos, Vec<LightSeed>>,
}

#[derive(Component)]
pub struct ChunkViewer {
    pub horizontal_radius: i32,
}

fn prune_generate_queue(streaming: &mut ChunkStreamingState, center: ColumnPos) {
    let mut retained: Vec<_> = streaming
        .generate
        .to_run
        .drain(..)
        .filter(|pos| streaming.desired.contains(pos))
        .collect();

    retained.sort_by_key(|pos| chunk_priority(center, *pos));
    retained.truncate(MAX_TOTAL_GENERATE_QUEUE);

    streaming.generate.to_run = retained.into_iter().collect();
    streaming.generate.queued = streaming.generate.to_run.iter().copied().collect();
}

fn prune_mesh_queue(streaming: &mut ChunkStreamingState, center: ColumnPos) {
    let mut retained: Vec<_> = streaming
        .mesh
        .to_run
        .drain(..)
        .filter(|pos| {
            let column = ColumnPos::new(pos.x, pos.z);
            streaming.active.contains(&column) && streaming.desired.contains(&column)
        })
        .collect();

    retained.sort_by_key(|pos| chunk_priority(center, ColumnPos::new(pos.x, pos.z)));

    streaming.mesh.to_run = retained.into_iter().collect();
    streaming.mesh.queued = streaming.mesh.to_run.iter().copied().collect();
}

fn prune_light_queue(streaming: &mut ChunkStreamingState, center: ColumnPos) {
    let mut retained: Vec<_> = streaming
        .light
        .to_run
        .drain(..)
        .filter(|pos| {
            let column = ColumnPos::new(pos.x, pos.z);
            streaming.active.contains(&column) && streaming.desired.contains(&column)
        })
        .collect();

    retained.sort_by_key(|pos| chunk_priority(center, ColumnPos::new(pos.x, pos.z)));

    streaming.light.to_run = retained.into_iter().collect();
    streaming.light.queued = streaming.light.to_run.iter().copied().collect();
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
    mut timing: ResMut<DebugSystemTimes>,
) {
    let start = Instant::now();
    let Some(center_transform) = viewers.iter().next() else {
        return;
    };

    let center_chunk = ChunkPos::from_translation(center_transform.translation);
    let center = ColumnPos::new(center_chunk.x, center_chunk.z);

    if streaming.generate.to_run.len() > MAX_TOTAL_GENERATE_QUEUE {
        prune_generate_queue(&mut streaming, center);
    }

    if streaming.mesh.to_run.len() > MAX_TOTAL_MESH_QUEUE {
        prune_mesh_queue(&mut streaming, center);
    }

    if streaming.light.to_run.len() > MAX_TOTAL_LIGHT_QUEUE {
        prune_light_queue(&mut streaming, center);
    }

    let mut missing: Vec<ColumnPos> = streaming
        .desired
        .iter()
        .copied()
        .filter(|pos| {
            !streaming.active.contains(pos)
                && !streaming.generate.running.contains(pos)
                && !streaming.generate.queued.contains(pos)
        })
        .collect();

    missing.sort_by_key(|pos| chunk_priority(center, *pos));

    let free_slots = MAX_TOTAL_GENERATE_QUEUE.saturating_sub(streaming.generate.to_run.len());
    let budget = free_slots.min(MAX_GENERATE_QUEUE_PER_FRAME);

    for pos in missing.into_iter().take(budget) {
        streaming.generate.enqueue_back(pos);
        // streaming.to_generate.push_back(pos);
        // streaming.queued_generate.insert(pos);
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
    timing.push_update_chunk_queues(start.elapsed().as_secs_f64() * 1000.0);
}

pub fn mark_chunk_for_light(
    world: &mut WorldState,
    streaming: &mut ChunkStreamingState,
    chunk_pos: ChunkPos,
) {
    streaming.pending_light_seeds.remove(&chunk_pos);

    if let Some(chunk) = world.get_chunk_mut(&chunk_pos) {
        chunk.light_version = chunk.light_version.wrapping_add(1);
        // debug!("[streaming]: updated light version for {chunk_pos}");
    }

    if streaming.light.running.contains(&chunk_pos) {
        streaming.relight_again.insert(chunk_pos);
        // debug!("[streaming]: flagged relight_again for {chunk_pos}");
    } else {
        streaming.light.enqueue_front(chunk_pos);
        // debug!("[streaming]: pushed to front lighting {chunk_pos}");
    }
}

pub fn mark_chunk_for_mesh(
    world: &mut WorldState,
    streaming: &mut ChunkStreamingState,
    chunk_pos: ChunkPos,
) {
    if let Some(chunk) = world.get_chunk_mut(&chunk_pos) {
        chunk.mesh_version = chunk.mesh_version.wrapping_add(1);
    }

    streaming.mesh.enqueue_front(chunk_pos);
}

pub fn clear_light_state_radius_1(streaming: &mut ChunkStreamingState, center: ChunkPos) {
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let pos = ChunkPos::new(center.x + dx, center.y + dy, center.z + dz);
                streaming.pending_light_seeds.remove(&pos);
            }
        }
    }
}

pub fn mark_chunk_and_neighbors_for_light(
    world: &mut WorldState,
    streaming: &mut ChunkStreamingState,
    chunk: ChunkPos,
) {
    mark_chunk_for_light(world, streaming, chunk);
    mark_chunk_for_light(
        world,
        streaming,
        ChunkPos::new(chunk.x + 1, chunk.y, chunk.z),
    );
    mark_chunk_for_light(
        world,
        streaming,
        ChunkPos::new(chunk.x - 1, chunk.y, chunk.z),
    );
    mark_chunk_for_light(
        world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y + 1, chunk.z),
    );
    mark_chunk_for_light(
        world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y - 1, chunk.z),
    );
    mark_chunk_for_light(
        world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y, chunk.z + 1),
    );
    mark_chunk_for_light(
        world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y, chunk.z - 1),
    );
}

pub fn mark_chunk_and_neighbors_for_mesh(
    mut world: &mut WorldState,
    streaming: &mut ChunkStreamingState,
    chunk: ChunkPos,
) {
    mark_chunk_for_mesh(&mut world, streaming, chunk);
    mark_chunk_for_mesh(
        &mut world,
        streaming,
        ChunkPos::new(chunk.x + 1, chunk.y, chunk.z),
    );
    mark_chunk_for_mesh(
        &mut world,
        streaming,
        ChunkPos::new(chunk.x - 1, chunk.y, chunk.z),
    );
    mark_chunk_for_mesh(
        &mut world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y + 1, chunk.z),
    );
    mark_chunk_for_mesh(
        &mut world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y - 1, chunk.z),
    );
    mark_chunk_for_mesh(
        &mut world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y, chunk.z + 1),
    );
    mark_chunk_for_mesh(
        &mut world,
        streaming,
        ChunkPos::new(chunk.x, chunk.y, chunk.z - 1),
    );
}
