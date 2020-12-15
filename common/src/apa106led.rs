pub const ON_BYTE: u8 = 0b1111_1100;
pub const OFF_BYTE: u8 = 0b1100_0000;

pub const WARM_WHITE: Apa106Led = Apa106Led {
    red: 255,
    green: 183,
    blue: 76,
};

pub const OFF: Apa106Led = Apa106Led {
    red: 0,
    green: 0,
    blue: 0,
};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Apa106Led {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

fn lerp(a: f32, b: f32, c: f32) -> f32 {
    (1.0 - c) * a + c * b
}

impl Apa106Led {
    pub const WARM_WHITE: Apa106Led = Apa106Led {
        red: 255,
        green: 183,
        blue: 76,
    };

    pub const OFF: Apa106Led = Apa106Led {
        red: 0,
        green: 0,
        blue: 0,
    };

    pub fn fade(self, multiplier: f32) -> Apa106Led {
        Apa106Led {
            red: (self.red as f32 * multiplier) as u8,
            green: (self.green as f32 * multiplier) as u8,
            blue: (self.blue as f32 * multiplier) as u8,
        }
    }

    pub fn lerp(&self, other: Self, ratio: f32) -> Self {
        let r1 = self.red as f32;
        let g1 = self.green as f32;
        let b1 = self.blue as f32;
        let r2 = other.red as f32;
        let g2 = other.green as f32;
        let b2 = other.blue as f32;

        Self {
            red: lerp(r1, r2, ratio) as u8,
            green: lerp(g1, g2, ratio) as u8,
            blue: lerp(b1, b2, ratio) as u8,
        }
    }

    pub fn as_bitbang_data(&self) -> [u8; 24] {
        colour_to_raw(&self)
    }
}

const fn bit_is_set(byte: u8, bit_index: u8) -> bool {
    (byte & (1 << bit_index)) != 0
}

fn colour_to_raw(input: &Apa106Led) -> [u8; 24] {
    let mut bytes: [u8; 24] = [0; 24];

    // Gamma correct colours
    // let gamma_corrected_input = Apa106Led {
    //  red: GAMMA_MAP[input.red as usize],
    //  green: GAMMA_MAP[input.green as usize],
    //  blue: GAMMA_MAP[input.blue as usize],
    // };

    // SPI transmits MSB first
    // Gamma correction
    // for pos in 0..8 {
    //  bytes[7 - pos as usize] = if bit_is_set(gamma_corrected_input.red, pos as u8) { ON_BYTE } else { OFF_BYTE };

    //  bytes[8 + (7 - pos as usize)] = if bit_is_set(gamma_corrected_input.green, pos as u8) { ON_BYTE } else { OFF_BYTE };

    //  bytes[16 + (7 - pos as usize)] = if bit_is_set(gamma_corrected_input.blue, pos as u8) { ON_BYTE } else { OFF_BYTE };
    // }

    // No gamma correction
    for pos in 0..8 {
        bytes[7 - pos as usize] = if bit_is_set(input.red, pos as u8) {
            ON_BYTE
        } else {
            OFF_BYTE
        };

        bytes[8 + (7 - pos as usize)] = if bit_is_set(input.green, pos as u8) {
            ON_BYTE
        } else {
            OFF_BYTE
        };

        bytes[16 + (7 - pos as usize)] = if bit_is_set(input.blue, pos as u8) {
            ON_BYTE
        } else {
            OFF_BYTE
        };
    }

    bytes
}

impl core::ops::Add<Apa106Led> for Apa106Led {
    type Output = Self;

    fn add(self, rhs: Apa106Led) -> Self::Output {
        Apa106Led {
            red: self.red.saturating_add(rhs.red),
            green: self.green.saturating_add(rhs.green),
            blue: self.blue.saturating_add(rhs.blue),
        }
    }
}

/// Red - green - white colour wheel.
pub fn christmas_wheel(wheelpos: u8) -> Apa106Led {
    let mut thingy = wheelpos;

    // Ramp red down to 0, green up to 255
    if thingy < 85 {
        Apa106Led {
            red: 255 - thingy * 3,
            green: thingy * 3,
            blue: 0,
        }
    } else if thingy < 170 {
        // Ramp red and blue up, leave green at 255
        thingy -= 85;

        Apa106Led {
            red: thingy * 3,
            green: 255,
            blue: thingy * 3,
        }
    } else {
        // Ramp green and blue down, leave red at 255
        thingy -= 170;

        Apa106Led {
            red: 255,
            green: 255 - thingy * 3,
            blue: 255 - thingy * 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_full() {
        assert_eq!(
            Apa106Led {
                red: 255,
                green: 255,
                blue: 255
            }
            .fade(0.0),
            Apa106Led {
                red: 0,
                green: 0,
                blue: 0
            }
        );

        assert_eq!(
            Apa106Led {
                red: 255,
                green: 255,
                blue: 255
            }
            .fade(1.0),
            Apa106Led {
                red: 255,
                green: 255,
                blue: 255
            }
        );
    }

    #[test]
    fn lerping() {
        let white = Apa106Led {
            red: 255,
            green: 255,
            blue: 255,
        };

        let off = Apa106Led {
            red: 0,
            green: 0,
            blue: 0,
        };

        assert_eq!(white.lerp(off, 0.0), white);

        assert_eq!(white.lerp(off, 1.0), off);

        assert_eq!(
            white.lerp(off, 0.5),
            Apa106Led {
                red: 127,
                green: 127,
                blue: 127
            }
        );
    }
}
