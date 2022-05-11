use crate::{io::Deserialize, io::Loaded, io::Serialize, texture::*, Result};
use image::{io::Reader, *};
use std::io::Cursor;
use std::path::Path;

impl Deserialize for Texture2D {
    fn deserialize(raw_assets: &mut Loaded, path: impl AsRef<std::path::Path>) -> Result<Self> {
        let bytes = raw_assets.remove_bytes(path)?;
        Self::deserialize_internal(&bytes)
    }
}

impl Texture2D {
    pub(crate) fn deserialize_internal(bytes: &[u8]) -> Result<Self> {
        let reader = Reader::new(Cursor::new(bytes))
            .with_guessed_format()
            .expect("Cursor io never fails");
        #[cfg(feature = "hdr")]
        if reader.format() == Some(image::ImageFormat::Hdr) {
            use image::codecs::hdr::*;
            let decoder = HdrDecoder::new(bytes)?;
            let metadata = decoder.metadata();
            let img = decoder.read_image_native()?;
            return Ok(Texture2D {
                data: TextureData::RgbF32(
                    img.iter()
                        .map(|rgbe| {
                            let Rgb(values) = rgbe.to_hdr();
                            [values[0], values[1], values[2]]
                        })
                        .collect::<Vec<_>>(),
                ),
                width: metadata.width,
                height: metadata.height,
                ..Default::default()
            });
        }
        let img: DynamicImage = reader.decode()?;
        let width = img.width();
        let height = img.height();
        let data = match img {
            DynamicImage::ImageLuma8(_) => TextureData::RU8(img.into_bytes()),
            DynamicImage::ImageLumaA8(_) => {
                let bytes = img.as_bytes();
                let mut data = Vec::new();
                for i in 0..bytes.len() / 2 {
                    data.push([bytes[i * 2], bytes[i * 2 + 1]]);
                }
                TextureData::RgU8(data)
            }
            DynamicImage::ImageRgb8(_) => {
                let bytes = img.as_bytes();
                let mut data = Vec::new();
                for i in 0..bytes.len() / 3 {
                    data.push([bytes[i * 3], bytes[i * 3 + 1], bytes[i * 3 + 2]]);
                }
                TextureData::RgbU8(data)
            }
            DynamicImage::ImageRgba8(_) => {
                let bytes = img.as_bytes();
                let mut data = Vec::new();
                for i in 0..bytes.len() / 4 {
                    data.push([
                        bytes[i * 4],
                        bytes[i * 4 + 1],
                        bytes[i * 4 + 2],
                        bytes[i * 4 + 3],
                    ]);
                }
                TextureData::RgbaU8(data)
            }
            _ => unimplemented!(),
        };
        Ok(Self {
            data,
            width,
            height,
            ..Default::default()
        })
    }
}

impl Serialize for Texture2D {
    fn serialize(&self, path: impl AsRef<Path>) -> Result<Loaded> {
        // TODO: Put actual pixel data
        let img = match &self.data {
            TextureData::RgbaU8(data) => DynamicImage::new_rgba8(self.width, self.height),
            _ => unimplemented!(),
        };
        let mut bytes: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;
        let mut raw_assets = Loaded::new();
        raw_assets.insert_bytes(path, bytes);
        Ok(raw_assets)
    }
}

impl Loaded {
    ///
    /// Deserialize the loaded image resource at the given path into a [Texture2D].
    ///
    pub fn image<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture2D> {
        self.deserialize(path)
    }
}
