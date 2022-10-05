//!
//! Contains functionality to load any type of asset runtime as well as parsers for common 3D assets.
//! Also includes functionality to save data which is limited to native.
//!
//!
//! A typical use-case is to load and deserialize assets:
//! ```
//! use three_d_asset::io::*;
//! use three_d_asset::{Texture2D, Model};
//!
//! let mut assets = load(&["test_data/test.png", "test_data/cube.obj"]).unwrap();
//! let texture: Texture2D = assets.deserialize("test.png").unwrap();
//! let model: Model = assets.deserialize("cube.obj").unwrap();
//! ```
//!
//! Or serialize and save assets:
//! ```
//! use three_d_asset::io::*;
//! use three_d_asset::{Texture2D, TextureData};
//!
//! let texture = Texture2D {
//!     data: TextureData::RgbaU8(vec![
//!         [0, 0, 0, 255],
//!         [255, 0, 0, 255],
//!         [0, 255, 0, 255],
//!         [0, 0, 255, 255],
//!     ]),
//!     width: 2,
//!     height: 2,
//!     ..Default::default()
//! };
//! let assets = texture.serialize("test_data/test.png").unwrap();
//! save(&assets).unwrap();
//! ```
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

#[cfg(feature = "pcd")]
mod pcd;

///
/// Loads and deserialize a single file. If the file depends on other files, those files are also loaded.
///
#[cfg(not(target_arch = "wasm32"))]
pub fn load_and_deserialize<T: Deserialize>(path: impl AsRef<std::path::Path>) -> crate::Result<T> {
    load(&[&path])?.deserialize(path)
}

///
/// Async loads and deserialize a single file. If the file depends on other files, those files are also loaded.
///
pub async fn load_and_deserialize_async<T: Deserialize>(
    path: impl AsRef<std::path::Path>,
) -> crate::Result<T> {
    load_async(&[&path]).await?.deserialize(path)
}

///
/// Save and serialize a single file.
///
#[cfg(not(target_arch = "wasm32"))]
pub fn serialize_and_save<T: Serialize>(
    path: impl AsRef<std::path::Path>,
    data: T,
) -> crate::Result<()> {
    save(&data.serialize(path)?)
}

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
use std::collections::HashSet;
use std::path::{Path, PathBuf};

impl Deserialize for crate::Texture2D {
    fn deserialize(path: impl AsRef<std::path::Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let path = raw_assets.match_path(path.as_ref())?;
        #[allow(unused_variables)]
        let bytes = raw_assets.get(&path)?;

        #[cfg(not(feature = "image"))]
        return Err(Error::FeatureMissing(
            path.extension()
                .map(|e| e.to_str().unwrap())
                .unwrap_or("image")
                .to_string(),
        ));

        #[cfg(feature = "image")]
        img::deserialize_img(path, bytes)
    }
}

impl Serialize for crate::Texture2D {
    fn serialize(&self, path: impl AsRef<Path>) -> Result<RawAssets> {
        let path = path.as_ref();

        #[cfg(not(feature = "image"))]
        return Err(Error::FeatureMissing(
            path.extension()
                .map(|e| e.to_str().unwrap())
                .unwrap_or("image")
                .to_string(),
        ));

        #[cfg(feature = "image")]
        img::serialize_img(self, path)
    }
}

impl Deserialize for crate::Model {
    fn deserialize(path: impl AsRef<Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let path = raw_assets.match_path(path.as_ref())?;
        match path.extension().map(|e| e.to_str().unwrap()).unwrap_or("") {
            "gltf" | "glb" => {
                #[cfg(not(feature = "gltf"))]
                return Err(Error::FeatureMissing("gltf".to_string()));

                #[cfg(feature = "gltf")]
                gltf::deserialize_gltf(raw_assets, &path)
            }
            "obj" => {
                #[cfg(not(feature = "obj"))]
                return Err(Error::FeatureMissing("obj".to_string()));

                #[cfg(feature = "obj")]
                obj::deserialize_obj(raw_assets, &path)
            }
            _ => Err(Error::FailedDeserialize(path.to_str().unwrap().to_string())),
        }
    }
}

impl Deserialize for crate::VoxelGrid {
    fn deserialize(path: impl AsRef<Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let path = raw_assets.match_path(path.as_ref())?;
        match path.extension().map(|e| e.to_str().unwrap()).unwrap_or("") {
            "vol" => {
                #[cfg(feature = "vol")]
                let result = vol::deserialize_vol(raw_assets, path);

                #[cfg(not(feature = "vol"))]
                let result = Err(Error::FeatureMissing("vol".to_string()));
                result
            }
            _ => Err(Error::FailedDeserialize(path.to_str().unwrap().to_string())),
        }
    }
}

impl Deserialize for crate::PointCloud {
    fn deserialize(path: impl AsRef<Path>, raw_assets: &mut RawAssets) -> Result<Self> {
        let path = raw_assets.match_path(path.as_ref())?;
        match path.extension().map(|e| e.to_str().unwrap()).unwrap_or("") {
            "pcd" => {
                #[cfg(feature = "pcd")]
                let result = pcd::deserialize_pcd(raw_assets, path);

                #[cfg(not(feature = "pcd"))]
                let result = Err(Error::FeatureMissing("pcd".to_string()));
                result
            }
            _ => Err(Error::FailedDeserialize(path.to_str().unwrap().to_string())),
        }
    }
}

fn get_dependencies(raw_assets: &RawAssets) -> Vec<PathBuf> {
    #[allow(unused_mut)]
    let mut dependencies = HashSet::new();
    for (path, _) in raw_assets.iter() {
        match path.extension().map(|e| e.to_str().unwrap()).unwrap_or("") {
            "gltf" | "glb" => {
                #[cfg(feature = "gltf")]
                dependencies.extend(gltf::dependencies(raw_assets, path));
            }
            "obj" => {
                #[cfg(feature = "obj")]
                dependencies.extend(obj::dependencies_obj(raw_assets, path));
            }
            "mtl" => {
                #[cfg(feature = "obj")]
                dependencies.extend(obj::dependencies_mtl(raw_assets, path));
            }
            _ => {}
        }
    }
    dependencies
        .into_iter()
        .filter(|d| !raw_assets.contains_key(d))
        .collect()
}
