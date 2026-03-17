use bevy::prelude::*;

use super::aabb;
use crate::world::WorldState;

pub struct CharacterBody3DConfig {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub gravity: f32,
    pub jump_force: f32,
}

impl Default for CharacterBody3DConfig {
    fn default() -> Self {
        Self {
            walk_speed: 24.0,
            run_speed: 18.0,
            gravity: 25.0,
            jump_force: 8.0,
        }
    }
}

#[derive(Component)]
pub struct BoxCollider3D {
    pub size: Vec3,

    half_size: Vec3,
}

impl BoxCollider3D {
    pub fn new(size: Vec3) -> Self {
        Self {
            size,
            half_size: size / 2.0,
        }
    }

    pub fn half_size(&self) -> Vec3 {
        self.half_size
    }

    pub fn aabb_min_max(&self, transform: &Transform) -> (Vec3, Vec3) {
        let half = self.half_size();
        let aabb_min = transform.translation - half;
        let aabb_max = transform.translation + half;

        (aabb_min, aabb_max)
    }
}

#[derive(Component)]
pub struct CharacterBody3D {
    pub current_velocity: Vec3,
    pub affected_by_gravity: bool,
    pub grounded: bool,
    pub noclip: bool,

    pub config: CharacterBody3DConfig,
}

impl Default for CharacterBody3D {
    fn default() -> Self {
        Self {
            current_velocity: Vec3::ZERO,
            affected_by_gravity: true,
            grounded: false,
            noclip: false,

            config: CharacterBody3DConfig::default(),
        }
    }
}

pub fn move_bodies(
    time: Res<Time>,
    world: Res<WorldState>,
    mut bodies: Query<(&mut Transform, &BoxCollider3D, &mut CharacterBody3D)>,
) {
    let dt = time.delta_secs();

    for (mut transform, collider, mut body) in &mut bodies {
        if body.noclip {
            transform.translation += body.current_velocity * dt;
            body.grounded = false;
            continue;
        }

        body.grounded = false;

        if body.affected_by_gravity {
            body.current_velocity.y -= body.config.gravity * dt;
        }

        move_axis_x(&mut transform, collider, &mut body, &world, dt);
        move_axis_y(&mut transform, collider, &mut body, &world, dt);
        move_axis_z(&mut transform, collider, &mut body, &world, dt);
    }
}

fn move_axis_x(
    transform: &mut Transform,
    collider: &BoxCollider3D,
    body: &mut CharacterBody3D,
    world: &WorldState,
    dt: f32,
) {
    transform.translation.x += body.current_velocity.x * dt;

    let (min, max) = collider.aabb_min_max(transform);

    if let Some(hit) = aabb::collides_with_world(min, max, world) {
        let half_x = collider.half_size().x;

        if body.current_velocity.x > 0.0 {
            transform.translation.x = hit.position.x as f32 - half_x;
        } else if body.current_velocity.x < 0.0 {
            transform.translation.x = hit.position.x as f32 + 1.0 + half_x;
        }

        body.current_velocity.x = 0.0;
    }
}

fn move_axis_y(
    transform: &mut Transform,
    collider: &BoxCollider3D,
    body: &mut CharacterBody3D,
    world: &WorldState,
    dt: f32,
) {
    transform.translation.y += body.current_velocity.y * dt;

    let (min, max) = collider.aabb_min_max(transform);

    if let Some(hit) = aabb::collides_with_world(min, max, world) {
        let half_y = collider.half_size().y;

        if body.current_velocity.y > 0.0 {
            transform.translation.y = hit.position.y as f32 - half_y;
        } else if body.current_velocity.y < 0.0 {
            transform.translation.y = hit.position.y as f32 + 1.0 + half_y;
            body.grounded = true;
        }

        body.current_velocity.y = 0.0;
    }
}

fn move_axis_z(
    transform: &mut Transform,
    collider: &BoxCollider3D,
    body: &mut CharacterBody3D,
    world: &WorldState,
    dt: f32,
) {
    transform.translation.z += body.current_velocity.z * dt;

    let (min, max) = collider.aabb_min_max(transform);

    if let Some(hit) = aabb::collides_with_world(min, max, world) {
        let half_z = collider.half_size().z;

        if body.current_velocity.z > 0.0 {
            transform.translation.z = hit.position.z as f32 - half_z;
        } else if body.current_velocity.z < 0.0 {
            transform.translation.z = hit.position.z as f32 + 1.0 + half_z;
        }

        body.current_velocity.z = 0.0;
    }
}
