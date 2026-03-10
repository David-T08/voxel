use bevy::prelude::*;

use crate::interpolation::exp_smooth;
use crate::player::{Player, controller::PlayerController};

pub fn step(
    mut player: Single<&mut Transform, With<Player>>,
    mut controller: Single<&mut PlayerController, With<Player>>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    let local = controller.target_horiz_velocity;

    let mut forward = *player.forward();
    forward.y = 0.0;
    forward = forward.normalize_or_zero();

    let mut right = *player.right();
    right.y = 0.0;
    right = right.normalize_or_zero();

    let desired_world = right * local.x + forward * local.y;

    controller.current_velocity.x =
        exp_smooth(controller.current_velocity.x, desired_world.x, 20.0, dt);

    controller.current_velocity.z =
        exp_smooth(controller.current_velocity.z, desired_world.z, 20.0, dt);

    if controller.jump_requested && controller.grounded {
        if !controller.flying {
            controller.current_velocity.y = controller.jump_force;
        }

        controller.jump_requested = false;
    }

    if !controller.grounded && !controller.flying {
        controller.current_velocity.y -= controller.gravity * dt;
    }

    player.translation += controller.current_velocity * dt;

    if controller.flying {
        controller.current_velocity.y = 0.0;
        controller.grounded = false;
        player.translation.y +=
            (controller.holding_jump as i8 as f32 - controller.crouching as i8 as f32) * dt * 6.5
    }

    if player.translation.y <= 0.0 && !controller.flying {
        player.translation.y = 0.0;

        if controller.current_velocity.y < 0.0 {
            controller.current_velocity.y = 0.0;
        }

        controller.grounded = true;
    } else {
        controller.grounded = false;
    }
}
