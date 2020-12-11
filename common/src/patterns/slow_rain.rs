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
    /// How many ms between seeding new drops on top layer.
    restart_delay: u32,

    /// How long a drop takes to go from the top to the bottom of the cube.
    drop_duration: u32,

    /// Pattern cache.
    cube: Cube,

    // Column offsets
    offsets: [u8; 16],

    rng: SmallRng,

    threshold: u32,
}

impl SlowRain {
    fn seed_drops(&mut self) {
        for index in 0..16 {
            let i = self.rng.next_u32() % 64;

            self.cube.set_at_index(
                (index + 16 * 3) as usize,
                if i < 16 { WARM_WHITE } else { OFF },
            );
        }
    }
}

impl Default for SlowRain {
    fn default() -> Self {
        let mut rng = SmallRng::seed_from_u64(0xdead_beef_cafe_babe);

        let mut offsets = [0u8; 16];

        rng.fill_bytes(&mut offsets);

        Self {
            restart_delay: 3000,
            drop_duration: 3000,
            cube: Cube::default(),
            rng,
            threshold: 0,
            offsets,
        }
    }
}

impl PatternUpdate for SlowRain {
    type CycleCounter = u32;

    fn pixel_at(&mut self, idx: usize, time: u32, _frame_delta: u32) -> Apa106Led {
        if time >= self.threshold {
            self.threshold = time + self.restart_delay;
            self.seed_drops();
        }

        let voxel = Voxel::from_index(idx);

        let column_offset = self.offsets[(voxel.x + voxel.y * 4) as usize];

        // How far down the cube we are.
        let z_position = ((time + column_offset as u32 * 5) % self.drop_duration) as f32
            / self.drop_duration as f32;

        // Z position of pixel, 0.0 - 1.0
        let pixel_pos =
            (voxel.z as u32 * (self.drop_duration / 4)) as f32 / self.drop_duration as f32;

        let distance = 1.0 - (z_position - pixel_pos).abs();

        // Offset each column by some amount
        // let distance = distance * self.offsets[(voxel.x + voxel.y * 4) as usize] as f32 / 255.0;

        WARM_WHITE.fade_sin(distance * PI * 2.0)
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / 5000
    }
}

fn scale(i: f32) -> u8 {
    ((i + 1.0) * 127.0) as u8
}
