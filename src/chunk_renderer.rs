use bevy::{asset::RenderAssetUsages, prelude::*};

use crate::chunk::{NeedsRemesh, ChunkData, ChunkPos};

pub struct ChunkRendererPlugin;

impl Plugin for ChunkRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mesh_dirty);
    }
}

fn mesh_dirty(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    chunks: Query<(Entity, &ChunkData, &ChunkPos, &Mesh3d), With<NeedsRemesh>>,
) {
    for (entity, chunk, pos, mesh_handle) in &chunks {
        // 1) Build mesh from chunk.blocks (+ neighbors later)
        let new_mesh = build_chunk_mesh(chunk, pos);

        // 2) Write into existing handle (preferred: avoids changing handles)
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            *mesh = new_mesh;
        } else {
            meshes.insert(mesh_handle.clone(), new_mesh);
        }
        
        commands.entity(entity).remove::<NeedsRemesh>();
    }
}

fn build_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    pos: IVec3,
) {
    let handle = meshes.add(Mesh::from(Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::all()
    )));
    
    commands.spawn((
        ChunkPos(pos),
        ChunkData::new(),
        NeedsRemesh,
        Mesh3d(handle),
        Transform::from_translation(chunk_world_pos),
        GlobalTransform::default(),
    ));
}