use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
};

use crate::{
    async_util::pipeline::VersionedTask, blocks::{AIR_ID, BlockId, BlockRegistry}, chunks::{
        CHUNK_SIZE, CHUNK_VOLUME, ChunkData, ChunkPos,
        render::ChunkRenderMap,
        streaming::{self, ChunkStreamingState, ColumnPos},
    }, debugging::DebugSystemTimes, world::WorldState
};

const MAX_LIGHTING_TASKS_PER_FRAME: usize = 64;
const MAX_ACTIVE_LIGHTING_TASKS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightSeed {
    pub target: ChunkPos,
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub light: u8,
}

pub struct LightTaskInput {
    pub center_voxels: [BlockId; CHUNK_VOLUME],
    pub center_light: [u8; CHUNK_VOLUME],

    pub left: NeighborChunkLightInput,
    pub right: NeighborChunkLightInput,
    pub bottom: NeighborChunkLightInput,
    pub top: NeighborChunkLightInput,
    pub back: NeighborChunkLightInput,
    pub front: NeighborChunkLightInput,

    pub incoming: Vec<LightSeed>,
}

pub struct LightTaskResult {
    pub light: [u8; CHUNK_VOLUME],
    pub overflow: Vec<LightSeed>,
}

#[derive(Clone)]
pub struct NeighborChunkLightInput {
    pub voxels: Option<[BlockId; CHUNK_VOLUME]>,
    pub light: Option<[u8; CHUNK_VOLUME]>,
}

pub type VersionedLightTaskInput = VersionedTask<ChunkPos, LightTaskInput>;
pub type VersionedLightTaskResult= VersionedTask<ChunkPos, LightTaskResult>;

#[derive(Component)]
pub struct ChunkLightTask(pub Task<VersionedLightTaskResult>);

pub fn generate_light(input: VersionedLightTaskInput, registry: &BlockRegistry) -> VersionedLightTaskResult {
    let mut light = [0u8; CHUNK_VOLUME];//input.data.center_light;
    let mut queue = VecDeque::<(usize, usize, usize)>::new();
    let mut overflow = Vec::<LightSeed>::new();

    let data = &input.data;
    fill_vertical_sun(data, registry, &mut light);

    // Add local light seeds
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let i = ChunkData::index(x, y, z);
                let id = data.center_voxels[i];

                if id == AIR_ID {
                    continue;
                }

                let emit = registry.get_block(id).unwrap().emission;
                if emit > block(light[i]) {
                    light[i] = with_block(light[i], emit);
                    queue.push_back((x, y, z));
                }
            }
        }
    }

    // Seed light that overflowed in
    for seed in &data.incoming {
        let x = seed.x as usize;
        let y = seed.y as usize;
        let z = seed.z as usize;
        let i = ChunkData::index(x, y, z);

        if registry.is_opaque(data.center_voxels[i]) {
            continue;
        }

        if seed.light > block(light[i]) {
            light[i] = with_block(light[i], seed.light);
            queue.push_back((x, y, z));
        }
    }

    // Seed light from borders
    seed_from_neighbor_borders(&input, registry, &mut light, &mut queue);

    while let Some((x, y, z)) = queue.pop_front() {
        let i = ChunkData::index(x, y, z);
        let current = block(light[i]);

        if current <= 1 {
            continue;
        }

        try_step(
            input.key,
            data,
            registry,
            &mut light,
            &mut queue,
            &mut overflow,
            x as i32 - 1,
            y as i32,
            z as i32,
            current - 1,
        );

        try_step(
            input.key,
            data,
            registry,
            &mut light,
            &mut queue,
            &mut overflow,
            x as i32 + 1,
            y as i32,
            z as i32,
            current - 1,
        );

        try_step(
            input.key,
            data,
            registry,
            &mut light,
            &mut queue,
            &mut overflow,
            x as i32,
            y as i32 - 1,
            z as i32,
            current - 1,
        );

        try_step(
            input.key,
            data,
            registry,
            &mut light,
            &mut queue,
            &mut overflow,
            x as i32,
            y as i32 + 1,
            z as i32,
            current - 1,
        );

        try_step(
            input.key,
            data,
            registry,
            &mut light,
            &mut queue,
            &mut overflow,
            x as i32,
            y as i32,
            z as i32 - 1,
            current - 1,
        );

        try_step(
            input.key,
            data,
            registry,
            &mut light,
            &mut queue,
            &mut overflow,
            x as i32,
            y as i32,
            z as i32 + 1,
            current - 1,
        );
    }

    remove_duplicates_overflow_max(&mut overflow);

    VersionedTask {
        key: input.key,
        version: input.version,
        data: LightTaskResult { light, overflow }
    }
}

fn queue_chunk_for_light(streaming: &mut ChunkStreamingState, pos: ChunkPos) {
    if streaming.light.running.contains(&pos) {
        streaming.relight_again.insert(pos);
    } else {
        streaming.light.enqueue_back(pos);
    }
}

fn queue_chunk_neighbors_for_light(streaming: &mut ChunkStreamingState, center: ChunkPos) {
    queue_chunk_for_light(streaming, ChunkPos::new(center.x - 1, center.y, center.z));
    queue_chunk_for_light(streaming, ChunkPos::new(center.x + 1, center.y, center.z));
    queue_chunk_for_light(streaming, ChunkPos::new(center.x, center.y - 1, center.z));
    queue_chunk_for_light(streaming, ChunkPos::new(center.x, center.y + 1, center.z));
    queue_chunk_for_light(streaming, ChunkPos::new(center.x, center.y, center.z - 1));
    queue_chunk_for_light(streaming, ChunkPos::new(center.x, center.y, center.z + 1));
}

fn try_step(
    pos: ChunkPos,
    input: &LightTaskInput,
    registry: &BlockRegistry,
    light: &mut [u8; CHUNK_VOLUME],
    queue: &mut VecDeque<(usize, usize, usize)>,
    overflow: &mut Vec<LightSeed>,
    nx: i32,
    ny: i32,
    nz: i32,
    candidate: u8,
) {
    if candidate == 0 {
        return;
    }

    let size = CHUNK_SIZE as i32;

    // is inside local chunk
    if nx >= 0 && nx < size && ny >= 0 && ny < size && nz >= 0 && nz < size {
        let x = nx as usize;
        let y = ny as usize;
        let z = nz as usize;
        let i = ChunkData::index(x, y, z);

        if registry.is_opaque(input.center_voxels[i]) {
            return;
        }

        if candidate > block(light[i]) {
            light[i] = with_block(light[i], candidate);
            queue.push_back((x, y, z));
        }

        return;
    }

    // overflowed
    let (target_chunk, tx, ty, tz, neighbor) = map_overflow_target(input, pos, nx, ny, nz);
    let Some(neighbor) = neighbor else {
        return;
    };

    let ni = ChunkData::index(tx as usize, ty as usize, tz as usize);

    let Some(neighbor_voxels) = neighbor.voxels.as_ref() else {
        return;
    };

    if registry.is_opaque(neighbor_voxels[ni]) {
        return;
    }

    if let Some(neighbor_light) = neighbor.light.as_ref() {
        if candidate <= block(neighbor_light[ni]) {
            return;
        }
    }

    overflow.push(LightSeed {
        target: target_chunk,
        x: tx,
        y: ty,
        z: tz,
        light: candidate,
    });
}

fn map_overflow_target<'a>(
    input: &'a LightTaskInput,
    pos: ChunkPos,
    nx: i32,
    ny: i32,
    nz: i32,
) -> (ChunkPos, u8, u8, u8, Option<&'a NeighborChunkLightInput>) {
    let max = CHUNK_SIZE as i32 - 1;

    if nx < 0 {
        return (
            ChunkPos::new(pos.x - 1, pos.y, pos.z),
            max as u8,
            ny as u8,
            nz as u8,
            Some(&input.left),
        );
    }

    if nx >= CHUNK_SIZE as i32 {
        return (
            ChunkPos::new(pos.x + 1, pos.y, pos.z),
            0,
            ny as u8,
            nz as u8,
            Some(&input.right),
        );
    }

    if ny < 0 {
        return (
            ChunkPos::new(pos.x, pos.y - 1, pos.z),
            nx as u8,
            max as u8,
            nz as u8,
            Some(&input.bottom),
        );
    }

    if ny >= CHUNK_SIZE as i32 {
        return (
            ChunkPos::new(pos.x, pos.y + 1, pos.z),
            nx as u8,
            0,
            nz as u8,
            Some(&input.top),
        );
    }

    if nz < 0 {
        return (
            ChunkPos::new(pos.x, pos.y, pos.z - 1),
            nx as u8,
            ny as u8,
            max as u8,
            Some(&input.back),
        );
    }

    (
        ChunkPos::new(pos.x, pos.y, pos.z + 1),
        nx as u8,
        ny as u8,
        0,
        Some(&input.front),
    )
}

fn try_seed_cell(
    input: &VersionedLightTaskInput,
    registry: &BlockRegistry,
    light: &mut [u8; CHUNK_VOLUME],
    queue: &mut VecDeque<(usize, usize, usize)>,
    dst_x: usize,
    dst_y: usize,
    dst_z: usize,
    candidate: u8,
) {
    if candidate == 0 {
        return;
    }

    let dst_i = ChunkData::index(dst_x, dst_y, dst_z);

    if registry.is_opaque(input.data.center_voxels[dst_i]) {
        return;
    }

    if candidate > block(light[dst_i]) {
        light[dst_i] = with_block(light[dst_i], candidate);
        queue.push_back((dst_x, dst_y, dst_z));
    }
}

fn seed_from_neighbor_borders(
    input: &VersionedLightTaskInput,
    registry: &BlockRegistry,
    light: &mut [u8; CHUNK_VOLUME],
    queue: &mut VecDeque<(usize, usize, usize)>,
) {
    let max = CHUNK_SIZE - 1;

    // left neighbor seeds
    if let (Some(neighbor_voxels), Some(neighbor_light)) =
        (input.data.left.voxels.as_ref(), input.data.left.light.as_ref())
    {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let src_i = ChunkData::index(max, y, z);
                if registry.is_opaque(neighbor_voxels[src_i]) {
                    continue;
                }

                let candidate = block(neighbor_light[src_i]).saturating_sub(1);
                try_seed_cell(input, registry, light, queue, 0, y, z, candidate);
            }
        }
    }

    // right neighbor seeds
    if let (Some(neighbor_voxels), Some(neighbor_light)) =
        (input.data.right.voxels.as_ref(), input.data.right.light.as_ref())
    {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let src_i = ChunkData::index(0, y, z);
                if registry.is_opaque(neighbor_voxels[src_i]) {
                    continue;
                }

                let candidate = block(neighbor_light[src_i]).saturating_sub(1);
                try_seed_cell(input, registry, light, queue, max, y, z, candidate);
            }
        }
    }

    // bottom neighbor seeds
    if let (Some(neighbor_voxels), Some(neighbor_light)) =
        (input.data.bottom.voxels.as_ref(), input.data.bottom.light.as_ref())
    {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let src_i = ChunkData::index(x, max, z);
                if registry.is_opaque(neighbor_voxels[src_i]) {
                    continue;
                }

                let candidate = block(neighbor_light[src_i]).saturating_sub(1);
                try_seed_cell(input, registry, light, queue, x, 0, z, candidate);
            }
        }
    }

    // top neighbor seeds
    if let (Some(neighbor_voxels), Some(neighbor_light)) =
        (input.data.top.voxels.as_ref(), input.data.top.light.as_ref())
    {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let src_i = ChunkData::index(x, 0, z);
                if registry.is_opaque(neighbor_voxels[src_i]) {
                    continue;
                }

                let candidate = block(neighbor_light[src_i]).saturating_sub(1);
                try_seed_cell(input, registry, light, queue, x, max, z, candidate);
            }
        }
    }

    // back neighbor seeds
    if let (Some(neighbor_voxels), Some(neighbor_light)) =
        (input.data.back.voxels.as_ref(), input.data.back.light.as_ref())
    {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let src_i = ChunkData::index(x, y, max);
                if registry.is_opaque(neighbor_voxels[src_i]) {
                    continue;
                }

                let candidate = block(neighbor_light[src_i]).saturating_sub(1);
                try_seed_cell(input, registry, light, queue, x, y, 0, candidate);
            }
        }
    }

    // front neighbor seeds
    if let (Some(neighbor_voxels), Some(neighbor_light)) =
        (input.data.front.voxels.as_ref(), input.data.front.light.as_ref())
    {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let src_i = ChunkData::index(x, y, 0);
                if registry.is_opaque(neighbor_voxels[src_i]) {
                    continue;
                }

                let candidate = block(neighbor_light[src_i]).saturating_sub(1);
                try_seed_cell(input, registry, light, queue, x, y, max, candidate);
            }
        }
    }
}

fn fill_vertical_sun(
    input: &LightTaskInput,
    registry: &BlockRegistry,
    light: &mut [u8; CHUNK_VOLUME],
) {
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let mut s = 15;

            if let (Some(top_voxels), Some(top_light)) =
                (input.top.voxels.as_ref(), input.top.light.as_ref())
            {
                let src_i = ChunkData::index(x, 0, z);

                if registry.is_opaque(top_voxels[src_i]) {
                    s = 0;
                } else {
                    s = sun(top_light[src_i]);
                }
            }

            if s == 0 {
                continue;
            }

            for y in (0..CHUNK_SIZE).rev() {
                let i = ChunkData::index(x, y, z);

                if registry.is_opaque(input.center_voxels[i]) {
                    break;
                }

                light[i] = with_sun(light[i], s);
            }
        }
    }
}

fn remove_duplicates_overflow_max(overflow: &mut Vec<LightSeed>) {
    let mut best = HashMap::<(ChunkPos, u8, u8, u8), u8>::new();

    for seed in overflow.drain(..) {
        let key = (seed.target, seed.x, seed.y, seed.z);
        best.entry(key)
            .and_modify(|v| *v = (*v).max(seed.light))
            .or_insert(seed.light);
    }

    overflow.extend(
        best.into_iter()
            .map(|((target, x, y, z), light)| LightSeed {
                target,
                x,
                y,
                z,
                light,
            }),
    );
}

pub fn spawn_lighting_tasks(
    mut timing: ResMut<DebugSystemTimes>,
    mut commands: Commands,
    mut streaming: ResMut<ChunkStreamingState>,
    world: Res<WorldState>,
    block_reg: Res<BlockRegistry>,
) {
    let start = Instant::now();

    let pool = AsyncComputeTaskPool::get();

    for _ in 0..MAX_LIGHTING_TASKS_PER_FRAME {
        if streaming.light.running.len() >= MAX_ACTIVE_LIGHTING_TASKS {
            break;
        }

        let Some(center) = streaming.light.pop_next() else {
            break;
        };

        // streaming.queued_light.remove(&center);
        // if streaming.lighting.contains(&center) {
        //     continue;
        // }
        // streaming.lighting.insert(center);

        let reg = block_reg.clone();

        let Some(center_chunk) = world.get_chunk(&center) else {
            streaming.light.finish(&center);
            continue;
        };

        let input = VersionedLightTaskInput {
            key: center,
            version: center_chunk.light_version,
            data: LightTaskInput {
                center_voxels: center_chunk.blocks.clone(),
                center_light: center_chunk.light,

                left: neighbor_input(&world, ChunkPos::new(center.x - 1, center.y, center.z)),
                right: neighbor_input(&world, ChunkPos::new(center.x + 1, center.y, center.z)),
                bottom: neighbor_input(&world, ChunkPos::new(center.x, center.y - 1, center.z)),
                top: neighbor_input(&world, ChunkPos::new(center.x, center.y + 1, center.z)),
                back: neighbor_input(&world, ChunkPos::new(center.x, center.y, center.z - 1)),
                front: neighbor_input(&world, ChunkPos::new(center.x, center.y, center.z + 1)),

                incoming: streaming
                    .pending_light_seeds
                    .remove(&center)
                    .unwrap_or_default(),
            }
        };

        let task = pool.spawn(async move { generate_light(input, &reg) });

        commands.spawn(ChunkLightTask(task));
    }

    timing.push_spawn_light_tasks(start.elapsed().as_secs_f64() * 1000.0);
}

fn neighbor_input(world: &WorldState, pos: ChunkPos) -> NeighborChunkLightInput {
    if let Some(chunk) = world.get_chunk(&pos) {
        NeighborChunkLightInput {
            voxels: Some(chunk.blocks.clone()),
            light: Some(chunk.light),
        }
    } else {
        NeighborChunkLightInput {
            voxels: None,
            light: None,
        }
    }
}

pub fn collect_lighting_tasks(
    mut timing: ResMut<DebugSystemTimes>,
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkLightTask)>,
    mut world: ResMut<WorldState>,
    mut streaming_data: ResMut<ChunkStreamingState>,
    render_map: Res<ChunkRenderMap>,
) {
    let start = Instant::now();
    for (entity, mut task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            streaming_data.light.finish(&result.key);

            let mut valid_center = false;
            let mut changed = false;

            if let Some(chunk) = world.get_chunk_mut(&result.key) {
                if chunk.light_version == result.version {
                    valid_center = true;

                    let needs_rerun = streaming_data.relight_again.contains(&result.key);

                    if !needs_rerun {
                        let old_light = chunk.light;
                        changed = old_light != result.data.light;
                        chunk.light = result.data.light;

                        if old_light != result.data.light
                            || !render_map.entities.contains_key(&result.key)
                            || true
                        {
                            debug!("[lighting] {} marking for mesh", result.key);
                            streaming::mark_chunk_for_mesh(
                                &mut world,
                                &mut streaming_data,
                                result.key,
                            );
                        }
                    }
                }
            }

            if valid_center && changed {
                queue_chunk_neighbors_for_light(&mut streaming_data, result.key);
            }

            if valid_center {
                for seed in result.data.overflow {
                    let target = seed.target;

                    if merge_pending_seed(&mut streaming_data.pending_light_seeds, seed) {
                        queue_chunk_for_light(&mut streaming_data, target);
                    }
                }
            }

            if valid_center && streaming_data.relight_again.remove(&result.key) {
                streaming_data.light.enqueue_back(result.key);
            }

            commands.entity(entity).despawn();
        }
    }

    timing.push_collect_light_tasks(start.elapsed().as_secs_f64() * 1000.0);
}

fn merge_pending_seed(
    pending: &mut std::collections::HashMap<ChunkPos, Vec<LightSeed>>,
    seed: LightSeed,
) -> bool {
    let seeds = pending.entry(seed.target).or_default();

    for existing in seeds.iter_mut() {
        if existing.x == seed.x && existing.y == seed.y && existing.z == seed.z {
            if seed.light > existing.light {
                existing.light = seed.light;
                return true;
            } else {
                return false;
            }
        }
    }

    seeds.push(seed);
    true
}

#[inline]
fn pack_light(sun: u8, block: u8) -> u8 {
    ((sun & 0x0F) << 4) | (block & 0x0F)
}

#[inline]
pub fn sun(light: u8) -> u8 {
    light >> 4
}

#[inline]
pub fn block(light: u8) -> u8 {
    light & 0x0F
}

#[inline]
pub fn with_sun(light: u8, sun: u8) -> u8 {
    (light & 0x0F) | ((sun & 0x0F) << 4)
}

#[inline]
pub fn with_block(light: u8, block: u8) -> u8 {
    (light & 0xF0) | (block & 0x0F)
}

#[inline]
pub fn max_channel(light: u8) -> u8 {
    sun(light).max(block(light))
}

pub fn light_to_vec2(light: u8) -> [f32; 2] {
    [sun(light) as f32 / 15.0, block(light) as f32 / 15.0]
}

pub fn light_to_color(light: u8, sky_color: [f32; 3]) -> [f32; 4] {
    let s = sun(light) as f32 / 15.0;
    let b = block(light) as f32 / 15.0;

    let r = s * sky_color[0] + b * 1.00;
    let g = s * sky_color[1] + b * 0.85;
    let bl = s * sky_color[2] + b * 0.60;

    [
        r.clamp(0.0045, 1.0),
        g.clamp(0.0045, 1.0),
        bl.clamp(0.0045, 1.0),
        1.0,
    ]
}

pub fn light_to_debug_color(light: u8) -> [f32; 4] {
    [
        sun(light) as f32 / 15.0,
        block(light) as f32 / 15.0,
        0.0,
        1.0,
    ]
}
