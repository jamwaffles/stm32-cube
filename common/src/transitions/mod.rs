mod cross_fade;
mod fade_from_black;
mod fade_to_black;

use crate::apa106led::Apa106Led;
pub use cross_fade::*;
pub use fade_from_black::*;
pub use fade_to_black::*;

#[derive(Debug, Copy, Clone)]
pub enum Transition {
    FadeToBlack(FadeToBlack),
    FadeFromBlack(FadeFromBlack),
    CrossFade(CrossFade),
}

impl TransitionUpdate for Transition {
    fn transition_pixel(&self, time: u32, current: Apa106Led, next: Apa106Led) -> Apa106Led {
        match self {
            Self::FadeToBlack(t) => t.transition_pixel(time, current, next),
            Self::FadeFromBlack(t) => t.transition_pixel(time, current, next),
            Self::CrossFade(t) => t.transition_pixel(time, current, next),
        }
    }

    fn next_start_offset(&self) -> u32 {
        match self {
            Self::FadeToBlack(t) => t.next_start_offset(),
            Self::FadeFromBlack(t) => t.next_start_offset(),
            Self::CrossFade(t) => t.next_start_offset(),
        }
    }

    fn duration(&self) -> u32 {
        match self {
            Self::FadeToBlack(t) => t.duration(),
            Self::FadeFromBlack(t) => t.duration(),
            Self::CrossFade(t) => t.duration(),
        }
    }

    fn is_complete(&self, time: u32) -> bool {
        match self {
            Self::FadeToBlack(t) => t.is_complete(time),
            Self::FadeFromBlack(t) => t.is_complete(time),
            Self::CrossFade(t) => t.is_complete(time),
        }
    }
}

pub trait TransitionUpdate {
    fn transition_pixel(&self, time: u32, current: Apa106Led, next: Apa106Led) -> Apa106Led;

    fn next_start_offset(&self) -> u32;

    fn duration(&self) -> u32;

    fn is_complete(&self, time: u32) -> bool;
}
