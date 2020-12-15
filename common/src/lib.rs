#![no_std]

pub mod apa106led;
pub mod cube;
pub mod patterns;
pub mod state;
pub mod transitions;
pub mod voxel;

pub use state::State;

/// Shim to fix link error. See here: https://github.com/rust-lang/rust/issues/62729
#[no_mangle]
fn fminf(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

/// Shim to fix link error. See here: https://github.com/rust-lang/rust/issues/62729
#[no_mangle]
fn fmaxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}
