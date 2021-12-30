
extern crate num;
extern crate time;

use cg::{self, EuclideanVector};

pub mod mat;
pub use self::mat::*;
pub mod non_zero;
pub use self::non_zero::*;
pub mod cardinal_direction;
pub use self::cardinal_direction::*;
pub mod array_map;
pub use self::array_map::*;

pub trait MapRange: Sized {
    fn map_range(&self, from: [Self; 2], to: [Self; 2]) -> Self;
}

impl MapRange for f32 {
    fn map_range(&self, from: [f32; 2], to: [f32; 2]) -> f32 {
        (self - from[0]) / (from[1] - from[0]) * (to[1] - to[0]) + to[0]
    }
}

pub struct Clock {
    last: f64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            last: time::precise_time_s(),
        }
    }

    pub fn delta(&mut self) -> f64 {
        let now = time::precise_time_s();
        let diff = now - self.last;
        self.last = now;
        diff
    }

    /// Time of the last delta() call
    pub fn time(&self) -> f64 {
        self.last
    }
}

pub struct FirstPersonCam {
    pub pos: cg::Point3<f32>,
    pub dir: cg::Vector3<f32>,
    pub move_x: f32,
    pub move_y: f32,
    pub move_z: f32,
}

impl FirstPersonCam {
    pub fn new(pos: cg::Point3<f32>, dir: cg::Vector3<f32>) -> FirstPersonCam {
        FirstPersonCam {
            pos: pos,
            dir: dir.normalize(),
            move_x: 0.0,
            move_y: 0.0,
            move_z: 0.0,
        }
    }

    pub fn rotate(&mut self, pitch: cg::Rad<f32>, yaw: cg::Rad<f32>) {
        let a = -cg::Vector2::new(self.dir.x, self.dir.z).angle(cg::Vector2::new(0.0, 1.0));
        self.dir = cg::Matrix3::from_angle_y(-a + pitch) * cg::Matrix3::from_angle_x(-yaw) * cg::Matrix3::from_angle_y(a) * self.dir;
    }

    pub fn set_movement(&mut self, movement: cg::Vector3<f32>) {
        self.move_x = movement.x;
        self.move_y = movement.y;
        self.move_z = movement.z;
    }

    pub fn update_pos(&mut self, delta: f32) {
        let up = cg::Vector3::new(0.0, 1.0, 0.0);
        let cam_right = self.dir.cross(up).normalize();
        let cam_forw = -cam_right.cross(up);

        if !cam_right.length().is_nan() && !cam_forw.length().is_nan() {
            self.pos = self.pos
                + cam_forw * self.move_z * delta * 8.0
                + cam_right * self.move_x * delta * 8.0
                + up * self.move_y * delta * 8.0;
        }
    }
}
