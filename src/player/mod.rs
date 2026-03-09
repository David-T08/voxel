use bevy::{math::VectorSpace, prelude::*, window::CursorOptions};

use crate::{fsm::StateMachine, player::camera::PlayerCamera};

pub mod controller;
pub mod movement;
pub mod input;
pub mod camera;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (lock_cursor, spawn_player))
            .add_systems(FixedUpdate, 
                (controller::tick, movement::step).chain()
            )
            .add_systems(Update, (
                input::capture,
                camera::update
            ));
    }
}

fn spawn_player(
    mut commands: Commands,
) {
    commands.spawn((
        Player,
        Transform::default(),
        GlobalTransform::default(),
        input::PlayerInput::default(),
        controller::PlayerController {
            fsm: StateMachine::new(controller::MoveState::Idle),
            walk_speed: 6.0,
            run_speed: 12.0,
            gravity: 20.0,
            grounded: false,
            jump_force: 6.5,
            jump_requested: false,
            target_horiz_velocity: Vec2::ZERO,
            current_velocity: Vec3::ZERO
        }
    ));
    
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        GlobalTransform::default(),
        PlayerCamera::default()
    ));
    
}

fn lock_cursor(
    mut cursor: Single<&mut CursorOptions>,
) {
    cursor.grab_mode = bevy::window::CursorGrabMode::Confined;
    cursor.visible = false;
}

#[derive(Component)]
pub struct Player;