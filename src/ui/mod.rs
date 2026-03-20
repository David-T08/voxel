use bevy::{input_focus::InputFocus, prelude::*};

use crate::{blocks::{BlockRegistry, BlockRegistryReady}, textures::{BlockTextureRegistryReady, atlas::BlockAtlas}, ui::screens::hotbar};

pub mod components;
pub mod screens;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputFocus>()
            .add_plugins(
                hotbar::CoreHotbarPlugin
            )
            .add_systems(
                Startup, 
                (
                    setup,
                    components::block_viewport::setup,
                    )
            )
            .add_systems(
                Update,
                (
                        components::block_viewport::populate,
                        components::block_viewport::bake_images,
                        components::block_viewport::finalize_bake
                    )
                    .run_if(resource_exists::<BlockAtlas>)
                    .run_if(resource_exists::<BlockRegistryReady>)
                    .run_if(resource_exists::<BlockTextureRegistryReady>)
                    .chain()
            );
            
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

}