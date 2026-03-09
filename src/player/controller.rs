use bevy::prelude::*;
use crate::fsm::{StateLifecycle, StateMachine, StateUpdate, Transition};
use super::{Player, input::PlayerInput};

#[derive(Component)]
pub struct PlayerController {
    pub fsm: StateMachine<MoveState>,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub gravity: f32,
    pub grounded: bool,
    pub jump_force: f32,
    pub jump_requested: bool,
    
    pub target_horiz_velocity: Vec2,
    pub current_velocity: Vec3
}

pub fn tick(
    mut controller: Single<&mut PlayerController, With<Player>>,
    mut input: Single<&mut PlayerInput, With<Player>>,
    time: Res<Time<Fixed>>
) {
    let input = &mut input.movement;
    
    let ctx = MoveContext {
        input_direction: input.direction,
        speed: controller.walk_speed
    };
    
    let cmds = controller.fsm.tick(time.delta_secs(), &ctx);
    for cmd in cmds {
        apply_cmd(&mut controller, cmd);
    }

    if input.jump_pressed && controller.grounded {
        controller.jump_requested = true;
        input.jump_pressed = false;
    }
}

fn apply_cmd(
    mut controller: &mut PlayerController,
    cmd: MoveCmd
) {
    match cmd {
        MoveCmd::SetVelocityTarget(vec) => {
            controller.target_horiz_velocity = vec
        }
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
    pub speed: f32,
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
        delta: f32,
        ctx: &MoveContext,
        out: &mut Vec<MoveCmd>,
    ) -> Transition<Self> {
        match self {
            MoveState::Idle => {
                if !is_still(&ctx.input_direction) {
                    return Transition::Switch(MoveState::Run);
                }

                out.push(MoveCmd::SetVelocityTarget(Vec2::ZERO));
                Transition::Stay
            }

            MoveState::Walking | MoveState::Run  => {
                if is_still(&ctx.input_direction) {
                    return Transition::Switch(MoveState::Idle);
                }

                out.push(MoveCmd::SetVelocityTarget(ctx.input_direction * ctx.speed));
                Transition::Stay
            }
        }
    }
}
