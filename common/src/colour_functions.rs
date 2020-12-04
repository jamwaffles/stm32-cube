// use tables::BLACKBODY_MAP;
use crate::apa106led::Apa106Led;

pub fn rgb_wheel(wheelpos: u8) -> Apa106Led {
    let mut thingy = wheelpos;

    if thingy < 85 {
        Apa106Led {
            red: thingy * 3,
            green: 255 - thingy * 3,
            blue: 0,
        }
    } else if thingy < 170 {
        thingy -= 85;

        Apa106Led {
            red: 255 - thingy * 3,
            green: 0,
            blue: thingy * 3,
        }
    } else {
        thingy -= 170;

        Apa106Led {
            red: 0,
            green: thingy * 3,
            blue: 255 - thingy * 3,
        }
    }
}

// Red - green - white colour wheel
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

// pub fn temp_to_rgb(kelvin: u32) -> Apa106Led {
// 	let interval = 20;

// 	// Clamp
// 	let temp = if kelvin < 500 {
// 		500
// 	} else if kelvin > 16000 {
// 		16000
// 	} else {
// 		kelvin
// 	};

// 	let index = (temp - 500) / interval;

// 	BLACKBODY_MAP[index as usize]

// 	// Round to nearest unit (in our case, 20)
// 	// (n + 4) / 5 * 5

// 	// let nearest =

// 	// Actual good implementation using lots of intrinsics that isn't supported well/at all/something else on ARM CPUs
// 	//
// 	// let temp: f32 = kelvin as f32 / 100.0;

// 	// if temp <= 66.0 {
// 	// 	Apa106Led {
// 	// 		red: 255,
// 	// 		green: clamp_to_u8(99.4708025861 * unsafe { intrinsics::logf32(temp) } - 161.1195681661),
// 	// 		blue:
// 	// 			if temp <= 19.0 {
// 	// 				0
// 	// 			} else {
// 	// 				clamp_to_u8(138.5177312231 * unsafe { intrinsics::logf32(temp - 10.0) } - 305.0447927307)
// 	// 			},
// 	// 	}
// 	// } else {
// 	// 	Apa106Led {
// 	// 		red: clamp_to_u8(329.698727446 * unsafe { intrinsics::powf32(temp - 60.0, -0.1332047592) }),
// 	// 		green: clamp_to_u8(288.1221695283 * unsafe { intrinsics::powf32(temp - 60.0, -0.1332047592) }),
// 	// 		blue: clamp_to_u8(255.0),
// 	// 	}
// 	// }
// }

pub fn fade(colour: Apa106Led, multiplier: f32) -> Apa106Led {
    Apa106Led {
        red: (colour.red as f32 * multiplier) as u8,
        green: (colour.green as f32 * multiplier) as u8,
        blue: (colour.blue as f32 * multiplier) as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_full() {
        assert_eq!(
            fade(
                Apa106Led {
                    red: 255,
                    green: 255,
                    blue: 255
                },
                0.0
            ),
            Apa106Led {
                red: 0,
                green: 0,
                blue: 0
            }
        );

        assert_eq!(
            fade(
                Apa106Led {
                    red: 255,
                    green: 255,
                    blue: 255
                },
                1.0
            ),
            Apa106Led {
                red: 255,
                green: 255,
                blue: 255
            }
        );
    }
}
