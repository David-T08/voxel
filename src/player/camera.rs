use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::PrimaryWindow};
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
            offset: Vec3::new(0., 1.6, 0.)
        }
    }
}

pub fn update(
    mut player: Single<&mut Transform, With<Player>>,
    camera: Single<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    
    motion: Res<AccumulatedMouseMotion>,
    window: Single<&Window, With<PrimaryWindow>>,
    time: Res<Time>
) {
    if !window.focused {
        return;
    }
    
    let dt = time.delta_secs();
    let sens = 200. / window.width().min(window.height());

    let (mut camera_transform, mut player_camera) = camera.into_inner();
    let (mut yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);
    
    yaw -= motion.delta.x * dt * sens;
    player_camera.pitch -= motion.delta.y * dt * sens;
    player_camera.pitch = player_camera.pitch.clamp(-PI/2., PI/2.);
    
    player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0., 0.);
    
    camera_transform.translation = player.translation + player_camera.offset;
    camera_transform.rotation =
        player.rotation * Quat::from_euler(EulerRot::YXZ, 0., player_camera.pitch, 0.)
}