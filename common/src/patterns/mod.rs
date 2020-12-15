mod christmas_puke;
mod rainbow;
mod slices;
mod slow_rain;

use crate::apa106led::Apa106Led;
pub use christmas_puke::*;
use core::iter::Iterator;
pub use rainbow::*;
pub use slices::*;
pub use slow_rain::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Rainbow(Rainbow),
    SlowRain(SlowRain),
    ChristmasPuke(ChristmasPuke),
    Slices(Slices),
}

impl Pattern {
    pub fn update_iter(&'_ mut self, time: u32) -> PatternIter<'_> {
        PatternIter {
            pattern: self,
            idx: 0,
            time,
        }
    }
}

pub trait PatternUpdate {
    type CycleCounter;

    fn pixel_at(&mut self, idx: usize, time: u32) -> Apa106Led;

    /// Get number of complete cycles this pattern will have run at a certain time.
    ///
    /// If the number is not known or cannot be computed, `None` should be returned.
    fn completed_cycles(&self, time: u32) -> Self::CycleCounter;
}

/// Iterator over all voxels in a frame, used to update the cube display buffer.
pub struct PatternIter<'a> {
    pattern: &'a mut Pattern,
    idx: usize,
    time: u32,
}

impl<'a> Iterator for PatternIter<'a> {
    type Item = Apa106Led;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > 63 {
            return None;
        }

        let pixel = match self.pattern {
            Pattern::Rainbow(p) => p.pixel_at(self.idx, self.time),
            Pattern::SlowRain(p) => p.pixel_at(self.idx, self.time),
            Pattern::ChristmasPuke(p) => p.pixel_at(self.idx, self.time),
            Pattern::Slices(p) => p.pixel_at(self.idx, self.time),
        };

        self.idx += 1;

        Some(pixel)
    }
}
