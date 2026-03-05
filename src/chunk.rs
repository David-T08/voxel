use bevy::prelude::*;

pub const CHUNK_SIZE: usize = 16;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Component)]
pub struct ChunkPos(IVec3);

#[derive(Component)]
pub struct ChunkData {
    blocks: [u16; CHUNK_VOLUME]
}

#[derive(Component)]
pub struct NeedsRemesh;

impl ChunkData {
    pub fn new() -> Self {
        Self { 
            blocks: [0; CHUNK_VOLUME]
        }
    }
    
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + CHUNK_SIZE * (y + CHUNK_SIZE * z)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> u16 {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: u16) {
        let i = Self::index(x, y, z);
        self.blocks[i] = block;
    }
}