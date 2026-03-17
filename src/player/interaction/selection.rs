use bevy::prelude::*;

use crate::{
    debugging::DebugRenderStats,
    player::camera::PlayerCamera,
    simulation::raycast::{VoxelHit, raycast_voxels},
    world::WorldState,
};

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct CurrentBlockTarget(pub Option<VoxelHit>);

#[derive(Component)]
pub struct SelectionBox;

pub fn update_block_target(
    cam_transform: Single<&GlobalTransform, With<PlayerCamera>>,
    world: Res<WorldState>,
    mut stats: ResMut<DebugRenderStats>,

    mut target: ResMut<CurrentBlockTarget>,
) {
    let origin = cam_transform.translation();
    let dir = cam_transform.forward().as_vec3();

    let hit = raycast_voxels(origin, dir, 4.0, &world);

    target.0 = hit;
    stats.raycast_voxel_hit = hit.map(|h| h.voxel).unwrap_or(IVec3::ZERO)
}

pub fn setup_selection_box(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.01, 1.01, 1.01))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.1),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Hidden,
        SelectionBox,
    ));
}

pub fn update_selection_box(
    target: Res<CurrentBlockTarget>,
    mut query: Query<(&mut Transform, &mut Visibility), With<SelectionBox>>,
) {
    let Ok((mut transform, mut visibility)) = query.single_mut() else {
        return;
    };

    if let Some(hit) = &target.0 {
        let block_pos = hit.voxel.as_vec3();

        transform.translation = block_pos + Vec3::splat(0.5);
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}
