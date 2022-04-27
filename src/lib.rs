#![warn(missing_docs)]
//!
//! Contains functionality to load any type of asset runtime on both desktop and web as well as parsers for different image and 3D model formats.
//! Also includes functionality to save data which is limited to desktop.
//!

mod loader;
#[doc(inline)]
pub use loader::*;

mod parser;
#[doc(inline)]
pub use parser::*;

#[cfg(not(target_arch = "wasm32"))]
mod saver;
#[doc(inline)]
#[cfg(not(target_arch = "wasm32"))]
pub use saver::*;

/// A result for this crate.
pub type IOResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub use three_d::{
    Color, CpuMaterial, CpuMesh, CpuTexture, CpuTexture3D, CpuTextureCube, CpuVolume,
    GeometryFunction, Indices, LightingModel, NormalDistributionFunction, Positions,
    TextureCubeData, TextureData, Vector2, Vector3, Vector4,
};

use thiserror::Error;
///
/// Error from the [io](crate::io) module.
///
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum IOError {
    #[cfg(feature = "image")]
    #[error("error while parsing an image file")]
    Image(#[from] image::ImageError),
    #[cfg(all(feature = "obj", feature = "image"))]
    #[error("error while parsing an .obj file")]
    Obj(#[from] wavefront_obj::ParseError),
    #[cfg(all(feature = "gltf", feature = "image"))]
    #[error("error while parsing a .gltf file")]
    Gltf(#[from] ::gltf::Error),
    #[cfg(all(feature = "gltf", feature = "image"))]
    #[error("the .gltf file contain corrupt buffer data")]
    GltfCorruptData,
    #[cfg(all(feature = "gltf", feature = "image"))]
    #[error("the .gltf file contain missing buffer data")]
    GltfMissingData,
    #[error("the .vol file contain wrong data size")]
    VolCorruptData,
    #[cfg(not(target_arch = "wasm32"))]
    #[error("error while loading the file {0}: {1}")]
    FailedLoading(String, std::io::Error),
    #[cfg(feature = "reqwest")]
    #[error("error while loading the url {0}: {1}")]
    FailedLoadingUrl(String, reqwest::Error),
    #[cfg(not(feature = "reqwest"))]
    #[error("error while loading the url {0}: feature 'reqwest' not enabled")]
    FailedLoadingUrl(String),
    #[error("tried to use {0} which was not loaded")]
    NotLoaded(String),
}
