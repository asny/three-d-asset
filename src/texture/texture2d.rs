#[doc(inline)]
pub use super::{Interpolation, Mipmap, TextureData, Wrapping};

///
/// A CPU-side version of a 2D texture.
///
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Texture2D {
    /// Name of this texture.
    pub name: String,
    /// The pixel data for the image
    pub data: TextureData,
    /// The width of the image
    pub width: u32,
    /// The height of the image
    pub height: u32,
    /// The way the pixel data is interpolated when the texture is far away
    pub min_filter: Interpolation,
    /// The way the pixel data is interpolated when the texture is close
    pub mag_filter: Interpolation,
    /// Specifies the [Mipmap] settings for this texture. If this is `None`, no mipmaps are created.
    pub mipmap: Option<Mipmap>,
    /// Determines how the texture is sampled outside the [0..1] s coordinate range (the first value of the uv coordinates).
    pub wrap_s: Wrapping,
    /// Determines how the texture is sampled outside the [0..1] t coordinate range (the second value of the uv coordinates).
    pub wrap_t: Wrapping,
}

impl Default for Texture2D {
    fn default() -> Self {
        Self {
            name: "default".to_owned(),
            data: TextureData::RgbaU8(vec![[0, 0, 0, 0]]),
            width: 1,
            height: 1,
            min_filter: Interpolation::Linear,
            mag_filter: Interpolation::Linear,
            mipmap: Some(Mipmap::default()),
            wrap_s: Wrapping::Repeat,
            wrap_t: Wrapping::Repeat,
        }
    }
}
