//! A debugging tool for thick lines
//!
//! Use the up/down arrow keys to increase or decrease the line thickness. Click and drag to move
//! the end point of the line around.
//!
//! The thickness, DX and DY components of the line are displayed in the top right corner of the
//! window.

extern crate embedded_graphics;
extern crate embedded_graphics_simulator;

use std::time::Instant;

use common::{apa106led::Apa106Led, cube::Cube};
use core::f32::consts::PI;
use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Line,
    primitives::{common::LineJoin, common::StrokeOffset, line::Intersection, Circle, Polyline},
    style::PrimitiveStyleBuilder,
    style::{MonoTextStyle, PrimitiveStyle},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use sdl2::keyboard::Keycode;

const SIZE: i32 = 15;
const SPACING: i32 = 5;

fn draw_layer(
    pixels: &[Rgb888],
    display: &mut impl DrawTarget<Error = core::convert::Infallible, Color = Rgb888>,
) -> Result<(), core::convert::Infallible> {
    for (idx, p) in pixels.iter().enumerate() {
        let x = (idx % 4) as i32 * (SIZE + SPACING);
        let y = (idx / 4) as i32 * (SIZE + SPACING);

        Circle::new(Point::new(x, y), SIZE as u32)
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .stroke_width(1)
                    .stroke_color(Rgb888::CSS_DARK_GRAY)
                    .fill_color(*p)
                    .build(),
            )
            .draw(display)?;
    }

    Ok(())
}

fn draw(
    display: &mut SimulatorDisplay<Rgb888>,
    _time: u32,
    cube: &mut Cube,
) -> Result<(), core::convert::Infallible> {
    display.clear(Rgb888::BLACK)?;

    for (idx, layer) in cube.frame().chunks(16).enumerate() {
        let colours = layer
            .iter()
            .map(|led| Rgb888::new(led.red, led.green, led.blue))
            .collect::<Vec<_>>();

        draw_layer(
            &colours,
            &mut display.translated(Point::new(
                idx as i32 * ((SIZE + SPACING) * 4 + SPACING * 2) + 10,
                10,
            )),
        )?;
    }

    Ok(())
}

fn scale(i: f32) -> u8 {
    ((i + 1.0) * 127.0) as u8
}

trait PatternUpdate {
    type CycleCounter;

    fn update(&mut self, time: u32, frame_delta: u32, cube: &mut Cube);

    /// Get number of complete cycles this pattern will have run at a certain time.
    ///
    /// If the number is not known or cannot be computed, `None` should be returned.
    fn completed_cycles(&self, time: u32) -> Self::CycleCounter;
}

struct Rainbow {
    duration: u32,
}

impl Default for Rainbow {
    fn default() -> Self {
        Self { duration: 1000 }
    }
}

impl PatternUpdate for Rainbow {
    type CycleCounter = u32;

    fn update(&mut self, time: u32, frame_delta: u32, cube: &mut Cube) {
        for (idx, _) in cube.frame().iter().enumerate() {
            let step = idx as f32 / 64.0;
            let offset = step * PI;

            // 1 second cycle time
            let t = time as f32 / (self.duration as f32 / PI);

            let r = scale((t + offset).sin());
            let g = scale((t + offset + ((2.0 * PI) / 3.0)).sin());
            let b = scale((t + offset + ((4.0 * PI) / 3.0)).sin());

            let colour = Apa106Led {
                red: r,
                green: g,
                blue: b,
            };

            cube.set_at_index(idx, colour);
        }
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        time / self.duration
    }
}

struct Police {
    is_red: bool,
    speed: u32,
    counter: u32,
}

impl Default for Police {
    fn default() -> Self {
        Self {
            is_red: true,
            speed: 300,
            counter: 0,
        }
    }
}

impl PatternUpdate for Police {
    type CycleCounter = u32;

    fn update(&mut self, _time: u32, frame_delta: u32, cube: &mut Cube) {
        cube.fill(if self.is_red {
            Apa106Led {
                red: 255,
                green: 0,
                blue: 0,
            }
        } else {
            Apa106Led {
                red: 0,
                green: 0,
                blue: 255,
            }
        });

        self.counter += frame_delta;

        if self.counter > self.speed {
            self.counter = 0;
            self.is_red = !self.is_red;
        }
    }

    fn completed_cycles(&self, time: u32) -> Self::CycleCounter {
        // Red/blue counts as one cycle
        time / (self.speed * 2)
    }
}

enum Pattern {
    Rainbow(Rainbow),
    Police(Police),
}

enum Transition {
    FadeOut { duration: u32 },
}

struct State {
    current_start: u32,
    pattern: Pattern,
    frame_delta: u32,
}

fn update(time: u32, state: &mut State, cube: &mut Cube) {
    match state.pattern {
        Pattern::Rainbow(ref mut rainbow) => {
            let delta = time - state.current_start;

            rainbow.update(time, state.frame_delta, cube);

            if rainbow.completed_cycles(delta) > 5 {
                state.current_start = time;
                state.pattern = Pattern::Police(Police::default())
            }
        }
        Pattern::Police(ref mut police) => {
            police.update(time, state.frame_delta, cube);
        }
    }
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(400, 120));
    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(1)
        .build();
    let mut window = Window::new("Cube sim", &output_settings);

    let start = Instant::now();

    let mut state = State {
        pattern: Pattern::Rainbow(Rainbow::default()),
        current_start: 0,
        frame_delta: 0,
    };
    let mut cube = Cube::new();

    let mut prev_time = 0;

    update(0, &mut state, &mut cube);
    draw(&mut display, 0, &mut cube)?;

    'running: loop {
        window.update(&display);

        for event in window.events() {
            #[allow(clippy::single_match)]
            match event {
                SimulatorEvent::Quit => break 'running,
                // SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                //     Keycode::Up => time += 1,
                //     Keycode::Down => time = time.saturating_sub(1),
                //     _ => (),
                // },
                //     draw(&mut display, time, &mut cube)?;
                // }
                // SimulatorEvent::MouseButtonDown { point, .. } => {
                //     mouse_down = true;
                //     position = point;

                //     draw(&mut display, time, &mut cube)?;
                // }
                // SimulatorEvent::MouseButtonUp { .. } => mouse_down = false,
                // SimulatorEvent::MouseMove { point, .. } => {
                //     if mouse_down {
                //         position = point;
                //         draw(&mut display, time, &mut cube)?;
                //     }
                // }
                _ => {}
            }
        }

        let time = start.elapsed().as_millis();

        state.frame_delta = time as u32 - prev_time;

        prev_time = time as u32;

        update(time as u32, &mut state, &mut cube);
        draw(&mut display, time as u32, &mut cube)?;
    }

    Ok(())
}
