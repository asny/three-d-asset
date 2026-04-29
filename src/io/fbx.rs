use crate::{geometry::*, io::*, material::*, Error, KeyFrames, Node, Result, Scene};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn dependencies(raw_assets: &RawAssets, path: &PathBuf) -> HashSet<PathBuf> {
    let mut deps = HashSet::new();
    let base_path = path.parent().unwrap_or(Path::new(""));

    let Ok(bytes) = raw_assets.get(path) else {
        return deps;
    };

    use fbxcel::tree::any::AnyTree;
    let cursor = std::io::Cursor::new(bytes);
    let Ok(any_tree) = AnyTree::from_seekable_reader(cursor) else {
        return deps;
    };
    let AnyTree::V7400(_, tree, _) = any_tree else {
        return deps;
    };
    let root = tree.root();
    let Some(objects) = root.first_child_by_name("Objects") else {
        return deps;
    };

    for obj in objects.children() {
        if obj.name() != "Texture" && obj.name() != "Video" {
            continue;
        }
        if let Some(rel_path) = fbx_texture_filename(&obj) {
            let resolved = base_path.join(&rel_path);
            deps.insert(resolved);
        }
    }

    deps
}

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

    // --- Parse axis system from GlobalSettings ---
    let axis_conversion = {
        let (mut up_axis, mut up_sign) = (1i32, 1i32);
        let (mut front_axis, mut front_sign) = (2i32, 1i32);
        let (mut coord_sign,) = (1i32,);
        if let Some(gs) = root.first_child_by_name("GlobalSettings") {
            if let Some(props) = gs.first_child_by_name("Properties70") {
                for p in props.children_by_name("P") {
                    let a = p.attributes();
                    if a.len() > 4 {
                        let val = a[4..]
                            .iter()
                            .find_map(|v| {
                                v.get_i32()
                                    .or_else(|| v.get_i64().map(|v| v as i32))
                                    .or_else(|| v.get_f64().map(|v| v as i32))
                                    .or_else(|| v.get_f32().map(|v| v as i32))
                            })
                            .unwrap_or(0);
                        match a[0].get_string() {
                            Some("UpAxis") => up_axis = val,
                            Some("UpAxisSign") => up_sign = val,
                            Some("FrontAxis") => front_axis = val,
                            Some("FrontAxisSign") => front_sign = val,
                            Some("CoordAxisSign") => coord_sign = val,
                            _ => {}
                        }
                    }
                }
            }
        }
        fbx_axis_conversion(up_axis, up_sign, front_axis, front_sign, coord_sign)
    };

    // --- Build connection graph ---
    let mut children_of: HashMap<i64, Vec<i64>> = HashMap::new();
    // "OP" connections: (child_id, parent_id, property_name) — used for texture→material
    let mut op_connections: Vec<(i64, i64, String)> = Vec::new();
    for conn in connections.children_by_name("C") {
        let attrs = conn.attributes();
        if attrs.len() >= 3 {
            match attrs[0].get_string() {
                Some("OO") => {
                    if let (Some(child_id), Some(parent_id)) =
                        (attrs[1].get_i64(), attrs[2].get_i64())
                    {
                        children_of.entry(parent_id).or_default().push(child_id);
                    }
                }
                Some("OP") => {
                    if let (Some(child_id), Some(parent_id)) =
                        (attrs[1].get_i64(), attrs[2].get_i64())
                    {
                        let prop = attrs
                            .get(3)
                            .and_then(|a| a.get_string())
                            .unwrap_or("")
                            .to_string();
                        op_connections.push((child_id, parent_id, prop));
                    }
                }
                _ => {}
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
        // Transparency handling based on Blender/Three.js approach:
        // 1. opacity = 1 - TransparencyFactor
        // 2. If that gives exactly 0 or 1, fall back to Opacity property
        // 3. If Opacity is also missing, default to fully opaque
        let mut opacity = 1.0
            - fbx_props_f64(&obj, "TransparencyFactor")
                .and_then(|v| v.first().copied())
                .unwrap_or(0.0);
        if opacity == 1.0 || opacity == 0.0 {
            opacity = fbx_props_f64(&obj, "Opacity")
                .and_then(|v| v.first().copied())
                .unwrap_or(1.0);
        }
        mat.albedo.a = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
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

    // --- Parse textures and assign to materials ---
    let base_path = path.parent().unwrap_or(Path::new(""));
    let mut texture_paths: HashMap<i64, String> = HashMap::new();
    for obj in objects.children() {
        if obj.name() != "Texture" {
            continue;
        }
        let Some(id) = obj.attributes().first().and_then(|a| a.get_i64()) else {
            continue;
        };
        if let Some(filename) = fbx_texture_filename(&obj) {
            texture_paths.insert(id, filename);
        }
    }

    // Resolve OP connections: texture → material with property name
    for (texture_id, material_id, prop_name) in &op_connections {
        let Some(&mat_idx) = mat_id_to_index.get(material_id) else {
            continue;
        };
        let Some(filename) = texture_paths.get(texture_id) else {
            continue;
        };
        let texture_path = base_path.join(filename);
        let Ok(texture) = raw_assets.deserialize(&texture_path) else {
            continue;
        };
        let mat = &mut mat_list[mat_idx].1;
        match prop_name.as_str() {
            "DiffuseColor" | "Maya|baseColor" | "BaseColor" => {
                mat.albedo_texture = Some(texture);
            }
            "NormalMap" | "Bump" | "Maya|normalCamera" => {
                mat.normal_texture = Some(texture);
            }
            "EmissiveColor" | "EmissiveFactor" | "Maya|emissionColor" => {
                mat.emissive_texture = Some(texture);
            }
            "ShininessExponent"
            | "SpecularColor"
            | "ReflectionColor"
            | "Maya|metalness"
            | "Maya|specularRoughness"
            | "Metalness"
            | "Roughness" => {
                mat.metallic_roughness_texture = Some(texture);
            }
            "AmbientOcclusion" | "Maya|TEX_ao_map" => {
                mat.occlusion_texture = Some(texture);
            }
            "TransparentColor" | "TransparencyFactor" | "Maya|opacity" => {
                mat.albedo_texture.get_or_insert(texture);
            }
            _ => {}
        }
    }

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
                            uvs_out.push(vec2(
                                u.data[idx * 2] as f32,
                                1.0 - u.data[idx * 2 + 1] as f32,
                            ));
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
        pre_rotation: [f64; 3],
        post_rotation: [f64; 3],
        rotation_offset: [f64; 3],
        rotation_pivot: [f64; 3],
        scaling: [f64; 3],
        scaling_offset: [f64; 3],
        scaling_pivot: [f64; 3],
        geometric_translation: [f64; 3],
        geometric_rotation: [f64; 3],
        geometric_scaling: [f64; 3],
        rotation_order: u8,
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
            pre_rotation: [0.0; 3],
            post_rotation: [0.0; 3],
            rotation_offset: [0.0; 3],
            rotation_pivot: [0.0; 3],
            scaling: [1.0, 1.0, 1.0],
            scaling_offset: [0.0; 3],
            scaling_pivot: [0.0; 3],
            geometric_translation: [0.0; 3],
            geometric_rotation: [0.0; 3],
            geometric_scaling: [1.0, 1.0, 1.0],
            rotation_order: 0,
        };
        let set_vec3 = |dst: &mut [f64; 3], v: &[f64]| {
            if v.len() >= 3 {
                *dst = [v[0], v[1], v[2]];
            }
        };
        if let Some(v) = fbx_props_f64(&obj, "Lcl Translation") {
            set_vec3(&mut info.translation, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "Lcl Rotation") {
            set_vec3(&mut info.rotation, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "PreRotation") {
            set_vec3(&mut info.pre_rotation, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "PostRotation") {
            set_vec3(&mut info.post_rotation, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "RotationOffset") {
            set_vec3(&mut info.rotation_offset, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "RotationPivot") {
            set_vec3(&mut info.rotation_pivot, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "Lcl Scaling") {
            set_vec3(&mut info.scaling, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "ScalingOffset") {
            set_vec3(&mut info.scaling_offset, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "ScalingPivot") {
            set_vec3(&mut info.scaling_pivot, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "GeometricTranslation") {
            set_vec3(&mut info.geometric_translation, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "GeometricRotation") {
            set_vec3(&mut info.geometric_rotation, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "GeometricScaling") {
            set_vec3(&mut info.geometric_scaling, &v);
        }
        if let Some(v) = fbx_props_f64(&obj, "RotationOrder") {
            info.rotation_order = v[0] as u8;
        }
        model_map.insert(id, info);
    }

    let model_transform = |m: &ModelInfo| -> Mat4 {
        let t = Mat4::from_translation(vec3(
            m.translation[0] as f32,
            m.translation[1] as f32,
            m.translation[2] as f32,
        ));
        let r_off = Mat4::from_translation(vec3(
            m.rotation_offset[0] as f32,
            m.rotation_offset[1] as f32,
            m.rotation_offset[2] as f32,
        ));
        let r_piv = Mat4::from_translation(vec3(
            m.rotation_pivot[0] as f32,
            m.rotation_pivot[1] as f32,
            m.rotation_pivot[2] as f32,
        ));
        let r_piv_inv = Mat4::from_translation(vec3(
            -m.rotation_pivot[0] as f32,
            -m.rotation_pivot[1] as f32,
            -m.rotation_pivot[2] as f32,
        ));
        let r_pre = fbx_euler_to_matrix(&m.pre_rotation, 0);
        let r_local = fbx_euler_to_matrix(&m.rotation, m.rotation_order);
        let r_post_inv = fbx_euler_to_matrix(&m.post_rotation, 0)
            .invert()
            .unwrap_or(Mat4::identity());
        let s_off = Mat4::from_translation(vec3(
            m.scaling_offset[0] as f32,
            m.scaling_offset[1] as f32,
            m.scaling_offset[2] as f32,
        ));
        let s_piv = Mat4::from_translation(vec3(
            m.scaling_pivot[0] as f32,
            m.scaling_pivot[1] as f32,
            m.scaling_pivot[2] as f32,
        ));
        let s_piv_inv = Mat4::from_translation(vec3(
            -m.scaling_pivot[0] as f32,
            -m.scaling_pivot[1] as f32,
            -m.scaling_pivot[2] as f32,
        ));
        let s = Mat4::from_nonuniform_scale(
            m.scaling[0] as f32,
            m.scaling[1] as f32,
            m.scaling[2] as f32,
        );
        // Full FBX local transform:
        // T * Roff * Rp * Rpre * R * Rpost^-1 * Rp^-1 * Soff * Sp * S * Sp^-1
        t * r_off * r_piv * r_pre * r_local * r_post_inv * r_piv_inv * s_off * s_piv * s * s_piv_inv
    };

    let geometric_transform = |m: &ModelInfo| -> Mat4 {
        let gt = Mat4::from_translation(vec3(
            m.geometric_translation[0] as f32,
            m.geometric_translation[1] as f32,
            m.geometric_translation[2] as f32,
        ));
        let gr = fbx_euler_to_matrix(&m.geometric_rotation, 0);
        let gs = Mat4::from_nonuniform_scale(
            m.geometric_scaling[0] as f32,
            m.geometric_scaling[1] as f32,
            m.geometric_scaling[2] as f32,
        );
        gt * gr * gs
    };

    // --- Parse animations ---
    const FBX_TICKS_PER_SECOND: f64 = 46186158000.0;

    struct AnimCurve {
        times: Vec<f32>,
        values: Vec<f32>,
    }
    // AnimationCurve objects: id → curve data
    let mut anim_curves: HashMap<i64, AnimCurve> = HashMap::new();
    for obj in objects.children() {
        if obj.name() != "AnimationCurve" {
            continue;
        }
        let Some(id) = obj.attributes().first().and_then(|a| a.get_i64()) else {
            continue;
        };
        let times: Vec<f32> = obj
            .first_child_by_name("KeyTime")
            .and_then(|n| n.attributes().first()?.get_arr_i64().map(|v| v.to_vec()))
            .unwrap_or_default()
            .iter()
            .map(|&t| (t as f64 / FBX_TICKS_PER_SECOND) as f32)
            .collect();
        let values: Vec<f32> = obj
            .first_child_by_name("KeyValueFloat")
            .and_then(|n| {
                let attr = n.attributes().first()?;
                attr.get_arr_f32().map(|v| v.to_vec()).or_else(|| {
                    attr.get_arr_f64()
                        .map(|v| v.iter().map(|&x| x as f32).collect())
                })
            })
            .unwrap_or_default();
        if times.is_empty() || values.is_empty() {
            continue;
        }
        anim_curves.insert(id, AnimCurve { times, values });
    }

    // AnimationCurveNode objects: id → property type (T/R/S)
    #[derive(Clone, Copy, PartialEq)]
    enum AnimProp {
        Translation,
        Rotation,
        Scaling,
    }
    let mut curve_node_props: HashMap<i64, AnimProp> = HashMap::new();
    for obj in objects.children() {
        if obj.name() != "AnimationCurveNode" {
            continue;
        }
        let Some(id) = obj.attributes().first().and_then(|a| a.get_i64()) else {
            continue;
        };
        let attr_name = obj
            .attributes()
            .get(1)
            .and_then(|a| a.get_string())
            .unwrap_or("");
        let name_part = attr_name.split('\0').next().unwrap_or("");
        let prop = match name_part {
            "T" | "AnimCurveNode::T" => Some(AnimProp::Translation),
            "R" | "AnimCurveNode::R" => Some(AnimProp::Rotation),
            "S" | "AnimCurveNode::S" => Some(AnimProp::Scaling),
            n if n.contains("Translation") || n.contains("Translate") => {
                Some(AnimProp::Translation)
            }
            n if n.contains("Rotation") || n.contains("Rotate") => Some(AnimProp::Rotation),
            n if n.contains("Scaling") || n.contains("Scale") => Some(AnimProp::Scaling),
            _ => None,
        };
        if let Some(p) = prop {
            curve_node_props.insert(id, p);
        }
    }

    // Also infer AnimCurveNode type from OP connections to models
    // (in case the attribute name didn't match)
    for &(child_id, parent_id, ref prop) in &op_connections {
        if model_map.contains_key(&parent_id) && !curve_node_props.contains_key(&child_id) {
            let inferred = match prop.as_str() {
                p if p.contains("Translation") || p.contains("Translate") => {
                    Some(AnimProp::Translation)
                }
                p if p.contains("Rotation") || p.contains("Rotate") => Some(AnimProp::Rotation),
                p if p.contains("Scaling") || p.contains("Scale") => Some(AnimProp::Scaling),
                _ => None,
            };
            if let Some(p) = inferred {
                curve_node_props.insert(child_id, p);
            }
        }
    }

    // AnimationStack objects: id → name
    let mut anim_stack_names: HashMap<i64, String> = HashMap::new();
    for obj in objects.children() {
        if obj.name() != "AnimationStack" {
            continue;
        }
        let attrs = obj.attributes();
        let Some(id) = attrs.first().and_then(|a| a.get_i64()) else {
            continue;
        };
        anim_stack_names.insert(id, fbx_name(attrs));
    }

    // Build: curve_node_id → { x_curve, y_curve, z_curve }
    struct CurveNodeChannels {
        x: Option<i64>,
        y: Option<i64>,
        z: Option<i64>,
    }
    let mut curve_node_channels: HashMap<i64, CurveNodeChannels> = HashMap::new();
    // Build: curve_node_id → model_id (via OP connections with "Lcl ..." property)
    let mut curve_node_to_model: HashMap<i64, i64> = HashMap::new();

    for &(child_id, parent_id, ref prop) in &op_connections {
        if curve_node_props.contains_key(&parent_id) && anim_curves.contains_key(&child_id) {
            // AnimCurve → AnimCurveNode connection (channel d|X, d|Y, d|Z)
            let channels = curve_node_channels
                .entry(parent_id)
                .or_insert(CurveNodeChannels {
                    x: None,
                    y: None,
                    z: None,
                });
            if prop.contains('X') {
                channels.x = Some(child_id);
            } else if prop.contains('Y') {
                channels.y = Some(child_id);
            } else if prop.contains('Z') {
                channels.z = Some(child_id);
            }
        }
        if curve_node_props.contains_key(&child_id) && model_map.contains_key(&parent_id) {
            // AnimCurveNode → Model connection
            curve_node_to_model.insert(child_id, parent_id);
        }
    }

    // Determine which AnimationStack each curve node belongs to (via AnimationLayer)
    // AnimCurveNode → AnimLayer (OO, in children_of)
    // AnimLayer → AnimStack (OO, in children_of)
    let mut layer_to_stack: HashMap<i64, i64> = HashMap::new();
    for (&stack_id, _) in &anim_stack_names {
        if let Some(layer_ids) = children_of.get(&stack_id) {
            for &layer_id in layer_ids {
                layer_to_stack.insert(layer_id, stack_id);
            }
        }
    }
    let mut curve_node_to_stack: HashMap<i64, i64> = HashMap::new();
    for (&layer_id, &stack_id) in &layer_to_stack {
        if let Some(cn_ids) = children_of.get(&layer_id) {
            for &cn_id in cn_ids {
                if curve_node_props.contains_key(&cn_id) {
                    curve_node_to_stack.insert(cn_id, stack_id);
                }
            }
        }
    }

    // Assemble per-model animations: model_id → { stack_name → (T curves, R curves, S curves) }
    struct ModelAnimData {
        t_channels: Option<CurveNodeChannels>,
        r_channels: Option<CurveNodeChannels>,
        s_channels: Option<CurveNodeChannels>,
    }
    let mut model_anims: HashMap<i64, HashMap<i64, ModelAnimData>> = HashMap::new();
    for (&cn_id, &prop) in &curve_node_props {
        let Some(&model_id) = curve_node_to_model.get(&cn_id) else {
            continue;
        };
        let stack_id = curve_node_to_stack.get(&cn_id).copied().unwrap_or(0);
        let Some(channels) = curve_node_channels.remove(&cn_id) else {
            continue;
        };
        let anim_data = model_anims
            .entry(model_id)
            .or_default()
            .entry(stack_id)
            .or_insert(ModelAnimData {
                t_channels: None,
                r_channels: None,
                s_channels: None,
            });
        match prop {
            AnimProp::Translation => anim_data.t_channels = Some(channels),
            AnimProp::Rotation => anim_data.r_channels = Some(channels),
            AnimProp::Scaling => anim_data.s_channels = Some(channels),
        }
    }

    // Helper: merge time arrays and sample curves at unified times
    let build_keyframes = |anim_data: ModelAnimData, model: &ModelInfo| -> KeyFrames {
        // Collect all unique times
        let mut all_times: Vec<f32> = Vec::new();
        let collect_times = |channels: &Option<CurveNodeChannels>, times: &mut Vec<f32>| {
            if let Some(ch) = channels {
                for curve_id in [ch.x, ch.y, ch.z].into_iter().flatten() {
                    if let Some(curve) = anim_curves.get(&curve_id) {
                        times.extend_from_slice(&curve.times);
                    }
                }
            }
        };
        collect_times(&anim_data.t_channels, &mut all_times);
        collect_times(&anim_data.r_channels, &mut all_times);
        collect_times(&anim_data.s_channels, &mut all_times);
        all_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        all_times.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

        if all_times.is_empty() {
            return KeyFrames::default();
        }

        let sample_at = |curve_id: Option<i64>, time: f32, default: f32| -> f32 {
            let Some(id) = curve_id else {
                return default;
            };
            let Some(curve) = anim_curves.get(&id) else {
                return default;
            };
            if curve.times.is_empty() {
                return default;
            }
            if time <= curve.times[0] {
                return *curve.values.first().unwrap_or(&default);
            }
            if time >= *curve.times.last().unwrap() {
                return *curve.values.last().unwrap_or(&default);
            }
            // Linear interpolation
            let pos = curve
                .times
                .partition_point(|&t| t < time)
                .min(curve.times.len() - 1);
            let i = pos.saturating_sub(1);
            let t0 = curve.times[i];
            let t1 = curve.times[pos];
            let v0 = curve.values.get(i).copied().unwrap_or(default);
            let v1 = curve.values.get(pos).copied().unwrap_or(default);
            if (t1 - t0).abs() < 1e-10 {
                v0
            } else {
                let alpha = (time - t0) / (t1 - t0);
                v0 + alpha * (v1 - v0)
            }
        };

        let translations = anim_data.t_channels.as_ref().map(|ch| {
            all_times
                .iter()
                .map(|&t| {
                    vec3(
                        sample_at(ch.x, t, model.translation[0] as f32),
                        sample_at(ch.y, t, model.translation[1] as f32),
                        sample_at(ch.z, t, model.translation[2] as f32),
                    )
                })
                .collect()
        });

        let rotations = anim_data.r_channels.as_ref().map(|ch| {
            all_times
                .iter()
                .map(|&t| {
                    let rx = sample_at(ch.x, t, model.rotation[0] as f32);
                    let ry = sample_at(ch.y, t, model.rotation[1] as f32);
                    let rz = sample_at(ch.z, t, model.rotation[2] as f32);
                    let euler = [rx as f64, ry as f64, rz as f64];
                    fbx_euler_to_quat(&euler, model.rotation_order)
                })
                .collect()
        });

        let scales = anim_data.s_channels.as_ref().map(|ch| {
            all_times
                .iter()
                .map(|&t| {
                    vec3(
                        sample_at(ch.x, t, model.scaling[0] as f32),
                        sample_at(ch.y, t, model.scaling[1] as f32),
                        sample_at(ch.z, t, model.scaling[2] as f32),
                    )
                })
                .collect()
        });

        KeyFrames {
            interpolation: Interpolation::Linear,
            loop_time: None,
            times: all_times,
            translations,
            rotations,
            scales,
            weights: None,
        }
    };

    // Build final animations map: model_id → Vec<(name, KeyFrames)>
    let mut node_animations: HashMap<i64, Vec<(Option<String>, KeyFrames)>> = HashMap::new();
    for (model_id, stacks) in model_anims {
        let model = match model_map.get(&model_id) {
            Some(m) => m,
            None => continue,
        };
        let anims = node_animations.entry(model_id).or_default();
        for (stack_id, anim_data) in stacks {
            let name = anim_stack_names.get(&stack_id).cloned();
            let key_frames = build_keyframes(anim_data, model);
            if !key_frames.times.is_empty() {
                anims.push((name, key_frames));
            }
        }
    }

    fn build_node(
        model_id: i64,
        model_map: &HashMap<i64, ModelInfo>,
        geom_map: &HashMap<i64, GeomData>,
        mat_id_to_index: &HashMap<i64, usize>,
        children_of: &HashMap<i64, Vec<i64>>,
        node_animations: &HashMap<i64, Vec<(Option<String>, KeyFrames)>>,
        model_transform: &dyn Fn(&ModelInfo) -> Mat4,
        geometric_transform: &dyn Fn(&ModelInfo) -> Mat4,
        triangulate: &dyn Fn(&GeomData) -> TriMesh,
        visited: &mut HashSet<i64>,
    ) -> Node {
        visited.insert(model_id);
        let model = &model_map[&model_id];
        let transformation = model_transform(model);
        let geo_transform = geometric_transform(model);

        let mut geometry = None;
        let mut material_index = None;
        let mut child_nodes = Vec::new();

        for &child_id in children_of.get(&model_id).unwrap_or(&Vec::new()) {
            if let Some(gd) = geom_map.get(&child_id) {
                let mut mesh = triangulate(gd);
                if geo_transform != Mat4::identity() {
                    apply_transform_to_mesh(&mut mesh, &geo_transform);
                }
                geometry = Some(Geometry::Triangles(mesh));
            } else if let Some(&mi) = mat_id_to_index.get(&child_id) {
                material_index = Some(mi);
            } else if model_map.contains_key(&child_id) && !visited.contains(&child_id) {
                child_nodes.push(build_node(
                    child_id,
                    model_map,
                    geom_map,
                    mat_id_to_index,
                    children_of,
                    node_animations,
                    model_transform,
                    geometric_transform,
                    triangulate,
                    visited,
                ));
            }
        }

        let animations = node_animations.get(&model_id).cloned().unwrap_or_default();

        Node {
            name: model.name.clone(),
            children: child_nodes,
            transformation,
            animations,
            geometry,
            material_index,
        }
    }

    // Walk from scene root (id 0) through the connection graph
    let mut visited: HashSet<i64> = HashSet::new();
    let mut scene_children: Vec<Node> = Vec::new();

    for &id in children_of.get(&0).unwrap_or(&Vec::new()) {
        if model_map.contains_key(&id) && !visited.contains(&id) {
            let mut node = build_node(
                id,
                &model_map,
                &geom_map,
                &mat_id_to_index,
                &children_of,
                &node_animations,
                &model_transform,
                &geometric_transform,
                &triangulate,
                &mut visited,
            );
            node.transformation = axis_conversion * node.transformation;
            scene_children.push(node);
        }
    }

    // Include orphaned models not reachable from the scene root
    let orphans: Vec<i64> = model_map
        .keys()
        .copied()
        .filter(|id| !visited.contains(id))
        .collect();
    for id in orphans {
        let mut node = build_node(
            id,
            &model_map,
            &geom_map,
            &mat_id_to_index,
            &children_of,
            &node_animations,
            &model_transform,
            &geometric_transform,
            &triangulate,
            &mut visited,
        );
        node.transformation = axis_conversion * node.transformation;
        scene_children.push(node);
    }

    let materials: Vec<PbrMaterial> = mat_list.into_iter().map(|(_, m)| m).collect();

    Ok(Scene {
        name: String::new(),
        children: scene_children,
        materials,
    })
}

/// Extract the relative filename from a Texture or Video FBX object.
/// Prefers "RelativeFilename" child, falls back to "FileName", strips directory prefix heuristics.
fn fbx_texture_filename(node: &fbxcel::tree::v7400::NodeHandle) -> Option<String> {
    let get_child_string = |name: &str| -> Option<String> {
        node.first_child_by_name(name)?
            .attributes()
            .first()?
            .get_string()
            .map(|s| s.to_string())
    };

    let raw = get_child_string("RelativeFilename").or_else(|| get_child_string("FileName"))?;

    if raw.is_empty() {
        return None;
    }

    // Normalize backslashes to forward slashes
    let normalized = raw.replace('\\', "/");
    // Take just the filename portion if it's an absolute path
    let path = Path::new("tex").join(Path::new(&normalized).file_name()?);
    Some(path.to_str()?.to_string())
}

fn fbx_attr_to_f64(attr: &fbxcel::low::v7400::AttributeValue) -> Option<f64> {
    attr.get_f64()
        .or_else(|| attr.get_f32().map(|v| v as f64))
        .or_else(|| attr.get_i64().map(|v| v as f64))
        .or_else(|| attr.get_i32().map(|v| v as f64))
}

/// Compose an Euler rotation matrix respecting the FBX rotation order enum.
/// PreRotation always uses order 0 (XYZ) per the FBX spec.
fn fbx_euler_to_matrix(degrees: &[f64; 3], rotation_order: u8) -> Mat4 {
    let rx = Mat4::from_angle_x(Rad((degrees[0] as f32).to_radians()));
    let ry = Mat4::from_angle_y(Rad((degrees[1] as f32).to_radians()));
    let rz = Mat4::from_angle_z(Rad((degrees[2] as f32).to_radians()));
    match rotation_order {
        0 => rz * ry * rx, // eEulerXYZ  (intrinsic X→Y→Z = extrinsic Z·Y·X)
        1 => ry * rz * rx, // eEulerXZY
        2 => rx * rz * ry, // eEulerYZX
        3 => rz * rx * ry, // eEulerYXZ
        4 => ry * rx * rz, // eEulerZXY
        5 => rx * ry * rz, // eEulerZYX
        _ => rz * ry * rx, // fallback to XYZ
    }
}

/// Convert Euler angles (degrees) to a quaternion, respecting FBX rotation order.
fn fbx_euler_to_quat(degrees: &[f64; 3], rotation_order: u8) -> Quat {
    let mat = fbx_euler_to_matrix(degrees, rotation_order);
    let rot3 = Mat3::from_cols(mat.x.truncate(), mat.y.truncate(), mat.z.truncate());
    Quat::from(rot3)
}

/// Build a conversion matrix from the FBX file's axis system to OpenGL (Y-up, right-handed).
///
/// Each FBX axis maps to exactly one OpenGL axis:
///   FBX coord_axis → OpenGL X (right)
///   FBX up_axis    → OpenGL Y (up)
///   FBX front_axis → OpenGL Z (toward viewer)
fn fbx_axis_conversion(
    up_axis: i32,
    up_sign: i32,
    front_axis: i32,
    front_sign: i32,
    coord_sign: i32,
) -> Mat4 {
    let coord_axis = (3 - up_axis - front_axis) as usize;
    let up_axis = up_axis as usize;
    let front_axis = front_axis as usize;

    let mut cols = [[0.0f32; 4]; 4];
    cols[coord_axis][0] = coord_sign as f32;
    cols[up_axis][1] = up_sign as f32;
    cols[front_axis][2] = front_sign as f32;
    cols[3][3] = 1.0;

    Mat4::new(
        cols[0][0], cols[0][1], cols[0][2], cols[0][3], cols[1][0], cols[1][1], cols[1][2],
        cols[1][3], cols[2][0], cols[2][1], cols[2][2], cols[2][3], cols[3][0], cols[3][1],
        cols[3][2], cols[3][3],
    )
}

/// Apply a transformation matrix to all positions and normals in a mesh.
/// Geometric transforms in FBX affect only the node's content, not its children.
fn apply_transform_to_mesh(mesh: &mut TriMesh, transform: &Mat4) {
    let normal_matrix = transform.invert().unwrap_or(Mat4::identity()).transpose();
    match &mut mesh.positions {
        Positions::F32(ref mut positions) => {
            for p in positions.iter_mut() {
                let v = *transform * vec4(p.x, p.y, p.z, 1.0);
                *p = vec3(v.x, v.y, v.z);
            }
        }
        Positions::F64(ref mut positions) => {
            for p in positions.iter_mut() {
                let v = *transform * vec4(p.x as f32, p.y as f32, p.z as f32, 1.0);
                *p = vec3(v.x as f64, v.y as f64, v.z as f64);
            }
        }
    }
    if let Some(ref mut normals) = mesh.normals {
        for n in normals.iter_mut() {
            let v = normal_matrix * vec4(n.x, n.y, n.z, 0.0);
            let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
            if len > 0.0 {
                *n = vec3(v.x / len, v.y / len, v.z / len);
            }
        }
    }
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
