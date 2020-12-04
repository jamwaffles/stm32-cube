use crate::patterns::PatternUpdate;
use common::{apa106led::Apa106Led, cube::Cube};

pub struct Police {
    is_red: bool,
    speed: u32,
    counter: u32,
}

impl Default for Police {
    fn default() -> Self {
        Self {
            is_red: true,
            speed: 300,
            counter: 0,
        }
    }
}

impl PatternUpdate for Police {
    type CycleCounter = u32;

    fn update(&mut self, _time: u32, frame_delta: u32, cube: &mut Cube) {
        cube.fill(if self.is_red {
            Apa106Led {
                red: 255,
                green: 0,
                blue: 0,
            }
        } else {
            Apa106Led {
                red: 0,
                green: 0,
                blue: 255,
            }
        });

        self.counter += frame_delta;

        if self.counter > self.speed {
            self.counter = 0;
            self.is_red = !self.is_red;
        }
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        // Red/blue counts as one cycle
        time / (self.speed * 2)
    }
}
