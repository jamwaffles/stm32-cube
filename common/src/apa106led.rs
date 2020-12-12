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
