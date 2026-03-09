use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::{CursorOptions, PrimaryWindow}};
use std::f32::consts::PI;

mod voxel;
mod chunks;
mod blocks;
mod textures;
mod registry_base;
mod debugging;
mod player;
mod simulation;
mod fsm;
mod interpolation;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(chunks::ChunkPlugin)
        .add_plugins(textures::TexturePlugin)
        .add_plugins(blocks::BlockPlugin)
        .add_plugins(debugging::DebuggingPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        DirectionalLight::default(),
        Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -PI/13.0, PI/6., 0.),
            ..Default::default()
        }
    ));
}