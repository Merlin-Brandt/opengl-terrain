
#[macro_use]
extern crate glium;
extern crate cgmath as cg;


use std::collections::{HashSet};
use std::thread;
use std::time::Duration;
use std::hash::{Hasher};

use cg::{Point3, Vector2, Vector3, Matrix4, Rad, Rotation};
type Point3f = Point3<f32>;
type Vector2f = Vector2<f32>;
type Vector3f = Vector3<f32>;
type Matrix4f = Matrix4<f32>;

use glium::glutin::{GlProfile, Event, ElementState, VirtualKeyCode as KeyCode};

use util::*;
use renderer::{Renderer};

mod util;
mod terrain;
mod renderer;
mod mesh;

fn main() {
    use glium::DisplayBuild;

    let monitor = glium::glutin::get_primary_monitor();
    let (w, h) = monitor.get_dimensions();
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(w, h)
        .with_fullscreen(monitor)
        .with_gl_profile(GlProfile::Compatibility)
        .with_depth_buffer(24)
        .with_multisampling(8)
        .build_glium().expect("Error building display");

    let mut clock = Clock::new();
    let mut cursor_pos = (0, 0);
    let mut cursor_jump = true;
    let mut pressed_keys = HashSet::new();
    let mut cam = FirstPersonCam::new(Point3f::new(60.0, 30.0, 60.0), Vector3f::new(0.0, -0.3, 0.0));

    let fovy: Rad<f32> = cg::deg(90.0).into();
    let aspect = w as f32 / h as f32;
    let fovx = fovy * aspect;
    let proj = cg::perspective(fovy, aspect, 0.1, 100.0);

    let mut renderer = Renderer::new(&display).expect("Error creating Renderer.");

    'main: loop {
        let delta = clock.delta() as f32;

        for event in display.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::MouseMoved((new_x, new_y)) => {

                    if !cursor_jump {
                        let dx = (cursor_pos.0 - new_x) as f32;
                        let dy = (cursor_pos.1 - new_y) as f32;

                        let pitch = (fovx / w as f32) * dx * 2.0;
                        let yaw = (fovy / h as f32) * dy * 2.0;

                        cam.rotate(pitch, yaw);
                    } else {
                        cursor_jump = false;
                    }

                    cursor_pos = (new_x, new_y)
                },
                Event::KeyboardInput(state, _, Some(key_code)) => {
                    if state == ElementState::Pressed {
                        pressed_keys.insert(key_code);
                    } else {
                        pressed_keys.remove(&key_code);
                    }
                }
                _ => (),
            }
        }

        if let Some(new_cp) = wrap_cursor_pos([cursor_pos.0, cursor_pos.1], [w, h]) {
            cursor_jump = true;
            display.get_window().unwrap().set_cursor_position(new_cp[0], new_cp[1]).unwrap();
        }

        cam.set_movement(movement_from_pressed_keys(&pressed_keys));

        // update cam pos
        cam.update_pos(delta);

        let mut target = display.draw();

        {
            let view = Matrix4f::look_at(cam.pos, cam.pos + cam.dir, Vector3f::new(0.0, 1.0, 0.0));
            renderer.render(&mut target, (proj * view).as_ref(), clock.time() as f32);
        }

        target.finish().expect("Error swapping");

        thread::sleep(Duration::from_millis(1));
    }
}

fn wrap_cursor_pos(cursor: [i32; 2], window: [u32; 2]) -> Option<[i32; 2]> {
    let cx = cursor[0];
    let cy = cursor[1];
    let w = window[0] as i32;
    let h = window[1] as i32;
    let b = 80;

    let new_cx =
        if cx > w - b {
            b + 1
        } else if cx < b {
            w - b - 1
        } else {
            cx
        };
    let new_cy =
        if cy > h - b {
            b + 1
        } else if cy < b {
            h - b - 1
        } else {
            cy
        };

    if new_cx != cx || new_cy != cy {
        Some([new_cx, new_cy])
    } else {
        None
    }
}

fn movement_from_pressed_keys(pressed_keys: &HashSet<KeyCode>) -> Vector3<f32> {
    use glium::glutin::VirtualKeyCode::*;

    let mut movement = Vector3::new(0.0, 0.0, 0.0);

    if pressed_keys.contains(&W) {
        movement.z+= 1.0;
    }
    if pressed_keys.contains(&S) {
        movement.z-= 1.0;
    }
    if pressed_keys.contains(&D) {
        movement.x+= 1.0;
    }
    if pressed_keys.contains(&A) {
        movement.x-= 1.0;
    }
    if pressed_keys.contains(&Space) {
        movement.y+= 1.0;
    }
    if pressed_keys.contains(&LShift) {
        movement.y-= 1.0;
    }

    movement
}
