use crate::{io::RawAssets, texture::*, Error, Result};
use image::{io::Reader, *};
use std::io::Cursor;
use std::path::Path;

pub fn deserialize_img(path: impl AsRef<Path>, bytes: &[u8]) -> Result<Texture2D> {
    let name = path
        .as_ref()
        .to_str()
        .filter(|s| !s.starts_with("data:"))
        .unwrap_or("default")
        .to_owned();
    let mut reader = Reader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("Cursor io never fails");

    if reader.format().is_none() {
        reader.set_format(ImageFormat::from_path(path)?);
    }
    #[cfg(feature = "hdr")]
    if reader.format() == Some(image::ImageFormat::Hdr) {
        use image::codecs::hdr::*;
        let decoder = HdrDecoder::new(&*bytes)?;
        let metadata = decoder.metadata();
        let img = decoder.read_image_native()?;
        return Ok(Texture2D {
            name,
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
        DynamicImage::ImageLumaA8(img) => TextureData::RgU8(
            img.into_raw()
                .chunks(2)
                .map(|c| [c[0], c[1]])
                .collect::<Vec<_>>(),
        ),
        DynamicImage::ImageRgb8(img) => TextureData::RgbU8(
            img.into_raw()
                .chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect::<Vec<_>>(),
        ),
        DynamicImage::ImageRgba8(img) => TextureData::RgbaU8(
            img.into_raw()
                .chunks(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect::<Vec<_>>(),
        ),
        _ => unimplemented!(),
    };
    Ok(Texture2D {
        name,
        data,
        width,
        height,
        ..Default::default()
    })
}

pub fn serialize_img(tex: &Texture2D, path: &Path) -> Result<RawAssets> {
    #![allow(unreachable_code)]
    #![allow(unused_variables)]
    let format: image::ImageOutputFormat = match path.extension().unwrap().to_str().unwrap() {
        "png" => {
            #[cfg(not(feature = "png"))]
            return Err(Error::FeatureMissing("png".to_string()));
            #[cfg(feature = "png")]
            image::ImageOutputFormat::Png
        }
        "jpeg" | "jpg" => {
            #[cfg(not(feature = "jpeg"))]
            return Err(Error::FeatureMissing("jpeg".to_string()));
            #[cfg(feature = "jpeg")]
            image::ImageOutputFormat::Jpeg(100)
        }
        "bmp" => {
            #[cfg(not(feature = "bmp"))]
            return Err(Error::FeatureMissing("bmp".to_string()));
            #[cfg(feature = "bmp")]
            image::ImageOutputFormat::Bmp
        }
        "tga" => {
            #[cfg(not(feature = "tga"))]
            return Err(Error::FeatureMissing("tga".to_string()));
            #[cfg(feature = "tga")]
            image::ImageOutputFormat::Tga
        }
        "tiff" | "tif" => {
            #[cfg(not(feature = "tiff"))]
            return Err(Error::FeatureMissing("tiff".to_string()));
            #[cfg(feature = "tiff")]
            image::ImageOutputFormat::Tiff
        }
        "gif" => {
            #[cfg(not(feature = "gif"))]
            return Err(Error::FeatureMissing("gif".to_string()));
            #[cfg(feature = "gif")]
            image::ImageOutputFormat::Gif
        }
        "webp" => {
            #[cfg(not(feature = "webp"))]
            return Err(Error::FeatureMissing("webp".to_string()));
            #[cfg(feature = "webp")]
            image::ImageOutputFormat::WebP
        }
        _ => return Err(Error::FailedSerialize(path.to_str().unwrap().to_string())),
    };
    let img = match &tex.data {
        TextureData::RU8(data) => DynamicImage::ImageLuma8(
            ImageBuffer::from_raw(tex.width, tex.height, data.clone()).unwrap(),
        ),
        TextureData::RgU8(data) => DynamicImage::ImageLumaA8(
            ImageBuffer::from_raw(
                tex.width,
                tex.height,
                data.iter().flat_map(|v| *v).collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        TextureData::RgbU8(data) => DynamicImage::ImageRgb8(
            ImageBuffer::from_raw(
                tex.width,
                tex.height,
                data.iter().flat_map(|v| *v).collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        TextureData::RgbaU8(data) => DynamicImage::ImageRgba8(
            ImageBuffer::from_raw(
                tex.width,
                tex.height,
                data.iter().flat_map(|v| *v).collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        _ => unimplemented!(),
    };
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), format)?;
    let mut raw_assets = RawAssets::new();
    raw_assets.insert(path, bytes);
    Ok(raw_assets)
}

#[cfg(test)]
mod test {
    fn tex() -> crate::Texture2D {
        crate::Texture2D {
            data: crate::TextureData::RgbaU8(vec![
                [0, 0, 0, 255],
                [255, 0, 0, 255],
                [0, 255, 0, 255],
                [0, 0, 255, 255],
            ]),
            width: 2,
            height: 2,
            ..Default::default()
        }
    }

    fn test_deserialize(format: &str) {
        let path = format!("test_data/test.{}", format);
        let tex: crate::Texture2D = crate::io::load_and_deserialize(&path).unwrap();

        if format == "jpeg" || format == "jpg" {
            if let crate::TextureData::RgbU8(data) = tex.data {
                assert_eq!(data, vec![[4, 0, 0], [250, 0, 1], [0, 254, 1], [1, 2, 253]]);
            } else {
                panic!("Wrong texture data: {:?}", tex.data)
            }
        } else {
            if let crate::TextureData::RgbaU8(data) = tex.data {
                assert_eq!(
                    data,
                    vec![
                        [0, 0, 0, 255],
                        [255, 0, 0, 255],
                        [0, 255, 0, 255],
                        [0, 0, 255, 255],
                    ]
                );
            } else {
                panic!("Wrong texture data: {:?}", tex.data)
            }
        }
        assert_eq!(tex.width, 2);
        assert_eq!(tex.height, 2);
    }

    fn test_serialize(format: &str) {
        let path = format!("test_data/test.{}", format);
        use crate::io::Serialize;
        let mut img = tex().serialize(&path).unwrap();
        img.save().unwrap();

        assert_eq!(
            crate::io::load(&[path]).unwrap().get("").unwrap(),
            img.get("").unwrap()
        );
    }

    #[cfg(feature = "png")]
    #[test]
    pub fn png() {
        test_serialize("png");
        test_deserialize("png");
    }

    #[cfg(feature = "jpeg")]
    #[test]
    pub fn jpeg() {
        test_serialize("jpeg");
        test_deserialize("jpeg");
        test_serialize("jpg");
        test_deserialize("jpg");
    }

    #[cfg(feature = "gif")]
    #[test]
    pub fn gif() {
        test_serialize("gif");
        test_deserialize("gif");
    }

    #[cfg(feature = "tga")]
    #[test]
    pub fn tga() {
        test_serialize("tga");
        test_deserialize("tga");
    }

    #[cfg(feature = "tiff")]
    #[test]
    pub fn tiff() {
        test_serialize("tiff");
        test_deserialize("tiff");
        test_serialize("tif");
        test_deserialize("tif");
    }

    #[cfg(feature = "bmp")]
    #[test]
    pub fn bmp() {
        test_serialize("bmp");
        test_deserialize("bmp");
    }

    #[cfg(feature = "hdr")]
    #[test]
    pub fn hdr() {
        let tex: crate::Texture2D = crate::io::load_and_deserialize("test_data/test.hdr").unwrap();
        if let crate::TextureData::RgbF32(data) = tex.data {
            assert_eq!(data[0], [0.16503906, 0.24609375, 0.20019531]);
        } else {
            panic!("Wrong texture data")
        }
        assert_eq!(tex.width, 1024);
        assert_eq!(tex.height, 512);
    }

    #[cfg(feature = "webp")]
    #[test]
    pub fn webp() {
        test_serialize("webp");
        test_deserialize("webp");
    }
}
