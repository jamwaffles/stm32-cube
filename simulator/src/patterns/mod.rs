mod police;
mod rainbow;

use common::cube::Cube;
pub use police::*;
pub use rainbow::*;

pub enum Pattern {
    Rainbow(Rainbow),
    Police(Police),
}

pub trait PatternUpdate {
    type CycleCounter;

    fn update(&mut self, time: u32, frame_delta: u32, cube: &mut Cube);

    /// Get number of complete cycles this pattern will have run at a certain time.
    ///
    /// If the number is not known or cannot be computed, `None` should be returned.
    fn completed_cycles(&self, time: u32) -> Self::CycleCounter;
}
