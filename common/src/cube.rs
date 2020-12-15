use crate::{apa106led::Apa106Led, voxel::Voxel};

const ON_BYTE: u8 = 0b1111_1100;
const OFF_BYTE: u8 = 0b1100_0000;

#[derive(Debug, Clone, Copy)]
pub struct Cube {
    frame: [Apa106Led; 64],
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            frame: [Apa106Led::default(); 64],
        }
    }
}

impl Cube {
    pub fn new() -> Cube {
        let blank_frame: [Apa106Led; 64] = [Apa106Led {
            red: 1,
            green: 0,
            blue: 0,
        }; 64];

        Cube { frame: blank_frame }
    }

    pub fn frame(&self) -> &[Apa106Led; 64] {
        &self.frame
    }

    pub fn frame_mut(&mut self) -> &mut [Apa106Led; 64] {
        &mut self.frame
    }

    pub fn set_at_index(&mut self, index: usize, colour: Apa106Led) {
        self.frame[index] = colour;
    }

    pub fn set_at_coord(&mut self, coord: Voxel, colour: Apa106Led) {
        let idx = coord.into_index();

        self.frame[idx] = colour;
    }

    pub fn get_at_coord(&self, coord: Voxel) -> Apa106Led {
        let idx = coord.into_index();

        self.frame[idx]
    }

    pub fn fill(&mut self, fill_colour: Apa106Led) {
        self.frame = [fill_colour; 64];
    }

    pub fn fill_layer(&mut self, layer: u8, fill_colour: Apa106Led) {
        for x in 0..4 {
            for y in 0..4 {
                self.set_at_coord(Voxel { x, y, z: layer }, fill_colour);
            }
        }
    }

    pub fn fill_slice(&mut self, slice: u8, fill_colour: Apa106Led) {
        for y in 0..4 {
            for z in 0..4 {
                self.set_at_coord(Voxel { x: slice, y, z }, fill_colour);
            }
        }
    }

    pub fn fill_panel(&mut self, panel: u8, fill_colour: Apa106Led) {
        for x in 0..4 {
            for z in 0..4 {
                self.set_at_coord(Voxel { x, y: panel, z }, fill_colour);
            }
        }
    }

    pub fn fill_column(&mut self, column: Voxel, fill_colour: Apa106Led) {
        for z in 0..4 {
            self.set_at_coord(
                Voxel {
                    x: column.x,
                    y: column.y,
                    z,
                },
                fill_colour,
            );
        }
    }

    /// Fill by index with a pixel iterator.
    ///
    /// The iterator should return 64 items to fill the cube. Any items produced after 64 will be
    /// ignored. Shorter iterators will not fail, but will leave the cube in a broken state.
    pub fn fill_iter(&mut self, iter: impl IntoIterator<Item = Apa106Led>) {
        for (idx, colour) in iter.into_iter().take(64).enumerate() {
            self.set_at_index(idx, colour)
        }
    }
}
