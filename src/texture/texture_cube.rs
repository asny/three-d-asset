#[doc(inline)]
pub use crate::texture::{f16, Interpolation, Wrapping};

///
/// The pixel data for a [TextureCube].
///
#[derive(Clone)]
pub enum TextureCubeData {
    /// byte in the red channel.
    RU8(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>),
    /// byte in the red and green channel.
    RgU8(
        Vec<[u8; 2]>,
        Vec<[u8; 2]>,
        Vec<[u8; 2]>,
        Vec<[u8; 2]>,
        Vec<[u8; 2]>,
        Vec<[u8; 2]>,
    ),
    /// byte in the red, green and blue channel.
    RgbU8(
        Vec<[u8; 3]>,
        Vec<[u8; 3]>,
        Vec<[u8; 3]>,
        Vec<[u8; 3]>,
        Vec<[u8; 3]>,
        Vec<[u8; 3]>,
    ),
    /// byte in the red, green, blue and alpha channel.
    RgbaU8(
        Vec<[u8; 4]>,
        Vec<[u8; 4]>,
        Vec<[u8; 4]>,
        Vec<[u8; 4]>,
        Vec<[u8; 4]>,
        Vec<[u8; 4]>,
    ),

    /// 16-bit float in the red channel.
    RF16(Vec<f16>, Vec<f16>, Vec<f16>, Vec<f16>, Vec<f16>, Vec<f16>),
    /// 16-bit float in the red and green channel.
    RgF16(
        Vec<[f16; 2]>,
        Vec<[f16; 2]>,
        Vec<[f16; 2]>,
        Vec<[f16; 2]>,
        Vec<[f16; 2]>,
        Vec<[f16; 2]>,
    ),
    /// 16-bit float in the red, green and blue channel.
    RgbF16(
        Vec<[f16; 3]>,
        Vec<[f16; 3]>,
        Vec<[f16; 3]>,
        Vec<[f16; 3]>,
        Vec<[f16; 3]>,
        Vec<[f16; 3]>,
    ),
    /// 16-bit float in the red, green, blue and alpha channel.
    RgbaF16(
        Vec<[f16; 4]>,
        Vec<[f16; 4]>,
        Vec<[f16; 4]>,
        Vec<[f16; 4]>,
        Vec<[f16; 4]>,
        Vec<[f16; 4]>,
    ),

    /// 32-bit float in the red channel.
    RF32(Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>),
    /// 32-bit float in the red and green channel.
    RgF32(
        Vec<[f32; 2]>,
        Vec<[f32; 2]>,
        Vec<[f32; 2]>,
        Vec<[f32; 2]>,
        Vec<[f32; 2]>,
        Vec<[f32; 2]>,
    ),
    /// 32-bit float in the red, green and blue channel.
    RgbF32(
        Vec<[f32; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 3]>,
    ),
    /// 32-bit float in the red, green, blue and alpha channel.
    RgbaF32(
        Vec<[f32; 4]>,
        Vec<[f32; 4]>,
        Vec<[f32; 4]>,
        Vec<[f32; 4]>,
        Vec<[f32; 4]>,
        Vec<[f32; 4]>,
    ),
}

///
/// A CPU-side version of a cube map texture. All 6 images must have the same dimensions.
///
pub struct TextureCube {
    /// The pixel data for the cube image
    pub data: TextureCubeData,
    /// The width of each of the 6 images
    pub width: u32,
    /// The height of each of the 6 images
    pub height: u32,
    /// The way the pixel data is interpolated when the texture is far away
    pub min_filter: Interpolation,
    /// The way the pixel data is interpolated when the texture is close
    pub mag_filter: Interpolation,
    /// Specifies whether mipmaps should be created for this texture and what type of interpolation to use between the two closest mipmaps.
    /// Note, however, that the mipmaps only will be created if the width and height of the texture are power of two.
    pub mip_map_filter: Option<Interpolation>,
    /// Determines how the texture is sampled outside the [0..1] s coordinate range.
    pub wrap_s: Wrapping,
    /// Determines how the texture is sampled outside the [0..1] t coordinate range.
    pub wrap_t: Wrapping,
    /// Determines how the texture is sampled outside the [0..1] r coordinate range.
    pub wrap_r: Wrapping,
}

impl Default for TextureCube {
    fn default() -> Self {
        Self {
            data: TextureCubeData::RgbaU8(
                vec![[255, 0, 0, 255]],
                vec![[255, 0, 0, 255]],
                vec![[255, 0, 0, 255]],
                vec![[255, 0, 0, 255]],
                vec![[255, 0, 0, 255]],
                vec![[255, 0, 0, 255]],
            ),
            width: 1,
            height: 1,
            min_filter: Interpolation::Linear,
            mag_filter: Interpolation::Linear,
            mip_map_filter: Some(Interpolation::Linear),
            wrap_s: Wrapping::Repeat,
            wrap_t: Wrapping::Repeat,
            wrap_r: Wrapping::Repeat,
        }
    }
}

impl std::fmt::Debug for TextureCube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureCube")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("min_filter", &self.min_filter)
            .field("mag_filter", &self.mag_filter)
            .field("mip_map_filter", &self.mip_map_filter)
            .field("wrap_s", &self.wrap_s)
            .field("wrap_t", &self.wrap_t)
            .field("wrap_r", &self.wrap_r)
            .finish()
    }
}
