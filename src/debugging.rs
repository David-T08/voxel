use bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::*};

pub struct DebuggingPlugin;
impl Plugin for DebuggingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugRenderStats::default());
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_debug_text);
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
}

fn update_debug_text(
    diagnostics: Res<DiagnosticsStore>,
    stats: Res<DebugRenderStats>,
    mut query: Query<&mut Text, With<DebugText>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    for mut text in &mut query {
        *text = Text::new(format!(
            "FPS: {:.1}\nMeshes: {}\nFaces: {}\nTriangles: {}\nVertices: {}",
            fps,
            stats.meshes,
            stats.faces,
            stats.triangles,
            stats.vertices,
        ));
    }
}