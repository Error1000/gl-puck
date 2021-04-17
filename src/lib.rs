use glam::{Mat3, Mat4};

#[macro_use]
extern crate lazy_static;

pub mod camera;
pub mod input;
pub mod mesh;
pub mod model;
pub mod obj;

// TODO: This is a hack, usually you would use a mat4 but i don't want to waste 7 floats
// I know a bit stupid but i'll figure something better out soon(tm)
#[inline]
pub fn make_ortho_2d(width: f32, height: f32) -> Mat3 {
    let proj4: [f32; 16] = Mat4::orthographic_lh(
        -width / 2.0,
        width / 2.0,
        -height / 2.0,
        height / 2.0,
        0.0,
        2.0,
    )
    .to_cols_array();

    Mat3::from_cols_array(&[
        proj4[0], proj4[1], proj4[2], proj4[4], proj4[5], proj4[6], proj4[8], proj4[9], proj4[10],
    ])
}
