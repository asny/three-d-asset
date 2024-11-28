//!
//! Basic math functionality. Mostly just an re-export of [cgmath](https://crates.io/crates/cgmath).
//!

pub use cgmath::{
    dot, frustum, ortho, perspective, planar, vec2, vec3, vec4, Deg, Matrix2, Matrix3, Matrix4,
    Point2, Point3, Quaternion, Rad, Vector2, Vector3, Vector4,
};
pub use cgmath::{
    Angle, EuclideanSpace, InnerSpace, Matrix, MetricSpace, One, Rotation, Rotation2, Rotation3,
    SquareMatrix, Transform, Transform2, Transform3, VectorSpace, Zero,
};

///
/// A [Vector2] with f32 data type.
///
pub type Vec2 = Vector2<f32>;

///
/// A [Vector3] with f32 data type.
///
pub type Vec3 = Vector3<f32>;

///
/// A [Vector4] with f32 data type.
///
pub type Vec4 = Vector4<f32>;

///
/// A [Matrix2] with f32 data type.
///
pub type Mat2 = Matrix2<f32>;

///
/// A [Matrix3] with f32 data type.
///
pub type Mat3 = Matrix3<f32>;

///
/// A [Matrix4] with f32 data type.
///
pub type Mat4 = Matrix4<f32>;

///
/// A [Quaternion] with f32 data type.
///
pub type Quat = Quaternion<f32>;

///
/// A [Degrees] with f32 data type.
///
pub type Degrees = Deg<f32>;

///
/// A [Radians] with f32 data type.
///
pub type Radians = Rad<f32>;

///
/// Constructs a an angle in degrees.
///
pub const fn degrees<T>(v: T) -> Deg<T> {
    cgmath::Deg(v)
}

///
/// Constructs a an angle in radians.
///
pub const fn radians<T>(v: T) -> Rad<T> {
    cgmath::Rad(v)
}

///
/// Constructs a rotation matrix that rotates from the source direction to the target direction.
///
pub fn rotation_matrix_from_dir_to_dir(source_dir: Vec3, target_dir: Vec3) -> Mat4 {
    Mat4::from(Mat3::from(cgmath::Basis3::between_vectors(
        source_dir, target_dir,
    )))
}
