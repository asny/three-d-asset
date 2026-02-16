//!
//! Contain texture asset definitions.
//!

pub(crate) mod texture2d;
pub use texture2d::*;

pub(crate) mod texture3d;
pub use texture3d::*;

pub use crate::prelude::f16;
use crate::Srgba;

///
/// Possible modes of interpolation which determines the texture output between texture pixels.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Interpolation {
    Nearest,
    #[default]
    Linear,
    CubicSpline,
}

/// Mipmap settings for a texture.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Mipmap {
    /// Specifies what type of interpolation to use between the two closest mipmaps.
    pub filter: Interpolation,
    /// Specifies the maximum number of mipmap levels that should be created for the texture.
    /// If this is 1, no mip maps will be created.
    pub max_levels: u32,
    /// Specifies the maximum ratio of anisotropy to be used when creating mipmaps for the texture.
    /// If this is 1, only isotropic mipmaps will be created.
    pub max_ratio: u32,
}

impl Default for Mipmap {
    fn default() -> Self {
        Self {
            filter: Interpolation::Linear,
            max_levels: u32::MAX,
            max_ratio: 1,
        }
    }
}

///
/// Possible wrapping modes for a texture which determines how the texture is applied outside of the
/// [0..1] uv coordinate range.
///
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl TextureData {
    ///
    /// Converts the texture data to linear sRGB color space if the data is either
    /// [TextureData::RgbU8] (assuming sRGB color space) or [TextureData::RgbaU8] (assuming sRGB color space with an alpha channel).
    /// Does nothing if the data is any other data type.
    ///
    pub fn to_linear_srgb(&mut self) {
        match self {
            TextureData::RgbU8(data) => data.iter_mut().for_each(|color| {
                *color = Srgba::from(Srgba::from(*color).to_linear_srgb()).into();
            }),
            TextureData::RgbaU8(data) => data.iter_mut().for_each(|color| {
                *color = Srgba::from(Srgba::from(*color).to_linear_srgb()).into();
            }),
            _ => {}
        };
    }

    ///
    /// Converts the texture data to color [TextureData::RgbU8] if the data is [TextureData::RU8] (assuming gray scale colors).
    /// Does nothing if the data is any other data type.
    ///
    pub fn to_color(&mut self) {
        if let TextureData::RU8(data) = self {
            *self = TextureData::RgbU8(data.iter().map(|color| [*color, *color, *color]).collect())
        };
    }
}
