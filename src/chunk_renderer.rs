use std::f32::consts::PI;

use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*};
use crate::{chunk::{ChunkData, ChunkPos, NeedsRemesh}, textures::{atlas::BlockAtlas, registry::BlockTextureRegistry}};

use crate::voxel::VOXEL_SIZE;

#[derive(Component)]
struct Rotates;

pub struct ChunkRendererPlugin;
impl Plugin for ChunkRendererPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup);
        app.add_systems(Update, (spawn_test_mesh.run_if(resource_added::<BlockAtlas>), do_rotate));
    }
}

fn rotate_uvs_cw(uvs: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    [uvs[3], uvs[0], uvs[1], uvs[2]]
}

fn rotate_uvs_ccw(uvs: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    [uvs[1], uvs[2], uvs[3], uvs[0]]
}

fn rotate_uvs_180(uvs: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    [uvs[2], uvs[3], uvs[0], uvs[1]]
}

fn flip_uvs_y(uvs: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    [uvs[3], uvs[2], uvs[1], uvs[0]]
}

fn flip_uvs_x(uvs: [[f32; 2]; 4]) -> [[f32; 2]; 4] {
    [uvs[1], uvs[0], uvs[3], uvs[2]]
}

fn build_block_mesh(
    textures: [&str; 6],
    atlas: &BlockAtlas,
    registry: &BlockTextureRegistry,
) -> Mesh {
    let uvs = [
        atlas.face_uvs(registry.name_to_id(textures[0]).unwrap()).unwrap(),
        atlas.face_uvs(registry.name_to_id(textures[1]).unwrap()).unwrap(),
        atlas.face_uvs(registry.name_to_id(textures[2]).unwrap()).unwrap(),
        atlas.face_uvs(registry.name_to_id(textures[3]).unwrap()).unwrap(),
        atlas.face_uvs(registry.name_to_id(textures[4]).unwrap()).unwrap(),
        atlas.face_uvs(registry.name_to_id(textures[5]).unwrap()).unwrap(),
    ];
    
    build_cube_mesh(uvs)
}

fn spawn_test_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    atlas: Res<BlockAtlas>,
    reg: Res<BlockTextureRegistry>
) {
    for x in 0..4 {
        for y in 0..4 {
            let mesh = meshes.add(build_block_mesh(
                [
                    "core:grass_top",
                    "core:grass_bottom",
                    "core:grass_front",
                    "core:grass_back",
                    "core:grass_left",
                    "core:grass_right",
                ],
                &atlas,
                &reg
            ));
            
            let material = materials.add(StandardMaterial {
                base_color_texture: Some(atlas.atlas.clone()),
                ..Default::default()
            });
            
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform {
                    // rotation: Quat::from_euler(EulerRot::YXZ, PI/4., PI/6., PI/8.),
                    translation: Vec3::new(x as f32 * VOXEL_SIZE, 0.0, y as f32 * VOXEL_SIZE),
                    scale: Vec3::splat(VOXEL_SIZE),
                    ..Default::default()
                },
                // Rotates
            ));
        }
    }
}

fn do_rotate(
    blocks: Query<&mut Transform, With<Rotates>>,
    time: Res<Time>
) {
    let dt = time.delta_secs();
    for mut transform in blocks {
        transform.rotate_y(PI / 8. * dt);
    }
}

#[rustfmt::skip]
fn build_cube_mesh(
    face_uvs: [[ [f32;2]; 4]; 6]
) -> Mesh {
    let uvs: Vec<[f32; 2]> = face_uvs.into_iter().flatten().collect();
    
    Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![
            // Top face
            [-0.5,  0.5,  0.5], // BL
            [ 0.5,  0.5,  0.5], // BR
            [ 0.5,  0.5, -0.5], // TR
            [-0.5,  0.5, -0.5], // TL
            
            // Bottom face
            [-0.5, -0.5, -0.5], // BL
            [ 0.5, -0.5, -0.5], // BR
            [ 0.5, -0.5,  0.5], // TR
            [-0.5, -0.5,  0.5], // TL
            
            // Front face
            [-0.5, -0.5,  0.5], // BL
            [ 0.5, -0.5,  0.5], // BR
            [ 0.5,  0.5,  0.5], // TR
            [-0.5,  0.5,  0.5], // TL
            
            // Back face
            [ 0.5, -0.5, -0.5], // BL
            [-0.5, -0.5, -0.5], // BR
            [-0.5,  0.5, -0.5], // TR
            [ 0.5,  0.5, -0.5], // TL
            
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
            // Top
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            
            // Bottom
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            
            // Front
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            
            // Back
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            
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
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
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