pub use crate::geometry::*;
pub use crate::material::*;

use crate::{PbrMaterial, TriMesh};

pub struct Model {
    pub geometries: Vec<TriMesh>,
    pub materials: Vec<PbrMaterial>,
}
