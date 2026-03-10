use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::blocks::BlockRegistry;
use crate::chunks::streaming::{ChunkStreamingState, ColumnPos};
use crate::chunks::{ChunkData, ChunkPos};
use crate::world::generation::{Generator, generate_chunk};
use crate::world::{WORLD_MAX_CHUNK_Y, WORLD_MIN_CHUNK_Y, WorldState};

#[derive(Component)]
pub struct ChunkGenTask(pub Task<(ColumnPos, Vec<(ChunkPos, ChunkData)>)>);

const MAX_GEN_TASKS_PER_FRAME: usize = 8;
const MAX_ACTIVE_GEN_TASKS: usize = 32;

pub fn generate_column(
    generator: &Generator,
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

        let task = pool.spawn(async move {
            let generator = Generator::new(seed);
            let chunk = generate_column(&generator, column, &registry);
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

                        if streaming.queued_mesh.insert(chunk_pos) {
                            streaming.to_mesh.push_back(chunk_pos);
                        }
                    }
                }
            }

            commands.entity(entity).despawn();
        }
    }
}
