#[doc(inline)]
pub use crate::texture::{Interpolation, TextureData, Wrapping};

///
/// A CPU-side version of a 2D texture.
///
#[derive(Clone, Debug, PartialEq)]
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
    /// Specifies whether mipmaps should be created for this texture and what type of interpolation to use between the two closest mipmaps.
    /// Note, however, that the mipmaps only will be created if the width and height of the texture are power of two.
    pub mip_map_filter: Option<Interpolation>,
    /// Determines how the texture is sampled outside the [0..1] s coordinate range (the first value of the uv coordinates).
    pub wrap_s: Wrapping,
    /// Determines how the texture is sampled outside the [0..1] t coordinate range (the second value of the uv coordinates).
    pub wrap_t: Wrapping,
}

impl Texture2D {
    ///
    /// Returns a clone of this texture where the data is converted to linear sRGB color space if the data is either
    /// [TextureData::RgbU8] (assuming sRGB color space) or [TextureData::RgbaU8] (assuming sRGB color space with an alpha channel),
    /// otherwise it returns `None`.
    ///
    pub fn to_linear_srgb(&self) -> Option<Self> {
        let convert = |rgb: &[u8]| {
            let mut linear_rgb = [0u8; 3];
            for i in 0..3 {
                let c = rgb[i] as f32 / 255.0;
                let c = if c < 0.04045 {
                    c / 12.92
                } else {
                    ((c + 0.055) / 1.055).powf(2.4)
                };
                linear_rgb[i] = (c * 255.0) as u8;
            }
            linear_rgb
        };

        let data = match &self.data {
            TextureData::RgbU8(data) => {
                TextureData::RgbU8(data.iter().map(|color| convert(color)).collect())
            }
            TextureData::RgbaU8(data) => TextureData::RgbaU8(
                data.into_iter()
                    .map(|color| {
                        let rgb = convert(color);
                        let mut rgba = color.clone();
                        for i in 0..3 {
                            rgba[i] = rgb[i];
                        }
                        rgba
                    })
                    .collect(),
            ),
            _ => return None,
        };
        Some(Self {
            data,
            ..self.clone()
        })
    }
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
            mip_map_filter: Some(Interpolation::Linear),
            wrap_s: Wrapping::Repeat,
            wrap_t: Wrapping::Repeat,
        }
    }
}
