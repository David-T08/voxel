use bevy::{input_focus::InputFocus, prelude::*};

use crate::ui::screens::hotbar;

pub mod components;
pub mod screens;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputFocus>()
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        }
    ));
    hotbar::init(commands);
}