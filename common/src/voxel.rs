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

        let y = remaining / 4;

        let y = match z {
            0 | 2 => y,
            1 | 3 => 3 - y,
            _ => unreachable!(),
        };

        let x = remaining % 4;

        let x = match z {
            0 | 2 => match y {
                0 | 2 => x,
                1 | 3 => 3 - x,
                _ => unreachable!(),
            },
            1 | 3 => match y {
                0 | 2 => 3 - x,
                1 | 3 => x,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        Self {
            x: x as u8,
            y: y as u8,
            z: z as u8,
        }
    }

    pub fn into_index(self) -> usize {
        let index = match self.z {
            0 | 2 => match self.y {
                0 | 2 => (4 * self.y) + self.x,
                1 | 3 => (4 * self.y) + 3 - self.x,
                _ => unreachable!(),
            },
            1 | 3 => match self.y {
                0 => 15 - self.x,
                2 => 7 - self.x,
                1 => self.x + 7 + self.y,
                3 => self.x + 3 - self.y,
                _ => unreachable!(),
            },
            _ => unreachable!(),
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
        assert_eq!(Voxel::from_index(index), voxel, "index {}", index);
    }

    #[test]
    fn round_trip() {
        rt(Voxel { x: 0, y: 0, z: 0 }, 0);
        rt(Voxel { x: 3, y: 3, z: 3 }, 51);
        rt(Voxel { x: 1, y: 0, z: 0 }, 1);

        rt(Voxel { x: 0, y: 0, z: 0 }, 0);
        rt(Voxel { x: 0, y: 1, z: 0 }, 7);
        rt(Voxel { x: 0, y: 2, z: 0 }, 8);
        rt(Voxel { x: 0, y: 3, z: 0 }, 15);

        rt(Voxel { x: 1, y: 0, z: 0 }, 1);
        rt(Voxel { x: 1, y: 1, z: 0 }, 6);
        rt(Voxel { x: 1, y: 2, z: 0 }, 9);
        rt(Voxel { x: 1, y: 3, z: 0 }, 14);

        rt(Voxel { x: 1, y: 2, z: 3 }, 54);
        rt(Voxel { x: 3, y: 3, z: 2 }, 44);
    }
}
