use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::blocks::BlockRegistry;
use crate::chunks::streaming::{ChunkStreamingState, ColumnPos};
use crate::chunks::{ChunkData, ChunkPos};
use crate::world::generation::{TerrainNoise, generate_chunk};
use crate::world::{WORLD_MAX_CHUNK_Y, WORLD_MIN_CHUNK_Y, WorldState};

#[derive(Component)]
pub struct ChunkGenTask(pub Task<(ColumnPos, Vec<(ChunkPos, ChunkData)>)>);

const MAX_GEN_TASKS_PER_FRAME: usize = 16;
const MAX_ACTIVE_GEN_TASKS: usize = 16;

pub fn generate_column(
    generator: &TerrainNoise,
    column: ColumnPos,
    registry: &BlockRegistry,
) -> Vec<(ChunkPos, ChunkData)> {
    let mut out = Vec::new();

    for cy in WORLD_MIN_CHUNK_Y..=WORLD_MAX_CHUNK_Y {
        let chunk_pos = ChunkPos::new(column.x, cy, column.z);
        let chunk = generate_chunk(generator, &chunk_pos, registry);
        out.push((chunk_pos, chunk));
    }

    out
}

pub fn spawn_chunk_gen_tasks(
    mut commands: Commands,
    mut streaming: ResMut<ChunkStreamingState>,
    world: Res<WorldState>,
    block_reg: Res<BlockRegistry>,
) {
    let pool = AsyncComputeTaskPool::get();
    let seed = world.seed;

    for _ in 0..MAX_GEN_TASKS_PER_FRAME {
        if streaming.generating.len() >= MAX_ACTIVE_GEN_TASKS {
            break;
        }

        let Some(column) = streaming.to_generate.pop_front() else {
            break;
        };

        streaming.queued_generate.remove(&column);
        if !streaming.desired.contains(&column)
            || streaming.active.contains(&column)
            || streaming.generating.contains(&column)
        {
            continue;
        }

        streaming.generating.insert(column);

        let registry = block_reg.clone();
        let task_pos = column;

        let cloned = world.generator.clone();
        let task = pool.spawn(async move {
            let chunk = generate_column(&cloned, column, &registry);
            (task_pos, chunk)
        });

        commands.spawn(ChunkGenTask(task));
    }
}

pub fn collect_chunk_gen_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkGenTask)>,
    mut world: ResMut<WorldState>,
    mut streaming: ResMut<ChunkStreamingState>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some((column, chunks)) = future::block_on(future::poll_once(&mut task.0)) {
            streaming.generating.remove(&column);

            if streaming.desired.contains(&column) {
                streaming.active.insert(column);

                for (chunk_pos, chunk) in chunks {
                    if world.get_chunk(&chunk_pos).is_none() {
                        world.insert_chunk(chunk_pos, chunk);
                    }
                }
                
                for cy in WORLD_MIN_CHUNK_Y..=WORLD_MAX_CHUNK_Y {
                    for column in [
                        column,
                        ColumnPos::new(column.x - 1, column.z),
                        ColumnPos::new(column.x + 1, column.z),
                        ColumnPos::new(column.x, column.z - 1),
                        ColumnPos::new(column.x, column.z + 1),
                    ] {
                        let pos = ChunkPos::new(column.x, cy, column.z);
                        if streaming.queued_light.insert(pos) {
                            streaming.to_light.push_back(pos);
                        }
                    }
                }
            }

            commands.entity(entity).despawn();
        }
    }
}
