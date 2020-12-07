mod police;
mod rainbow;
mod slow_rain;

use common::{apa106led::Apa106Led, cube::Cube};
use core::iter::Iterator;
pub use police::*;
pub use rainbow::*;
pub use slow_rain::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Rainbow(Rainbow),
    Police(Police),
    SlowRain(SlowRain),
}

impl Pattern {
    pub fn update_iter<'a>(&'a mut self, time: u32, frame_delta: u32) -> PatternIter<'a> {
        PatternIter {
            pattern: self,
            idx: 0,
            time,
            frame_delta,
        }
    }
}

pub trait PatternUpdate {
    type CycleCounter;

    fn pixel_at(&mut self, idx: usize, time: u32, frame_delta: u32) -> Apa106Led;

    // type Iter: Iterator;

    // fn update_iter(&self, time: u32, frame_delta: u32, cube: &Cube) -> Self::Iter;

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
    frame_delta: u32,
}

impl<'a> Iterator for PatternIter<'a> {
    type Item = Apa106Led;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > 63 {
            return None;
        }

        let pixel = match self.pattern {
            Pattern::Rainbow(p) => p.pixel_at(self.idx, self.time, self.frame_delta),
            Pattern::Police(p) => p.pixel_at(self.idx, self.time, self.frame_delta),
            Pattern::SlowRain(p) => p.pixel_at(self.idx, self.time, self.frame_delta),
        };

        self.idx += 1;

        Some(pixel)
    }
}
