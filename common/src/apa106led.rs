use core::f32::consts::PI;
use micromath::F32Ext;

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

fn scale_sin(i: f32) -> f32 {
    (i + 1.0) * 127.0
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
}
