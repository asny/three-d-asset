use crate::{geometry::*, io::*, material::*, Error, Node, Result, Scene};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn deserialize_fbx(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let bytes = raw_assets.remove(path)?;
    use fbxcel::tree::any::AnyTree;
    use fbxcel::tree::v7400::NodeHandle;

    let cursor = std::io::Cursor::new(bytes);
    let any_tree = AnyTree::from_seekable_reader(cursor)?;
    let AnyTree::V7400(_, tree, _) = any_tree else {
        return Err(Error::FbxVersion(any_tree.fbx_version()));
    };
    let root = tree.root();
    let Some(objects) = root.first_child_by_name("Objects") else {
        return Ok(Scene::default());
    };
    let Some(connections) = root.first_child_by_name("Connections") else {
        return Ok(Scene::default());
    };

    // --- Build connection graph ---
    let mut children_of: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut parent_of: HashMap<i64, i64> = HashMap::new();
    for conn in connections.children_by_name("C") {
        let attrs = conn.attributes();
        if attrs.len() >= 3 {
            if let (Some("OO"), Some(child_id), Some(parent_id)) = (
                attrs[0].get_string(),
                attrs[1].get_i64(),
                attrs[2].get_i64(),
            ) {
                children_of.entry(parent_id).or_default().push(child_id);
                parent_of.insert(child_id, parent_id);
            }
        }
    }

    fn fbx_name(attrs: &[fbxcel::low::v7400::AttributeValue]) -> String {
        attrs
            .get(1)
            .and_then(|a| a.get_string())
            .unwrap_or("")
            .split('\0')
            .next()
            .unwrap_or("")
            .to_string()
    }

    fn fbx_props_f64(node: &NodeHandle, prop_name: &str) -> Option<Vec<f64>> {
        let props = node.first_child_by_name("Properties70")?;
        for p in props.children_by_name("P") {
            let a = p.attributes();
            if a.len() > 4 && a[0].get_string() == Some(prop_name) {
                let vals: Vec<f64> = a[4..].iter().filter_map(fbx_attr_to_f64).collect();
                if !vals.is_empty() {
                    return Some(vals);
                }
            }
        }
        None
    }

    fn fbx_layer_f64(
        node: &NodeHandle,
        layer_name: &str,
        data_name: &str,
        index_name: &str,
    ) -> (Vec<f64>, Vec<i32>, String, String) {
        let Some(layer) = node.first_child_by_name(layer_name) else {
            return Default::default();
        };
        let data = layer
            .first_child_by_name(data_name)
            .and_then(|n| n.attributes().first()?.get_arr_f64().map(|v| v.to_vec()))
            .unwrap_or_default();
        let indices = layer
            .first_child_by_name(index_name)
            .and_then(|n| n.attributes().first()?.get_arr_i32().map(|v| v.to_vec()))
            .unwrap_or_default();
        let mapping = layer
            .first_child_by_name("MappingInformationType")
            .and_then(|n| n.attributes().first()?.get_string().map(|s| s.to_string()))
            .unwrap_or_default();
        let reference = layer
            .first_child_by_name("ReferenceInformationType")
            .and_then(|n| n.attributes().first()?.get_string().map(|s| s.to_string()))
            .unwrap_or_default();
        (data, indices, mapping, reference)
    }

    // --- Parse materials ---
    let mut mat_list: Vec<(i64, PbrMaterial)> = Vec::new();
    for obj in objects.children() {
        if obj.name() != "Material" {
            continue;
        }
        let attrs = obj.attributes();
        let Some(id) = attrs.first().and_then(|a| a.get_i64()) else {
            continue;
        };
        let mut mat = PbrMaterial {
            name: fbx_name(attrs),
            ..Default::default()
        };
        let color_prop = |node: &NodeHandle, names: &[&str]| -> Option<[f64; 3]> {
            for &n in names {
                if let Some(v) = fbx_props_f64(node, n) {
                    if v.len() >= 3 {
                        return Some([v[0], v[1], v[2]]);
                    }
                }
            }
            None
        };
        if let Some(c) = color_prop(&obj, &["DiffuseColor", "BaseColor", "Maya|baseColor"]) {
            mat.albedo = Srgba::new(
                (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                255,
            );
        }
        if let Some(v) = fbx_props_f64(&obj, "Opacity") {
            mat.albedo.a = (v[0].clamp(0.0, 1.0) * 255.0) as u8;
        }
        for name in [
            "Metallic",
            "Metalness",
            "ReflectionFactor",
            "Maya|metalness",
        ] {
            if let Some(v) = fbx_props_f64(&obj, name) {
                mat.metallic = v[0] as f32;
                break;
            }
        }
        for name in ["Roughness", "Maya|specularRoughness"] {
            if let Some(v) = fbx_props_f64(&obj, name) {
                mat.roughness = v[0] as f32;
                break;
            }
        }
        if let Some(c) = color_prop(&obj, &["EmissiveColor"]) {
            mat.emissive = Srgba::new(
                (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                255,
            );
        }
        mat_list.push((id, mat));
    }
    let mat_id_to_index: HashMap<i64, usize> = mat_list
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i))
        .collect();

    // --- Parse geometries ---
    struct GeomLayer {
        data: Vec<f64>,
        indices: Vec<i32>,
        mapping: String,
        reference: String,
    }
    struct GeomData {
        vertices: Vec<f64>,
        poly_indices: Vec<i32>,
        normals: GeomLayer,
        uvs: GeomLayer,
        colors: GeomLayer,
    }

    let mut geom_map: HashMap<i64, GeomData> = HashMap::new();
    for obj in objects.children() {
        if obj.name() != "Geometry" {
            continue;
        }
        let attrs = obj.attributes();
        if attrs.get(2).and_then(|a| a.get_string()) != Some("Mesh") {
            continue;
        }
        let Some(id) = attrs.first().and_then(|a| a.get_i64()) else {
            continue;
        };

        let vertices = obj
            .first_child_by_name("Vertices")
            .and_then(|n| n.attributes().first()?.get_arr_f64().map(|v| v.to_vec()))
            .unwrap_or_default();
        let poly_indices = obj
            .first_child_by_name("PolygonVertexIndex")
            .and_then(|n| n.attributes().first()?.get_arr_i32().map(|v| v.to_vec()))
            .unwrap_or_default();

        let (nd, ni, nm, nr) = fbx_layer_f64(&obj, "LayerElementNormal", "Normals", "NormalsIndex");
        let (ud, ui, um, ur) = fbx_layer_f64(&obj, "LayerElementUV", "UV", "UVIndex");
        let (cd, ci, cm, cr) = fbx_layer_f64(&obj, "LayerElementColor", "Colors", "ColorIndex");

        geom_map.insert(
            id,
            GeomData {
                vertices,
                poly_indices,
                normals: GeomLayer {
                    data: nd,
                    indices: ni,
                    mapping: nm,
                    reference: nr,
                },
                uvs: GeomLayer {
                    data: ud,
                    indices: ui,
                    mapping: um,
                    reference: ur,
                },
                colors: GeomLayer {
                    data: cd,
                    indices: ci,
                    mapping: cm,
                    reference: cr,
                },
            },
        );
    }

    // --- Triangulate a geometry into a TriMesh ---
    let triangulate = |geom: &GeomData| -> TriMesh {
        let verts = &geom.vertices;
        let poly_idx = &geom.poly_indices;

        // Split into polygons: negative index marks end of polygon (actual = !raw)
        let mut polygons: Vec<Vec<(usize, usize)>> = Vec::new(); // (polygon_vertex_idx, vertex_idx)
        let mut current: Vec<(usize, usize)> = Vec::new();
        for (pv, &raw) in poly_idx.iter().enumerate() {
            if raw < 0 {
                current.push((pv, (!raw) as usize));
                polygons.push(std::mem::take(&mut current));
            } else {
                current.push((pv, raw as usize));
            }
        }

        let est = polygons
            .iter()
            .map(|p| p.len().saturating_sub(2))
            .sum::<usize>();
        let mut positions = Vec::with_capacity(est * 3);
        let mut normals_out = Vec::with_capacity(est * 3);
        let mut uvs_out = Vec::with_capacity(est * 3);
        let mut colors_out = Vec::with_capacity(est * 3);

        let has_n = !geom.normals.data.is_empty();
        let has_uv = !geom.uvs.data.is_empty();
        let has_c = !geom.colors.data.is_empty();

        for poly in &polygons {
            if poly.len() < 3 {
                continue;
            }
            for i in 1..(poly.len() - 1) {
                for &(pv, vi) in &[poly[0], poly[i], poly[i + 1]] {
                    if vi * 3 + 2 < verts.len() {
                        positions.push(vec3(
                            verts[vi * 3] as f32,
                            verts[vi * 3 + 1] as f32,
                            verts[vi * 3 + 2] as f32,
                        ));
                    }
                    if has_n {
                        let n = &geom.normals;
                        let idx = fbx_get_layer_index(pv, vi, &n.mapping, &n.reference, &n.indices);
                        if idx * 3 + 2 < n.data.len() {
                            normals_out.push(vec3(
                                n.data[idx * 3] as f32,
                                n.data[idx * 3 + 1] as f32,
                                n.data[idx * 3 + 2] as f32,
                            ));
                        }
                    }
                    if has_uv {
                        let u = &geom.uvs;
                        let idx = fbx_get_layer_index(pv, vi, &u.mapping, &u.reference, &u.indices);
                        if idx * 2 + 1 < u.data.len() {
                            uvs_out.push(vec2(u.data[idx * 2] as f32, u.data[idx * 2 + 1] as f32));
                        }
                    }
                    if has_c {
                        let c = &geom.colors;
                        let idx = fbx_get_layer_index(pv, vi, &c.mapping, &c.reference, &c.indices);
                        if idx * 4 + 3 < c.data.len() {
                            colors_out.push(Srgba::new(
                                (c.data[idx * 4].clamp(0.0, 1.0) * 255.0) as u8,
                                (c.data[idx * 4 + 1].clamp(0.0, 1.0) * 255.0) as u8,
                                (c.data[idx * 4 + 2].clamp(0.0, 1.0) * 255.0) as u8,
                                (c.data[idx * 4 + 3].clamp(0.0, 1.0) * 255.0) as u8,
                            ));
                        }
                    }
                }
            }
        }

        TriMesh {
            positions: Positions::F32(positions),
            indices: Indices::None,
            normals: if normals_out.is_empty() {
                None
            } else {
                Some(normals_out)
            },
            tangents: None,
            uvs: if uvs_out.is_empty() {
                None
            } else {
                Some(uvs_out)
            },
            colors: if colors_out.is_empty() {
                None
            } else {
                Some(colors_out)
            },
        }
    };

    // --- Parse models and build scene tree ---
    struct ModelInfo {
        name: String,
        translation: [f64; 3],
        rotation: [f64; 3],
        scaling: [f64; 3],
    }
    let mut model_map: HashMap<i64, ModelInfo> = HashMap::new();
    for obj in objects.children() {
        if obj.name() != "Model" {
            continue;
        }
        let attrs = obj.attributes();
        let Some(id) = attrs.first().and_then(|a| a.get_i64()) else {
            continue;
        };
        let mut info = ModelInfo {
            name: fbx_name(attrs),
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scaling: [1.0, 1.0, 1.0],
        };
        if let Some(v) = fbx_props_f64(&obj, "Lcl Translation") {
            if v.len() >= 3 {
                info.translation = [v[0], v[1], v[2]];
            }
        }
        if let Some(v) = fbx_props_f64(&obj, "Lcl Rotation") {
            if v.len() >= 3 {
                info.rotation = [v[0], v[1], v[2]];
            }
        }
        if let Some(v) = fbx_props_f64(&obj, "Lcl Scaling") {
            if v.len() >= 3 {
                info.scaling = [v[0], v[1], v[2]];
            }
        }
        model_map.insert(id, info);
    }

    let model_transform = |m: &ModelInfo| -> Mat4 {
        let t = Mat4::from_translation(vec3(
            m.translation[0] as f32,
            m.translation[1] as f32,
            m.translation[2] as f32,
        ));
        let r = Mat4::from_angle_z(Rad((m.rotation[2] as f32).to_radians()))
            * Mat4::from_angle_y(Rad((m.rotation[1] as f32).to_radians()))
            * Mat4::from_angle_x(Rad((m.rotation[0] as f32).to_radians()));
        let s = Mat4::from_nonuniform_scale(
            m.scaling[0] as f32,
            m.scaling[1] as f32,
            m.scaling[2] as f32,
        );
        t * r * s
    };

    // Recursive node builder
    fn build_node(
        model_id: i64,
        model_map: &HashMap<i64, ModelInfo>,
        geom_map: &HashMap<i64, GeomData>,
        mat_id_to_index: &HashMap<i64, usize>,
        children_of: &HashMap<i64, Vec<i64>>,
        model_transform: &dyn Fn(&ModelInfo) -> Mat4,
        triangulate: &dyn Fn(&GeomData) -> TriMesh,
    ) -> Node {
        let model = &model_map[&model_id];
        let transformation = model_transform(model);

        let mut geometry = None;
        let mut material_index = None;
        let mut child_nodes = Vec::new();

        for &child_id in children_of.get(&model_id).unwrap_or(&Vec::new()) {
            if let Some(gd) = geom_map.get(&child_id) {
                geometry = Some(Geometry::Triangles(triangulate(gd)));
            } else if let Some(&mi) = mat_id_to_index.get(&child_id) {
                material_index = Some(mi);
            } else if model_map.contains_key(&child_id) {
                child_nodes.push(build_node(
                    child_id,
                    model_map,
                    geom_map,
                    mat_id_to_index,
                    children_of,
                    model_transform,
                    triangulate,
                ));
            }
        }

        Node {
            name: model.name.clone(),
            children: child_nodes,
            transformation,
            geometry,
            material_index,
            ..Default::default()
        }
    }

    // Root models: parent is 0 (scene root) or not another model
    let root_ids: Vec<i64> = model_map
        .keys()
        .filter(|&&id| match parent_of.get(&id) {
            Some(&pid) => pid == 0 || !model_map.contains_key(&pid),
            None => true,
        })
        .copied()
        .collect();

    let scene_children: Vec<Node> = root_ids
        .iter()
        .map(|&id| {
            build_node(
                id,
                &model_map,
                &geom_map,
                &mat_id_to_index,
                &children_of,
                &model_transform,
                &triangulate,
            )
        })
        .collect();

    let materials: Vec<PbrMaterial> = mat_list.into_iter().map(|(_, m)| m).collect();

    Ok(Scene {
        name: String::new(),
        children: scene_children,
        materials,
    })
}

fn fbx_attr_to_f64(attr: &fbxcel::low::v7400::AttributeValue) -> Option<f64> {
    attr.get_f64()
        .or_else(|| attr.get_f32().map(|v| v as f64))
        .or_else(|| attr.get_i64().map(|v| v as f64))
        .or_else(|| attr.get_i32().map(|v| v as f64))
}

fn fbx_get_layer_index(
    pv_idx: usize,
    vert_idx: usize,
    mapping: &str,
    reference: &str,
    indices: &[i32],
) -> usize {
    match (mapping, reference) {
        ("ByPolygonVertex", "Direct") => pv_idx,
        ("ByPolygonVertex", "IndexToDirect") => {
            indices.get(pv_idx).map(|&i| i as usize).unwrap_or(0)
        }
        ("ByVertex" | "ByVertice", "Direct") => vert_idx,
        ("ByVertex" | "ByVertice", "IndexToDirect") => {
            indices.get(vert_idx).map(|&i| i as usize).unwrap_or(0)
        }
        _ => pv_idx,
    }
}

#[cfg(test)]
mod test {
    use crate::Model;

    #[test]
    pub fn load_fbx() {
        let mut loaded = crate::io::load(&["test_data/Cube.fbx"]).unwrap();
        let model: Model = loaded.deserialize(".fbx").unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 1);
    }

    #[test]
    pub fn deserialize_fbx() {
        let model: Model = crate::io::RawAssets::new()
            .insert(
                "Cube.fbx",
                include_bytes!("../../test_data/Cube.fbx").to_vec(),
            )
            .deserialize("fbx")
            .unwrap();
        assert_eq!(model.geometries.len(), 1);
        assert_eq!(model.materials.len(), 1);
    }
}
