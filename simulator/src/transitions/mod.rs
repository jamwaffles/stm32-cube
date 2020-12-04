mod fade_to_black;

use common::{apa106led::Apa106Led, cube::Cube};
pub use fade_to_black::*;

pub enum Transition {
    FadeToBlack(FadeToBlack),
}

impl TransitionUpdate for Transition {
    fn transition_pixel(
        &self,
        time: u32,
        frame_delta: u32,
        current: Apa106Led,
        next: Apa106Led,
    ) -> Apa106Led {
        match self {
            Self::FadeToBlack(t) => t.transition_pixel(time, frame_delta, current, next),
        }
    }

    fn is_complete(&self, time: u32) -> bool {
        match self {
            Self::FadeToBlack(t) => t.is_complete(time),
        }
    }
}

pub trait TransitionUpdate {
    fn transition_pixel(
        &self,
        time: u32,
        frame_delta: u32,
        current: Apa106Led,
        next: Apa106Led,
    ) -> Apa106Led;

    fn is_complete(&self, time: u32) -> bool;
}
