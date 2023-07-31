use crate::{prelude::*, Error, Indices, Positions, Result};

///
/// A CPU-side version of a triangle mesh.
///
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TriMesh {
    /// The positions of the vertices.
    /// If there is no indices associated with this mesh, three contiguous positions defines a triangle, in that case, the length must be divisable by 3.
    pub positions: Positions,
    /// The indices into the positions, normals, uvs and colors arrays which defines the three vertices of a triangle. Three contiguous indices defines a triangle, therefore the length must be divisable by 3.
    pub indices: Indices,
    /// The normals of the vertices.
    pub normals: Option<Vec<Vec3>>,
    /// The tangents of the vertices, orthogonal direction to the normal.
    /// The fourth value specifies the handedness (either -1.0 or 1.0).
    pub tangents: Option<Vec<Vec4>>,
    /// The uv coordinates of the vertices.
    pub uvs: Option<Vec<Vec2>>,
    /// The colors of the vertices.
    pub colors: Option<Vec<Srgba>>,
}

impl std::default::Default for TriMesh {
    fn default() -> Self {
        Self {
            positions: Positions::default(),
            indices: Indices::None,
            normals: None,
            tangents: None,
            uvs: None,
            colors: None,
        }
    }
}

impl std::fmt::Debug for TriMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Mesh");
        d.field("positions", &self.positions.len());
        d.field("indices", &self.indices);
        d.field("normals", &self.normals.as_ref().map(|v| v.len()));
        d.field("tangents", &self.tangents.as_ref().map(|v| v.len()));
        d.field("uvs", &self.uvs.as_ref().map(|v| v.len()));
        d.field("colors", &self.colors.as_ref().map(|v| v.len()));
        d.finish()
    }
}

impl TriMesh {
    /// Returns the number of vertices in this mesh.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the number of triangles in this mesh.
    pub fn triangle_count(&self) -> usize {
        self.indices
            .len()
            .map(|i| i / 3)
            .unwrap_or(self.positions.len() / 3)
    }

    ///
    /// Transforms the mesh by the given transformation.
    ///
    pub fn transform(&mut self, transform: &Mat4) -> Result<()> {
        match self.positions {
            Positions::F32(ref mut positions) => {
                for pos in positions.iter_mut() {
                    *pos = (transform * pos.extend(1.0)).truncate();
                }
            }
            Positions::F64(ref mut positions) => {
                let t = transform.cast::<f64>().unwrap();
                for pos in positions.iter_mut() {
                    *pos = (t * pos.extend(1.0)).truncate();
                }
            }
        };

        if self.normals.is_some() || self.tangents.is_some() {
            let normal_transform = transform
                .invert()
                .ok_or(Error::FailedInvertingTransformationMatrix)?
                .transpose();

            if let Some(ref mut normals) = self.normals {
                for n in normals.iter_mut() {
                    *n = (normal_transform * n.extend(1.0)).truncate();
                }
            }
            if let Some(ref mut tangents) = self.tangents {
                for t in tangents.iter_mut() {
                    *t = (normal_transform * t.truncate().extend(1.0))
                        .truncate()
                        .extend(t.w);
                }
            }
        }
        Ok(())
    }

    ///
    /// Returns a square mesh spanning the xy-plane with positions in the range `[-1..1]` in the x and y axes.
    ///
    pub fn square() -> Self {
        let indices = vec![0u8, 1, 2, 2, 3, 0];
        let halfsize = 1.0;
        let positions = vec![
            Vec3::new(-halfsize, -halfsize, 0.0),
            Vec3::new(halfsize, -halfsize, 0.0),
            Vec3::new(halfsize, halfsize, 0.0),
            Vec3::new(-halfsize, halfsize, 0.0),
        ];
        let normals = vec![
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let tangents = vec![
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
        ];
        let uvs = vec![
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];
        TriMesh {
            indices: Indices::U8(indices),
            positions: Positions::F32(positions),
            normals: Some(normals),
            tangents: Some(tangents),
            uvs: Some(uvs),
            ..Default::default()
        }
    }

    ///
    /// Returns a circle mesh spanning the xy-plane with radius 1 and center in `(0, 0, 0)`.
    ///
    pub fn circle(angle_subdivisions: u32) -> Self {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            positions.push(Vec3::new(angle.cos(), angle.sin(), 0.0));
            normals.push(Vec3::new(0.0, 0.0, 1.0));
        }

        for j in 0..angle_subdivisions {
            indices.push(0);
            indices.push(j as u16);
            indices.push(((j + 1) % angle_subdivisions) as u16);
        }
        TriMesh {
            indices: Indices::U16(indices),
            positions: Positions::F32(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }

    ///
    /// Returns a sphere mesh with radius 1 and center in `(0, 0, 0)`.
    ///
    pub fn sphere(angle_subdivisions: u32) -> Self {
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();

        positions.push(Vec3::new(0.0, 0.0, 1.0));
        normals.push(Vec3::new(0.0, 0.0, 1.0));

        for j in 0..angle_subdivisions * 2 {
            let j1 = (j + 1) % (angle_subdivisions * 2);
            indices.push(0);
            indices.push((1 + j) as u16);
            indices.push((1 + j1) as u16);
        }

        for i in 0..angle_subdivisions - 1 {
            let theta = std::f32::consts::PI * (i + 1) as f32 / angle_subdivisions as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let i0 = 1 + i * angle_subdivisions * 2;
            let i1 = 1 + (i + 1) * angle_subdivisions * 2;

            for j in 0..angle_subdivisions * 2 {
                let phi = std::f32::consts::PI * j as f32 / angle_subdivisions as f32;
                let x = sin_theta * phi.cos();
                let y = sin_theta * phi.sin();
                let z = cos_theta;
                positions.push(Vec3::new(x, y, z));
                normals.push(Vec3::new(x, y, z));

                if i != angle_subdivisions - 2 {
                    let j1 = (j + 1) % (angle_subdivisions * 2);
                    indices.push((i0 + j) as u16);
                    indices.push((i1 + j1) as u16);
                    indices.push((i0 + j1) as u16);
                    indices.push((i1 + j1) as u16);
                    indices.push((i0 + j) as u16);
                    indices.push((i1 + j) as u16);
                }
            }
        }
        positions.push(Vec3::new(0.0, 0.0, -1.0));
        normals.push(Vec3::new(0.0, 0.0, -1.0));

        let i = 1 + (angle_subdivisions - 2) * angle_subdivisions * 2;
        for j in 0..angle_subdivisions * 2 {
            let j1 = (j + 1) % (angle_subdivisions * 2);
            indices.push((i + j) as u16);
            indices.push(((angle_subdivisions - 1) * angle_subdivisions * 2 + 1) as u16);
            indices.push((i + j1) as u16);
        }

        TriMesh {
            indices: Indices::U16(indices),
            positions: Positions::F32(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }

    ///
    /// Returns an axis aligned unconnected cube mesh with positions in the range `[-1..1]` in all axes.
    ///
    pub fn cube() -> Self {
        let positions = vec![
            // Up
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            // Down
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            // Back
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            // Front
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            // Right
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, -1.0),
            // Left
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
        ];
        let uvs = vec![
            // Up
            Vec2::new(0.25, 0.0),
            Vec2::new(0.25, 1.0 / 3.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(0.5, 1.0 / 3.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(0.25, 1.0 / 3.0),
            // Down
            Vec2::new(0.25, 2.0 / 3.0),
            Vec2::new(0.25, 1.0),
            Vec2::new(0.5, 1.0),
            Vec2::new(0.5, 1.0),
            Vec2::new(0.5, 2.0 / 3.0),
            Vec2::new(0.25, 2.0 / 3.0),
            // Back
            Vec2::new(0.0, 2.0 / 3.0),
            Vec2::new(0.25, 2.0 / 3.0),
            Vec2::new(0.0, 1.0 / 3.0),
            Vec2::new(0.25, 1.0 / 3.0),
            Vec2::new(0.0, 1.0 / 3.0),
            Vec2::new(0.25, 2.0 / 3.0),
            // Front
            Vec2::new(0.5, 2.0 / 3.0),
            Vec2::new(0.75, 2.0 / 3.0),
            Vec2::new(0.75, 1.0 / 3.0),
            Vec2::new(0.75, 1.0 / 3.0),
            Vec2::new(0.5, 1.0 / 3.0),
            Vec2::new(0.5, 2.0 / 3.0),
            // Right
            Vec2::new(1.0, 2.0 / 3.0),
            Vec2::new(1.0, 1.0 / 3.0),
            Vec2::new(0.75, 1.0 / 3.0),
            Vec2::new(0.75, 1.0 / 3.0),
            Vec2::new(0.75, 2.0 / 3.0),
            Vec2::new(1.0, 2.0 / 3.0),
            // Left
            Vec2::new(0.25, 1.0 / 3.0),
            Vec2::new(0.25, 2.0 / 3.0),
            Vec2::new(0.5, 1.0 / 3.0),
            Vec2::new(0.5, 2.0 / 3.0),
            Vec2::new(0.5, 1.0 / 3.0),
            Vec2::new(0.25, 2.0 / 3.0),
        ];
        let mut mesh = TriMesh {
            positions: Positions::F32(positions),
            uvs: Some(uvs),
            ..Default::default()
        };
        mesh.compute_normals();
        mesh.compute_tangents();
        mesh
    }

    ///
    /// Returns a cylinder mesh around the x-axis in the range `[0..1]` and with radius 1.
    ///
    pub fn cylinder(angle_subdivisions: u32) -> Self {
        let length_subdivisions = 1;
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for i in 0..length_subdivisions + 1 {
            let x = i as f32 / length_subdivisions as f32;
            for j in 0..angle_subdivisions {
                let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

                positions.push(Vec3::new(x, angle.cos(), angle.sin()));
            }
        }
        for i in 0..length_subdivisions {
            for j in 0..angle_subdivisions {
                indices.push((i * angle_subdivisions + j) as u16);
                indices.push((i * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);

                indices.push((i * angle_subdivisions + j) as u16);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
                indices.push(((i + 1) * angle_subdivisions + j) as u16);
            }
        }
        let mut mesh = Self {
            positions: Positions::F32(positions),
            indices: Indices::U16(indices),
            ..Default::default()
        };
        mesh.compute_normals();
        mesh
    }

    ///
    /// Returns a cone mesh around the x-axis in the range `[0..1]` and with radius 1 at -1.0.
    ///
    pub fn cone(angle_subdivisions: u32) -> Self {
        let length_subdivisions = 1;
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for i in 0..length_subdivisions + 1 {
            let x = i as f32 / length_subdivisions as f32;
            for j in 0..angle_subdivisions {
                let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

                positions.push(Vec3::new(
                    x,
                    angle.cos() * (1.0 - x),
                    angle.sin() * (1.0 - x),
                ));
            }
        }
        for i in 0..length_subdivisions {
            for j in 0..angle_subdivisions {
                indices.push((i * angle_subdivisions + j) as u16);
                indices.push((i * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);

                indices.push((i * angle_subdivisions + j) as u16);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
                indices.push(((i + 1) * angle_subdivisions + j) as u16);
            }
        }
        let mut mesh = Self {
            positions: Positions::F32(positions),
            indices: Indices::U16(indices),
            ..Default::default()
        };
        mesh.compute_normals();
        mesh
    }

    ///
    /// Returns an arrow mesh around the x-axis in the range `[0..1]` and with radius 1.
    /// The tail length and radius should be in the range `]0..1[`.
    ///
    pub fn arrow(tail_length: f32, tail_radius: f32, angle_subdivisions: u32) -> Self {
        let mut arrow = Self::cylinder(angle_subdivisions);
        arrow
            .transform(&Mat4::from_nonuniform_scale(
                tail_length,
                tail_radius,
                tail_radius,
            ))
            .unwrap();
        let mut cone = Self::cone(angle_subdivisions);
        cone.transform(
            &(Mat4::from_translation(Vec3::new(tail_length, 0.0, 0.0))
                * Mat4::from_nonuniform_scale(1.0 - tail_length, 1.0, 1.0)),
        )
        .unwrap();
        let mut indices = arrow.indices.into_u32().unwrap();
        let cone_indices = cone.indices.into_u32().unwrap();
        let offset = indices.iter().max().unwrap() + 1;
        indices.extend(cone_indices.iter().map(|i| i + offset));
        arrow.indices = Indices::U16(indices.iter().map(|i| *i as u16).collect());

        if let Positions::F32(ref mut p) = arrow.positions {
            if let Positions::F32(ref p2) = cone.positions {
                p.extend(p2);
            }
        }
        arrow
            .normals
            .as_mut()
            .unwrap()
            .extend(cone.normals.as_ref().unwrap());
        arrow
    }

    ///
    /// Computes the per vertex normals and updates the normals of the mesh.
    /// It will override the current normals if they already exist.
    ///
    pub fn compute_normals(&mut self) {
        let mut normals = vec![Vec3::new(0.0, 0.0, 0.0); self.positions.len()];
        self.for_each_triangle(|i0, i1, i2| {
            let normal = match self.positions {
                Positions::F32(ref positions) => {
                    let p0 = positions[i0];
                    let p1 = positions[i1];
                    let p2 = positions[i2];
                    (p1 - p0).cross(p2 - p0)
                }
                Positions::F64(ref positions) => {
                    let p0 = positions[i0];
                    let p1 = positions[i1];
                    let p2 = positions[i2];
                    let n = (p1 - p0).cross(p2 - p0);
                    Vec3::new(n.x as f32, n.y as f32, n.z as f32)
                }
            };
            normals[i0] += normal;
            normals[i1] += normal;
            normals[i2] += normal;
        });

        for n in normals.iter_mut() {
            *n = n.normalize();
        }
        self.normals = Some(normals);
    }

    ///
    /// Computes the per vertex tangents and updates the tangents of the mesh.
    /// It will override the current tangents if they already exist.
    ///
    pub fn compute_tangents(&mut self) {
        if self.normals.is_none() || self.uvs.is_none() {
            panic!("mesh must have both normals and uv coordinates to be able to compute tangents");
        }
        let mut tan1 = vec![Vec3::new(0.0, 0.0, 0.0); self.positions.len()];
        let mut tan2 = vec![Vec3::new(0.0, 0.0, 0.0); self.positions.len()];

        self.for_each_triangle(|i0, i1, i2| {
            let (a, b, c) = match self.positions {
                Positions::F32(ref positions) => (positions[i0], positions[i1], positions[i2]),
                Positions::F64(ref positions) => {
                    let (a, b, c) = (positions[i0], positions[i1], positions[i2]);
                    (
                        Vec3::new(a.x as f32, a.y as f32, a.z as f32),
                        Vec3::new(b.x as f32, b.y as f32, b.z as f32),
                        Vec3::new(c.x as f32, c.y as f32, c.z as f32),
                    )
                }
            };
            let uva = self.uvs.as_ref().unwrap()[i0];
            let uvb = self.uvs.as_ref().unwrap()[i1];
            let uvc = self.uvs.as_ref().unwrap()[i2];

            let ba = b - a;
            let ca = c - a;

            let uvba = uvb - uva;
            let uvca = uvc - uva;

            let d = uvba.x * uvca.y - uvca.x * uvba.y;
            if d.abs() > 0.00001 {
                let r = 1.0 / d;
                let sdir = (ba * uvca.y - ca * uvba.y) * r;
                let tdir = (ca * uvba.x - ba * uvca.x) * r;
                tan1[i0] += sdir;
                tan1[i1] += sdir;
                tan1[i2] += sdir;
                tan2[i0] += tdir;
                tan2[i1] += tdir;
                tan2[i2] += tdir;
            }
        });

        let mut tangents = vec![Vec4::new(0.0, 0.0, 0.0, 0.0); self.positions.len()];
        self.for_each_vertex(|index| {
            let normal = self.normals.as_ref().unwrap()[index];
            let t = tan1[index];
            let tangent = (t - normal * normal.dot(t)).normalize();
            let handedness = if normal.cross(tangent).dot(tan2[index]) < 0.0 {
                1.0
            } else {
                -1.0
            };
            tangents[index] = tangent.extend(handedness);
        });

        self.tangents = Some(tangents);
    }

    ///
    ///  Iterates over all vertices in this mesh and calls the callback function with the index for each vertex.
    ///
    pub fn for_each_vertex(&self, mut callback: impl FnMut(usize)) {
        for i in 0..self.positions.len() {
            callback(i);
        }
    }

    ///
    /// Iterates over all triangles in this mesh and calls the callback function with the three indices, one for each vertex in the triangle.
    ///
    pub fn for_each_triangle(&self, mut callback: impl FnMut(usize, usize, usize)) {
        match self.indices {
            Indices::U8(ref indices) => {
                for face in 0..indices.len() / 3 {
                    let index0 = indices[face * 3] as usize;
                    let index1 = indices[face * 3 + 1] as usize;
                    let index2 = indices[face * 3 + 2] as usize;
                    callback(index0, index1, index2);
                }
            }
            Indices::U16(ref indices) => {
                for face in 0..indices.len() / 3 {
                    let index0 = indices[face * 3] as usize;
                    let index1 = indices[face * 3 + 1] as usize;
                    let index2 = indices[face * 3 + 2] as usize;
                    callback(index0, index1, index2);
                }
            }
            Indices::U32(ref indices) => {
                for face in 0..indices.len() / 3 {
                    let index0 = indices[face * 3] as usize;
                    let index1 = indices[face * 3 + 1] as usize;
                    let index2 = indices[face * 3 + 2] as usize;
                    callback(index0, index1, index2);
                }
            }
            Indices::None => {
                for face in 0..self.triangle_count() {
                    callback(face * 3, face * 3 + 1, face * 3 + 2);
                }
            }
        }
    }

    ///
    /// Computes the [AxisAlignedBoundingBox] for this triangle mesh.
    ///
    pub fn compute_aabb(&self) -> AxisAlignedBoundingBox {
        self.positions.compute_aabb()
    }

    ///
    /// Returns an error if the mesh is not valid.
    ///
    pub fn validate(&self) -> Result<()> {
        if self.indices.len().map(|i| i % 3 != 0).unwrap_or(false) {
            Err(Error::InvalidNumberOfIndices(self.indices.len().unwrap()))?;
        }
        let vertex_count = self.vertex_count();
        let max_index = match &self.indices {
            Indices::U8(ind) => ind.iter().max().map(|m| *m as usize),
            Indices::U16(ind) => ind.iter().max().map(|m| *m as usize),
            Indices::U32(ind) => ind.iter().max().map(|m| *m as usize),
            Indices::None => None,
        };
        if max_index.map(|i| i >= vertex_count).unwrap_or(false) {
            Err(Error::InvalidIndices(max_index.unwrap(), vertex_count))?;
        }
        let buffer_check = |length: Option<usize>, name: &str| -> Result<()> {
            if let Some(length) = length {
                if length < vertex_count {
                    Err(Error::InvalidBufferLength(
                        name.to_string(),
                        vertex_count,
                        length,
                    ))?;
                }
            }
            Ok(())
        };

        buffer_check(Some(self.positions.len()), "position")?;
        buffer_check(self.normals.as_ref().map(|b| b.len()), "normal")?;
        buffer_check(self.tangents.as_ref().map(|b| b.len()), "tangent")?;
        buffer_check(self.colors.as_ref().map(|b| b.len()), "color")?;
        buffer_check(self.uvs.as_ref().map(|b| b.len()), "uv coordinate")?;

        Ok(())
    }
}
