use crate::{apa106led::Apa106Led, transitions::TransitionUpdate};

#[derive(Debug, Copy, Clone)]
pub struct FadeToBlack {
    duration: u32,
}

impl Default for FadeToBlack {
    fn default() -> Self {
        Self { duration: 2000 }
    }
}

impl TransitionUpdate for FadeToBlack {
    fn transition_pixel(&self, time: u32, current: Apa106Led, _next: Apa106Led) -> Apa106Led {
        let multiplier = time as f32 / self.duration as f32;

        current.fade(1.0 - multiplier.min(1.0))
    }

    fn next_start_offset(&self) -> u32 {
        self.duration
    }

    fn duration(&self) -> u32 {
        self.duration
    }

    fn is_complete(&self, time: u32) -> bool {
        time > self.duration
    }
}
