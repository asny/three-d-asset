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

mod animation;
pub use animation::*;

///
/// Representation of a set of objects as a scene graph.
/// Specifically, a [Scene] contains a tree of [Node]s, where the nodes contain the [Geometry] data.
/// A [Scene] can easily be converted into a [Model], if it is more desirable with a flat arrays instead of a tree structure.
///
/// To visualise the [Geometry] in the [Scene] correctly, it is necessary to traverse the scene from the root (the [Scene]) to the leaves
/// and along the way calculate a transformation.
/// For each node containing [Geometry], the [Geometry] should be visualised with the calculated transformation applied.
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Scene {
    /// The name. Might not be anything meaningful.
    pub name: String,
    /// Children nodes.
    pub children: Vec<Node>,
    /// A list of materials used in this scene. The materials are referenced by index in the relevant nodes.
    pub materials: Vec<PbrMaterial>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: "scene".to_owned(),
            children: Vec::new(),
            materials: Vec::new(),
        }
    }
}

///
/// A node in a [Scene] graph. Each node may contain a set of children nodes, hence the whole [Scene] representaion has a tree structure.
///
/// Each node may also contain a transformation, animations, geometry and an index to the [Scene::materials].
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
    /// The name. Might not be anything meaningful.
    pub name: String,
    /// Children [Node]s.
    pub children: Vec<Node>,
    /// A transformation that should be applied to all [Geometry] referenced by this and all children nodes.
    pub transformation: Mat4,
    /// Optional animation applied to this node and all of its children.
    /// A transformation should be computed for a specific time and then multiplied together with [Node::transformation].
    pub animations: Vec<(Option<String>, KeyFrames)>,
    /// Optional geometry for this node.
    pub geometry: Option<Geometry>,
    /// Optional index into [Scene::materials], indicating which material should be applied to geometry below this node in the tree.
    pub material_index: Option<usize>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            name: "node".to_owned(),
            children: Vec::new(),
            transformation: Mat4::identity(),
            animations: Vec::new(),
            geometry: None,
            material_index: None,
        }
    }
}

///
/// A [Model] contain the same data as a [Scene], it's just stored in flat arrays instead of in a tree structure.
/// You can convert from a [Scene] to a [Model], but not the other way, because the tree structure is lost in the conversion.
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Model {
    /// The name. Might not be anything meaningful.
    pub name: String,
    /// A list of geometries for this model.
    pub geometries: Vec<Primitive>,
    /// A list of materials for this model
    pub materials: Vec<PbrMaterial>,
}

///
/// A part of a [Model] containing exactly one [Geometry], an optional reference to a material and information necessary to calculate the transformation that
/// should be applied to the geometry.
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Primitive {
    /// The name. Might not be anything meaningful.
    pub name: String,
    /// A transformation that should be applied to the [Primitive::geometry].
    pub transformation: Mat4,
    /// Optional animation applied to the [Primitive::geometry].
    /// A transformation should be computed for a specific time and then multiplied together with [Node::transformation].
    pub animations: Vec<KeyFrameAnimation>,
    /// The geometry of this primitive.
    pub geometry: Geometry,
    /// Optional index into [Model::materials], indicating which material should be applied to [Primitive::geometry].
    pub material_index: Option<usize>,
}

impl std::ops::Deref for Primitive {
    type Target = Geometry;
    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl std::ops::DerefMut for Primitive {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geometry
    }
}

impl std::convert::From<Scene> for Model {
    fn from(scene: Scene) -> Self {
        let mut geometries = Vec::new();
        for child in scene.children {
            visit(child, Vec::new(), Mat4::identity(), &mut geometries);
        }
        Self {
            name: scene.name,
            materials: scene.materials,
            geometries,
        }
    }
}

fn visit(
    node: Node,
    mut animations: Vec<KeyFrameAnimation>,
    transformation: Mat4,
    geometries: &mut Vec<Primitive>,
) {
    let mut transformation = transformation * node.transformation;
    if !node.animations.is_empty() {
        for (animation_name, key_frames) in node.animations {
            if let Some(i) = animations.iter().position(|a| a.name == animation_name) {
                animations[i]
                    .key_frames
                    .push((transformation, std::sync::Arc::new(key_frames)));
            } else {
                animations.push(KeyFrameAnimation {
                    name: animation_name,
                    key_frames: vec![(transformation, std::sync::Arc::new(key_frames))],
                });
            }
        }
        transformation = Mat4::identity();
    };
    if let Some(geometry) = node.geometry {
        geometries.push(Primitive {
            name: node.name.clone(),
            transformation,
            animations: animations.clone(),
            geometry,
            material_index: node.material_index,
        });
    }
    for child in node.children {
        visit(child, animations.clone(), transformation, geometries);
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

    #[cfg(feature = "svg")]
    #[error("error while parsing svg file")]
    Svg(#[from] resvg::usvg::Error),

    #[cfg(feature = "obj")]
    #[error("error while parsing an .obj file")]
    Obj(#[from] wavefront_obj::ParseError),

    #[cfg(feature = "3mf")]
    #[error("error while parsing a .3mf file")]
    ThreeMf(#[from] lib3mf::Error),

    #[cfg(feature = "pcd")]
    #[error("error while parsing an .pcd file")]
    Pcd(#[from] pcd_rs::Error),

    #[cfg(any(not(target_arch = "wasm32"), feature = "stl"))]
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
    FailedLoadingUrlWithReqwest(String, reqwest::Error),
    #[cfg(feature = "reqwest")]
    #[error("error while loading the url {0}: {1}")]
    FailedLoadingUrl(String, String),
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
    #[error("failed to find {0} in the file {1}")]
    FailedConvertion(String, String),
}
