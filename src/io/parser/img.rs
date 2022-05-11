use crate::{io::RawAssets, texture::*, Result};
use image::{io::Reader, *};
use std::io::Cursor;
use std::path::Path;

pub fn deserialize_img(bytes: &[u8]) -> Result<Texture2D> {
    let reader = Reader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("Cursor io never fails");
    #[cfg(feature = "hdr")]
    if reader.format() == Some(image::ImageFormat::Hdr) {
        use image::codecs::hdr::*;
        let decoder = HdrDecoder::new(&*bytes)?;
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
    Ok(Texture2D {
        data,
        width,
        height,
        ..Default::default()
    })
}

pub fn serialize_img(tex: &Texture2D, path: impl AsRef<Path>) -> Result<RawAssets> {
    // TODO: Put actual pixel data
    let img = match &tex.data {
        TextureData::RgbaU8(data) => DynamicImage::new_rgba8(tex.width, tex.height),
        _ => unimplemented!(),
    };
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;
    let mut raw_assets = RawAssets::new();
    raw_assets.insert(path, bytes);
    Ok(raw_assets)
}
