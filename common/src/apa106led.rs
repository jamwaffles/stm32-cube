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

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct Apa106Led {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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
