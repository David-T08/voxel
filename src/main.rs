use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::{CursorOptions, PrimaryWindow}};
use std::f32::consts::PI;

mod voxel;
mod chunks;
mod blocks;
mod textures;
mod registry_base;
mod debugging;

#[derive(Component)]
struct Player;

fn get_movement_vec(keyboard: &Res<ButtonInput<KeyCode>>) -> (f32, f32, f32) {
    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;
    let mut dz: f32 = 0.0;
    
    if keyboard.pressed(KeyCode::KeyW) {
        dx -= 1.;
    }
    
    if keyboard.pressed(KeyCode::KeyS) {
        dx += 1.;
    }
    
    if keyboard.pressed(KeyCode::KeyE) {
        dy += 1.;
    }
    
    if keyboard.pressed(KeyCode::KeyQ) {
        dy -= 1.;
    }
    
    if keyboard.pressed(KeyCode::KeyA) {
        dz -= 1.;
    }
    
    if keyboard.pressed(KeyCode::KeyD) {
        dz += 1.;
    }
    
    (dz, dy, dx)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(chunks::ChunkPlugin)
        .add_plugins(textures::TexturePlugin)
        .add_plugins(blocks::BlockPlugin)
        .add_plugins(debugging::DebuggingPlugin)
        .add_systems(Startup, (setup, lock_cursor))
        .add_systems(Update, (player_look, player_move))
        .run();
}

fn lock_cursor(
    mut cursor: Single<&mut CursorOptions>,
) {
    cursor.grab_mode = bevy::window::CursorGrabMode::Confined;
    cursor.visible = false;
}

fn player_look(
    mut player: Single<&mut Transform, With<Player>>,
    motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if !window.focused {
        return;
    }
    
    let dt = time.delta_secs();
    let sens = 200. / window.width().min(window.height());

    let (mut yaw, mut pitch, _) = player.rotation.to_euler(EulerRot::YXZ);
    pitch -= motion.delta.y * dt * sens;
    yaw -= motion.delta.x * dt * sens;
    pitch = pitch.clamp(-PI/2., PI/2.);
    
    player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.);
}

fn player_move(
    mut player: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>
) {
    let dt = time.delta_secs();
    let (dx, dy, dz) = get_movement_vec(&keyboard);
    let (yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);

    let speed = if keyboard.pressed(KeyCode::ShiftLeft) { 4. } else {12.} * dt;
    
    player.translation += Vec3::new(dx, dy, dz)
        // .normalize()
        .rotate_axis(Vec3::Y, yaw)
        * speed
    ;
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((Camera3d::default(), Player));
    commands.spawn((
        DirectionalLight::default(),
        Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -PI/13.0, PI/6., 0.),
            ..Default::default()
        }
    ));
}