use std::path::PathBuf;

use crate::{Node, Positions, Result, TriMesh};

use crate::{io::RawAssets, Scene};

use cgmath::Vector3;

pub fn deserialize_stl(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let stl_bytes = raw_assets.remove(path)?;
    let mut stl_bytes = std::io::Cursor::new(stl_bytes.to_vec());
    let stl = stl_io::read_stl(&mut stl_bytes)?;

    let positions = stl
        .vertices
        .iter()
        .map(|vertex| Vector3 {
            x: vertex[0],
            y: vertex[1],
            z: vertex[2],
        })
        .collect();

    let mut indices = Vec::with_capacity(stl.faces.len() * 3);
    let mut normals = Vec::with_capacity(stl.faces.len());
    for face in stl.faces {
        let face_indices = face.vertices;
        indices.push(face_indices[0] as u32);
        indices.push(face_indices[1] as u32);
        indices.push(face_indices[2] as u32);

        normals.push(Vector3 {
            x: face.normal[0],
            y: face.normal[1],
            z: face.normal[2],
        });
    }

    let mesh = TriMesh {
        positions: Positions::F32(positions),
        indices: crate::Indices::U32(indices),
        normals: Some(normals),
        tangents: None,
        uvs: None,
        colors: None,
    };

    // STL files contain only one object, so only one node
    let node = Node {
        geometry: Some(crate::Geometry::Triangles(mesh)),
        ..Default::default()
    };

    Ok(Scene {
        // stl_io does not expose the name it seems, so using path instead
        name: path.to_str().unwrap_or("default").to_owned(),
        children: vec![node],
        materials: vec![],
    })
}

#[cfg(test)]
mod test {
    #[test]
    pub fn deserialize_stl_ascii() {
        let model: crate::Model = crate::io::load_and_deserialize("test_data/cube.stl").unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 0);
    }

    #[test]
    pub fn deserialize_stl_binary() {
        let model: crate::Model = crate::io::load_and_deserialize("test_data/suzanne.stl").unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 0);
    }
}
