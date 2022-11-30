#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(clippy::all)]
#![warn(missing_docs)]

//!
//! A set of common assets that are useful when doing graphics, for example [TriMesh], [Texture2D] or [PbrMaterial].
//! These assets can be loaded using the [io] module or constructed manually.
//! When in memory, the assets can be for example be
//! - visualised, for example using the [three-d](https://github.com/asny/three-d) crate or in a CPU ray tracer
//! - imported into a rust-based game engine
//! - edited and saved again
//!

pub mod prelude;

mod camera;
pub use camera::*;

pub mod texture;
pub use texture::*;

pub mod material;
pub use material::*;

pub mod geometry;
pub use geometry::*;

pub mod volume;
pub use volume::*;

pub mod animation;
pub use animation::*;

///
/// Model consisting of a set of [geometries](Model::geometries) and [materials](Model::materials).
///
#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub parts: Vec<Part>,
    pub materials: Vec<PbrMaterial>,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub name: String,
    pub transformation: Mat4,
    pub animations: Vec<(Mat4, Vec<KeyFrames>)>,
    pub geometry: TriMesh,
    pub material_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub name: String,
    pub children: Vec<Node>,
    pub materials: Vec<PbrMaterial>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub transformation: Mat4,
    pub key_frames: Vec<KeyFrames>,
    pub children: Vec<Node>,
    pub primitives: Vec<(TriMesh, Option<usize>)>,
}

impl std::convert::From<Scene> for Model {
    fn from(scene: Scene) -> Self {
        let mut parts = Vec::new();
        for child in scene.children {
            visit(child, Vec::new(), &mut parts);
        }
        Self {
            name: scene.name,
            materials: scene.materials,
            parts,
        }
    }
}

fn visit(mut node: Node, mut animations: Vec<(Mat4, Vec<KeyFrames>)>, parts: &mut Vec<Part>) {
    animations.push((node.transformation, node.key_frames));
    parts.extend(node.primitives.drain(..).map(|p| {
        let mut animations = animations.clone();
        let transformation = if animations
            .last()
            .and_then(|a| Some(a.1.is_empty()))
            .unwrap_or(false)
        {
            animations.pop().unwrap().0
        } else {
            Mat4::identity()
        };
        animations.reverse();
        Part {
            name: node.name.clone(),
            transformation,
            animations,
            geometry: p.0,
            material_index: p.1,
        }
    }));
    for child in node.children {
        visit(child, animations.clone(), parts);
    }
}

pub mod io;

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
    #[error("the number of indices must be divisable by 3, actual count is {0}")]
    InvalidNumberOfIndices(usize),
    #[error("the max index {0} must be less than the number of vertices {1}")]
    InvalidIndices(usize, usize),
    #[error("the transformation matrix cannot be inverted and is therefore invalid")]
    FailedInvertingTransformationMatrix,
    #[cfg(feature = "image")]
    #[error("error while parsing an image file")]
    Image(#[from] image::ImageError),
    #[cfg(feature = "obj")]
    #[error("error while parsing an .obj file")]
    Obj(#[from] wavefront_obj::ParseError),

    #[cfg(feature = "pcd")]
    #[error("error while parsing an .pcd file")]
    Pcd(#[from] pcd_rs::anyhow::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("io error")]
    IO(#[from] std::io::Error),
    #[cfg(feature = "gltf")]
    #[error("error while parsing a .gltf file")]
    Gltf(#[from] ::gltf::Error),
    #[cfg(feature = "gltf")]
    #[error("the .gltf file contain corrupt buffer data")]
    GltfCorruptData,
    #[cfg(feature = "gltf")]
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
    #[cfg(feature = "data-url")]
    #[error("error while parsing data-url {0}: {1}")]
    FailedParsingDataUrl(String, String),
    #[error("tried to use {0} which was not loaded or otherwise added to the raw assets")]
    NotLoaded(String),
    #[error("the feature {0} is needed")]
    FeatureMissing(String),
    #[error("failed to deserialize the file {0}")]
    FailedDeserialize(String),
    #[error("failed to serialize the file {0}")]
    FailedSerialize(String),
}
