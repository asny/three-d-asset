//!
//! Contains functionality to load any type of asset runtime as well as parsers for common 3D assets.
//! Also includes functionality to save data which is limited to native.
//!

mod loader;
pub use loader::*;

mod raw_assets;
pub use raw_assets::*;

#[cfg(not(target_arch = "wasm32"))]
mod saver;
#[cfg(not(target_arch = "wasm32"))]
pub use saver::*;

#[cfg(feature = "obj")]
mod obj;

#[cfg(feature = "gltf")]
mod gltf;

#[cfg(feature = "image")]
mod img;

#[cfg(feature = "vol")]
mod vol;

///
/// Implemented for assets that can be deserialized after being loaded (see also [load] and [RawAssets::deserialize]).
///
pub trait Deserialize: Sized {
    ///
    /// See [RawAssets::deserialize].
    ///
    fn deserialize(
        path: impl AsRef<std::path::Path>,
        raw_assets: &mut RawAssets,
    ) -> crate::Result<Self>;
}

///
/// Implemented for assets that can be serialized before being saved (see also [save]).
///
pub trait Serialize: Sized {
    ///
    /// Serialize the asset into a list of raw assets which consist of byte arrays and related path to where they should be saved (see also [save]).
    /// The path given as input is the path to the main raw asset.
    ///
    fn serialize(&self, path: impl AsRef<std::path::Path>) -> crate::Result<RawAssets>;
}

use crate::{Error, Result};
use std::path::Path;

impl Deserialize for crate::Texture2D {
    fn deserialize(path: impl AsRef<std::path::Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let bytes = raw_assets.get(path)?;
        img::deserialize_img(bytes)
    }
}

impl Serialize for crate::Texture2D {
    fn serialize(&self, path: impl AsRef<Path>) -> Result<RawAssets> {
        img::serialize_img(self, path)
    }
}

impl Deserialize for crate::Models {
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

impl Deserialize for crate::VoxelGrid {
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
