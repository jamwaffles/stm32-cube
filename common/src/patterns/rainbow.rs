use crate::{apa106led::Apa106Led, cube::Cube, patterns::PatternUpdate};
use core::f32::consts::PI;
use micromath::F32Ext;

#[derive(Clone, Debug)]
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
    // type Iter = RainbowIter;

    fn pixel_at(&mut self, idx: usize, time: u32) -> Apa106Led {
        let step = idx as f32 / 64.0;
        let offset = step * PI;

        // 1 second cycle time
        let t = time as f32 / (self.duration as f32 / PI);

        let r = scale((t + offset).sin());
        let g = scale((t + offset + ((2.0 * PI) / 3.0)).sin());
        let b = scale((t + offset + ((4.0 * PI) / 3.0)).sin());

        Apa106Led {
            red: r,
            green: g,
            blue: b,
        }
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / self.duration
    }
}

fn scale(i: f32) -> u8 {
    ((i + 1.0) * 127.0) as u8
}
