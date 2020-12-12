use crate::{apa106led::Apa106Led, cube::Cube, patterns::PatternUpdate};
use core::f32::consts::PI;
use micromath::F32Ext;

#[derive(Clone, Debug)]
pub struct ChristmasPuke {
    duration: u32,
}

impl ChristmasPuke {
    // Red - green - white colour wheel
    fn wheel(wheelpos: u8) -> Apa106Led {
        let mut thingy = wheelpos;

        // Ramp red down to 0, green up to 255
        if thingy < 85 {
            Apa106Led {
                red: 255 - thingy * 3,
                green: thingy * 3,
                blue: 0,
            }
        } else if thingy < 170 {
            // Ramp red and blue up, leave green at 255
            thingy -= 85;

            Apa106Led {
                red: thingy * 3,
                green: 255,
                blue: thingy * 3,
            }
        } else {
            // Ramp green and blue down, leave red at 255
            thingy -= 170;

            Apa106Led {
                red: 255,
                green: 255 - thingy * 3,
                blue: 255 - thingy * 3,
            }
        }
    }
}

impl Default for ChristmasPuke {
    fn default() -> Self {
        Self { duration: 4000 }
    }
}

impl PatternUpdate for ChristmasPuke {
    type CycleCounter = u32;

    fn pixel_at(&mut self, idx: usize, time: u32, _frame_delta: u32) -> Apa106Led {
        let pos = time % self.duration;
        let scaler = self.duration as f32 / 255.0;

        // Add an offset 0 -> duration for each voxel to distribute pattern throughout cube.
        let offset = self.duration / 64 * idx as u32;

        // Get 0 -> duration position for wheel function
        let wheelpos = (pos + offset) % self.duration;

        // Scale to 0 -> 255 for the u8 wheel input
        let wheelpos = (wheelpos as f32 / scaler) as u8;

        Self::wheel(wheelpos)
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / self.duration
    }
}
