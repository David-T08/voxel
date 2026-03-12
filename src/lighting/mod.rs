use std::collections::VecDeque;

use bevy::{prelude::*, tasks::{AsyncComputeTaskPool, Task, futures_lite::future}};

use crate::{blocks::{AIR_ID, BlockId, BlockRegistry}, chunks::{CHUNK_SIZE, CHUNK_VOLUME, ChunkData, ChunkPos, streaming::{ChunkStreamingState, ColumnPos}}, world::WorldState};

const MAX_LIGHTING_TASKS_PER_FRAME: usize = 16;
const MAX_ACTIVE_LIGHTING_TASKS: usize = 8;

pub struct LightTaskResult {
    pub center: ChunkPos,
    pub updated: Vec<(ChunkPos, [u8; 16 * 16 * 16])>,
}

#[derive(Component)]
pub struct ChunkLightTask(pub Task<LightTaskResult>);

pub fn generate_light(chunk: ChunkPos, voxels: [BlockId; CHUNK_VOLUME], registry: &BlockRegistry) -> LightTaskResult {
    let mut light = [0u8; CHUNK_VOLUME];
    let mut queue = VecDeque::<(usize, usize, usize)>::new();

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let i = ChunkData::index(x,y,z);
                let id = voxels[i];
                
                if id == AIR_ID {
                    continue;
                }
                
                let emit = registry.get_block(id).unwrap().emission;
                if emit > 0 {
                    light[i] = with_block(light[i], emit);
                    queue.push_back((x, y, z));
                }
            }
        }
    }

    while let Some((x, y, z)) = queue.pop_front() {
        let i = ChunkData::index(x, y, z);
        let current = block(light[i]);
        
        if current <= 1 {
            continue;
        }

        let neighbors = [
            (x > 0, x.wrapping_sub(1), y, z),
            (x + 1 < CHUNK_SIZE, x + 1, y, z),
            (y > 0, x, y.wrapping_sub(1), z),
            (y + 1 < CHUNK_SIZE, x, y + 1, z),
            (z > 0, x, y, z.wrapping_sub(1)),
            (z + 1 < CHUNK_SIZE, x, y, z + 1),
        ];

        for (valid, nx, ny, nz) in neighbors {
            if !valid {
                continue;
            }
            
            let ni = ChunkData::index(nx, ny, nz);
            let nid = voxels[ni];

            if registry.is_opaque(nid) {
                continue;
            }
            
            let candidate = current - 1;
            if candidate > block(light[ni]) {
                light[ni] = with_block(light[ni], candidate);
                queue.push_back((nx, ny, nz));
                
            }
        }
    }
    
    LightTaskResult {
        center: chunk,
        updated: vec![(chunk, light)],
    }
}

pub fn spawn_lighting_tasks(
    mut commands: Commands,
    mut streaming: ResMut<ChunkStreamingState>,
    world: Res<WorldState>,
    block_reg: Res<BlockRegistry>
) {
    let pool = AsyncComputeTaskPool::get();
    
    for _ in 0..MAX_LIGHTING_TASKS_PER_FRAME {
        if streaming.lighting.len() >= MAX_ACTIVE_LIGHTING_TASKS {
            break;
        }
        
        let Some(center) = streaming.to_light.pop_front() else {
            break;
        };
        
        streaming.queued_light.remove(&center);
        if streaming.lighting.contains(&center) {
            continue;
        }
        streaming.lighting.insert(center);
            
        let reg = block_reg.clone();
        let Some(chunk) = world.get_chunk(&center) else {
            streaming.lighting.remove(&center);
            continue;
        };
        
        let voxels = chunk.blocks.clone();
        let task_pos = center;
        
        let task = pool.spawn(async move {
            generate_light(task_pos, voxels, &reg)
        });
        
        commands.spawn(ChunkLightTask(task));
    }
}

pub fn collect_lighting_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkLightTask)>,
    mut world: ResMut<WorldState>,
    mut streaming: ResMut<ChunkStreamingState>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            streaming.lighting.remove(&result.center);

            for (chunk_pos, light_data) in result.updated {
                let column = ColumnPos::new(chunk_pos.x, chunk_pos.z);
                
                if !streaming.active.contains(&column) || !streaming.desired.contains(&column) {
                    continue;
                }

                let Some(chunk) = world.get_chunk_mut(&chunk_pos) else {
                    continue;
                };

                // if chunk.light != light_data {
                    if streaming.queued_mesh.insert(chunk_pos) {
                        streaming.to_mesh.push_back(chunk_pos);
                    }
                // }
                
                chunk.light = light_data;
                
            }

            commands.entity(entity).despawn();
        }
    }
}

#[inline]
fn pack_light(sun: u8, block: u8) -> u8 {
    ((sun & 0x0F) << 4) | (block & 0x0F)
}

#[inline]
fn sun(light: u8) -> u8 {
    light >> 4
}

#[inline]
fn block(light: u8) -> u8 {
    light & 0x0F
}

#[inline]
fn with_sun(light: u8, sun: u8) -> u8 {
    (light & 0x0F) | ((sun & 0x0F) << 4)
}

#[inline]
fn with_block(light: u8, block: u8) -> u8 {
    (light & 0xF0) | (block & 0x0F)
}

#[inline]
fn max_channel(light: u8) -> u8 {
    sun(light).max(block(light))
}

#[inline]
pub fn light_to_color(light: u8) -> [f32; 4] {
    let s = sun(light) as f32 / 15.0;
    let b = block(light) as f32 / 15.0;

    let r = (s * 1.00 + b * 1.00).min(1.0);
    let g = (s * 1.00 + b * 0.85).min(1.0);
    let bl = (s * 1.00 + b * 0.60).min(1.0);

    [
        r.max(0.05),
        g.max(0.05),
        bl.max(0.05),
        1.0,
    ]
}