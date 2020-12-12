use crate::{apa106led::Apa106Led, cube::Cube, patterns::PatternUpdate};
use core::f32::consts::PI;
use micromath::F32Ext;

#[derive(Debug, Copy, Clone)]
enum Direction {
    /// Left to right
    X,

    /// Front to back
    Y,

    /// Top down
    Z,
}

impl Direction {
    fn next_dir(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::Z,
            Self::Z => Self::X,
        }
    }

    fn colour(self) -> Apa106Led {
        match self {
            // Red
            Self::X => Apa106Led {
                red: 255,
                green: 0,
                blue: 0,
            },
            // Green
            Self::Y => Apa106Led {
                red: 0,
                green: 255,
                blue: 0,
            },
            // White
            Self::Z => Apa106Led {
                red: 255,
                green: 255,
                blue: 255,
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Stage {
    FadeIn { idx: usize },
    FadeOut,
}

#[derive(Clone, Debug)]
pub struct Slices {
    /// Slice fade in/out time, ms.
    fade_time: u32,

    current_colour: Apa106Led,

    /// Slice brightnesses
    brightnesses: [f32; 4],

    /// Animation direction
    dir: Direction,

    // /// Slice index
    // slice_idx: usize,
    stage: Stage,

    threshold: u32,
}

impl Default for Slices {
    fn default() -> Self {
        let dir = Direction::Z;

        Self {
            fade_time: 1000,
            current_colour: dir.colour(),
            dir,
            brightnesses: [0.0f32; 4],
            stage: Stage::FadeIn { idx: 0 },
            threshold: 0,
        }
    }
}

impl PatternUpdate for Slices {
    type CycleCounter = u32;

    fn pixel_at(&mut self, idx: usize, time: u32, _frame_delta: u32) -> Apa106Led {
        // dbg!(time % (self.fade_time * 2), self.fade_time);

        if time > self.threshold {
            self.threshold = time + self.fade_time;

            // Past end of current stage. Transition state to next phase.
            if idx == 0 && time % (self.fade_time * 2) > self.fade_time {
                let old = self.stage;
                self.stage = match self.stage {
                    // Move on to next slice
                    Stage::FadeIn { idx } if idx < 3 => Stage::FadeIn { idx: idx + 1 },
                    // Reached end of fade in, move on to fade whole cube out
                    Stage::FadeIn { idx: _ } => Stage::FadeOut,
                    // Finished fading out. Reset to zero slice index, change direction
                    Stage::FadeOut => {
                        self.dir = self.dir.next_dir();

                        Stage::FadeIn { idx: 0 }
                    }
                };

                println!("Next phase {:?} -> {:?}, {:?}", old, self.stage, self.dir);
            }
        }

        Apa106Led::OFF
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        // time / self.duration
        todo!()
    }
}
