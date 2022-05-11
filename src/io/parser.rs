#[cfg(feature = "obj")]
mod obj;

#[cfg(feature = "gltf")]
mod gltf;

#[cfg(feature = "image")]
mod img;

#[cfg(feature = "vol")]
mod vol;

use crate::io::{Deserialize, RawAssets, Serialize};
use crate::{Error, Model, Result, Texture2D, VoxelGrid};
use std::path::Path;

impl Deserialize for Texture2D {
    fn deserialize(path: impl AsRef<std::path::Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let bytes = raw_assets.remove(path)?;
        img::deserialize_img(&bytes)
    }
}

impl Serialize for Texture2D {
    fn serialize(&self, path: impl AsRef<Path>) -> Result<RawAssets> {
        img::serialize_img(self, path)
    }
}

impl Deserialize for Model {
    fn deserialize(path: impl AsRef<Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let path = raw_assets.match_path(path)?;
        match path.extension().map(|e| e.to_str().unwrap()).unwrap_or("") {
            "gltf" | "glb" => {
                #[cfg(feature = "gltf")]
                let result = gltf::deserialize_gltf(raw_assets, path);

                #[cfg(not(feature = "gltf"))]
                let result = Err(Error::FeatureMissing(
                    "gltf".to_string(),
                    path.to_str().unwrap().to_string(),
                ));
                result
            }
            "obj" => {
                #[cfg(feature = "obj")]
                let result = obj::deserialize_obj(raw_assets, path);

                #[cfg(not(feature = "obj"))]
                let result = Err(Error::FeatureMissing(
                    "obj".to_string(),
                    path.to_str().unwrap().to_string(),
                ));
                result
            }
            _ => Err(Error::FailedDeserialize(path.to_str().unwrap().to_string())),
        }
    }
}

impl Deserialize for VoxelGrid {
    fn deserialize(path: impl AsRef<Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let path = raw_assets.match_path(path)?;
        match path.extension().map(|e| e.to_str().unwrap()).unwrap_or("") {
            "vol" => {
                #[cfg(feature = "vol")]
                let result = vol::deserialize_vol(raw_assets, path);

                #[cfg(not(feature = "vol"))]
                let result = Err(Error::FeatureMissing(
                    "vol".to_string(),
                    path.to_str().unwrap().to_string(),
                ));
                result
            }
            _ => Err(Error::FailedDeserialize(path.to_str().unwrap().to_string())),
        }
    }
}

mod test {

    #[test]
    pub fn deserialize_obj() {
        let model: crate::Model = crate::io::RawAssets::new()
            .insert(
                "cube.obj",
                include_bytes!("../../test_data/cube.obj").to_vec(),
            )
            .deserialize("")
            .unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 0);
    }

    #[test]
    pub fn deserialize_gltf() {
        let model: crate::Model = crate::io::RawAssets::new()
            .insert(
                "Cube.gltf",
                include_bytes!("../../test_data/Cube.gltf").to_vec(),
            )
            .insert(
                "Cube.bin",
                include_bytes!("../../test_data/Cube.bin").to_vec(),
            )
            .insert(
                "Cube_BaseColor.png",
                include_bytes!("../../test_data/Cube_BaseColor.png").to_vec(),
            )
            .insert(
                "Cube_MetallicRoughness.png",
                include_bytes!("../../test_data/Cube_MetallicRoughness.png").to_vec(),
            )
            .deserialize("gltf")
            .unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 1);
    }

    #[test]
    pub fn deserialize_png() {
        let png = include_bytes!("../../test_data/test.png").to_vec();
        let tex: crate::Texture2D = crate::io::RawAssets::new()
            .insert("test.png", png)
            .deserialize("")
            .unwrap();
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
            panic!("Wrong texture data")
        }
        assert_eq!(tex.width, 2);
        assert_eq!(tex.height, 2);
    }

    #[test]
    pub fn serialize_png() {
        use crate::io::Serialize;
        let tex = crate::Texture2D {
            data: crate::TextureData::RgbaU8(vec![
                [0, 0, 0, 255],
                [255, 0, 0, 255],
                [0, 255, 0, 255],
                [0, 0, 255, 255],
            ]),
            width: 2,
            height: 2,
            ..Default::default()
        };
        let img = tex.serialize("test.png").unwrap();

        assert_eq!(
            include_bytes!("../../test_data/test.png"),
            img.get("test.png").unwrap()
        );
    }
}
