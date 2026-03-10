use bevy::prelude::*;

pub mod aabb;
pub mod body_3D;
pub mod raycast;

pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate, 
            body_3D::move_bodies
        );
    }
}
