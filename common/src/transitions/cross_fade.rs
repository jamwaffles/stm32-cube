use crate::{apa106led::Apa106Led, transitions::TransitionUpdate};

pub struct CrossFade {
    duration: u32,
}

impl Default for CrossFade {
    fn default() -> Self {
        Self { duration: 1000 }
    }
}

impl TransitionUpdate for CrossFade {
    fn transition_pixel(
        &self,
        time: u32,
        frame_delta: u32,
        current: Apa106Led,
        next: Apa106Led,
    ) -> Apa106Led {
        // Take modulus of time so we can fade out an infinite number of times if necessary.
        let multiplier = (time % self.duration) as f32 / self.duration as f32;

        let current = current.fade(1.0 - multiplier);
        let new = current.fade(multiplier);

        current + new
    }

    fn is_complete(&self, time: u32) -> bool {
        time > self.duration
    }
}