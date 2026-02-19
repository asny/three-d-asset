use crate::{io::RawAssets, texture::*, Error, Result};
use image::*;
use std::io::Cursor;
use std::path::Path;

pub fn deserialize_img(path: impl AsRef<Path>, bytes: &[u8]) -> Result<Texture2D> {
    let name = path
        .as_ref()
        .to_str()
        .filter(|s| !s.starts_with("data:"))
        .unwrap_or("default")
        .to_owned();
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("Cursor io never fails");

    if reader.format().is_none() {
        reader.set_format(ImageFormat::from_path(path)?);
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
        DynamicImage::ImageRgb32F(img) => TextureData::RgbF32(
            img.into_raw()
                .chunks(3)
                .map(|c| [c[0], c[1], c[2]])
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

#[cfg(feature = "svg")]
pub fn deserialize_svg(path: impl AsRef<Path>, bytes: &[u8]) -> Result<Texture2D> {
    use cgmath::num_traits::ToPrimitive;

    let name = path
        .as_ref()
        .to_str()
        .filter(|s| !s.starts_with("data:"))
        .unwrap_or("default")
        .to_owned();
    let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default())?;
    // TODO: should we have more error checking here?
    let (width, height) = (
        tree.size().width().to_u32().unwrap(),
        tree.size().height().to_u32().unwrap(),
    );
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    // process the data to our desired RGBAU8 format
    let texture_data: Vec<[u8; 4]> = pixmap
        .pixels()
        .iter()
        .map(|pixel| [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()])
        .collect();

    Ok(Texture2D {
        name,
        data: TextureData::RgbaU8(texture_data),
        width,
        height,
        ..Default::default()
    })
}

pub fn serialize_img(tex: &Texture2D, path: &Path) -> Result<RawAssets> {
    #![allow(unreachable_code)]
    #![allow(unused_variables)]
    let format: ImageFormat = match path.extension().unwrap().to_str().unwrap() {
        "png" => {
            #[cfg(not(feature = "png"))]
            return Err(Error::FeatureMissing("png".to_string()));
            #[cfg(feature = "png")]
            ImageFormat::Png
        }
        "jpeg" | "jpg" => {
            #[cfg(not(feature = "jpeg"))]
            return Err(Error::FeatureMissing("jpeg".to_string()));
            #[cfg(feature = "jpeg")]
            ImageFormat::Jpeg
        }
        "bmp" => {
            #[cfg(not(feature = "bmp"))]
            return Err(Error::FeatureMissing("bmp".to_string()));
            #[cfg(feature = "bmp")]
            ImageFormat::Bmp
        }
        "tga" => {
            #[cfg(not(feature = "tga"))]
            return Err(Error::FeatureMissing("tga".to_string()));
            #[cfg(feature = "tga")]
            ImageFormat::Tga
        }
        "tiff" | "tif" => {
            #[cfg(not(feature = "tiff"))]
            return Err(Error::FeatureMissing("tiff".to_string()));
            #[cfg(feature = "tiff")]
            ImageFormat::Tiff
        }
        "gif" => {
            #[cfg(not(feature = "gif"))]
            return Err(Error::FeatureMissing("gif".to_string()));
            #[cfg(feature = "gif")]
            ImageFormat::Gif
        }
        "webp" => {
            #[cfg(not(feature = "webp"))]
            return Err(Error::FeatureMissing("webp".to_string()));
            #[cfg(feature = "webp")]
            ImageFormat::WebP
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
        TextureData::RgbaU8(data) => {
            if format == ImageFormat::Jpeg {
                DynamicImage::ImageRgb8(
                    ImageBuffer::from_raw(
                        tex.width,
                        tex.height,
                        data.iter()
                            .flat_map(|v| [v[0], v[1], v[2]])
                            .collect::<Vec<_>>(),
                    )
                    .unwrap(),
                )
            } else {
                DynamicImage::ImageRgba8(
                    ImageBuffer::from_raw(
                        tex.width,
                        tex.height,
                        data.iter().flat_map(|v| *v).collect::<Vec<_>>(),
                    )
                    .unwrap(),
                )
            }
        }
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
    use cgmath::AbsDiffEq;

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
                // Jpeg is not lossless
                assert!(data
                    .iter()
                    .zip(vec![[48, 0, 17], [227, 0, 14], [0, 244, 0], [16, 36, 253]].iter())
                    .all(|(data_pixel, test_pixel)| data_pixel.abs_diff_eq(test_pixel, 2)));
            } else {
                panic!("Wrong texture data: {:?}", tex.data)
            }
        } else if let crate::TextureData::RgbaU8(data) = tex.data {
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

    #[cfg(feature = "svg")]
    #[test]
    pub fn svg() {
        let tex: crate::Texture2D = crate::io::load_and_deserialize("test_data/test.svg").unwrap();
        if let crate::TextureData::RgbaU8(data) = tex.data {
            assert_eq!(data[0], [0, 0, 0, 0]);
            assert_eq!(data[25036], [0, 51, 255, 255]);
            assert_eq!(data[20062], [255, 0, 0, 255]);
            assert_eq!(data[58095], [0, 255, 0, 255]);
        } else {
            panic!("Wrong texture data");
        }

        assert_eq!(tex.width, 320);
        assert_eq!(tex.height, 240);
    }
}
