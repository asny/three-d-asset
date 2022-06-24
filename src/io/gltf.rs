use crate::{geometry::*, io::*, material::*, Error, Model, Result};
use ::gltf::Gltf;
use std::path::Path;

pub fn deserialize_gltf(raw_assets: &mut RawAssets, path: impl AsRef<Path>) -> Result<Model> {
    let mut cpu_meshes = Vec::new();
    let mut cpu_materials = Vec::new();

    let Gltf { document, mut blob } = Gltf::from_slice(&raw_assets.remove(path.as_ref())?)?;
    let base_path = path.as_ref().parent().unwrap_or(Path::new(""));
    let mut buffers = Vec::new();
    for buffer in document.buffers() {
        let mut data = match buffer.source() {
            ::gltf::buffer::Source::Uri(uri) => raw_assets.remove(base_path.join(uri))?,
            ::gltf::buffer::Source::Bin => blob.take().ok_or(Error::GltfMissingData)?,
        };
        if data.len() < buffer.length() {
            Err(Error::GltfCorruptData)?;
        }
        while data.len() % 4 != 0 {
            data.push(0);
        }
        buffers.push(::gltf::buffer::Data(data));
    }

    for scene in document.scenes() {
        for node in scene.nodes() {
            parse_tree(
                &Mat4::identity(),
                &node,
                raw_assets,
                &base_path,
                &buffers,
                &mut cpu_meshes,
                &mut cpu_materials,
            )?;
        }
    }
    Ok(Model {
        geometries: cpu_meshes,
        materials: cpu_materials,
    })
}

fn parse_tree<'a>(
    parent_transform: &Mat4,
    node: &::gltf::Node,
    raw_assets: &mut RawAssets,
    path: &Path,
    buffers: &[::gltf::buffer::Data],
    cpu_meshes: &mut Vec<TriMesh>,
    cpu_materials: &mut Vec<PbrMaterial>,
) -> Result<()> {
    let node_transform = parse_transform(node.transform());
    if node_transform.determinant() == 0.0 {
        return Ok(()); // glTF say that if the scale is all zeroes, the node should be ignored.
    }
    let transform = parent_transform * node_transform;

    if let Some(mesh) = node.mesh() {
        let name: String = mesh
            .name()
            .map(|s| s.to_string())
            .unwrap_or(format!("index {}", mesh.index()));
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(read_positions) = reader.read_positions() {
                let positions = read_positions.map(|p| p.into()).collect();

                let normals = reader
                    .read_normals()
                    .map(|values| values.map(|n| n.into()).collect());

                let tangents = reader
                    .read_tangents()
                    .map(|values| values.map(|t| t.into()).collect());

                let indices = reader.read_indices().map(|values| match values {
                    ::gltf::mesh::util::ReadIndices::U8(iter) => Indices::U8(iter.collect()),
                    ::gltf::mesh::util::ReadIndices::U16(iter) => Indices::U16(iter.collect()),
                    ::gltf::mesh::util::ReadIndices::U32(iter) => Indices::U32(iter.collect()),
                });

                let material = primitive.material();
                let material_name: String = material.name().map(|s| s.to_string()).unwrap_or(
                    material
                        .index()
                        .map(|i| format!("index {}", i))
                        .unwrap_or("default".to_string()),
                );
                let parsed = cpu_materials
                    .iter()
                    .any(|material| material.name == material_name);

                if !parsed {
                    let pbr = material.pbr_metallic_roughness();
                    let color = pbr.base_color_factor();
                    let albedo_texture = if let Some(info) = pbr.base_color_texture() {
                        Some(parse_texture(raw_assets, path, buffers, info.texture())?)
                    } else {
                        None
                    };
                    let metallic_roughness_texture =
                        if let Some(info) = pbr.metallic_roughness_texture() {
                            Some(parse_texture(raw_assets, path, buffers, info.texture())?)
                        } else {
                            None
                        };
                    let (normal_texture, normal_scale) =
                        if let Some(normal) = material.normal_texture() {
                            (
                                Some(parse_texture(raw_assets, path, buffers, normal.texture())?),
                                normal.scale(),
                            )
                        } else {
                            (None, 1.0)
                        };
                    let (occlusion_texture, occlusion_strength) =
                        if let Some(occlusion) = material.occlusion_texture() {
                            (
                                Some(parse_texture(
                                    raw_assets,
                                    path,
                                    buffers,
                                    occlusion.texture(),
                                )?),
                                occlusion.strength(),
                            )
                        } else {
                            (None, 1.0)
                        };
                    let emissive_texture = if let Some(info) = material.emissive_texture() {
                        Some(parse_texture(raw_assets, path, buffers, info.texture())?)
                    } else {
                        None
                    };
                    cpu_materials.push(PbrMaterial {
                        name: material_name.clone(),
                        albedo: Color::from_rgba_slice(&color),
                        albedo_texture,
                        metallic: pbr.metallic_factor(),
                        roughness: pbr.roughness_factor(),
                        metallic_roughness_texture,
                        normal_texture,
                        normal_scale,
                        occlusion_texture,
                        occlusion_strength,
                        occlusion_metallic_roughness_texture: None,
                        emissive: Color::from_rgb_slice(&material.emissive_factor()),
                        emissive_texture,
                        alpha_cutout: None,
                        lighting_model: LightingModel::Cook(
                            NormalDistributionFunction::TrowbridgeReitzGGX,
                            GeometryFunction::SmithSchlickGGX,
                        ),
                    });
                }

                let colors = reader.read_colors(0).map(|values| {
                    values
                        .into_rgba_u8()
                        .map(|c| Color::new(c[0], c[1], c[2], c[3]))
                        .collect()
                });

                let uvs = reader
                    .read_tex_coords(0)
                    .map(|values| values.into_f32().map(|uv| uv.into()).collect());

                let mut cpu_mesh = TriMesh {
                    name: name.clone(),
                    positions: Positions::F32(positions),
                    normals,
                    tangents,
                    indices,
                    colors,
                    uvs,
                    material_name: Some(material_name),
                };
                if transform != Mat4::identity() {
                    cpu_mesh.transform(&transform)?;
                }
                cpu_meshes.push(cpu_mesh);
            }
        }
    }

    for child in node.children() {
        parse_tree(
            &transform,
            &child,
            raw_assets,
            path,
            buffers,
            cpu_meshes,
            cpu_materials,
        )?;
    }
    Ok(())
}

fn parse_texture<'a>(
    raw_assets: &mut RawAssets,
    path: &Path,
    buffers: &[::gltf::buffer::Data],
    gltf_texture: ::gltf::texture::Texture,
) -> Result<Texture2D> {
    let gltf_image = gltf_texture.source();
    let gltf_source = gltf_image.source();
    let tex = match gltf_source {
        ::gltf::image::Source::Uri { uri, .. } => {
            raw_assets.deserialize(path.join(Path::new(uri)))?
        }
        ::gltf::image::Source::View { view, .. } => {
            if view.stride() != None {
                unimplemented!();
            }
            #[allow(unused_variables)]
            let buffer = &buffers[view.buffer().index()];
            #[cfg(not(feature = "image"))]
            return Err(Error::FeatureMissing("image".to_string()));
            #[cfg(feature = "image")]
            super::img::deserialize_img("", &buffer[view.offset()..view.offset() + view.length()])?
        }
    };
    // TODO: Parse sampling parameters
    Ok(tex)
}

fn parse_transform(transform: ::gltf::scene::Transform) -> Mat4 {
    let [c0, c1, c2, c3] = transform.matrix();
    Mat4::from_cols(c0.into(), c1.into(), c2.into(), c3.into())
}

#[cfg(test)]
mod test {

    #[test]
    pub fn deserialize_gltf() {
        let model: crate::Model = crate::io::RawAssets::new()
            .insert(
                "Cube.gltf",
                include_bytes!("../../test_data/Cube.gltf").to_vec(),
            )
            .insert(
                "Cube.bin",
                include_bytes!("../../test_data/Cube.bin").to_vec(),
            )
            .insert(
                "Cube_BaseColor.png",
                include_bytes!("../../test_data/Cube_BaseColor.png").to_vec(),
            )
            .insert(
                "Cube_MetallicRoughness.png",
                include_bytes!("../../test_data/Cube_MetallicRoughness.png").to_vec(),
            )
            .deserialize("gltf")
            .unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 1);
    }

    #[test]
    pub fn deserialize_gltf_with_data_url() {
        let model: crate::Model = crate::io::RawAssets::new()
            .insert(
                "data_url.gltf",
                include_bytes!("../../test_data/data_url.gltf").to_vec(),
            )
            .deserialize("gltf")
            .unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 1);
    }
}
