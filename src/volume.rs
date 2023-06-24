//!
//! Contain volume asset definitions.
//!
pub use crate::prelude::*;
#[doc(inline)]
pub use crate::texture::texture3d::*;

pub use serde::{Deserialize, Serialize};

///
/// Volume data consisting of voxel data inside a cube.
///
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoxelGrid {
    /// Name.
    pub name: String,

    /// Voxel data, ie. small cubes in 3D (analogue to pixels in 2D) that contain 1-4 values.
    pub voxels: Texture3D,

    /// The size of the cube that is spanned by the voxel data.
    pub size: Vec3,
}

impl std::default::Default for VoxelGrid {
    fn default() -> Self {
        Self {
            name: String::default(),
            voxels: Texture3D::default(),
            size: Vec3::new(2.0, 2.0, 2.0),
        }
    }
}
