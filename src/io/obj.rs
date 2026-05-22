use crate::{geometry::*, io::RawAssets, material::*, prelude::Srgba, Node, Result, Scene};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub fn dependencies_obj(raw_assets: &RawAssets, path: &PathBuf) -> HashSet<PathBuf> {
    let mut dependencies = HashSet::new();
    let bytes = raw_assets.get(path).unwrap();
    let base_path = path.parent().unwrap_or(Path::new(""));

    for line in std::str::from_utf8(bytes).unwrap_or("").lines() {
        let line = line.trim();
        if let Some(mtl_file) = line.strip_prefix("mtllib ") {
            dependencies.insert(base_path.join(mtl_file.trim()));
        }
    }
    dependencies
}

pub fn dependencies_mtl(raw_assets: &RawAssets, path: &PathBuf) -> HashSet<PathBuf> {
    let mut dependencies = HashSet::new();
    let bytes = raw_assets.get(path).unwrap();
    let base_path = path.parent().unwrap_or(Path::new(""));

    let mut reader = Cursor::new(bytes);
    if let Ok(materials) = tobj::load_mtl_buf(&mut reader) {
        for material in materials.0 {
            if let Some(ref tex) = material.ambient_texture {
                dependencies.insert(base_path.join(tex));
            }
            if let Some(ref tex) = material.diffuse_texture {
                dependencies.insert(base_path.join(tex));
            }
            if let Some(ref tex) = material.specular_texture {
                dependencies.insert(base_path.join(tex));
            }
            if let Some(ref tex) = material.shininess_texture {
                dependencies.insert(base_path.join(tex));
            }
            if let Some(ref tex) = material.dissolve_texture {
                dependencies.insert(base_path.join(tex));
            }
            if let Some(ref tex) = material.normal_texture {
                dependencies.insert(base_path.join(tex));
            }
        }
    }
    dependencies
}

pub fn deserialize_obj(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let obj_bytes = raw_assets.remove(path)?;
    let base_path = path.parent().unwrap_or(Path::new("")).to_owned();

    let mut mtl_cache: HashMap<PathBuf, Vec<u8>> = HashMap::new();
    for line in std::str::from_utf8(&obj_bytes).unwrap_or("").lines() {
        let line = line.trim();
        if let Some(mtl_file) = line.strip_prefix("mtllib ") {
            let full_path = base_path.join(mtl_file.trim());
            if let Ok(bytes) = raw_assets.remove(&full_path) {
                mtl_cache.insert(full_path, bytes);
            }
        }
    }

    let mut reader = Cursor::new(&obj_bytes);
    let (models, obj_materials) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |mtl_path| {
            let full_path = base_path.join(mtl_path);
            if let Some(mtl_bytes) = mtl_cache.get(&full_path) {
                let mut mtl_reader = Cursor::new(mtl_bytes);
                tobj::load_mtl_buf(&mut mtl_reader)
            } else {
                Err(tobj::LoadError::ReadError)
            }
        },
    )?;

    let tobj_materials = obj_materials.unwrap_or_default();

    let mut materials = Vec::new();
    for mat in &tobj_materials {
        let diffuse = mat.diffuse.unwrap_or([0.8, 0.8, 0.8]);
        let specular = mat.specular.unwrap_or([0.0, 0.0, 0.0]);
        let ambient = mat.ambient.unwrap_or([0.0, 0.0, 0.0]);

        let color = if diffuse[0] != diffuse[1] || diffuse[1] != diffuse[2] {
            diffuse
        } else if specular[0] != specular[1] || specular[1] != specular[2] {
            specular
        } else if ambient[0] != ambient[1] || ambient[1] != ambient[2] {
            ambient
        } else {
            diffuse
        };

        let normal_texture = if let Some(ref texture_name) = mat.normal_texture {
            Some(raw_assets.deserialize(base_path.join(texture_name))?)
        } else {
            None
        };
        let albedo_texture = if let Some(ref texture_name) = mat.diffuse_texture {
            Some(raw_assets.deserialize(base_path.join(texture_name))?)
        } else {
            None
        };

        let shininess = mat.shininess.unwrap_or(0.0);
        let alpha = mat.dissolve.unwrap_or(1.0);

        materials.push(PbrMaterial {
            name: mat.name.clone(),
            albedo: [color[0] as f32, color[1] as f32, color[2] as f32, alpha as f32].into(),
            albedo_texture,
            metallic: ((specular[0] + specular[1] + specular[2]) / 3.0) as f32,
            roughness: if shininess > 0.1 {
                ((1.999 / shininess).sqrt() as f32).min(1.0)
            } else {
                1.0
            },
            normal_texture,
            lighting_model: LightingModel::Blinn,
            ..Default::default()
        });
    }

    let mut nodes = Vec::new();
    for model in &models {
        let mesh = &model.mesh;
        let vertex_count = mesh.positions.len() / 3;

        let positions: Vec<Vector3<f64>> = mesh
            .positions
            .chunks_exact(3)
            .map(|c| Vector3::new(c[0], c[1], c[2]))
            .collect();

        let normals: Option<Vec<Vec3>> = if mesh.normals.len() == vertex_count * 3 {
            Some(
                mesh.normals
                    .chunks_exact(3)
                    .map(|c| Vec3::new(c[0] as f32, c[1] as f32, c[2] as f32))
                    .collect(),
            )
        } else {
            None
        };

        let uvs: Option<Vec<Vec2>> = if mesh.texcoords.len() == vertex_count * 2 {
            Some(
                mesh.texcoords
                    .chunks_exact(2)
                    .map(|c| Vec2::new(c[0] as f32, 1.0 - c[1] as f32))
                    .collect(),
            )
        } else {
            None
        };

        let colors: Option<Vec<Srgba>> = if mesh.vertex_color.len() == vertex_count * 3 {
            Some(
                mesh.vertex_color
                    .chunks_exact(3)
                    .map(|c| Srgba {
                        r: (c[0] * 255.0) as u8,
                        g: (c[1] * 255.0) as u8,
                        b: (c[2] * 255.0) as u8,
                        a: 255,
                    })
                    .collect(),
            )
        } else {
            None
        };

        let indices = if mesh.indices.is_empty() {
            Indices::None
        } else {
            Indices::U32(mesh.indices.clone())
        };

        let tri_mesh = TriMesh {
            positions: Positions::F64(positions),
            indices,
            normals,
            uvs,
            colors,
            tangents: None,
        };

        nodes.push(Node {
            name: model.name.clone(),
            geometry: Some(Geometry::Triangles(tri_mesh)),
            material_index: mesh
                .material_id
                .and_then(|id| if id < materials.len() { Some(id) } else { None }),
            ..Default::default()
        });
    }

    Ok(Scene {
        name: path.to_str().unwrap_or("default").to_owned(),
        children: nodes,
        materials,
    })
}

#[cfg(test)]
mod test {

    #[test]
    pub fn deserialize_obj() {
        let model: crate::Model = crate::io::load_and_deserialize("test_data/cube.obj").unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 0);
    }

    #[test]
    pub fn deserialize_obj_with_material() {
        let model: crate::Model = crate::io::load_and_deserialize("test_data/suzanne.obj").unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 1);
    }
}
