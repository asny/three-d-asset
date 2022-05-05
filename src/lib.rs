#![warn(missing_docs)]
//!
//! Contains functionality to load any type of asset runtime as well as parsers for different image and 3D model formats.
//! The parsers will output into data types defined in the [three-d-data-types](https://github.com/asny/three-d-data-types) crate.
//! Also includes functionality to save data which is limited to native.
//!

pub mod math;
pub use math::*;

mod aabb;
pub use aabb::*;

mod color;
pub use color::*;

///
/// Contain the basic types used by the 3D specific data types.
///
pub mod prelude {
    pub use crate::aabb::*;
    pub use crate::color::*;
    #[doc(inline)]
    pub use crate::math::*;
}

pub mod texture;
pub use texture::*;

pub mod material;
pub use material::*;

pub mod surface;
pub use surface::*;

pub mod volume;
pub use volume::*;

/// A result for this crate.
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
    #[error("{0} buffer length must be {1}, actual length is {2}")]
    InvalidBufferLength(String, usize, usize),
    #[error("mesh must have both normals and uv coordinates to be able to compute tangents")]
    FailedComputingTangents,
    #[error("the number of vertices must be divisable by 3, actual count is {0}")]
    InvalidNumberOfVertices(usize),
    #[error("the transformation matrix cannot be inverted and is therefore invalid")]
    FailedInvertingTransformationMatrix,
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
}
