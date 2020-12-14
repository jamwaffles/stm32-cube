use common::{apa106led::Apa106Led, cube::Cube, patterns::*, transitions::*, voxel::Voxel};
use core::f32::consts::PI;
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::post_processing::SobelEdgeHighlight;
use kiss3d::window::Window;
use kiss3d::{
    camera::{ArcBall, FirstPerson},
    scene::SceneNode,
};
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use sdl2::libc::RUN_LVL;
use std::time::Instant;

struct TransitionStuff {
    driver: Transition,
    next_pattern: Pattern,
    start: u32,
    next_pattern_start: u32,
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
                driver: transition,
                start: time,
                next_pattern_start: time + transition.next_start_offset(),
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
    // TODO: Move into else {}
    let pattern_run_time = time - state.current_start;

    println!(
        "--- Frame {} (delta {}, current start {}) ---",
        time, pattern_run_time, state.current_start
    );

    if let Some(t) = state.transition.as_mut() {
        let transition_run_time = time - t.start;

        // Next pattern starts at end of transition
        // TODO: Make this an enum
        let next_start = if t.driver.next_start_offset() > 0 {
            t.start + transition_run_time
        }
        // Next pattern starts at the same time as the transition
        else {
            t.start
        };

        let next_pattern_run_time = time - next_start;

        println!(
            "D {}, Transition D {}, offset {}",
            pattern_run_time,
            transition_run_time,
            t.driver.next_start_offset()
        );

        if !t.driver.is_complete(transition_run_time) {
            let update_iter = t
                .next_pattern
                .update_iter(next_pattern_run_time, state.frame_delta);

            for (current, next) in cube.frame_mut().iter_mut().zip(update_iter) {
                let new = t.driver.transition_pixel(
                    transition_run_time,
                    state.frame_delta,
                    *current,
                    next,
                );

                *current = new;
            }
        } else {
            println!(
                "Transition complete in {}. Next start: {}",
                transition_run_time, next_start
            );

            state.pattern = t.next_pattern.clone();
            state.current_start = next_start;
            state.transition = None;
        }

    //     // Absolute time at which next pattern should begin
    //     let next_pattern_start = t.driver.next_start_offset();

    //     // How long this transition has been running for
    //     let transition_run_time = time - t.start;

    //     // Pause next pattern at zero until `time` reaches the next start offset
    //     let next_pattern_time = if transition_run_time < next_pattern_start {
    //         0
    //     } else {
    //         transition_run_time
    //     };

    //     if !t.driver.is_complete(transition_run_time) {
    //         let frame_delta = state.frame_delta;

    //         println!(
    //             "Transitioning {:?}, at next pattern time {} (runtime {})",
    //             t.driver, next_pattern_time, transition_run_time
    //         );

    //         let update_iter = t
    //             .next_pattern
    //             .update_iter(next_pattern_time, state.frame_delta);

    //         for (current, next) in cube.frame_mut().iter_mut().zip(update_iter) {
    //             let new = t
    //                 .driver
    //                 .transition_pixel(delta, frame_delta, *current, next);

    //             *current = new;
    //         }
    //     } else {
    //         println!(
    //             "Transition complete in {} ms, next pat time {}, offset {}, transition duration {}",
    //             transition_run_time,
    //             next_pattern_time,
    //             t.driver.next_start_offset(),
    //             t.driver.duration(),
    //         );
    //         state.pattern = t.next_pattern.clone();
    //         state.current_start = t.start
    //             + if t.driver.next_start_offset() > 0 {
    //                 transition_run_time
    //             } else {
    //                 0
    //             };
    //         state.transition = None;

    //         // update(time, state, cube);
    //     }

    // // let transition_delta = time - transition.start;
    // // let next_time = transition_delta.saturating_sub(transition.transition.next_start_offset());

    // // if !transition.transition.is_complete(transition_delta) {
    // //     let frame_delta = state.frame_delta;

    // //     let update_iter = transition
    // //         .next_pattern
    // //         .update_iter(next_time, state.frame_delta);

    // //     for (current, next) in cube.frame_mut().iter_mut().zip(update_iter) {
    // //         let new = transition
    // //             .transition
    // //             .transition_pixel(time, frame_delta, *current, next);

    // //         *current = new;
    // //     }
    // // } else {
    // //     state.pattern = transition.next_pattern.clone();
    // //     state.current_start = next_time;
    // //     state.transition = None;

    // //     update(next_time, state, cube);
    // // }
    } else {
        cube.fill_iter(
            state
                .pattern
                .update_iter(pattern_run_time, state.frame_delta),
        );

        match state.pattern {
            Pattern::Rainbow(_) => {
                // Rainbow is disabled
                // unreachable!();

                // TESTING: Hijack as init state
                state.next_pattern(
                    time,
                    Pattern::SlowRain(SlowRain::default()),
                    Some(Transition::FadeFromBlack(FadeFromBlack::default())),
                );
            }
            Pattern::SlowRain(ref mut pattern) => {
                // "cycles" doesn't mean a lot here as drops have different offsets
                if pattern.completed_cycles(pattern_run_time) == 2 {
                    state.next_pattern(
                        time,
                        Pattern::ChristmasPuke(ChristmasPuke::default()),
                        Some(Transition::CrossFade(CrossFade::default())),
                    );
                }
            }
            Pattern::Slices(ref mut pattern) => {
                if pattern.completed_cycles(pattern_run_time) == 1 {
                    state.next_pattern(
                        time,
                        Pattern::SlowRain(SlowRain::default()),
                        Some(Transition::FadeFromBlack(FadeFromBlack::default())),
                    );
                }
            }
            Pattern::ChristmasPuke(ref mut pattern) => {
                if pattern.completed_cycles(pattern_run_time) == 2 {
                    state.next_pattern(
                        time,
                        Pattern::Slices(Slices::default()),
                        Some(Transition::CrossFade(CrossFade::default())),
                    );
                }
            }
        }
    }
}

fn main() {
    let eye = Point3::new(10.0f32, 10.0, 10.0);
    let at = Point3::origin();
    let mut arc_ball = ArcBall::new(eye, at);

    let align_z_up = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 2.0);

    let mut window = Window::new("cube sim");
    window.set_background_color(0.1, 0.1, 0.1);
    window.set_light(Light::StickToCamera);
    let mut floor = window.add_quad(7.0, 7.0, 1, 1);
    floor.set_color(0.2, 0.2, 0.2);
    floor.append_rotation_wrt_center(&align_z_up);

    let mut cube = Cube::new();

    let mut voxels = Vec::new();

    let sphere_scale = 0.35;
    let cube_scale = 1.5;
    let mut g = window.add_group();

    let cube_size = 3.0 * 1.5;

    g.append_translation(&Translation3::new(
        -cube_size / 2.0,
        sphere_scale + 0.1,
        -cube_size / 2.0,
    ));
    g.set_local_scale(sphere_scale, sphere_scale, sphere_scale);

    for idx in 0..64 {
        let pos = Voxel::from_index(idx);

        let x = pos.x as f32;
        let y = pos.y as f32;
        let z = pos.z as f32;

        let mut s = g.add_sphere(1.0);

        // NOTE: Weird ordering here as Z faces out of screen with KISS3D
        s.append_translation(&Translation3::new(
            x as f32 * cube_scale,
            (3.0 - z) as f32 * cube_scale,
            y as f32 * cube_scale,
        ));

        voxels.push(s);
    }

    let start = Instant::now();

    let mut state = State {
        pattern: Pattern::Rainbow(Rainbow::default()),
        // pattern: Pattern::SlowRain(SlowRain::default()),
        // pattern: Pattern::ChristmasPuke(ChristmasPuke::default()),
        // pattern: Pattern::Slices(Slices::default()),
        transition: None,
        current_start: 0,
        frame_delta: 0,
    };

    let mut prev_time = 0;

    cube.set_at_coord(Voxel { x: 0, y: 0, z: 0 }, Apa106Led::WARM_WHITE);

    while window.render_with_camera(&mut arc_ball) {
        let time = start.elapsed().as_millis();

        state.frame_delta = time as u32 - prev_time;

        prev_time = time as u32;

        update(time as u32, &mut state, &mut cube);

        // Update voxel colours
        for (sphere, c) in voxels.iter_mut().zip(cube.frame().iter()) {
            sphere.set_color(
                c.red as f32 / 255.0,
                c.green as f32 / 255.0,
                c.blue as f32 / 255.0,
            );
        }

        // Axes
        // X - red
        window.draw_line(
            &Point3::origin(),
            &Point3::new(1.0, 0.0, 0.0),
            &Point3::new(1.0, 0.0, 0.0),
        );
        // Y - green
        window.draw_line(
            &Point3::origin(),
            &Point3::new(0.0, 1.0, 0.0),
            &Point3::new(0.0, 1.0, 0.0),
        );
        // Z - blue
        window.draw_line(
            &Point3::origin(),
            &Point3::new(0.0, 0.0, 1.0),
            &Point3::new(0.0, 0.0, 1.0),
        );
    }
}
