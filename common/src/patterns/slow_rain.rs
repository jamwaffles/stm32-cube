use crate::{
    apa106led::{Apa106Led, OFF, WARM_WHITE},
    cube::Cube,
    patterns::PatternUpdate,
    voxel::Voxel,
};
use core::f32::consts::PI;
use micromath::F32Ext;
use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct SlowRain {
    // /// How many ms between seeding new drops on top layer.
    // restart_delay: u32,
    /// How long a drop takes to go from the top to the bottom of the cube.
    drop_duration: u32,

    /// Pattern cache.
    cube: Cube,

    offsets: [u8; 16],
    positions: [f32; 16],
    brightnesses: [bool; 16],

    rng: SmallRng,

    threshold: u32,
}

impl SlowRain {
    fn seed_drops(&mut self) {
        // let num_drops = 1;

        // for _ in 0..num_drops {
        //     let i = self.rng.next_u32() % 16;

        //     self.cube.set_at_index((i + 16 * 3) as usize, WARM_WHITE);
        // }

        // let num_drops = 1;

        // for _ in 0..16 {
        //     let i = self.rng.next_u32() % 16;

        //     self.brightnesses[i] = ;
        //     // self.cube.set_at_index((i + 16 * 3) as usize, WARM_WHITE);
        // }

        for b in self.brightnesses.iter_mut() {
            let i = self.rng.next_u32() % 16;

            *b = i < 4;
        }
    }
}

impl Default for SlowRain {
    fn default() -> Self {
        let mut rng = SmallRng::seed_from_u64(0xdead_beef_cafe_babe);

        let mut offsets = [0u8; 16];

        rng.fill_bytes(&mut offsets);

        let mut self_ = Self {
            // restart_delay: 3000,
            drop_duration: 2000,
            cube: Cube::default(),
            rng,
            threshold: 0,
            offsets,
            brightnesses: [false; 16],
            positions: [0.0; 16],
        };

        self_.seed_drops();

        self_
    }
}

impl PatternUpdate for SlowRain {
    type CycleCounter = u32;

    fn pixel_at(&mut self, idx: usize, time: u32, _frame_delta: u32) -> Apa106Led {
        if time > self.threshold {
            self.threshold = time + self.drop_duration;
            self.seed_drops();
        }

        let voxel = Voxel::from_index(idx);

        // Length in voxels away from leading point where brightness should be zero
        let tail_len = 4.0;
        let total_scale = 4.0 + (tail_len * 2.0);

        let column_offset = self.offsets[(voxel.x + voxel.y * 4) as usize];
        let column_brightness = self.brightnesses[(voxel.x + voxel.y * 4) as usize];

        let time_pos = (time % self.drop_duration) as f32 / self.drop_duration as f32;

        // Off the top of the cube by tail_len to below cube by tail_len
        let scaled_time_pos = -tail_len + (time_pos * total_scale);

        let voxel_pos = voxel.z as f32;

        // Can check sign later for different leading/trailing behaviours
        let distance = (voxel_pos - scaled_time_pos);

        if column_brightness {
            // 1.0 - 0.0 clamped
            let distance = (distance / tail_len).abs().min(1.0);

            // Smoother transition
            let distance = ((distance * PI).cos() + 1.0) / 2.0;

            WARM_WHITE.fade(distance)
        } else {
            OFF
        }

        //
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / 5000
    }
}

fn scale(i: f32) -> u8 {
    ((i + 1.0) * 127.0) as u8
}
