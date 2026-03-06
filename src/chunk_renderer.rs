use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*};

use crate::chunk::{NeedsRemesh, ChunkData, ChunkPos};

pub struct ChunkRendererPlugin;

impl Plugin for ChunkRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, mesh_dirty);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(build_cube_mesh());
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.4),
        ..default()
    });
    
    commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
        ));
}

#[rustfmt::skip]
fn build_cube_mesh() -> Mesh {
    Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![
            // Front face
            [-0.5, -0.5,  0.5], // BL
            [ 0.5, -0.5,  0.5], // BR
            [ 0.5,  0.5,  0.5], // TR
            [-0.5,  0.5,  0.5], // TL
            
            // Top face
            [-0.5,  0.5,  0.5], // TL
            [ 0.5,  0.5,  0.5], // TR
            [ 0.5,  0.5, -0.5], // BR
            [-0.5,  0.5, -0.5], // BL
            
            // Back face
            [-0.5,  0.5, -0.5], // TL
            [ 0.5,  0.5, -0.5], // TR
            [ 0.5, -0.5, -0.5], // BR
            [-0.5, -0.5, -0.5], // BL
            
            // Bottom face
            [-0.5, -0.5, -0.5], // TL
            [ 0.5, -0.5, -0.5], // TR
            [ 0.5, -0.5,  0.5], // BR
            [-0.5, -0.5,  0.5], // BL
            
            // Left face
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5,  0.5],
            [-0.5,  0.5,  0.5],
            [-0.5,  0.5, -0.5],
            
            // Right face
            [0.5, -0.5,  0.5],
            [0.5, -0.5, -0.5],
            [0.5,  0.5, -0.5],
            [0.5,  0.5,  0.5],
        ])
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
            // Front
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            
            // Top
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            
            // Back
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            
            // Bottom
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            
            // Left
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            
            // Right
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ])
        .with_inserted_indices(Indices::U32(vec![
            0,1,2,
            0,2,3,
            4,5,6,
            4,6,7,
            8,9,10,
            8,10,11,
            12,13,14,
            12,14,15,
            16,17,18,
            16,18,19,
            20,21,22,
            20,22,23
        ]))
}

fn mesh_dirty(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    chunks: Query<(Entity, &ChunkData, &ChunkPos), With<NeedsRemesh>>,
) {
    for (entity, chunk, pos) in &chunks {
        let new_mesh = build_cube_mesh();
        commands.spawn(
            Mesh3d(meshes.add(new_mesh))
        );
    }
}