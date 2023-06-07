use crate::{geometry::*, io::RawAssets, material::*, Node, Result, Scene};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub use serde::{Serialize,Deserialize};

pub fn dependencies_obj(raw_assets: &RawAssets, path: &PathBuf) -> HashSet<PathBuf> {
    let mut dependencies = HashSet::new();
    if let Ok(Ok(obj)) =
        std::str::from_utf8(raw_assets.get(path).unwrap()).map(|s| wavefront_obj::obj::parse(s))
    {
        let base_path = path.parent().unwrap_or(Path::new(""));
        if let Some(material_library) = obj.material_library {
            dependencies.insert(base_path.join(material_library));
        }
    }
    dependencies
}

pub fn dependencies_mtl(raw_assets: &RawAssets, path: &PathBuf) -> HashSet<PathBuf> {
    let mut dependencies = HashSet::new();
    if let Ok(Ok(materials)) =
        std::str::from_utf8(raw_assets.get(path).unwrap()).map(|s| wavefront_obj::mtl::parse(s))
    {
        let base_path = path.parent().unwrap_or(Path::new(""));
        for material in materials.materials {
            material
                .ambient_map
                .map(|p| dependencies.insert(base_path.join(p)));
            material
                .diffuse_map
                .map(|p| dependencies.insert(base_path.join(p)));
            material
                .specular_map
                .map(|p| dependencies.insert(base_path.join(p)));
            material
                .specular_exponent_map
                .map(|p| dependencies.insert(base_path.join(p)));
            material
                .displacement_map
                .map(|p| dependencies.insert(base_path.join(p)));
            material
                .dissolve_map
                .map(|p| dependencies.insert(base_path.join(p)));
            material
                .decal_map
                .map(|p| dependencies.insert(base_path.join(p)));
            material
                .bump_map
                .map(|p| dependencies.insert(base_path.join(p)));
        }
    }
    dependencies
}

pub fn deserialize_obj(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let obj_bytes = raw_assets.remove(path)?;
    let obj = wavefront_obj::obj::parse(std::str::from_utf8(&obj_bytes).unwrap())?;
    let p = path.parent().unwrap_or(Path::new(""));

    // Parse materials
    let mut materials = Vec::new();
    if let Some(material_library) = obj.material_library {
        let bytes = raw_assets.remove(p.join(material_library).to_str().unwrap())?;
        for material in wavefront_obj::mtl::parse(std::str::from_utf8(&bytes).unwrap())?.materials {
            let color = if material.color_diffuse.r != material.color_diffuse.g
                || material.color_diffuse.g != material.color_diffuse.b
            {
                material.color_diffuse
            } else if material.color_specular.r != material.color_specular.g
                || material.color_specular.g != material.color_specular.b
            {
                material.color_specular
            } else if material.color_ambient.r != material.color_ambient.g
                || material.color_ambient.g != material.color_ambient.b
            {
                material.color_ambient
            } else {
                material.color_diffuse
            };

            let normal_texture = if let Some(ref texture_name) = material.bump_map {
                Some(raw_assets.deserialize(p.join(texture_name))?)
            } else {
                None
            };
            let albedo_texture = if let Some(ref texture_name) = material.diffuse_map {
                Some(raw_assets.deserialize(p.join(texture_name))?)
            } else {
                None
            };

            materials.push(PbrMaterial {
                name: material.name,
                albedo: Color::from_rgba_slice(&[
                    color.r as f32,
                    color.g as f32,
                    color.b as f32,
                    material.alpha as f32,
                ]),
                albedo_texture,
                metallic: ((material.color_specular.r
                    + material.color_specular.g
                    + material.color_specular.b)
                    / 3.0) as f32,
                roughness: if material.specular_coefficient > 0.1 {
                    ((1.999 / material.specular_coefficient).sqrt() as f32).min(1.0)
                } else {
                    1.0
                },
                normal_texture,
                lighting_model: LightingModel::Blinn,
                ..Default::default()
            });
        }
    }

    // Parse meshes
    let mut nodes = Vec::new();
    for object in obj.objects.iter() {
        // Objects consisting of several meshes with different materials
        for mesh in object.geometry.iter() {
            // All meshes with different materials
            let mut positions = Vec::new();
            let mut normals: Vec<Vec3> = Vec::new();
            let mut uvs: Vec<Vec2> = Vec::new();
            let mut indices = Vec::new();

            let mut map: HashMap<usize, usize> = HashMap::new();

            let mut process = |i: wavefront_obj::obj::VTNIndex| {
                let mut index = map.get(&i.0).map(|v| *v);

                let uvw = i.1.map(|tex_index| object.tex_vertices[tex_index]);
                let normal = i.2.map(|normal_index| object.normals[normal_index]);

                if let Some(ind) = index {
                    if let Some(tex) = uvw {
                        if ((uvs[ind].x - tex.u as f32) as f32).abs() > std::f32::EPSILON
                            || ((uvs[ind].y - tex.v as f32) as f32).abs() > std::f32::EPSILON
                        {
                            index = None;
                        }
                    }
                    if let Some(n) = normal {
                        if ((normals[ind].x - n.x as f32) as f32).abs() > std::f32::EPSILON
                            || ((normals[ind].y - n.y as f32) as f32).abs() > std::f32::EPSILON
                            || ((normals[ind].z - n.z as f32) as f32).abs() > std::f32::EPSILON
                        {
                            index = None;
                        }
                    }
                }

                if index.is_none() {
                    index = Some(positions.len());
                    map.insert(i.0, index.unwrap());
                    let position = object.vertices[i.0];
                    positions.push(Vector3::new(position.x, position.y, position.z));

                    if let Some(tex) = uvw {
                        uvs.push(Vec2::new(tex.u as f32, 1.0 - tex.v as f32));
                    }
                    if let Some(n) = normal {
                        normals.push(Vec3::new(n.x as f32, n.y as f32, n.z as f32));
                    }
                }

                indices.push(index.unwrap() as u32);
            };
            for shape in mesh.shapes.iter() {
                // All triangles with same material
                match shape.primitive {
                    wavefront_obj::obj::Primitive::Triangle(i0, i1, i2) => {
                        process(i0);
                        process(i1);
                        process(i2);
                    }
                    _ => {}
                }
            }

            let vertex_count = positions.len();
            let tri_mesh = TriMesh {
                positions: Positions::F64(positions),
                indices: Indices::U32(indices),
                normals: if normals.len() == vertex_count {
                    Some(normals)
                } else {
                    None
                },
                uvs: if uvs.len() == vertex_count {
                    Some(uvs)
                } else {
                    None
                },
                colors: None,
                tangents: None,
            };
            nodes.push(Node {
                name: object.name.to_string(),
                geometry: Some(Geometry::Triangles(tri_mesh)),
                material_index: mesh
                    .material_name
                    .as_ref()
                    .map(|n| materials.iter().position(|m| &m.name == n))
                    .flatten(),
                ..Default::default()
            });
        }
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
