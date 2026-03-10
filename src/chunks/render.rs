use std::collections::HashMap;

use super::{ChunkData, ChunkPos};
use crate::{
    blocks::{AIR_ID, BlockRegistry},
    chunks::{
        CHUNK_SIZE,
        streaming::{ChunkStreamingState, ColumnPos},
    },
    debugging::DebugRenderStats,
    textures::{Face, atlas::BlockAtlas},
    world::{WORLD_MAX_CHUNK_Y, WORLD_MIN_CHUNK_Y, WorldState},
};
use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*};

use crate::VOXEL_SIZE;

const MAX_MESHES_PER_FRAME: usize = 64;

#[derive(Resource, Default)]
pub struct ChunkRenderMap {
    pub entities: HashMap<ChunkPos, Entity>,
}

#[derive(Resource, Deref, DerefMut)]
pub struct ChunkMaterial(pub Handle<StandardMaterial>);

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
fn is_air_world(world: &WorldState, chunk_pos: &ChunkPos, x: isize, y: isize, z: isize) -> bool {
    let cs = CHUNK_SIZE as isize;

    let neighbor_chunk = ChunkPos::new(
        chunk_pos.x + x.div_euclid(cs) as i32,
        chunk_pos.y + y.div_euclid(cs) as i32,
        chunk_pos.z + z.div_euclid(cs) as i32,
    );

    let local_x = x.rem_euclid(cs) as usize;
    let local_y = y.rem_euclid(cs) as usize;
    let local_z = z.rem_euclid(cs) as usize;

    let Some(chunk) = world.get_chunk(&neighbor_chunk) else {
        return true;
    };

    chunk.blocks[ChunkData::index(local_x, local_y, local_z)] == AIR_ID
}

pub fn mesh_requested_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut render_map: ResMut<ChunkRenderMap>,
    mut debug_stats: ResMut<DebugRenderStats>,
    mut streaming: ResMut<ChunkStreamingState>,

    mut world: ResMut<WorldState>,
    atlas: Res<BlockAtlas>,
    block_reg: Res<BlockRegistry>,
    chunk_material: Res<ChunkMaterial>,
) {
    for _ in 0..MAX_MESHES_PER_FRAME {
        let Some(pos) = streaming.to_mesh.pop_front() else {
            break;
        };

        streaming.queued_mesh.remove(&pos);
        let column = ColumnPos::new(pos.x, pos.z);
        if !streaming.active.contains(&column) || !streaming.desired.contains(&column) {
            continue;
        }

        let Some(chunk) = world.get_chunk(&pos) else {
            continue;
        };

        let mut builder = ChunkMeshBuilder::default();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let i = ChunkData::index(x as usize, y as usize, z as usize);
                    let block = chunk.blocks[i];

                    if block == AIR_ID {
                        continue;
                    }

                    let block_data = block_reg
                        .definitions
                        .get(block)
                        .expect("Expected block {block} to exist!");
                    let uvs = block_data.textures.get_uvs(&atlas);

                    builder.set_uv_lookup(uvs);

                    let (ix, iy, iz) = (x as isize, y as isize, z as isize);

                    if is_air_world(&world, &pos, ix, iy + 1, iz) {
                        builder.add_face(Face::Top, x, y, z);
                    }
                    if is_air_world(&world, &pos, ix, iy - 1, iz) {
                        builder.add_face(Face::Bottom, x, y, z);
                    }
                    if is_air_world(&world, &pos, ix, iy, iz + 1) {
                        builder.add_face(Face::Front, x, y, z);
                    }
                    if is_air_world(&world, &pos, ix, iy, iz - 1) {
                        builder.add_face(Face::Back, x, y, z);
                    }
                    if is_air_world(&world, &pos, ix - 1, iy, iz) {
                        builder.add_face(Face::Left, x, y, z);
                    }
                    if is_air_world(&world, &pos, ix + 1, iy, iz) {
                        builder.add_face(Face::Right, x, y, z);
                    }
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

        world.get_chunk_mut(&pos).unwrap().vertice_count = vert_count as u64;

        let mesh_handle = meshes.add(builder.build());

        if let Some(&entity) = render_map.entities.get(&pos) {
            commands.entity(entity).insert(Mesh3d(mesh_handle));
        } else {
            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(chunk_material.clone()),
                    Transform {
                        translation: pos.as_vec3() * CHUNK_SIZE as f32,
                        scale: Vec3::splat(VOXEL_SIZE as f32),
                        ..Default::default()
                    },
                ))
                .id();

            render_map.entities.insert(pos, entity);
        }
    }
}

pub fn unload_chunks(
    mut commands: Commands,
    mut render_map: ResMut<ChunkRenderMap>,
    mut debug_stats: ResMut<DebugRenderStats>,
    mut streaming: ResMut<ChunkStreamingState>,

    mut world: ResMut<WorldState>,
) {
    while let Some(column) = streaming.to_unload.pop_front() {
        for cy in WORLD_MIN_CHUNK_Y..=WORLD_MAX_CHUNK_Y {
            let pos = ChunkPos::new(column.x, cy, column.z);
            let Some(chunk) = world.get_chunk(&pos) else {
                continue;
            };

            let vert_count = chunk.vertice_count;
            debug_stats.vertices -= vert_count;
            debug_stats.faces -= vert_count / 4;
            debug_stats.triangles -= vert_count / 2;
            debug_stats.meshes = debug_stats.meshes.saturating_sub(1);

            if let Some(entity) = render_map.entities.remove(&pos) {
                commands.entity(entity).despawn()
            }

            world.remove_chunk(&pos);
        }

        streaming.active.remove(&column);
        streaming.queued_unload.remove(&column);
    }
}
