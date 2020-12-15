use common::{apa106led::Apa106Led, cube::Cube, patterns::*, state::State, voxel::Voxel};
use core::f32::consts::PI;
use kiss3d::camera::ArcBall;
use kiss3d::light::Light;
use kiss3d::window::Window;
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};

use std::time::Instant;

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

    let mut state = State::new(Pattern::Rainbow(Rainbow::default()));

    cube.set_at_coord(Voxel { x: 0, y: 0, z: 0 }, Apa106Led::WARM_WHITE);

    while window.render_with_camera(&mut arc_ball) {
        let time = start.elapsed().as_millis();

        state.drive(time as u32, &mut cube);

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
