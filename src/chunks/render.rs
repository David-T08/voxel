use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*};
use crate::{blocks::{AIR_ID, BlockId, BlockRegistry, BlockRegistryReady}, chunks::{CHUNK_SIZE, CHUNK_VOLUME}, debugging::DebugRenderStats, textures::atlas::BlockAtlas};
use super::{ChunkData, ChunkPos, NeedsRemesh};

use crate::voxel::VOXEL_SIZE;

pub struct ChunkRendererPlugin;
impl Plugin for ChunkRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update, 
            render_dirty_chunks
                .run_if(resource_exists::<BlockRegistryReady>)
                .run_if(resource_exists::<BlockAtlas>)
        );
    }
}

#[inline(always)]
fn is_air(blocks: &[BlockId; CHUNK_VOLUME], x: isize, y: isize, z: isize) -> bool {
    if x < 0
        || y < 0
        || z < 0
        || x >= CHUNK_SIZE as isize
        || y >= CHUNK_SIZE as isize
        || z >= CHUNK_SIZE as isize
    {
        return true;
    }
    
    blocks[ChunkData::index(x as usize, y as usize, z as usize)] == AIR_ID
}

fn render_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut debug_stats: ResMut<DebugRenderStats>,
    
    block_reg: Res<BlockRegistry>,
    chunks: Query<(Entity, &ChunkData, &ChunkPos), With<NeedsRemesh>>,
    atlas: Res<BlockAtlas>,
) {
    for (entity, chunk, chunk_pos) in chunks {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let i = ChunkData::index(x as usize,y as usize,z as usize);
                    let block = chunk.blocks[i];
                    
                    let (x, y, z) = (x as isize, y as isize, z as isize);
                    
                    if block == AIR_ID {
                        continue;
                    }

                    let visible = {
                        let mut faces = [false; 6];
                        
                        if is_air(&chunk.blocks, x, y + 1, z) { faces[0] = true; }
                        if is_air(&chunk.blocks, x, y - 1, z) { faces[1] = true; }
                        if is_air(&chunk.blocks, x, y, z + 1) { faces[2] = true; }
                        if is_air(&chunk.blocks, x, y, z - 1) { faces[3] = true; }
                        if is_air(&chunk.blocks, x - 1, y, z) { faces[4] = true; }
                        if is_air(&chunk.blocks, x + 1, y, z) { faces[5] = true; }
                        
                        faces
                    };
                    
                    let world_pos = ChunkData::index_to_world_pos(i, chunk_pos);
        
                    let block_data = block_reg.definitions.get(block).expect("Expected block {block} to exist!");
                    let uvs = block_data.textures.get_uvs(&atlas);
                    
                    let mesh = meshes.add(build_cube_mesh(uvs, visible, &mut debug_stats));
                    
                    let material = materials.add(StandardMaterial {
                        base_color_texture: Some(atlas.atlas.clone()),
                        ..Default::default()
                    });
                    
                    commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material),
                        Transform {
                            translation: world_pos.as_vec3(),
                            scale: Vec3::splat(VOXEL_SIZE as f32),
                            ..Default::default()
                        },
                    ));
                }
            }
        }
        
        commands.entity(entity).remove::<NeedsRemesh>();
    }
}

#[rustfmt::skip]
fn build_cube_mesh(
    face_uvs: [[ [f32;2]; 4]; 6],
    visible: [bool; 6],
    debug_stats: &mut DebugRenderStats
) -> Mesh {
    const POSITIONS: [[[f32; 3]; 4]; 6] = [
        // Top face
        [
            [-0.5,  0.5,  0.5], // BL
            [ 0.5,  0.5,  0.5], // BR
            [ 0.5,  0.5, -0.5], // TR
            [-0.5,  0.5, -0.5], // TL  
        ],
        
        // Bottom face
        [
            [-0.5, -0.5, -0.5], // BL
            [ 0.5, -0.5, -0.5], // BR
            [ 0.5, -0.5,  0.5], // TR
            [-0.5, -0.5,  0.5], // TL
        ],
        
        // Front face
        [
            [-0.5, -0.5,  0.5], // BL
            [ 0.5, -0.5,  0.5], // BR
            [ 0.5,  0.5,  0.5], // TR
            [-0.5,  0.5,  0.5], // TL
        ],
        
        // Back face
        [
            [ 0.5, -0.5, -0.5], // BL
            [-0.5, -0.5, -0.5], // BR
            [-0.5,  0.5, -0.5], // TR
            [ 0.5,  0.5, -0.5], // TL
        ],
        
        // Left face
        [
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5,  0.5],
            [-0.5,  0.5,  0.5],
            [-0.5,  0.5, -0.5],
        ],
        
        // Right face
        [
            [0.5, -0.5,  0.5],
            [0.5, -0.5, -0.5],
            [0.5,  0.5, -0.5],
            [0.5,  0.5,  0.5],
        ]
    ];
    
    const NORMALS: [[f32; 3]; 6] = [
        [ 0.0,  1.0,  0.0],
        [ 0.0, -1.0,  0.0],
        [ 0.0,  0.0,  1.0],
        [ 0.0,  0.0, -1.0],
        [-1.0,  0.0,  0.0],
        [ 1.0,  0.0,  0.0],
    ];
    
    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    
    let mut base_index: u32 = 0;
    
    for face in 0..6 {
        if !visible[face] {
            continue;
        }
        
        debug_stats.faces += 1;
        debug_stats.triangles += 2;
        debug_stats.vertices += 4;
        
        positions.extend_from_slice(&POSITIONS[face]);
        uvs.extend_from_slice(&face_uvs[face]);
        
        for _ in 0..4 {
            normals.push(NORMALS[face]);
        }
        
        indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ]);
        
        base_index += 4;
    }
    
    debug_stats.meshes += 1;
    
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}