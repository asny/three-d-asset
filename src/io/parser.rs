#[cfg(feature = "obj")]
mod obj;

#[cfg(feature = "gltf")]
mod gltf;

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
mod img;
#[cfg(feature = "image")]
#[doc(inline)]
pub use img::*;

#[cfg(feature = "vol")]
mod vol;

use crate::io::{Deserialize, RawAssets};
use crate::{Error, Model, Result, Texture2D, VoxelGrid};
use std::path::Path;

impl RawAssets {
    pub fn gltf<P: AsRef<Path>>(&mut self, path: P) -> Result<Model> {
        self.deserialize(path)
    }

    pub fn obj<P: AsRef<Path>>(&mut self, path: P) -> Result<Model> {
        self.deserialize(path)
    }

    pub fn vol<P: AsRef<Path>>(&mut self, path: P) -> Result<VoxelGrid> {
        self.deserialize(path)
    }

    ///
    /// Deserialize the image resource at the given path into a [Texture2D].
    ///
    pub fn image<P: AsRef<Path>>(&mut self, path: P) -> Result<Texture2D> {
        self.deserialize(path)
    }
}

impl Deserialize for Model {
    fn deserialize(raw_assets: &mut RawAssets, path: impl AsRef<Path>) -> Result<Self> {
        match path
            .as_ref()
            .extension()
            .map(|e| e.to_str().unwrap())
            .unwrap_or("")
        {
            "gltf" => {
                #[cfg(feature = "gltf")]
                let result = gltf::deserialize(raw_assets, path);

                #[cfg(not(feature = "gltf"))]
                let result = Err(Error::FeatureMissing(
                    "gltf".to_string(),
                    path.as_ref().to_str().unwrap().to_string(),
                ));
                result
            }
            _ => Err(Error::FailedDeserialize(
                path.as_ref().to_str().unwrap().to_string(),
            )),
        }
    }
}

impl Deserialize for VoxelGrid {
    fn deserialize(raw_assets: &mut RawAssets, path: impl AsRef<Path>) -> Result<Self> {
        match path
            .as_ref()
            .extension()
            .map(|e| e.to_str().unwrap())
            .unwrap_or("")
        {
            "vol" => {
                #[cfg(feature = "vol")]
                let result = vol::deserialize(raw_assets, path);

                #[cfg(not(feature = "vol"))]
                let result = Err(Error::FeatureMissing(
                    "vol".to_string(),
                    path.as_ref().to_str().unwrap().to_string(),
                ));
                result
            }
            _ => Err(Error::FailedDeserialize(
                path.as_ref().to_str().unwrap().to_string(),
            )),
        }
    }
}
