use crate::patterns::PatternUpdate;
use common::{apa106led::Apa106Led, cube::Cube};

#[derive(Clone, Debug)]
pub struct Police {
    speed: u32,
}

impl Default for Police {
    fn default() -> Self {
        Self { speed: 1000 }
    }
}

impl PatternUpdate for Police {
    type CycleCounter = u32;
    // type Iter = PoliceIter;

    fn pixel_at(&mut self, _idx: usize, time: u32, _frame_delta: u32) -> Apa106Led {
        let is_red = (time % self.speed) < self.speed / 2;

        if is_red {
            Apa106Led {
                red: 255,
                green: 0,
                blue: 0,
            }
        } else {
            Apa106Led {
                red: 0,
                green: 0,
                blue: 255,
            }
        }
    }

    // fn update_iter(& self, time: u32, _frame_delta: u32, _cube: &Cube) -> Self::Iter {
    //     Self::Iter {
    //         idx: 0,
    //         is_red: (time % self.speed) < self.speed / 2,
    //     }
    // }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / self.speed
    }
}

// pub struct PoliceIter {
//     idx: usize,
//     is_red: bool,
// }

// impl Iterator for PoliceIter {
//     type Item = Apa106Led;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.idx > 63 {
//             return None;
//         }

//         self.idx += 1;

//         let result = if self.is_red {
//             Apa106Led {
//                 red: 255,
//                 green: 0,
//                 blue: 0,
//             }
//         } else {
//             Apa106Led {
//                 red: 0,
//                 green: 0,
//                 blue: 255,
//             }
//         };

//         Some(result)
//     }
// }
