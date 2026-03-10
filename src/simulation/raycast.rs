use bevy::prelude::*;

use crate::world::WorldState;

#[derive(Debug, Clone, Copy)]
pub struct VoxelHit {
    pub voxel: IVec3,
    pub previous: IVec3,
    pub normal: IVec3,
    pub distance: f32,
}

pub fn raycast_voxels(
    origin: Vec3,
    dir: Vec3,
    max_distance: f32,
    world: &WorldState,
) -> Option<VoxelHit> {
    if dir.length_squared() == 0.0 {
        return None;
    }

    let dir = dir.normalize();
    let mut voxel = origin.floor().as_ivec3();

    let step = IVec3::new(signum_i(dir.x), signum_i(dir.y), signum_i(dir.z));
    let delta = Vec3::new(
        safe_inv_abs(dir.x),
        safe_inv_abs(dir.y),
        safe_inv_abs(dir.z),
    );

    let mut side_dist = Vec3::new(
        first_boundary_distance(origin.x, dir.x),
        first_boundary_distance(origin.y, dir.y),
        first_boundary_distance(origin.z, dir.z),
    );

    if world.is_solid(voxel.x, voxel.y, voxel.z) {
        return Some(VoxelHit {
            voxel,
            previous: voxel,
            normal: IVec3::ZERO,
            distance: 0.0,
        });
    }

    let mut previous = voxel;
    let mut last_normal = IVec3::ZERO;
    let mut distance = 0.0;

    while distance <= max_distance {
        previous = voxel;

        if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
            voxel.x += step.x;
            distance = side_dist.x;
            side_dist.x += delta.x;
            last_normal = IVec3::new(-step.x, 0, 0);
        } else if side_dist.y < side_dist.z {
            voxel.y += step.y;
            distance = side_dist.y;
            side_dist.y += delta.y;
            last_normal = IVec3::new(0, -step.y, 0);
        } else {
            voxel.z += step.z;
            distance = side_dist.z;
            side_dist.z += delta.z;
            last_normal = IVec3::new(0, 0, -step.z);
        }

        if distance > max_distance {
            break;
        }

        if world.is_solid(voxel.x, voxel.y, voxel.z) {
            return Some(VoxelHit {
                voxel,
                previous,
                normal: last_normal,
                distance,
            });
        }
    }

    None
}

fn signum_i(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

fn safe_inv_abs(v: f32) -> f32 {
    if v == 0.0 {
        f32::INFINITY
    } else {
        (1.0 / v).abs()
    }
}

fn first_boundary_distance(pos: f32, dir: f32) -> f32 {
    if dir > 0.0 {
        ((pos.floor() + 1.0) - pos) / dir.abs()
    } else if dir < 0.0 {
        (pos - pos.floor()) / dir.abs()
    } else {
        f32::INFINITY
    }
}