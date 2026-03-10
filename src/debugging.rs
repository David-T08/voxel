use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{chunks::ChunkData, player::Player};

pub struct DebuggingPlugin;
impl Plugin for DebuggingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(WireframePlugin::default())
            .add_systems(Startup, setup)
            .add_systems(Update, (update_debug_text, toggle_wireframe))
            .insert_resource(WireframeConfig {
                global: true,
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
            position_type: PositionType::Absolute,
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
        
        *text = Text::new(format!(
            "FPS: {:.1}\nMeshes: {}\nFaces: {}\nTriangles: {}\nVertices: {}\n\nPosition: {}\nChunk Position: {}\n\nChunk Generation Queue: {}\nChunk Unloading Queue: {}\nChunk Mesh Queue: {}\n\nRaycast Voxel: {}",
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
            stats.raycast_voxel_hit
        ));
    }
}
