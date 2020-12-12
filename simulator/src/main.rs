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
use std::time::Instant;

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

            for (current, next) in cube.frame_mut().iter_mut().zip(update_iter) {
                let new = transition
                    .transition
                    .transition_pixel(time, frame_delta, *current, next);

                *current = new;
            }
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
            Pattern::SlowRain(ref mut pattern) => {
                // Uncomment to cycle patterns
                // if pattern.completed_cycles(delta) > 5 {
                //     state.next_pattern(
                //         time,
                //         Pattern::Police(Police::default()),
                //         Some(Transition::FadeToBlack(FadeToBlack::default())),
                //     );
                // }
            }
            Pattern::Police(ref mut police) => {
                if police.completed_cycles(delta) > 5 {
                    state.next_pattern(
                        time,
                        Pattern::Rainbow(Rainbow::default()),
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

    let offset = -3.0 / 2.0;
    let sphere_scale = 0.25;
    let mut g = window.add_group();
    g.append_rotation_wrt_center(&align_z_up);
    g.append_translation(&Translation3::new(offset, 3.0 + sphere_scale + 0.2, offset));
    g.set_local_scale(sphere_scale, sphere_scale, sphere_scale);

    for idx in 0..64 {
        let pos = Voxel::from_index(idx);

        let x = pos.x as f32;
        let y = pos.y as f32;
        let z = pos.z as f32;

        let mut s = g.add_sphere(1.0);

        s.append_translation(&Translation3::new(x as f32, y as f32, z as f32));

        voxels.push(s);
    }

    let start = Instant::now();

    let mut state = State {
        // pattern: Pattern::Rainbow(Rainbow::default()),
        pattern: Pattern::SlowRain(SlowRain::default()),
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
    }
}
