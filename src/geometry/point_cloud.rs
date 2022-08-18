use super::Positions;
use crate::prelude::*;

#[derive(Default)]
pub struct PointCloud {
    /// Name.
    pub name: String,
    /// The positions of the points.
    pub positions: Positions,
    /// The colors of the points.
    pub colors: Option<Vec<Color>>,
}

impl std::fmt::Debug for PointCloud {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("PointCloud");
        d.field("name", &self.name);
        d.field("positions", &self.positions.len());
        d.field("colors", &self.colors.as_ref().map(|v| v.len()));
        d.finish()
    }
}

impl PointCloud {
    ///
    /// Returns a point cloud whose points lie on the corners of an axis aligned unconnected cube with positions in the range `[-1..1]` in all axes.
    ///
    pub fn cube() -> Self {
        let positions = vec![
            vec3(-1.0, -1.0, -1.0),
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, -1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, -1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, -1.0),
            vec3(1.0, 1.0, 1.0),
        ];
        Self {
            positions: Positions::F32(positions),
            ..Default::default()
        }
    }
}
