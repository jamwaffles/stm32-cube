use crate::{apa106led::Apa106Led, transitions::TransitionUpdate};
#[derive(Debug, Copy, Clone)]
pub struct CrossFade {
    duration: u32,
}

impl Default for CrossFade {
    fn default() -> Self {
        Self { duration: 4000 }
    }
}

impl TransitionUpdate for CrossFade {
    fn transition_pixel(&self, time: u32, current: Apa106Led, next: Apa106Led) -> Apa106Led {
        let multiplier = time as f32 / self.duration as f32;

        let multiplier = multiplier.min(1.0);

        current.lerp(next, multiplier)
    }

    fn next_start_offset(&self) -> u32 {
        0
    }

    fn duration(&self) -> u32 {
        self.duration
    }

    fn is_complete(&self, time: u32) -> bool {
        time > self.duration
    }
}
