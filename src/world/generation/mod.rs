use bevy::prelude::*;
use fastnoise_lite::{self, FastNoiseLite, FractalType, NoiseType};

use crate::{
    blocks::BlockRegistry,
    chunks::{CHUNK_SIZE, ChunkData, ChunkPos, streaming::ChunkStreamingState},
    world::WorldState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Ocean,
    Beach,
    Desert,
    Grassland,
    Forest,
    Rainforest,
    Tundra,
    Snow,
    Rocky,
}

impl Biome {
    fn new(height: i32, sea_level: i32, temp: f32, humid: f32, slope: i32) -> Biome {
        if height <= sea_level {
            return Biome::Ocean;
        }

        if height <= sea_level + 2 {
            return Biome::Beach;
        }

        if slope >= 4 || height >= 120 {
            if temp < 0.25 {
                return Biome::Snow;
            }
            return Biome::Rocky;
        }

        if temp < 0.2 {
            if humid < 0.5 {
                Biome::Tundra
            } else {
                Biome::Snow
            }
        } else if temp < 0.7 {
            if humid < 0.3 {
                Biome::Grassland
            } else if humid < 0.7 {
                Biome::Forest
            } else {
                Biome::Rainforest
            }
        } else {
            if humid < 0.25 {
                Biome::Desert
            } else if humid < 0.65 {
                Biome::Grassland
            } else {
                Biome::Rainforest
            }
        }
    }
}

pub mod tasks;

pub struct TerrainNoise {
    pub continent: FastNoiseLite,
    pub hills: FastNoiseLite,
    pub mountains: FastNoiseLite,
    pub detail: FastNoiseLite,

    pub temperature: FastNoiseLite,
    pub humidity: FastNoiseLite,

    pub seed: i32,
}

pub struct ClimateSample {
    pub temperature: f32,
    pub humidity: f32,
}

fn sample_climate(
    noise: &TerrainNoise,
    x: f32,
    z: f32,
    height: i32,
    sea_level: i32,
) -> ClimateSample {
    let raw_temp = noise01(noise.temperature.get_noise_2d(x, z));
    let raw_humid = noise01(noise.humidity.get_noise_2d(x, z));

    let altitude_cooling = ((height - sea_level).max(0) as f32 / 100.0) * 0.35;
    let temperature = (raw_temp - altitude_cooling).clamp(0.0, 1.0);

    let humidity = raw_humid.clamp(0.0, 1.0);

    ClimateSample {
        temperature,
        humidity,
    }
}

impl TerrainNoise {
    pub fn new(seed: i32) -> Self {
        let mut continent = FastNoiseLite::with_seed(seed);
        continent.set_noise_type(Some(NoiseType::OpenSimplex2));
        continent.set_frequency(Some(0.001));
        continent.set_fractal_type(Some(FractalType::FBm));
        continent.set_fractal_octaves(Some(4));
        continent.set_fractal_lacunarity(Some(1.0));
        continent.set_fractal_gain(Some(0.25));

        let mut hills = FastNoiseLite::with_seed(seed + 1);
        hills.set_noise_type(Some(NoiseType::OpenSimplex2));
        hills.set_frequency(Some(0.005));
        hills.set_fractal_type(Some(FractalType::FBm));
        hills.set_fractal_octaves(Some(3));
        hills.set_fractal_lacunarity(Some(1.6));
        hills.set_fractal_gain(Some(0.310));

        let mut mountains = FastNoiseLite::with_seed(seed - 12);
        mountains.set_noise_type(Some(NoiseType::OpenSimplex2));
        mountains.set_frequency(Some(0.001));
        mountains.set_fractal_type(Some(FractalType::Ridged));
        mountains.set_fractal_octaves(Some(2));
        mountains.set_fractal_lacunarity(Some(1.2));
        mountains.set_fractal_gain(Some(0.310));

        let mut detail = FastNoiseLite::with_seed(seed + 63);
        detail.set_noise_type(Some(NoiseType::OpenSimplex2));
        detail.set_frequency(Some(0.02));
        detail.set_fractal_type(Some(FractalType::FBm));
        detail.set_fractal_octaves(Some(3));
        detail.set_fractal_lacunarity(Some(1.6));
        detail.set_fractal_gain(Some(0.6));

        let mut temperature = FastNoiseLite::with_seed(seed + 100);
        temperature.set_noise_type(Some(NoiseType::OpenSimplex2));
        temperature.set_frequency(Some(0.0008));
        temperature.set_fractal_type(Some(FractalType::FBm));
        temperature.set_fractal_octaves(Some(3));
        temperature.set_fractal_lacunarity(Some(2.0));
        temperature.set_fractal_gain(Some(0.5));

        let mut humidity = FastNoiseLite::with_seed(seed + 200);
        humidity.set_noise_type(Some(NoiseType::OpenSimplex2));
        humidity.set_frequency(Some(0.0012));
        humidity.set_fractal_type(Some(FractalType::FBm));
        humidity.set_fractal_octaves(Some(3));
        humidity.set_fractal_lacunarity(Some(2.0));
        humidity.set_fractal_gain(Some(0.5));

        Self {
            continent,
            hills,
            mountains,
            detail,

            temperature,
            humidity,

            seed,
        }
    }

    pub fn clone(&self) -> Self {
        Self::new(self.seed)
    }
}

fn noise01(v: f32) -> f32 {
    (v + 1.0) * 0.5
}

fn sample_height(noise: &TerrainNoise, x: f32, z: f32) -> i32 {
    let continent = noise.continent.get_noise_2d(x, z);
    let hills = noise.hills.get_noise_2d(x, z);
    let mountains = noise.mountains.get_noise_2d(x, z).max(0.0);
    let detail = noise.detail.get_noise_2d(x, z);

    let continent_norm = (continent + 1.0) * 0.5;

    let mountain_mask = ((continent_norm - 0.45).max(0.0) / 0.55).powf(2.0);

    let hill_mask = continent_norm.powf(1.5);

    let mut height = 62.0;
    height += continent * 35.0;
    height += hills * 16.0 * hill_mask;
    height += mountains * 75.0 * mountain_mask;
    height += detail * 1.5;

    height as i32
}

pub fn generate_chunk(
    generator: &TerrainNoise,
    pos: &ChunkPos,
    registry: &BlockRegistry,
) -> ChunkData {
    let mut chunk = ChunkData::new();

    let sea_level = 62;
    let air = registry.names.name_to_id("core:air").unwrap();
    let grass = registry.names.name_to_id("core:grass").unwrap();
    let dirt = registry.names.name_to_id("core:dirt").unwrap();
    let stone = registry.names.name_to_id("core:stone").unwrap();
    let sand = registry.names.name_to_id("core:sand").unwrap();
    let water = registry.names.name_to_id("core:water_still").unwrap();

    for lx in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            let wx = pos.x * 16 + lx as i32;
            let wz = pos.z * 16 + lz as i32;

            let height = sample_height(generator, wx as f32, wz as f32);

            let hx = sample_height(generator, wx as f32 + 1.0, wz as f32);
            let hz = sample_height(generator, wx as f32, wz as f32 + 1.0);
            let slope = (hx - height).abs().max((hz - height).abs());

            let (top_block, filler_block, filler_depth) = if slope >= 2 {
                (stone, stone, 0)
            } else if height <= sea_level + 2 {
                (sand, sand, 4)
            } else {
                (grass, dirt, 4)
            };

            for ly in 0..CHUNK_SIZE {
                let wy = pos.y * 16 + ly as i32;
                let i = ChunkData::index(lx, ly, lz);

                chunk.blocks[i] = if wy > height {
                    if wy <= sea_level { water } else { air }
                } else if wy == height {
                    top_block
                } else if wy >= height - filler_depth {
                    filler_block
                } else {
                    stone
                };
            }
        }
    }

    chunk
}
