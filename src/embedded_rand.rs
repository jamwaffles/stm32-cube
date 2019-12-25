//! Really crappy rand() implementation for making "random" light displays. **This is not cryptographically secure.** (duh)

static mut seed: u32 = 348723;

pub fn rand() -> u32 {
	unsafe {
		seed = (1103515245 * seed + 12345) % 429496729;

		seed
	}
}

pub fn rand_u8() -> u8 {
	unsafe {
		(rand() & 0xff) as u8
	}
}

pub fn rand_range(min: u32, max: u32) -> u32 {
	unsafe {
		rand() % (max + 1 - min) + min
	}
}