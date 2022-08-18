//!
//! Contain geometry asset definitions.
//!

mod points;
pub use points::*;

mod tri_mesh;
pub use tri_mesh::*;

pub use crate::prelude::*;

///
/// An array of indices. Supports different data types.
///
#[derive(Clone)]
pub enum Indices {
    /// Uses unsigned 8 bit integer for each index.
    U8(Vec<u8>),
    /// Uses unsigned 16 bit integer for each index.
    U16(Vec<u16>),
    /// Uses unsigned 32 bit integer for each index.
    U32(Vec<u32>),
}

impl Indices {
    ///
    /// Converts all the indices as `u32` data type.
    ///
    pub fn into_u32(self) -> Vec<u32> {
        match self {
            Self::U8(mut values) => values.drain(..).map(|i| i as u32).collect::<Vec<u32>>(),
            Self::U16(mut values) => values.drain(..).map(|i| i as u32).collect::<Vec<u32>>(),
            Self::U32(values) => values,
        }
    }

    ///
    /// Clones and converts all the indices as `u32` data type.
    ///
    pub fn to_u32(&self) -> Vec<u32> {
        match self {
            Self::U8(values) => values.iter().map(|i| *i as u32).collect::<Vec<u32>>(),
            Self::U16(values) => values.iter().map(|i| *i as u32).collect::<Vec<u32>>(),
            Self::U32(values) => values.clone(),
        }
    }

    ///
    /// Returns the number of indices.
    ///
    pub fn len(&self) -> usize {
        match self {
            Self::U8(values) => values.len(),
            Self::U16(values) => values.len(),
            Self::U32(values) => values.len(),
        }
    }
}

impl std::default::Default for Indices {
    fn default() -> Self {
        Self::U32(Vec::new())
    }
}

impl std::fmt::Debug for Indices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Indices");
        match self {
            Self::U8(ind) => d.field("u8", &ind.len()),
            Self::U16(ind) => d.field("u16", &ind.len()),
            Self::U32(ind) => d.field("u32", &ind.len()),
        };
        d.finish()
    }
}

///
/// An array of positions. Supports f32 and f64 data types.
///
#[derive(Clone)]
pub enum Positions {
    /// Uses 32 bit float for the vertex positions.
    F32(Vec<Vec3>),
    /// Uses 64 bit float for the vertex positions.
    F64(Vec<Vector3<f64>>),
}

impl Positions {
    ///
    /// Converts and returns all the indices as `f32` data type.
    ///
    pub fn into_f32(self) -> Vec<Vec3> {
        match self {
            Self::F32(values) => values,
            Self::F64(mut values) => values
                .drain(..)
                .map(|v| Vec3::new(v.x as f32, v.y as f32, v.z as f32))
                .collect::<Vec<_>>(),
        }
    }

    ///
    /// Clones and converts all the positions as `f32` data type.
    ///
    pub fn to_f32(&self) -> Vec<Vec3> {
        match self {
            Self::F32(values) => values.clone(),
            Self::F64(values) => values
                .iter()
                .map(|v| Vec3::new(v.x as f32, v.y as f32, v.z as f32))
                .collect::<Vec<_>>(),
        }
    }

    ///
    /// Returns the number of positions.
    ///
    pub fn len(&self) -> usize {
        match self {
            Self::F32(values) => values.len(),
            Self::F64(values) => values.len(),
        }
    }
}

impl std::default::Default for Positions {
    fn default() -> Self {
        Self::F32(Vec::new())
    }
}

impl std::fmt::Debug for Positions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Positions");
        match self {
            Self::F32(ind) => d.field("f32", &ind.len()),
            Self::F64(ind) => d.field("f64", &ind.len()),
        };
        d.finish()
    }
}
