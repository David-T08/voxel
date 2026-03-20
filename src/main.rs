use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};

use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod async_util;
mod blocks;
mod chunks;
mod debugging;
mod fsm;
mod interpolation;
mod lighting;
mod player;
mod registry_base;
mod simulation;
mod textures;
mod world;
mod ui;

pub use world::VOXEL_SIZE;

use crate::blocks::material::VoxelMaterial;

fn main() {
    let timer = UtcTime::new(
        time::format_description::parse("[minute]:[second].[subsecond digits:3]").unwrap(),
    );

    tracing_subscriber::registry()
        // .with(EnvFilter::new("voxel=trace"))
        .with(EnvFilter::new("info,voxel=trace"))
        .with(fmt::layer().with_timer(timer))
        .init();

    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .set(ImagePlugin::default_nearest())
                .disable::<LogPlugin>(),
            MaterialPlugin::<VoxelMaterial>::default(),
        ))
        .add_plugins(chunks::ChunkPlugin)
        .add_plugins(textures::TexturePlugin)
        .add_plugins(blocks::BlockPlugin)
        .add_plugins(debugging::DebuggingPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(world::WorldPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(simulation::SimulationPlugin)
        .run();
}
