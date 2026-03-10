use bevy::prelude::*;
use std::collections::HashMap;

pub const VOXEL_SIZE: i32 = 1;
pub const WORLD_MIN_CHUNK_Y: i32 = -4;
pub const WORLD_MAX_CHUNK_Y: i32 = 8;

pub mod generation;
use crate::chunks::{ChunkData, ChunkPos};
use generation::Generator;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldState::new(135));
    }
}

#[derive(Resource)]
pub struct WorldState {
    pub generator: Generator,
    pub chunks: HashMap<ChunkPos, ChunkData>,
    pub seed: i32,
}

impl WorldState {
    pub fn new(seed: i32) -> Self {
        Self {
            generator: Generator::new(seed),
            chunks: HashMap::new(),
            seed: seed,
        }
    }
}

impl WorldState {
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
