pub use crate::geometry::*;
pub use crate::material::*;

pub struct Model {
    pub geometries: Vec<TriMesh>,
    pub materials: Vec<PbrMaterial>,
}

impl Model {
    ///
    /// Returns the material for this mesh in the given list of materials. Returns `None` if no suitable material can be found.
    ///
    pub fn material(&self, name: &str) -> Option<&'_ PbrMaterial> {
        self.materials
            .iter()
            .position(|mat| &mat.name == name)
            .map(|index| &self.materials[index])
    }
}
