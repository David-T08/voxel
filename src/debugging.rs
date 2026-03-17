use std::collections::VecDeque;
use std::fmt::{Display, Formatter, Result};

use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::chunks::streaming::ChunkStreamingState;
use crate::world::WorldState;
use crate::world::day::DayCycle;
use crate::{
    chunks::{ChunkData, ChunkPos},
    player::Player,
};

const TIMING_HISTORY: usize = 240;

#[derive(Debug, Clone)]
pub struct TimingSeries {
    samples: VecDeque<f64>,
    capacity: usize,
}

impl Default for TimingSeries {
    fn default() -> Self {
        Self {
            samples: VecDeque::with_capacity(TIMING_HISTORY),
            capacity: TIMING_HISTORY,
        }
    }
}

impl TimingSeries {
    pub fn push(&mut self, value_ms: f64) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(value_ms);
    }

    pub fn latest(&self) -> f64 {
        self.samples.back().copied().unwrap_or(0.0)
    }

    pub fn avg(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }

    pub fn min(&self) -> f64 {
        self.samples.iter().copied().reduce(f64::min).unwrap_or(0.0)
    }

    pub fn max(&self) -> f64 {
        self.samples.iter().copied().reduce(f64::max).unwrap_or(0.0)
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let mut values: Vec<f64> = self.samples.iter().copied().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p = p.clamp(0.0, 1.0);
        let idx = ((values.len() - 1) as f64 * p).round() as usize;
        values[idx]
    }

    pub fn p50(&self) -> f64 {
        self.percentile(0.50)
    }

    pub fn p95(&self) -> f64 {
        self.percentile(0.95)
    }

    pub fn p99(&self) -> f64 {
        self.percentile(0.99)
    }
}

pub struct DebuggingPlugin;
impl Plugin for DebuggingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(WireframePlugin::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_frame_timing_series,
                    update_debug_text,
                    toggle_wireframe,
                ),
            )
            .insert_resource(WireframeConfig {
                global: false,
                ..default()
            })
            .init_resource::<DebugRenderStats>()
            .init_resource::<DebugSystemTimes>();
    }
}

fn toggle_wireframe(keys: Res<ButtonInput<KeyCode>>, mut config: ResMut<WireframeConfig>) {
    if keys.just_pressed(KeyCode::F3) {
        config.global = !config.global;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Relative,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..Default::default()
        },
        Text::new("Debug"),
        TextFont {
            font_size: 16.0,
            ..Default::default()
        },
        TextColor(Color::WHITE),
        DebugText,
    ));
}

#[derive(Component)]
struct DebugText;

#[derive(Resource, Default)]
pub struct DebugRenderStats {
    pub meshes: u64,
    pub faces: u64,
    pub triangles: u64,
    pub vertices: u64,

    pub raycast_voxel_hit: IVec3,
}

#[derive(Resource, Default)]
pub struct DebugSystemTimes {
    pub frame_time: TimingSeries,
    pub update_chunk_queues: TimingSeries,
    pub spawn_mesh_tasks: TimingSeries,
    pub collect_mesh_tasks: TimingSeries,
    pub spawn_light_tasks: TimingSeries,
    pub collect_light_tasks: TimingSeries,
}

impl DebugSystemTimes {
    pub fn push_frame_time(&mut self, ms: f64) {
        self.frame_time.push(ms);
    }

    pub fn push_update_chunk_queues(&mut self, ms: f64) {
        self.update_chunk_queues.push(ms);
    }

    pub fn push_spawn_mesh_tasks(&mut self, ms: f64) {
        self.spawn_mesh_tasks.push(ms);
    }

    pub fn push_collect_mesh_tasks(&mut self, ms: f64) {
        self.collect_mesh_tasks.push(ms);
    }

    pub fn push_spawn_light_tasks(&mut self, ms: f64) {
        self.spawn_light_tasks.push(ms);
    }

    pub fn push_collect_light_tasks(&mut self, ms: f64) {
        self.collect_light_tasks.push(ms);
    }
}

impl Display for DebugSystemTimes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        fn line(f: &mut Formatter<'_>, name: &str, t: &TimingSeries) -> Result {
            writeln!(
                f,
                "{:<20} last {:>6.3}  avg {:>6.3}  p95 {:>6.3}  p99 {:>6.3}",
                name,
                t.latest(),
                t.avg(),
                t.p95(),
                t.p99(),
            )
        }

        writeln!(f, "Frame Timings (ms):")?;
        line(f, "\tframe", &self.frame_time)?;
        writeln!(f)?;
        writeln!(f, "System Timings (ms):")?;
        line(f, "\tupdate_chunk_queues", &self.update_chunk_queues)?;
        line(f, "\tspawn_mesh_tasks", &self.spawn_mesh_tasks)?;
        line(f, "\tcollect_mesh_tasks", &self.collect_mesh_tasks)?;
        line(f, "\tspawn_light_tasks", &self.spawn_light_tasks)?;
        line(f, "\tcollect_light_tasks", &self.collect_light_tasks)?;
        Ok(())
    }
}

fn update_frame_timing_series(
    diagnostics: Res<DiagnosticsStore>,
    mut timing: ResMut<DebugSystemTimes>,
) {
    if let Some(ms) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
    {
        timing.push_frame_time(ms);
    }
}

fn update_debug_text(
    diagnostics: Res<DiagnosticsStore>,
    stats: Res<DebugRenderStats>,
    timing: Res<DebugSystemTimes>,
    stream: Res<ChunkStreamingState>,
    mut text: Query<&mut Text, With<DebugText>>,
    world: Res<WorldState>,
    player: Single<&Transform, With<Player>>,
    day: Res<DayCycle>,
) {
    let (wx, wy, wz) = {
        let floored = player.translation.floor().as_ivec3();
        (floored.x, floored.y, floored.z)
    };

    let player_chunk = ChunkData::world_to_chunk_pos(wx, wy, wz);

    let hit = stats.raycast_voxel_hit;
    let hit_chunk_pos = ChunkPos::from_world(hit.x, hit.y, hit.z);

    let voxel_debug = world.get_chunk(&hit_chunk_pos).map(|hit_chunk| {
        let (lx, ly, lz) = ChunkData::world_to_local_pos(hit.x, hit.y, hit.z);
        let hit_index = ChunkData::index(lx, ly, lz);
        let self_light = hit_chunk.light[hit_index];

        let neighbors = [
            (hit.x, hit.y + 1, hit.z),
            (hit.x, hit.y - 1, hit.z),
            (hit.x, hit.y, hit.z + 1),
            (hit.x, hit.y, hit.z - 1),
            (hit.x - 1, hit.y, hit.z),
            (hit.x + 1, hit.y, hit.z),
        ];

        let light_neighbors: Vec<u8> = neighbors
            .iter()
            .copied()
            .map(|(nx, ny, nz)| {
                let neighbor_chunk = match world.get_chunk(&ChunkPos::from_world(nx, ny, nz)) {
                    Some(c) => c,
                    None => return 0,
                };

                let (lx, ly, lz) = ChunkData::world_to_local_pos(nx, ny, nz);
                let neighbor_index = ChunkData::index(lx, ly, lz);
                neighbor_chunk.light[neighbor_index]
            })
            .collect();

        let block_id_neighbors: Vec<u16> = neighbors
            .iter()
            .copied()
            .map(|(nx, ny, nz)| {
                let neighbor_chunk = match world.get_chunk(&ChunkPos::from_world(nx, ny, nz)) {
                    Some(c) => c,
                    None => return 0,
                };

                let (lx, ly, lz) = ChunkData::world_to_local_pos(nx, ny, nz);
                let neighbor_index = ChunkData::index(lx, ly, lz);
                neighbor_chunk.blocks[neighbor_index].0
            })
            .collect();

        (
            hit.to_string(),
            hit_chunk_pos.to_string(),
            self_light.to_string(),
            format!(
                "[{}, {}, {}, {}, {}, {}]",
                light_neighbors[0],
                light_neighbors[1],
                light_neighbors[2],
                light_neighbors[3],
                light_neighbors[4],
                light_neighbors[5],
            ),
            format!(
                "[{}, {}, {}, {}, {}, {}]",
                block_id_neighbors[0],
                block_id_neighbors[1],
                block_id_neighbors[2],
                block_id_neighbors[3],
                block_id_neighbors[4],
                block_id_neighbors[5],
            ),
        )
    });

    let (
        voxel_pos_text,
        voxel_chunk_text,
        self_light_text,
        light_neighbors_text,
        block_neighbors_text,
    ) = voxel_debug.unwrap_or_else(|| {
        (
            "None".to_string(),
            "None".to_string(),
            "None".to_string(),
            "[None, None, None, None, None, None]".to_string(),
            "[None, None, None, None, None, None]".to_string(),
        )
    });

    let debug_text = format!(
        concat!(
            // "FPS: {:.1}\n",
            "Meshes: {}\n",
            "Faces: {}\n",
            "Triangles: {}\n",
            "Vertices: {}\n\n",
            "Position: {}\n",
            "Chunk: {}\n",
            "Voxel: {}\n",
            "Voxel Chunk: {}\n\n",
            "Self Light: {}\n",
            "Light Neighbors: {}\n",
            "Light Neighbor IDs: {}\n\n",
            "Day: {}\n",
            "Sun Strength: {}\n\n",
            "{}\n\n",
            "Generation:\t{}\n",
            "Light:\t{}\n",
            "Mesh:\t{}"
        ),
        // fps,
        stats.meshes,
        stats.faces,
        stats.triangles,
        stats.vertices,
        player.translation.floor(),
        player_chunk,
        voxel_pos_text,
        voxel_chunk_text,
        self_light_text,
        light_neighbors_text,
        block_neighbors_text,
        day.current_day,
        day.baked_sun,
        *timing,
        stream.generate,
        stream.light,
        stream.mesh,
    );

    for mut text in &mut text {
        *text = Text::new(debug_text.clone());
    }
}
