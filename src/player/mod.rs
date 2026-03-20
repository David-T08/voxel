use bevy::{prelude::*};

use crate::blocks::BlockRegistry;
use crate::simulation::body_3D::{BoxCollider3D, CharacterBody3D};
use crate::{chunks::streaming::ChunkViewer, fsm::StateMachine, player::camera::PlayerCamera};

pub mod camera;
pub mod controller;
pub mod input;
pub mod interaction;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<interaction::selection::CurrentBlockTarget>()
            .add_systems(
                Startup,
                (spawn_player, interaction::selection::setup_selection_box),
            )
            .add_systems(
                FixedUpdate,
                (controller::tick, controller::drive_character_body).chain(),
            )
            .add_systems(
                Update,
                (
                    input::capture,
                    camera::update,
                    camera::set_mouse,
                    interaction::selection::update_block_target,
                    interaction::selection::update_selection_box,
                    interaction::placement::tick.run_if(resource_exists::<BlockRegistry>),
                    interaction::mining::tick,
                )
                    .chain(),
            );
    }
}

fn spawn_player(mut commands: Commands) {
    commands.init_resource::<input::PlayerInput>();
    commands.spawn((
        Player,
        Transform {
            translation: Vec3::new(0., 100., 0.),
            ..Default::default()
        },
        GlobalTransform::default(),
        CharacterBody3D::default(),
        BoxCollider3D::new(Vec3::new(0.55, 1.35, 0.55)),
        controller::PlayerController {
            fsm: StateMachine::new(controller::MoveState::Idle),
            flying: false,

            crouching: false,
            sprinting: false,

            jump_requested: false,
            holding_jump: false,

            target_horiz_velocity: Vec2::ZERO,
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        GlobalTransform::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 80.0_f32.to_radians(),
            ..default()
        }),
        PlayerCamera::default(),
        ChunkViewer {
            horizontal_radius: 12,
        },
    ));
}

#[derive(Component)]
pub struct Player;
