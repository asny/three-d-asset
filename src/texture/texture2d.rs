#[doc(inline)]
pub use crate::texture::{Interpolation, TextureData, Wrapping};

///
/// A CPU-side version of a 2D texture.
///
#[derive(Clone, Debug)]
pub struct Texture2D {
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
    /// Specifies whether mipmaps should be created for this texture and what type of interpolation to use between the two closest mipmaps.
    /// Note, however, that the mipmaps only will be created if the width and height of the texture are power of two.
    pub mip_map_filter: Option<Interpolation>,
    /// Determines how the texture is sampled outside the [0..1] s coordinate range (the first value of the uv coordinates).
    pub wrap_s: Wrapping,
    /// Determines how the texture is sampled outside the [0..1] t coordinate range (the second value of the uv coordinates).
    pub wrap_t: Wrapping,
}

impl Default for Texture2D {
    fn default() -> Self {
        Self {
            data: TextureData::RgbaU8(vec![[0, 0, 0, 0]]),
            width: 1,
            height: 1,
            min_filter: Interpolation::Linear,
            mag_filter: Interpolation::Linear,
            mip_map_filter: Some(Interpolation::Linear),
            wrap_s: Wrapping::Repeat,
            wrap_t: Wrapping::Repeat,
        }
    }
}
