use bevy::prelude::*;

use crate::player::Player;

#[derive(Default)]
pub struct MoveInput {
    pub direction: Vec2,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub sprint: bool,
    pub crouch: bool,

    pub set_fly: bool,

    last_forward_press_time: f32,
}

#[derive(Default)]
pub struct MouseInput {
    pub m1_held: bool,
    pub m1_pressed: bool,

    pub m2_held: bool,
    pub m2_pressed: bool,

    pub m3_held: bool,
    pub m3_pressed: bool,

    pub cursor_unlocked: bool,
}

#[derive(Resource, Default)]
pub struct PlayerInput {
    pub movement: MoveInput,
    pub mouse: MouseInput,
}

pub fn capture(
    mut input: ResMut<PlayerInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    capture_movement(&keyboard, &mut input.movement, &time);
    capture_mouse_clicks(&mouse, &mut input.mouse);

    if keyboard.just_pressed(KeyCode::Escape) {
        input.mouse.cursor_unlocked = !input.mouse.cursor_unlocked;
    }
}

fn capture_mouse_clicks(mouse: &ButtonInput<MouseButton>, input: &mut MouseInput) {
    input.m1_held = mouse.pressed(MouseButton::Left);
    input.m2_held = mouse.pressed(MouseButton::Right);
    input.m3_held = mouse.pressed(MouseButton::Middle);

    input.m1_pressed = mouse.just_pressed(MouseButton::Left);
    input.m2_pressed = mouse.just_pressed(MouseButton::Right);
    input.m3_pressed = mouse.just_pressed(MouseButton::Middle);
}

fn capture_movement(keyboard: &ButtonInput<KeyCode>, movement: &mut MoveInput, time: &Time) {
    let x =
        keyboard.pressed(KeyCode::KeyD) as i8 as f32 - keyboard.pressed(KeyCode::KeyA) as i8 as f32;

    let y =
        keyboard.pressed(KeyCode::KeyW) as i8 as f32 - keyboard.pressed(KeyCode::KeyS) as i8 as f32;

    if keyboard.just_pressed(KeyCode::KeyW) {
        let now = time.elapsed_secs();

        if now - movement.last_forward_press_time < 0.25 {
            movement.sprint = false;
        } else {
            movement.last_forward_press_time = now
        }
    }

    if keyboard.just_pressed(KeyCode::F2) {
        movement.set_fly = !movement.set_fly;
    }

    if keyboard.just_pressed(KeyCode::Space) {
        movement.jump_pressed = true;
    }

    movement.direction = Vec2::new(x, y).normalize_or_zero();
    movement.jump_held = keyboard.pressed(KeyCode::Space);
    movement.crouch = keyboard.pressed(KeyCode::ShiftLeft);
}
