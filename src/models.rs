pub use crate::geometry::*;
pub use crate::material::*;

pub struct Models {
    pub geometries: Vec<TriMesh>,
    pub materials: Vec<PbrMaterial>,
}
