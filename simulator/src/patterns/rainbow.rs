use crate::patterns::PatternUpdate;
use common::{apa106led::Apa106Led, cube::Cube};
use core::f32::consts::PI;

pub struct Rainbow {
    duration: u32,
}

impl Default for Rainbow {
    fn default() -> Self {
        Self { duration: 1000 }
    }
}

impl PatternUpdate for Rainbow {
    type CycleCounter = u32;

    fn update(&mut self, time: u32, frame_delta: u32, cube: &mut Cube) {
        for (idx, _) in cube.frame().iter().enumerate() {
            let step = idx as f32 / 64.0;
            let offset = step * PI;

            // 1 second cycle time
            let t = time as f32 / (self.duration as f32 / PI);

            let r = scale((t + offset).sin());
            let g = scale((t + offset + ((2.0 * PI) / 3.0)).sin());
            let b = scale((t + offset + ((4.0 * PI) / 3.0)).sin());

            let colour = Apa106Led {
                red: r,
                green: g,
                blue: b,
            };

            cube.set_at_index(idx, colour);
        }
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / self.duration
    }
}

fn scale(i: f32) -> u8 {
    ((i + 1.0) * 127.0) as u8
}
