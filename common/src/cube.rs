use crate::apa106led::Apa106Led;

#[derive(Copy, Clone)]
pub struct Voxel {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

pub struct Cube {
    frame: [Apa106Led; 64],
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

    fn coord_to_index(&self, coord: Voxel) -> usize {
        let index = match coord.z {
            0 | 2 => match coord.y {
                0 | 2 => (4 * coord.y) + coord.x,
                1 | 3 => (4 * coord.y) + 3 - coord.x,
                _ => 64,
            },
            1 | 3 => match coord.y {
                0 => 15 - coord.x,
                2 => 7 - coord.x,
                1 => coord.x + 7 + coord.y,
                3 => coord.x + 3 - coord.y,
                _ => 64,
            },
            _ => 64,
        };

        // Z coord is easy, just offset n * (num voxels in layer)
        (index + (coord.z * 16)) as usize
    }

    pub fn frame(&self) -> [Apa106Led; 64] {
        self.frame
    }

    pub fn set_at_index(&mut self, index: usize, colour: Apa106Led) {
        self.frame[index] = colour;
    }

    pub fn set_at_coord(&mut self, coord: Voxel, colour: Apa106Led) {
        let idx = self.coord_to_index(coord);

        self.frame[idx] = colour;
    }

    pub fn get_at_coord(&self, coord: Voxel) -> Apa106Led {
        let idx = self.coord_to_index(coord);

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
}
