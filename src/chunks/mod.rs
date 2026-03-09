use bevy::prelude::*;

pub mod render;
pub mod streaming;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

const CHUNK_BITS: usize = CHUNK_SIZE.trailing_zeros() as usize;
const CHUNK_MASK: usize = CHUNK_SIZE - 1;

use crate::{blocks::{AIR_ID, BlockId, BlockRegistry, BlockRegistryReady}, textures::atlas::BlockAtlas, voxel::VOXEL_SIZE};

pub struct ChunkPlugin;
impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(render::ChunkRendererPlugin);
        app.add_systems(
            Update, 
            setup_chunk_blocks
                .run_if(resource_exists::<BlockRegistryReady>)
                .run_if(resource_exists::<BlockAtlas>)
        );
        app.add_systems(Startup, setup);
    }
}

pub fn setup(
    mut commands: Commands
) {
    for x in -1..1 {
        for z in -1..1 {
            commands.spawn((
                ChunkPos(IVec3::new(x, 0, z)),
                TEMPNeedsBlocks,
                ChunkData::new()
            ));
        }
    }
    
}

#[derive(Component, Deref, DerefMut)]
pub struct ChunkPos(pub IVec3);

#[derive(Component)]
pub struct ChunkData {
    blocks: [BlockId; CHUNK_VOLUME]
}

#[derive(Component)]
pub struct NeedsRemesh;

#[derive(Component)]
pub struct TEMPNeedsBlocks;

fn setup_chunk_blocks(
    mut commands: Commands,
    block_reg: Res<BlockRegistry>,
    mut chunks: Query<(&mut ChunkData, Entity), With<TEMPNeedsBlocks>>,
) {
    for (mut chunk, entity) in &mut chunks {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let i = ChunkData::index(x,y,z);
                    let block = &mut chunk.blocks[i];
                    
                    if rand::random::<bool>() || true {
                        if y > CHUNK_SIZE - 2 {
                            *block = block_reg.names.name_to_id("core:grass").unwrap();
                        } else {
                            *block = block_reg.names.name_to_id("core:stone").unwrap();    
                        }
                    };
                }
            }
        }
        
        commands.entity(entity)
            .insert(NeedsRemesh)
            .remove::<TEMPNeedsBlocks>();
    }
}

impl ChunkData {
    pub fn new() -> Self {
        Self { 
            blocks: [AIR_ID; CHUNK_VOLUME]
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