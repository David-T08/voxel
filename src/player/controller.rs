use super::{Player, input::PlayerInput};
use crate::interpolation::exp_smooth;
use crate::{
    fsm::{StateLifecycle, StateMachine, StateUpdate, Transition},
    simulation::body_3D::CharacterBody3D,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerController {
    pub fsm: StateMachine<MoveState>,
    pub jump_requested: bool,
    pub holding_jump: bool,

    pub crouching: bool,
    pub sprinting: bool,

    // debug
    pub flying: bool,

    pub target_horiz_velocity: Vec2,
}

pub fn drive_character_body(
    player: Single<(&Transform, &mut PlayerController, &mut CharacterBody3D), With<Player>>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    let (transform, mut controller, mut body) = player.into_inner();

    let local = controller.target_horiz_velocity;

    let mut forward = *transform.forward();
    forward.y = 0.0;
    forward = forward.normalize_or_zero();

    let mut right = *transform.right();
    right.y = 0.0;
    right = right.normalize_or_zero();

    let move_speed = if controller.sprinting {
        body.config.run_speed
    } else {
        body.config.walk_speed
    };

    let desired_world = (right * local.x + forward * local.y) * move_speed;

    body.current_velocity.x = exp_smooth(body.current_velocity.x, desired_world.x, 20.0, dt);

    body.current_velocity.z = exp_smooth(body.current_velocity.z, desired_world.z, 20.0, dt);

    body.noclip = controller.flying;
    body.affected_by_gravity = !controller.flying;

    if controller.flying {
        body.current_velocity.y =
            (controller.holding_jump as i8 as f32 - controller.crouching as i8 as f32) * 6.5;
        body.grounded = false;
    } else if controller.jump_requested && body.grounded {
        body.current_velocity.y = body.config.jump_force;
        controller.jump_requested = false;
    } else {
        controller.jump_requested = false;
    }
}

pub fn tick(
    player: Single<(&mut PlayerController, &mut PlayerInput, &CharacterBody3D), With<Player>>,
    time: Res<Time<Fixed>>,
) {
    let (mut controller, mut input, body) = player.into_inner();
    let input = &mut input.movement;

    controller.flying = input.set_fly;
    controller.holding_jump = input.jump_held;
    controller.crouching = input.crouch;
    controller.sprinting = input.sprint;

    let ctx = MoveContext {
        input_direction: input.direction,
        sprinting: controller.sprinting,
    };

    let cmds = controller.fsm.tick(time.delta_secs(), &ctx);
    for cmd in cmds {
        apply_cmd(&mut controller, cmd);
    }

    if input.jump_pressed && body.grounded {
        controller.jump_requested = true;
        input.jump_pressed = false;
    }
}

fn apply_cmd(controller: &mut PlayerController, cmd: MoveCmd) {
    match cmd {
        MoveCmd::SetVelocityTarget(vec) => controller.target_horiz_velocity = vec,
    }
}

#[derive(Debug, Clone)]
pub enum MoveState {
    Idle,
    Walking,
    Run,
}

impl StateLifecycle<MoveContext, MoveCmd> for MoveState {}

#[derive(Debug, Clone, Copy)]
pub struct MoveContext {
    pub input_direction: Vec2,
    pub sprinting: bool,
}

#[derive(Debug, Clone)]
pub enum MoveCmd {
    SetVelocityTarget(Vec2),
}

#[inline(always)]
pub fn is_still(v: &Vec2) -> bool {
    v.length_squared() < 0.0001
}

impl StateUpdate<MoveContext, MoveCmd> for MoveState {
    fn update(
        &mut self,
        _delta: f32,
        ctx: &MoveContext,
        out: &mut Vec<MoveCmd>,
    ) -> Transition<Self> {
        match self {
            MoveState::Idle => {
                if !is_still(&ctx.input_direction) {
                    if ctx.sprinting {
                        return Transition::Switch(MoveState::Run);
                    } else {
                        return Transition::Switch(MoveState::Walking);
                    }
                }

                out.push(MoveCmd::SetVelocityTarget(Vec2::ZERO));
                Transition::Stay
            }

            MoveState::Walking => {
                if is_still(&ctx.input_direction) {
                    return Transition::Switch(MoveState::Idle);
                }

                if ctx.sprinting {
                    return Transition::Switch(MoveState::Run);
                }

                out.push(MoveCmd::SetVelocityTarget(
                    ctx.input_direction.normalize_or_zero(),
                ));
                Transition::Stay
            }

            MoveState::Run => {
                if is_still(&ctx.input_direction) {
                    return Transition::Switch(MoveState::Idle);
                }

                if !ctx.sprinting {
                    return Transition::Switch(MoveState::Walking);
                }

                out.push(MoveCmd::SetVelocityTarget(
                    ctx.input_direction.normalize_or_zero(),
                ));
                Transition::Stay
            }
        }
    }
}
