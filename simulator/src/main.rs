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

fn update(time: u32, cube: &mut Cube) {
    for (idx, _) in cube.frame().iter().enumerate() {
        let step = idx as f32 / 64.0;
        let offset = step * PI;

        // 1 second cycle time
        let t = time as f32 / (1000.0 / PI);

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

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(400, 120));
    let output_settings = OutputSettingsBuilder::new()
        // .pixel_spacing(1)
        .build();
    let mut window = Window::new("Cube sim", &output_settings);

    let start = Instant::now();

    let mut cube = Cube::new();

    update(0, &mut cube);
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

        update(time as u32, &mut cube);
        draw(&mut display, time as u32, &mut cube)?;
    }

    Ok(())
}
