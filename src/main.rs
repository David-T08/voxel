use bevy::prelude::*;

mod blocks;
mod chunks;
mod debugging;
mod fsm;
mod interpolation;
mod player;
mod registry_base;
mod textures;
mod world;
mod simulation;
mod lighting;

pub use world::VOXEL_SIZE;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(chunks::ChunkPlugin)
        .add_plugins(textures::TexturePlugin)
        .add_plugins(blocks::BlockPlugin)
        .add_plugins(debugging::DebuggingPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(world::WorldPlugin)
        .add_plugins(simulation::SimulationPlugin)
        .run();
}

