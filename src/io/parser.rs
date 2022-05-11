#[cfg(feature = "obj")]
#[cfg_attr(docsrs, doc(cfg(feature = "obj")))]
mod obj;
#[doc(inline)]
#[cfg(feature = "obj")]
pub use obj::*;

#[cfg(feature = "gltf")]
mod gltf;

#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
mod img;
#[cfg(feature = "image")]
#[doc(inline)]
pub use img::*;

#[cfg(feature = "vol")]
#[cfg_attr(docsrs, doc(cfg(feature = "vol")))]
mod vol;
#[cfg(feature = "vol")]
#[doc(inline)]
pub use vol::*;

use crate::io::{Deserialize, RawAssets};
use crate::{Model, PbrMaterial, Result, Texture2D, TriMesh};
use std::path::Path;

impl RawAssets {
    pub fn gltf<P: AsRef<Path>>(&mut self, path: P) -> Result<Model> {
        self.deserialize(path)
    }

    pub fn obj<P: AsRef<Path>>(&mut self, path: P) -> Result<Model> {
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
        let geometries: Vec<TriMesh> = Vec::new();
        let materials: Vec<PbrMaterial> = Vec::new();
        #[cfg(feature = "gltf")]
        let (geometries, materials) = gltf::gltf(raw_assets, path)?;
        Ok(Model {
            geometries,
            materials,
        })
    }
}
