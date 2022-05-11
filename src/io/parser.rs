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

use crate::io::{Deserialize, Loaded};
use crate::{Model, PbrMaterial, Result, TriMesh};
use std::path::Path;

impl Loaded {
    pub fn gltf<P: AsRef<Path>>(&mut self, path: P) -> Result<Model> {
        self.deserialize(path)
    }
}

impl Deserialize for Model {
    fn deserialize(raw_assets: &mut Loaded, path: impl AsRef<Path>) -> Result<Self> {
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
