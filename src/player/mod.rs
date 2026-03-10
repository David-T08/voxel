use bevy::{prelude::*, window::CursorOptions};

use crate::{chunks::streaming::ChunkViewer, fsm::StateMachine, player::camera::PlayerCamera};

pub mod camera;
pub mod controller;
pub mod input;
pub mod movement;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (lock_cursor, spawn_player))
            .add_systems(FixedUpdate, (controller::tick, movement::step).chain())
            .add_systems(Update, (input::capture, camera::update));
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::default(),
        GlobalTransform::default(),
        input::PlayerInput::default(),
        controller::PlayerController {
            fsm: StateMachine::new(controller::MoveState::Idle),
            walk_speed: 36.0,
            run_speed: 12.0,
            gravity: 20.0,

            grounded: false,
            flying: false,

            crouching: false,
            sprinting: false,

            jump_force: 6.5,
            jump_requested: false,
            holding_jump: false,

            target_horiz_velocity: Vec2::ZERO,
            current_velocity: Vec3::ZERO,
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        GlobalTransform::default(),
        PlayerCamera::default(),
        ChunkViewer {
            horizontal_radius: 32,
        },
    ));
}

fn lock_cursor(mut cursor: Single<&mut CursorOptions>) {
    cursor.grab_mode = bevy::window::CursorGrabMode::Confined;
    cursor.visible = false;
}

#[derive(Component)]
pub struct Player;
