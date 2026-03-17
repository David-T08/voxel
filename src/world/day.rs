use bevy::prelude::*;

use super::WorldState;
use crate::chunks::{
    ChunkPos,
    streaming::{self, ChunkStreamingState},
};

#[derive(Resource, Clone, Copy)]
pub struct DayCycle {
    pub current_day: u32,
    pub time_of_day: f32,
    pub sun_strength: f32,
    pub baked_sun: u8,
    pub sky_color: [f32; 3],
    pub day_length_seconds: f32,
}

impl Default for DayCycle {
    fn default() -> Self {
        let time_of_day = 6.0 / 24.0;
        let angle = time_of_day * std::f32::consts::TAU;
        let sun_strength = (angle.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let baked_sun = (sun_strength * 15.0).floor() as u8;

        Self {
            current_day: 0,
            time_of_day,
            sun_strength,
            baked_sun,
            sky_color: [1.0; 3],
            day_length_seconds: 20.0, //60.0 * 20.0,
        }
    }
}

fn compute_sky_color(time_of_day: f32) -> [f32; 3] {
    let angle = time_of_day * std::f32::consts::TAU;
    let sun = (angle.sin() * 0.5 + 0.5).clamp(0.0, 1.0);

    let night = [0.03, 0.04, 0.1];
    let day = [1.0, 1.0, 1.0];

    [
        night[0] + (day[0] - night[0]) * sun,
        night[1] + (day[1] - night[1]) * sun,
        night[2] + (day[2] - night[2]) * sun,
    ]
}

pub fn update_day_cycle(
    time: Res<Time>,
    mut world: ResMut<WorldState>,
    mut streaming: ResMut<ChunkStreamingState>,
    mut cycle: ResMut<DayCycle>,
) {
    let old = cycle.baked_sun;
    cycle.time_of_day += time.delta_secs() / cycle.day_length_seconds;
    cycle.sky_color = compute_sky_color(cycle.time_of_day);

    if cycle.time_of_day >= 1.0 {
        cycle.time_of_day -= 1.0;
        cycle.current_day += 1;
    }

    let angle = cycle.time_of_day * std::f32::consts::TAU;
    cycle.sun_strength = (angle.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    cycle.baked_sun = (cycle.sun_strength * 15.0).floor() as u8;

    // if cycle.baked_sun != old {
    //     let chunks: Vec<ChunkPos> = world.chunks.keys().copied().collect();
    //     for chunk in chunks {
    //         streaming::mark_chunk_for_mesh(&mut world, &mut streaming, chunk);
    //     }
    // }
}
