#![allow(missing_docs)]
//!
//! Basic math functionality. Mostly just an re-export of [cgmath](https://crates.io/crates/cgmath).
//!

pub use cgmath::ortho;
pub use cgmath::perspective;
#[doc(hidden)]
pub use cgmath::prelude::*;
pub use cgmath::{Matrix2, Matrix3, Matrix4, Quaternion, Vector2, Vector3, Vector4};

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Mat2 = Matrix2<f32>;
pub type Mat3 = Matrix3<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Point = cgmath::Point3<f32>;
pub type Degrees = cgmath::Deg<f32>;
pub type Radians = cgmath::Rad<f32>;
pub type Quat = Quaternion<f32>;

pub const fn vec2<T>(x: T, y: T) -> Vector2<T> {
    Vector2::new(x, y)
}

pub const fn vec3<T>(x: T, y: T, z: T) -> Vector3<T> {
    Vector3::new(x, y, z)
}

pub const fn vec4<T>(x: T, y: T, z: T, w: T) -> Vector4<T> {
    Vector4::new(x, y, z, w)
}

pub const fn degrees(v: f32) -> Degrees {
    cgmath::Deg(v)
}
pub const fn radians(v: f32) -> Radians {
    cgmath::Rad(v)
}

pub fn rotation_matrix_from_dir_to_dir(source_dir: Vec3, target_dir: Vec3) -> Mat4 {
    Mat4::from(Mat3::from(cgmath::Basis3::between_vectors(
        source_dir, target_dir,
    )))
}
