use crate::{apa106led::Apa106Led, transitions::TransitionUpdate};

pub struct FadeToBlack {
    duration: u32,
}

impl Default for FadeToBlack {
    fn default() -> Self {
        Self { duration: 1000 }
    }
}

impl TransitionUpdate for FadeToBlack {
    fn transition_pixel(
        &self,
        time: u32,
        frame_delta: u32,
        current: Apa106Led,
        next: Apa106Led,
    ) -> Apa106Led {
        // Take modulus of time so we can fade out an infinite number of times if necessary.
        let multiplier = (time % self.duration) as f32 / self.duration as f32;

        current.fade(1.0 - multiplier)
    }

    fn is_complete(&self, time: u32) -> bool {
        time > self.duration
    }
}
