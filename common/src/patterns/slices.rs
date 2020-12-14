use crate::{apa106led::Apa106Led, cube::Cube, patterns::PatternUpdate, voxel::Voxel};
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
    Init,
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

        let fade_time = 500;

        Self {
            fade_time,
            current_colour: dir.colour(),
            dir,
            brightnesses: [0.0f32; 4],
            stage: Stage::FadeIn { idx: 0 },
            threshold: fade_time,
        }
    }
}

impl PatternUpdate for Slices {
    type CycleCounter = u32;

    fn pixel_at(&mut self, idx: usize, time: u32, _frame_delta: u32) -> Apa106Led {
        let brightness = (time % self.fade_time) as f32 / self.fade_time as f32;

        // TODO: Move to a setup method
        // Past end of current stage. Transition state to next phase.
        if time >= self.threshold {
            self.threshold = time + self.fade_time;

            self.stage = match self.stage {
                // Noop - all the state is set up, we're just updating `self.threshold` correctly
                // in this iteration.
                Stage::Init => Stage::FadeIn { idx: 0 },
                // Move on to next slice
                Stage::FadeIn { idx } if idx < 3 => Stage::FadeIn { idx: idx + 1 },
                // Reached end of fade in, move on to fade whole cube out
                Stage::FadeIn { idx: _ } => Stage::FadeOut,
                // Finished fading out. Reset to zero slice index, change direction
                Stage::FadeOut => {
                    self.dir = self.dir.next_dir();
                    self.brightnesses.iter_mut().for_each(|b| *b = 0.0);

                    Stage::FadeIn { idx: 0 }
                }
            };
        }

        // TODO: Move to a pre-frame method
        if idx == 0 {
            match self.stage {
                Stage::Init => {}
                Stage::FadeIn { idx } => {
                    self.brightnesses[idx] = brightness.max(self.brightnesses[idx]);
                }
                Stage::FadeOut => self
                    .brightnesses
                    .iter_mut()
                    .for_each(|b| *b = (1.0 - brightness).min(*b)),
            }
        }

        let voxel = Voxel::from_index(idx);

        // Voxel coordinate along current slice axis
        let voxel_pos = match self.dir {
            Direction::X => voxel.x,
            Direction::Y => voxel.y,
            Direction::Z => voxel.z,
        };

        let colour = self.dir.colour();

        colour.fade(self.brightnesses[voxel_pos as usize])
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        // Fade for 4 voxels per direction + fadeout time * 3 directions
        let total_time = self.fade_time * 5 * 3;

        time / total_time
    }
}
