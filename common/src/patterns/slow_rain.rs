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
    /// How long a drop takes to go from the top to the bottom of the cube.
    drop_duration: u32,

    /// Pattern cache.
    cube: Cube,

    /// Each column gets an offset so drops don't all fall together.
    offsets: [u8; 16],

    /// Turn columns on or off.
    mask: [bool; 16],

    rng: SmallRng,
}

impl Default for SlowRain {
    fn default() -> Self {
        let mut rng = SmallRng::seed_from_u64(0xdead_beef_cafe_babe);

        let mut offsets = [0u8; 16];
        let mut mask = [false; 16];

        rng.fill_bytes(&mut offsets);

        for (idx, column_on) in mask.iter_mut().enumerate() {
            let i = rng.next_u32() % 16;

            *column_on = i < 4;
        }

        let mut self_ = Self {
            drop_duration: 2000,
            cube: Cube::default(),
            rng,
            offsets,
            mask,
        };

        self_
    }
}

impl PatternUpdate for SlowRain {
    type CycleCounter = u32;

    fn pixel_at(&mut self, idx: usize, time: u32, _frame_delta: u32) -> Apa106Led {
        let voxel = Voxel::from_index(idx);
        let column_idx = (voxel.x + voxel.y * 4) as usize;

        // Smaller values = drops closer together
        let offset_mul = 1.0;

        let column_offset = self.offsets[column_idx] as f32 / 255.0 * offset_mul;
        let mask = self.mask[column_idx];

        // Length in voxels away from leading point where brightness should be zero
        let tail_len = 3.0;

        // 4 voxels plus a front/back porch for each tail length. This ensures all voxels are blank
        // between each iteration.
        let total_scale = 4.0 + (tail_len * 2.0);

        let time_pos = (time % self.drop_duration) as f32 / self.drop_duration as f32;

        // Apply offset
        let time_pos = (time_pos + column_offset).rem_euclid(1.0);

        // Reset
        if time_pos >= 0.99 && mask && voxel.z == 3 {
            loop {
                let next_idx = (self.rng.next_u32() as usize + column_idx) % 16;

                // Add a bit of chaos
                if self.rng.next_u32() % 10 <= 1 {
                    continue;
                }

                // Find next unlit column
                if !self.mask[next_idx] {
                    self.mask[next_idx] = true;
                    self.mask[column_idx] = false;
                    self.offsets[next_idx] = self.offsets[column_idx];

                    break;
                }
            }
        }

        // Off the top of the cube by tail_len to below cube by tail_len
        let scaled_time_pos = -tail_len + (time_pos * total_scale);

        let voxel_pos = voxel.z as f32;

        // Can check sign later for different leading/trailing behaviours
        let distance = voxel_pos - scaled_time_pos;

        if mask {
            // 1.0 - 0.0 clamped
            let distance = (distance / tail_len).abs().min(1.0);

            // Smoother transition
            let distance = ((distance * PI).cos() + 1.0) / 2.0;

            WARM_WHITE.fade(distance)
        } else {
            OFF
        }
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / 5000
    }
}
