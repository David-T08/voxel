use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::world::WorldState;
use crate::{chunks::{ChunkData, ChunkPos}, player::Player};

pub struct DebuggingPlugin;
impl Plugin for DebuggingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(WireframePlugin::default())
            .add_systems(Startup, setup)
            .add_systems(Update, (update_debug_text, toggle_wireframe))
            .insert_resource(WireframeConfig {
                global: false,
                ..default()
            })
            .insert_resource(DebugRenderStats::default());
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

    pub chunks_to_generate: u64,
    pub chunks_to_unload: u64,
    pub chunks_to_mesh: u64,
    
    pub raycast_voxel_hit: IVec3,
}

fn update_debug_text(
    diagnostics: Res<DiagnosticsStore>,
    stats: Res<DebugRenderStats>,
    mut text: Query<&mut Text, With<DebugText>>,
    world: Res<WorldState>,
    player: Single<&Transform, With<Player>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    for mut text in &mut text {
        let (wx, wy, wz) = {
            let floored = player.translation.floor().as_ivec3();
            
            (floored.x, floored.y, floored.z)
        };
        
        let hit = stats.raycast_voxel_hit;
        let hit_chunk_pos = ChunkPos::from_world(hit.x, hit.y, hit.z);
        let hit_chunk = match world.get_chunk(&hit_chunk_pos) {
            Some(c) => c,
            None => {return}
        };
        
        let hit_index = {
            let (lx, ly, lz) = ChunkData::world_to_local_pos(hit.x, hit.y, hit.z);
            ChunkData::index(lx, ly, lz)
        };
        let self_light = hit_chunk.light[hit_index];
        
        let neighbors = [
            (hit.x, hit.y + 1, hit.z),
            (hit.x, hit.y - 1, hit.z),
            (hit.x, hit.y, hit.z + 1),
            (hit.x, hit.y, hit.z - 1),
            (hit.x - 1, hit.y, hit.z),
            (hit.x + 1, hit.y, hit.z)
        ];
        
        let light_neighbors: Vec<u8> = neighbors.iter().copied().map(|(nx, ny, nz)| {
            let neighbor_chunk = match world.get_chunk(&ChunkPos::from_world(nx, ny, nz)) {
                Some(c) => c,
                None => return 0
            };
            
            let neighbor_index = {
                let (lx, ly, lz) = ChunkData::world_to_local_pos(nx, ny, nz);
                ChunkData::index(lx, ly, lz)
            };
            
            neighbor_chunk.light[neighbor_index]
        }).collect();
        
        let block_id_neighbors: Vec<u16> = neighbors.iter().copied().map(|(nx, ny, nz)| {
            let neighbor_chunk = match world.get_chunk(&ChunkPos::from_world(nx, ny, nz)) {
                Some(c) => c,
                None => return 0
            };
            
            let neighbor_index = {
                let (lx, ly, lz) = ChunkData::world_to_local_pos(nx, ny, nz);
                ChunkData::index(lx, ly, lz)
            };
            
            neighbor_chunk.blocks[neighbor_index].0
        }).collect();
        
        *text = Text::new(format!(
            "FPS: {:.1}\nMeshes: {}\nFaces: {}\nTriangles: {}\nVertices: {}\n\nPosition: {}\nChunk Position: {}\n\nChunk Generation Queue: {}\nChunk Unloading Queue: {}\nChunk Mesh Queue: {}\n\nVoxel Position: {}\nVoxel Chunk: {}\nSelf Light: {}\nLight Neighbors: [{}, {}, {}, {}, {}, {}]\nLight Neighbor IDs: [{}, {}, {}, {}, {}, {}]",
            fps,
            stats.meshes,
            stats.faces,
            stats.triangles,
            stats.vertices,
            player.translation.floor(),
            ChunkData::world_to_chunk_pos(wx, wy, wz),
            stats.chunks_to_generate,
            stats.chunks_to_unload,
            stats.chunks_to_mesh,
            stats.raycast_voxel_hit,
            hit_chunk_pos,
            self_light,
            light_neighbors[0],
            light_neighbors[1],
            light_neighbors[2],
            light_neighbors[3],
            light_neighbors[4],
            light_neighbors[5],
            block_id_neighbors[0],
            block_id_neighbors[1],
            block_id_neighbors[2],
            block_id_neighbors[3],
            block_id_neighbors[4],
            block_id_neighbors[5],
        ));
    }
}
