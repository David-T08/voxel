use bevy::prelude::*;

use std::f32::consts::PI;

mod blocks;
mod chunks;
mod debugging;
mod fsm;
mod interpolation;
mod player;
mod registry_base;
mod simulation;
mod textures;
mod world;

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -PI / 13.0, PI / 6., 0.),
            ..Default::default()
        },
    ));
}
