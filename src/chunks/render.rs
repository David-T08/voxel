use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*};
use crate::{blocks::{AIR_ID, BlockId, BlockRegistry, BlockRegistryReady}, chunks::{CHUNK_SIZE, CHUNK_VOLUME}, debugging::DebugRenderStats, textures::{Face, atlas::BlockAtlas}};
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

#[derive(Default)]
struct ChunkMeshBuilder {
    uv_lookup: [[[f32; 2]; 4]; 6],
    
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl ChunkMeshBuilder {
    pub fn set_uv_lookup(&mut self, lookup: [[[f32; 2]; 4]; 6]) {
        self.uv_lookup = lookup;
    }
    
    #[rustfmt::skip]
    pub fn add_face(&mut self, f: Face, x: usize, y: usize, z: usize) {
        let f = f as usize;
        
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
        
        let vi = self.vertices.len() as u32;
        for p in POSITIONS[f] {
            self.vertices.push([
                p[0] + x as f32 + 0.5,
                p[1] + y as f32 + 0.5,
                p[2] + z as f32 + 0.5,
            ]);

            self.normals.push(NORMALS[f]);
        }
        
        self.uvs.extend_from_slice(&self.uv_lookup[f]);
        self.indices.extend_from_slice(&[
            vi, vi + 1, vi + 2,
            vi, vi + 2, vi + 3,
        ]);
    }
    
    pub fn build(self) -> Mesh {
        Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_indices(Indices::U32(self.indices))
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
        let mut builder = ChunkMeshBuilder::default();
        
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let i = ChunkData::index(x as usize,y as usize,z as usize);
                    let block = chunk.blocks[i];
                    
                    if block == AIR_ID {
                        continue;
                    }
                    
                    let block_data = block_reg.definitions.get(block).expect("Expected block {block} to exist!");
                    let uvs = block_data.textures.get_uvs(&atlas);
                    
                    builder.set_uv_lookup(uvs);
                    
                    let (ix, iy, iz) = (x as isize, y as isize, z as isize);
                    
                    if is_air(&chunk.blocks, ix, iy + 1, iz) { builder.add_face(Face::Top, x, y, z); }
                    if is_air(&chunk.blocks, ix, iy - 1, iz) { builder.add_face(Face::Bottom, x, y, z); }
                    if is_air(&chunk.blocks, ix, iy, iz + 1) { builder.add_face(Face::Front, x, y, z); }
                    if is_air(&chunk.blocks, ix, iy, iz - 1) { builder.add_face(Face::Back, x, y, z); }
                    if is_air(&chunk.blocks, ix - 1, iy, iz) { builder.add_face(Face::Left, x, y, z); }
                    if is_air(&chunk.blocks, ix + 1, iy, iz) { builder.add_face(Face::Right, x, y, z); }
                }
            }
        }
        
        let vert_count = builder.vertices.len();
        let faces = vert_count / 4;
        let triangles = vert_count / 2;
        
        debug_stats.faces += faces as u64;
        debug_stats.vertices += vert_count as u64;
        debug_stats.triangles += triangles as u64;
        debug_stats.meshes += 1;
        
        let mesh = meshes.add(builder.build());
        
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(atlas.atlas.clone()),
            ..Default::default()
        });
        
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform {
                translation: chunk_pos.as_vec3() * CHUNK_SIZE as f32,
                scale: Vec3::splat(VOXEL_SIZE as f32),
                ..Default::default()
            },
        ));
        
        commands.entity(entity).remove::<NeedsRemesh>();
    }
}