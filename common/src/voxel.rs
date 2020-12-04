#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Voxel {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl Voxel {
    pub fn from_index(idx: usize) -> Self {
        let z = idx / 16;

        let remaining = idx % 16;

        let (x, y) = match z {
            0 | 2 => match remaining {
                0..=3 => (remaining, 0),
                8..=11 => (remaining - 8, 2),
                4..=7 => (3 - (remaining - 4), 1),
                12..=15 => (3 - (remaining - 12), 3),
                idx => panic!("{} out of range", idx),
            },
            1 | 3 => match remaining {
                0..=3 => (3 - remaining, 0),
                8..=11 => (3 - (remaining - 8), 2),
                4..=7 => ((remaining - 4), 1),
                12..=15 => ((remaining - 12), 3),
                idx => panic!("{} out of range", idx),
            },
            z => panic!("Z coord {} out of bounds", z),
        };

        Self {
            x: x as u8,
            y,
            z: z as u8,
        }
    }

    pub fn into_index(self) -> usize {
        let index = match self.z {
            0 | 2 => match self.y {
                0 | 2 => (4 * self.y) + self.x,
                1 | 3 => (4 * self.y) + 3 - self.x,
                _ => 64,
            },
            1 | 3 => match self.y {
                0 => 15 - self.x,
                2 => 7 - self.x,
                1 => self.x + 7 + self.y,
                3 => self.x + 3 - self.y,
                _ => 64,
            },
            _ => 64,
        };

        // Z coord is easy, just offset n * (num voxels in layer)
        (index + (self.z * 16)) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rt(voxel: Voxel, index: usize) {
        assert_eq!(voxel.into_index(), index, "voxel {:?}", voxel);
        dbg!(voxel.into_index());
        dbg!(Voxel::from_index(index));
        // assert_eq!(Voxel::from_index(index), voxel, "index {}", index);
    }

    #[test]
    fn round_trip() {
        rt(Voxel { x: 0, y: 0, z: 0 }, 0);
        rt(Voxel { x: 0, y: 0, z: 3 }, 63);
        rt(Voxel { x: 1, y: 0, z: 0 }, 1);

        rt(Voxel { x: 0, y: 0, z: 0 }, 0);
        rt(Voxel { x: 0, y: 1, z: 0 }, 7);
        rt(Voxel { x: 0, y: 2, z: 0 }, 8);
        rt(Voxel { x: 0, y: 3, z: 0 }, 15);

        rt(Voxel { x: 1, y: 0, z: 0 }, 1);
        rt(Voxel { x: 1, y: 1, z: 0 }, 6);
        rt(Voxel { x: 1, y: 2, z: 0 }, 9);
        rt(Voxel { x: 1, y: 3, z: 0 }, 14);

        rt(Voxel { x: 3, y: 3, z: 3 }, 51);
        rt(Voxel { x: 1, y: 2, z: 3 }, 54);
        rt(Voxel { x: 3, y: 3, z: 2 }, 44);
    }
}
