use bevy::prelude::*;

pub mod render;
pub mod streaming;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

const CHUNK_BITS: usize = CHUNK_SIZE.trailing_zeros() as usize;
const CHUNK_MASK: usize = CHUNK_SIZE - 1;

use crate::world::generation;
use crate::{
    VOXEL_SIZE,
    blocks::{AIR_ID, BlockId, BlockRegistryReady},
    chunks::render::ChunkMaterial,
    textures::atlas::BlockAtlas,
};

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<streaming::ChunkStreamingState>()
            .init_resource::<render::ChunkRenderMap>()
            .add_systems(
                Update,
                (
                    setup_chunk_material
                        .run_if(resource_exists::<BlockRegistryReady>)
                        .run_if(resource_exists::<BlockAtlas>),
                    (
                        streaming::request_columns_for_viewers,
                        streaming::update_chunk_queues
                            .after(streaming::request_columns_for_viewers),
                        generation::tasks::spawn_chunk_gen_tasks
                            .after(streaming::update_chunk_queues),
                        generation::tasks::collect_chunk_gen_tasks
                            .after(generation::tasks::spawn_chunk_gen_tasks),
                        render::mesh_requested_chunks
                            .after(generation::tasks::collect_chunk_gen_tasks)
                            .run_if(resource_exists::<ChunkMaterial>),
                        render::unload_chunks.after(render::mesh_requested_chunks),
                    )
                        .run_if(resource_exists::<BlockRegistryReady>),
                ),
            );
        // .add_systems(Startup, setup)
    }
}

fn setup_chunk_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    atlas: Res<BlockAtlas>,
) {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(atlas.atlas.clone()),
        ..Default::default()
    });

    commands.insert_resource(ChunkMaterial(material));
}

#[derive(Component, Deref, DerefMut, Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ChunkPos(pub IVec3);

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn from_world(wx: i32, wy: i32, wz: i32) -> Self {
        Self(IVec3::new(
            wx.div_euclid(CHUNK_SIZE as i32),
            wy.div_euclid(CHUNK_SIZE as i32),
            wz.div_euclid(CHUNK_SIZE as i32),
        ))
    }

    pub fn from_translation(pos: Vec3) -> Self {
        Self(IVec3::new(
            (pos.x.floor() as i32).div_euclid(CHUNK_SIZE as i32),
            (pos.y.floor() as i32).div_euclid(CHUNK_SIZE as i32),
            (pos.z.floor() as i32).div_euclid(CHUNK_SIZE as i32),
        ))
    }
}

impl ChunkPos {
    pub fn face_neighbors(&self) -> [ChunkPos; 6] {
        [
            ChunkPos::new(self.x - 1, self.y, self.z),
            ChunkPos::new(self.x + 1, self.y, self.z),
            ChunkPos::new(self.x, self.y - 1, self.z),
            ChunkPos::new(self.x, self.y + 1, self.z),
            ChunkPos::new(self.x, self.y, self.z - 1),
            ChunkPos::new(self.x, self.y, self.z + 1),
        ]
    }
}

#[derive(Component)]
pub struct ChunkData {
    pub blocks: [BlockId; CHUNK_VOLUME],

    pub vertice_count: u64,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            blocks: [AIR_ID; CHUNK_VOLUME],
            vertice_count: 0,
        }
    }

    pub fn world_to_chunk_pos(world_pos: Vec3) -> IVec3 {
        let x = (world_pos.x / (CHUNK_SIZE as f32 * VOXEL_SIZE as f32)).floor() as i32;
        let y = (world_pos.y / (CHUNK_SIZE as f32 * VOXEL_SIZE as f32)).floor() as i32;
        let z = (world_pos.z / (CHUNK_SIZE as f32 * VOXEL_SIZE as f32)).floor() as i32;

        IVec3::new(x, y, z)
    }

    #[inline(always)]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + CHUNK_SIZE * (y + CHUNK_SIZE * z)
    }

    pub fn index_to_local_pos(i: usize) -> (usize, usize, usize) {
        let x = i & CHUNK_MASK;
        let y = (i >> CHUNK_BITS) & CHUNK_MASK;
        let z = i >> (CHUNK_BITS * 2);
        (x, y, z)
    }

    pub fn index_to_world_pos(i: usize, chunk_pos: &ChunkPos) -> IVec3 {
        let (lx, ly, lz) = ChunkData::index_to_local_pos(i);

        let x_offset = chunk_pos.x * CHUNK_SIZE as i32 * VOXEL_SIZE;
        let y_offset = chunk_pos.y * CHUNK_SIZE as i32 * VOXEL_SIZE;
        let z_offset = chunk_pos.z * CHUNK_SIZE as i32 * VOXEL_SIZE;

        IVec3::new(
            lx as i32 * VOXEL_SIZE + x_offset,
            ly as i32 * VOXEL_SIZE + y_offset,
            lz as i32 * VOXEL_SIZE + z_offset,
        )
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE);
        let i = Self::index(x, y, z);
        self.blocks[i] = block;
    }
}
