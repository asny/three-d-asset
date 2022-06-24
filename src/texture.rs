//!
//! Contain texture asset definitions.
//!

pub(crate) mod texture2d;
pub use texture2d::*;

pub(crate) mod texture3d;
pub use texture3d::*;

pub use crate::prelude::f16;

///
/// Possible modes of interpolation which determines the texture output between texture pixels.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Interpolation {
    Nearest,
    Linear,
}

///
/// Possible wrapping modes for a texture which determines how the texture is applied outside of the
/// [0..1] uv coordinate range.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Wrapping {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
}

///
/// The pixel/texel data for a [Texture2D] or [Texture3D].
///
/// If 2D data, the data array should start with the top left texel and then one row at a time.
/// The indices `(row, column)` into the 2D data would look like
/// ```notrust
/// [
/// (0, 0), (1, 0), .., // First row
/// (0, 1), (1, 1), .., // Second row
/// ..
/// ]
/// ```
/// If 3D data, the data array would look like the 2D data, one layer/image at a time.
/// The indices `(row, column, layer)` into the 3D data would look like
/// ```notrust
/// [
/// (0, 0, 0), (1, 0, 0), .., // First row in first layer
/// (0, 1, 0), (1, 1, 0), .., // Second row in first layer
/// ..
/// (0, 0, 1), (1, 0, 1), .., // First row in second layer
/// (0, 1, 1), (1, 1, 1), ..,  // Second row in second layer
/// ..
/// ]
/// ```
///
#[derive(Clone, PartialEq)]
pub enum TextureData {
    /// One byte in the red channel.
    RU8(Vec<u8>),
    /// One byte in the red and green channel.
    RgU8(Vec<[u8; 2]>),
    /// One byte in the red, green and blue channel.
    RgbU8(Vec<[u8; 3]>),
    /// One byte in the red, green, blue and alpha channel.
    RgbaU8(Vec<[u8; 4]>),

    /// 16-bit float in the red channel.
    RF16(Vec<f16>),
    /// 16-bit float in the red and green channel.
    RgF16(Vec<[f16; 2]>),
    /// 16-bit float in the red, green and blue channel.
    RgbF16(Vec<[f16; 3]>),
    /// 16-bit float in the red, green, blue and alpha channel.
    RgbaF16(Vec<[f16; 4]>),

    /// 32-bit float in the red channel.
    RF32(Vec<f32>),
    /// 32-bit float in the red and green channel.
    RgF32(Vec<[f32; 2]>),
    /// 32-bit float in the red, green and blue channel.
    RgbF32(Vec<[f32; 3]>),
    /// 32-bit float in the red, green, blue and alpha channel.
    RgbaF32(Vec<[f32; 4]>),
}

impl std::fmt::Debug for TextureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RU8(values) => write!(f, "R u8 ({:?})", values.len()),
            Self::RgU8(values) => write!(f, "RG u8 ({:?})", values.len()),
            Self::RgbU8(values) => write!(f, "RGB u8 ({:?})", values.len()),
            Self::RgbaU8(values) => write!(f, "RGBA u8 ({:?})", values.len()),
            Self::RF16(values) => write!(f, "R f16 ({:?})", values.len()),
            Self::RgF16(values) => write!(f, "RG f16 ({:?})", values.len()),
            Self::RgbF16(values) => write!(f, "RGB f16 ({:?})", values.len()),
            Self::RgbaF16(values) => write!(f, "RGBA f16 ({:?})", values.len()),
            Self::RF32(values) => write!(f, "R f32 ({:?})", values.len()),
            Self::RgF32(values) => write!(f, "RG f32 ({:?})", values.len()),
            Self::RgbF32(values) => write!(f, "RGB f32 ({:?})", values.len()),
            Self::RgbaF32(values) => write!(f, "RGBA f32 ({:?})", values.len()),
        }
    }
}
