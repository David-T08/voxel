use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::{CursorOptions, PrimaryWindow}};
use std::f32::consts::PI;

mod voxel;
mod chunk;
mod blocks;
mod chunk_renderer;
mod textures;
mod registry_base;

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
        .add_plugins(chunk_renderer::ChunkRendererPlugin)
        .add_plugins(textures::TexturePlugin)
        .add_plugins(blocks::BlockPlugin)
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
    keyboard: Res<ButtonInput<KeyCode>>
) {
    let (dx, dy, dz) = get_movement_vec(&keyboard);
    let (yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);

    let speed = if keyboard.pressed(KeyCode::ShiftLeft) { 0.025 } else {0.075};
    
    player.translation += Vec3::new(dx * speed, dy * speed, dz * speed)
        .rotate_axis(Vec3::Y, yaw);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((Camera3d::default(), Player));
    commands.spawn(DirectionalLight::default());
    
    let mesh = meshes.add(Sphere::new(1.0));
    for h in 0..16 {
        let material = materials.add(StandardMaterial {
            base_color: Color::hsl((h as f32 / 16.0) * 360.0, 1.0, 0.5),
            ..Default::default()
        });
        
        commands.spawn((
                Transform::from_xyz(h as f32 * 2. - 12.0, 0.5, -50.),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
            ));
    }
}