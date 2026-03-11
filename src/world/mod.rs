use bevy::prelude::*;
use std::collections::HashMap;

pub const VOXEL_SIZE: i32 = 1;
pub const WORLD_MIN_CHUNK_Y: i32 = -4;
pub const WORLD_MAX_CHUNK_Y: i32 = 8;

pub mod generation;
use crate::{blocks::{AIR_ID, BlockId}, chunks::{ChunkData, ChunkPos}};
use generation::TerrainNoise;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldState::new(452));
    }
}

#[derive(Resource)]
pub struct WorldState {
    pub generator: TerrainNoise,
    pub chunks: HashMap<ChunkPos, ChunkData>,
    pub seed: i32,
}

impl WorldState {
    pub fn new(seed: i32) -> Self {
        Self {
            generator: TerrainNoise::new(seed),
            chunks: HashMap::new(),
            seed: seed,
        }
    }
}

impl WorldState {
    pub fn get_block(&self, wx: i32, wy: i32, wz: i32) -> BlockId {
        let chunk_pos = ChunkPos::from_world(wx, wy, wz);
        let index = ChunkData::world_to_index(wx, wy, wz);
        
        self.get_chunk(&chunk_pos)
            .map(|chunk| chunk.blocks[index])
            .unwrap_or(AIR_ID)
    }
    
    pub fn set_block(&mut self, wx: i32, wy: i32, wz: i32, new: BlockId) {
        let chunk_pos = ChunkPos::from_world(wx, wy, wz);
        let index = ChunkData::world_to_index(wx, wy, wz);
        
        if let Some(chunk) = self.get_chunk_mut(&chunk_pos) {
            chunk.blocks[index] = new;
        }
    }
    
    pub fn is_solid(&self, wx: i32, wy: i32, wz: i32) -> bool {
        self.get_block(wx, wy, wz) != AIR_ID
    }
    
    pub fn insert_chunk(&mut self, pos: ChunkPos, data: ChunkData) {
        self.chunks.insert(pos, data);
    }

    pub fn remove_chunk(&mut self, pos: &ChunkPos) -> Option<ChunkData> {
        self.chunks.remove(pos)
    }

    pub fn get_chunk(&self, pos: &ChunkPos) -> Option<&ChunkData> {
        self.chunks.get(pos)
    }

    pub fn get_chunk_mut(&mut self, pos: &ChunkPos) -> Option<&mut ChunkData> {
        self.chunks.get_mut(pos)
    }
}
