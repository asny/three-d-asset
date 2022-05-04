#![warn(missing_docs)]
//!
//! Contains functionality to load any type of asset runtime as well as parsers for different image and 3D model formats.
//! The parsers will output into data types defined in the [three-d-data-types](https://github.com/asny/three-d-data-types) crate.
//! Also includes functionality to save data which is limited to native.
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
pub type Result<T> = std::result::Result<T, Error>;

use thiserror::Error;
///
/// Error from this crate.
///
#[derive(Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[cfg(feature = "image")]
    #[error("error while parsing an image file")]
    Image(#[from] image::ImageError),
    #[cfg(all(feature = "obj", feature = "image"))]
    #[error("error while parsing an .obj file")]
    Obj(#[from] wavefront_obj::ParseError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("io error")]
    IO(#[from] std::io::Error),
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
    #[cfg(feature = "reqwest")]
    #[error("error while parsing the url {0}")]
    FailedParsingUrl(String),
    #[cfg(not(feature = "reqwest"))]
    #[error("error while loading the url {0}: feature 'reqwest' not enabled")]
    FailedLoadingUrl(String),
    #[error("tried to use {0} which was not loaded")]
    NotLoaded(String),
    #[error("three-d-data-types error")]
    ThreeDDataTypes(#[from] ::three_d_data_types::Error),
}
