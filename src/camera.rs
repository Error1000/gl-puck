use glam::*;

use crate::model::{World2D, World3D};

pub struct Camera2D {
    pos: Vec2,
    mat: Option<Mat3>,
}

impl Default for Camera2D {
    fn default() -> Self {
        Camera2D {
            pos: Vec2::new(0.0, 0.0),
            mat: None,
        }
    }
}

impl Camera2D {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_mat(self: &mut Self) {
        self.mat = Some(Mat3::from_scale_angle_translation(
            Vec2::new(1.0, 1.0),
            0.0,
            -self.pos,
        ));
    }

    // snap_range is the range in u.m. that the target has to be within when you call the function to "snap" ( set the camera's pos) to the target's
    // acceleration is the amount of the distance it will travel per call to the function
    pub fn lerp_to(self: &mut Self, target: &Vec2, acceleration: Vec2, snap_range: f32) {
        if self.get_pos() != target {
            let distance = *target - *self.get_pos();
            self.strafe(distance * acceleration);
            let r: [f32; 2] = distance.abs().into();
            if r[0] < snap_range && r[1] < snap_range {
                self.set_pos(*target);
            }
        }
    }
}

impl World2D for Camera2D {
    #[inline]
    fn get_mat(self: &mut Self) -> &Mat3 {
        if self.mat.is_none() {
            self.update_mat();
        }
        self.mat.as_ref().unwrap()
    }

    fn get_aabb(self: &Self) -> &Vec2 {
        unimplemented!()
    }

    #[inline]
    fn get_pos(self: &Self) -> &Vec2 {
        &self.pos
    }

    #[inline]
    fn set_pos(self: &mut Self, val: Vec2) {
        self.pos = val;
        self.mat = None;
    }

    #[inline]
    fn strafe(self: &mut Self, val: Vec2) {
        self.pos += val;
        self.mat = None;
    }
}

pub struct Camera3D {
    pos: Vec3,
    mat: Option<Mat4>,
    looking_dir: Vec3,
    rot: Vec3, // in Degrees ( not radians )
    up: Vec3,  // in Degrees ( not radians )
}

impl Default for Camera3D {
    fn default() -> Self {
        Camera3D {
            pos: Vec3::new(0.0, 0.0, 0.0),
            mat: None,
            looking_dir: Vec3::new(1.0, 1.0, 1.0),
            rot: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

impl Camera3D {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_mat(self: &mut Self) {
        // Faster to cache
        let mut rot_x = self.rot.x;
        let rot_y = self.rot.y;
        if rot_x >= 89.9 {
            self.rot.x = 89.9;
            rot_x = 89.9;
        } else if rot_x <= -89.9 {
            self.rot.x = -89.9;
            rot_x = -89.9;
        }
        self.looking_dir = Vec3::new(
            rot_y.to_radians().cos() * rot_x.to_radians().cos(),
            rot_x.to_radians().sin(),
            rot_y.to_radians().sin() * rot_x.to_radians().cos(),
        );

        self.up = Vec3::new(
            self.rot.z.to_radians().sin(),
            self.rot.z.to_radians().cos(),
            0.0,
        );
        self.mat = Some(Mat4::look_at_lh(
            self.pos,
            self.pos + self.looking_dir.normalize(),
            self.up.normalize(),
        ));
    }

    #[inline]
    pub fn get_rot(self: &Self) -> &Vec3 {
        &self.rot
    }

    #[inline]
    pub fn get_looking_dir(self: &Self) -> &Vec3 {
        &self.looking_dir
    }

    #[inline]
    pub fn look_at(self: &mut Self, point: Vec3) {
        self.looking_dir = point;
        self.mat = None;
    }

    fn scale(b: &Vec3, mut v: Vec3) -> Vec3 {
        v.x *= b.x;
        v.y *= b.y;
        v.z *= b.z;
        v
    }
    pub fn masked_step(self: &mut Self, amount: Vec2, mask: Vec3) {
        self.pos += amount.y * (Self::scale(&mask, self.looking_dir.normalize())).normalize();
        self.pos -= amount.x * self.looking_dir.cross(self.up).normalize();
        self.mat = None;
    }

    pub fn step(self: &mut Self, amount: Vec2) {
        self.masked_step(amount, Vec3::new(1.0, 1.0, 1.0))
    }

    pub fn rotate(self: &mut Self, r: Vec3) {
        self.rot += r;
        self.mat = None;
    }

    pub fn set_rot(self: &mut Self, r: Vec3) {
        self.rot = r;
        self.mat = None;
    }
}

impl World3D for Camera3D {
    #[inline]
    fn get_mat(self: &mut Self) -> &Mat4 {
        if self.mat.is_none() {
            self.update_mat();
        }
        self.mat.as_ref().unwrap()
    }

    fn get_aabb(self: &Self) -> &Vec3 {
        unimplemented!()
    }

    #[inline]
    fn get_pos(self: &Self) -> &Vec3 {
        &self.pos
    }

    #[inline]
    fn set_pos(self: &mut Self, val: Vec3) {
        self.pos = val;
        self.mat = None;
    }

    #[inline]
    fn strafe(self: &mut Self, val: Vec3) {
        self.pos += val;
        self.mat = None;
    }
}
