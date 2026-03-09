pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

pub fn exp_smooth(current: f32, target: f32, sharpness: f32, delta: f32) -> f32 {
    let t = 1.0 - (-sharpness * delta).exp();
    current + (target - current) * t
}

pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_delta {
        target
    } else {
        current + diff.signum() * max_delta
    }
}

#[inline]
pub fn ease_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

pub fn smooth_damp(
    current: f32,
    target: f32,
    current_velocity: &mut f32,
    smooth_time: f32,
    max_speed: f32,
    delta: f32,
) -> f32 {
    let smooth_time = smooth_time.max(0.0001);
    let omega = 2.0 / smooth_time;

    let x = omega * delta;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);

    let mut change = current - target;
    let original_target = target;

    let max_change = max_speed * smooth_time;
    change = change.clamp(-max_change, max_change);

    let target = current - change;
    let temp = (*current_velocity + omega * change) * delta;
    *current_velocity = (*current_velocity - omega * temp) * exp;

    let output = target + (change + temp) * exp;

    if (original_target - current > 0.0) == (output > original_target) {
        *current_velocity = (output - original_target) / delta;
        return original_target;
    }

    output
}
