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
