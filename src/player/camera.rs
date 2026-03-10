use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};
use std::f32::consts::PI;

use super::Player;

#[derive(Component)]
pub struct PlayerCamera {
    pub pitch: f32,
    pub offset: Vec3,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            offset: Vec3::new(0., 1.05, 0.),
        }
    }
}

pub fn update(
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut player: Single<&mut Transform, With<Player>>,
    camera: Single<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if !window.focused {
        mouse_motion_events.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    let sensitivity = 0.0025;

    let (mut camera_transform, mut player_camera) = camera.into_inner();
    let (mut yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);

    if delta != Vec2::ZERO {
        yaw -= delta.x * sensitivity;
        player_camera.pitch -= delta.y * sensitivity;
        player_camera.pitch = player_camera.pitch.clamp(-PI / 2.0, PI / 2.0);

        player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
    }

    camera_transform.translation = player.translation + player_camera.offset;
    camera_transform.rotation =
        player.rotation * Quat::from_euler(EulerRot::YXZ, 0.0, player_camera.pitch, 0.0);
}
