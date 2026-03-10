use bevy::prelude::*;
use fastnoise_lite::{self, FastNoiseLite, FractalType, NoiseType};

use crate::{
    blocks::BlockRegistry,
    chunks::{CHUNK_SIZE, ChunkData, ChunkPos, streaming::ChunkStreamingState},
    world::WorldState,
};

pub mod tasks;

pub struct Generator {
    noise: FastNoiseLite,
}

impl Generator {
    pub fn new(seed: i32) -> Self {
        let mut noise = FastNoiseLite::with_seed(seed);
        noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        noise.set_frequency(Some(0.01));
        noise.set_fractal_type(Some(FractalType::FBm));
        noise.set_fractal_octaves(Some(4));
        noise.set_fractal_lacunarity(Some(2.0));
        noise.set_fractal_gain(Some(0.5));

        Self { noise }
    }
}

fn sample_height(generator: &Generator, x: i32, z: i32) -> i32 {
    let continental = generator.noise.get_noise_2d(x as f32 * 0.1, z as f32 * 0.1);
    let hills = generator
        .noise
        .get_noise_2d(x as f32 * 0.6 + 1000.0, z as f32 * 0.6 + 1000.0);
    let detail = generator
        .noise
        .get_noise_2d(x as f32 * 0.12 + 2000.0, z as f32 * 0.12 + 2000.0);

    (continental * 30.0 + hills * 18.0 + detail * 4.0 + 40.0) as i32
}

pub fn generate_chunk(
    generator: &Generator,
    pos: &ChunkPos,
    registry: &BlockRegistry,
) -> ChunkData {
    let mut chunk = ChunkData::new();

    for lx in 0..CHUNK_SIZE {
        for ly in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let i = ChunkData::index(lx, ly, lz);

                let wx = pos.x * 16 + lx as i32;
                let wy = pos.y * 16 + ly as i32;
                let wz = pos.z * 16 + lz as i32;

                let height = sample_height(generator, wx, wz);

                if wy > height {
                    chunk.blocks[i] = registry.names.name_to_id("core:air").unwrap()
                } else if wy == height {
                    chunk.blocks[i] = registry.names.name_to_id("core:grass").unwrap()
                } else {
                    chunk.blocks[i] = registry.names.name_to_id("core:stone").unwrap()
                }
            }
        }
    }

    chunk
}
