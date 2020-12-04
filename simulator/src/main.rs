mod patterns;
mod transitions;

use crate::{patterns::*, transitions::*};
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
use std::time::Instant;

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

struct TransitionStuff {
    transition: Transition,
    next_pattern: Pattern,
    transition_start: u32,
}

struct State {
    current_start: u32,
    pattern: Pattern,
    frame_delta: u32,
    transition: Option<TransitionStuff>,
}

impl State {
    fn next_pattern(&mut self, time: u32, new_pattern: Pattern, transition: Option<Transition>) {
        if let Some(transition) = transition {
            self.transition = Some(TransitionStuff {
                transition,
                transition_start: time,
                next_pattern: new_pattern,
            });
        } else {
            self.current_start = time;
            self.transition = None;
            self.pattern = new_pattern;
        }
    }
}

fn update(time: u32, state: &mut State, cube: &mut Cube) {
    let delta = time - state.current_start;

    cube.fill_iter(state.pattern.update_iter(time, state.frame_delta));

    if let Some(ref mut transition) = state.transition {
        let transition_delta = time - transition.transition_start;

        if !transition.transition.is_complete(transition_delta) {
            let frame_delta = state.frame_delta;
            let update_iter = transition.next_pattern.update_iter(time, state.frame_delta);

            cube.frame_mut()
                .iter_mut()
                .zip(update_iter)
                .for_each(|(current, next)| {
                    let new =
                        transition
                            .transition
                            .transition_pixel(time, frame_delta, *current, next);

                    *current = new;
                });
        } else {
            state.pattern = transition.next_pattern.clone();
            state.current_start = time;
            state.transition = None;

            update(time, state, cube);
        }
    } else {
        match state.pattern {
            Pattern::Rainbow(ref mut rainbow) => {
                if rainbow.completed_cycles(delta) > 5 {
                    state.next_pattern(
                        time,
                        Pattern::Police(Police::default()),
                        Some(Transition::FadeToBlack(FadeToBlack::default())),
                    );
                }
            }
            Pattern::Police(ref mut police) => {
                if police.completed_cycles(delta) > 5 {
                    state.next_pattern(time, Pattern::Rainbow(Rainbow::default()), None);
                }
            }
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
        transition: None,
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
