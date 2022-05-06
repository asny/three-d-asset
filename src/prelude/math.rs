//!
//! Basic math functionality. Mostly just an re-export of [cgmath](https://crates.io/crates/cgmath).
//!

pub use cgmath::{
    dot, frustum, ortho, perspective, vec2, vec3, vec4, Deg, Matrix2, Matrix3, Matrix4, Point2,
    Point3, Quaternion, Rad, Vector2, Vector3, Vector4,
};
pub use cgmath::{
    Angle, EuclideanSpace, InnerSpace, Matrix, MetricSpace, One, Rotation, Rotation2, Rotation3,
    SquareMatrix, Transform, Transform2, Transform3, VectorSpace, Zero,
};

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Mat2 = Matrix2<f32>;
pub type Mat3 = Matrix3<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Quat = Quaternion<f32>;
pub type Degrees = Deg<f32>;
pub type Radians = Rad<f32>;

pub const fn degrees<T>(v: T) -> Deg<T> {
    cgmath::Deg(v)
}
pub const fn radians<T>(v: T) -> Rad<T> {
    cgmath::Rad(v)
}

pub fn rotation_matrix_from_dir_to_dir(source_dir: Vec3, target_dir: Vec3) -> Mat4 {
    Mat4::from(Mat3::from(cgmath::Basis3::between_vectors(
        source_dir, target_dir,
    )))
}
