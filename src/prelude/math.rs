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

/// Create a planar projection matrix, which can be either perspective or orthographic.
///
/// The projection frustum is always `height` units high at the origin along the view direction,
/// making the focal point located at `(0.0, 0.0, cot(fovy / 2.0)) * height / 2.0`. Unlike
/// a standard perspective projection, this allows `fovy` to be zero or negative.
pub fn planar<S: cgmath::BaseFloat, A: Into<Rad<S>>>(
    fovy: A,
    aspect: S,
    height: S,
    near: S,
    far: S,
) -> Matrix4<S> {
    PlanarFov {
        fovy: fovy.into(),
        aspect,
        height,
        near,
        far,
    }
    .into()
}

/// A planar projection based on a vertical field-of-view angle.
#[derive(Copy, Clone, Debug, PartialEq)]
struct PlanarFov<S> {
    pub fovy: Rad<S>,
    pub aspect: S,
    pub height: S,
    pub near: S,
    pub far: S,
}

impl<S: cgmath::BaseFloat> From<PlanarFov<S>> for Matrix4<S> {
    fn from(persp: PlanarFov<S>) -> Matrix4<S> {
        assert!(
            persp.fovy > -Rad::turn_div_2(),
            "The vertical field of view cannot be less than a negative half turn, found: {:?}",
            persp.fovy
        );
        assert!(
            persp.fovy < Rad::turn_div_2(),
            "The vertical field of view cannot be greater than a half turn, found: {:?}",
            persp.fovy
        );
        assert! {
            persp.height >= S::zero(),
            "The projection plane height cannot be negative, found: {:?}",
            persp.height
        }

        let two: S = cgmath::num_traits::cast(2).unwrap();
        let inv_f = Rad::tan(persp.fovy / two) * two / persp.height;

        let focal_point = -inv_f.recip();

        assert!(
            cgmath::abs_diff_ne!(persp.aspect.abs(), S::zero()),
            "The absolute aspect ratio cannot be zero, found: {:?}",
            persp.aspect.abs()
        );
        assert!(
            cgmath::abs_diff_ne!(persp.far, persp.near),
            "The far plane and near plane are too close, found: far: {:?}, near: {:?}",
            persp.far,
            persp.near
        );
        assert!(
            focal_point < S::min(persp.far, persp.near) || focal_point > S::max(persp.far, persp.near),
            "The focal point cannot be between the far and near planes, found: focal: {:?}, far: {:?}, near: {:?}",
            focal_point,
            persp.far,
            persp.near,
        );

        let c0r0 = two / (persp.aspect * persp.height);
        let c0r1 = S::zero();
        let c0r2 = S::zero();
        let c0r3 = S::zero();

        let c1r0 = S::zero();
        let c1r1 = two / persp.height;
        let c1r2 = S::zero();
        let c1r3 = S::zero();

        let c2r0 = S::zero();
        let c2r1 = S::zero();
        let c2r2 = ((persp.far + persp.near) * inv_f + two) / (persp.near - persp.far);
        let c2r3 = -inv_f;

        let c3r0 = S::zero();
        let c3r1 = S::zero();
        let c3r2 = (two * persp.far * persp.near * inv_f + (persp.far + persp.near))
            / (persp.near - persp.far);
        let c3r3 = S::one();

        #[rustfmt::skip]
        let result = Matrix4::new(
            c0r0, c0r1, c0r2, c0r3,
            c1r0, c1r1, c1r2, c1r3,
            c2r0, c2r1, c2r2, c2r3,
            c3r0, c3r1, c3r2, c3r3,
        );
        result
    }
}
