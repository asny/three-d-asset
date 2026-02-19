use std::path::PathBuf;

use crate::{
    geometry::*,
    io::RawAssets,
    material::*,
    prelude::{Mat4, Srgba},
    texture::*,
    Node, Result, Scene,
};

/// Convert a 3MF 4x3 affine transform (12 floats, row-major) to a cgmath Mat4.
///
/// 3MF layout: `[m00 m01 m02 m10 m11 m12 m20 m21 m22 tx ty tz]`
///
/// This represents the matrix:
/// ```text
/// | m00 m01 m02 0 |
/// | m10 m11 m12 0 |
/// | m20 m21 m22 0 |
/// | tx  ty  tz  1 |
/// ```
///
/// cgmath `Matrix4` is column-major, so we fill column by column.
fn transform_to_mat4(t: &[f64; 12]) -> Mat4 {
    #[rustfmt::skip]
    let m = Mat4::new(
        t[0] as f32, t[3] as f32, t[6] as f32, t[9]  as f32,
        t[1] as f32, t[4] as f32, t[7] as f32, t[10] as f32,
        t[2] as f32, t[5] as f32, t[8] as f32, t[11] as f32,
        0.0,         0.0,         0.0,         1.0,
    );
    m
}

/// Convert a cgmath Mat4 back to a 3MF 4x3 affine transform (12 floats, row-major).
fn mat4_to_transform(m: &Mat4) -> [f64; 12] {
    [
        m.x.x as f64,
        m.x.y as f64,
        m.x.z as f64,
        m.y.x as f64,
        m.y.y as f64,
        m.y.z as f64,
        m.z.x as f64,
        m.z.y as f64,
        m.z.z as f64,
        m.w.x as f64,
        m.w.y as f64,
        m.w.z as f64,
    ]
}

/// Context built during deserialization to map 3MF resource IDs to scene materials.
struct MaterialContext {
    /// Scene-level materials collected so far.
    materials: Vec<PbrMaterial>,
    /// Maps (base_material_group_id, material_index_within_group) → scene material index.
    base_material_map: std::collections::HashMap<(usize, usize), usize>,
    /// Maps texture2d_group_id → (scene material index, Texture2DGroup ref index).
    /// The scene material already has its albedo_texture set.
    texture_group_map: std::collections::HashMap<usize, usize>,
}

pub fn deserialize_3mf(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let bytes = raw_assets.remove(path)?;

    // We need two passes over the bytes:
    // 1) Parse the Model (XML) to get objects, materials, texture group metadata
    // 2) Open as a Package to read texture image bytes from the ZIP
    let model = lib3mf::Model::from_reader(std::io::Cursor::new(&bytes))?;

    // Open the ZIP package so we can read texture image data
    let mut package = lib3mf::opc::Package::open(std::io::Cursor::new(&bytes))?;

    let mut ctx = MaterialContext {
        materials: Vec::new(),
        base_material_map: std::collections::HashMap::new(),
        texture_group_map: std::collections::HashMap::new(),
    };

    // --- Base material groups (standard 3MF material colors) ---
    for group in &model.resources.base_material_groups {
        for (idx, bm) in group.materials.iter().enumerate() {
            let (r, g, b, a) = bm.displaycolor;
            let scene_idx = ctx.materials.len();
            ctx.materials.push(PbrMaterial {
                name: bm.name.clone(),
                albedo: Srgba::new(r, g, b, a),
                ..Default::default()
            });
            ctx.base_material_map.insert((group.id, idx), scene_idx);
        }
    }

    // --- Legacy Material resources (keep for backward compat) ---
    for mat in &model.resources.materials {
        let albedo = match mat.color {
            Some((r, g, b, a)) => Srgba::new(r, g, b, a),
            None => Srgba::WHITE,
        };
        ctx.materials.push(PbrMaterial {
            name: mat.name.clone().unwrap_or_default(),
            albedo,
            ..Default::default()
        });
    }

    // --- Texture2D resources → decode images and create materials ---
    // Build a map from texture2d resource id → decoded Texture2D
    let mut texture_map: std::collections::HashMap<usize, crate::Texture2D> =
        std::collections::HashMap::new();
    for tex_res in &model.resources.texture2d_resources {
        // Read texture image bytes from the ZIP
        let tex_path = if tex_res.path.starts_with('/') {
            tex_res.path[1..].to_string()
        } else {
            tex_res.path.clone()
        };
        if let Ok(img_bytes) = package.get_file_binary(&tex_path) {
            let mut tex = decode_texture_image(&tex_path, &img_bytes)?;
            // Map 3MF tile/filter settings
            tex.wrap_s = convert_tile_style(&tex_res.tilestyleu);
            tex.wrap_t = convert_tile_style(&tex_res.tilestylev);
            let interp = convert_filter_mode(&tex_res.filter);
            tex.min_filter = interp;
            tex.mag_filter = interp;
            texture_map.insert(tex_res.id, tex);
        }
    }

    // --- Texture2DGroup → create a material per group with albedo_texture ---
    for tex_group in &model.resources.texture2d_groups {
        if let Some(tex) = texture_map.get(&tex_group.texid) {
            let scene_idx = ctx.materials.len();
            ctx.materials.push(PbrMaterial {
                name: format!("texture_{}", tex_group.id),
                albedo_texture: Some(tex.clone()),
                ..Default::default()
            });
            ctx.texture_group_map.insert(tex_group.id, scene_idx);
        }
    }

    // Build an index of objects by ID for fast lookup
    let objects: std::collections::HashMap<usize, &lib3mf::Object> = model
        .resources
        .objects
        .iter()
        .map(|obj| (obj.id, obj))
        .collect();

    // Walk build items → resolve objects → produce nodes
    let mut nodes = Vec::new();
    for item in &model.build.items {
        let item_transform = item
            .transform
            .as_ref()
            .map(transform_to_mat4)
            .unwrap_or_else(Mat4::identity);

        if let Some(object) = objects.get(&item.objectid) {
            let mut children = resolve_object(object, &objects, &model, &ctx, item_transform);
            nodes.append(&mut children);
        }
    }

    Ok(Scene {
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("3mf_scene")
            .to_owned(),
        children: nodes,
        materials: ctx.materials,
    })
}

/// Decode texture image bytes into a three-d-asset Texture2D.
///
/// If the `image` feature is enabled, uses the image crate to decode PNG/JPEG.
/// Otherwise returns a 1×1 placeholder.
fn decode_texture_image(path: &str, bytes: &[u8]) -> Result<crate::Texture2D> {
    #[cfg(feature = "image")]
    {
        return super::img::deserialize_img(path, bytes);
    }
    #[cfg(not(feature = "image"))]
    {
        let _ = (path, bytes);
        Ok(crate::Texture2D::default())
    }
}

fn convert_tile_style(style: &lib3mf::TileStyle) -> Wrapping {
    match style {
        lib3mf::TileStyle::Wrap => Wrapping::Repeat,
        lib3mf::TileStyle::Mirror => Wrapping::MirroredRepeat,
        lib3mf::TileStyle::Clamp => Wrapping::ClampToEdge,
        lib3mf::TileStyle::None => Wrapping::ClampToEdge,
    }
}

fn convert_filter_mode(mode: &lib3mf::FilterMode) -> Interpolation {
    match mode {
        lib3mf::FilterMode::Auto | lib3mf::FilterMode::Linear => Interpolation::Linear,
        lib3mf::FilterMode::Nearest => Interpolation::Nearest,
    }
}

/// Recursively resolve an object into scene nodes, accumulating transforms.
///
/// - If the object has a mesh, create a single Node with that mesh and the accumulated transform.
/// - If the object has components, recurse into each component, multiplying the component's
///   transform onto the accumulated parent transform.
fn resolve_object(
    object: &lib3mf::Object,
    objects: &std::collections::HashMap<usize, &lib3mf::Object>,
    model: &lib3mf::Model,
    ctx: &MaterialContext,
    accumulated_transform: Mat4,
) -> Vec<Node> {
    let mut result = Vec::new();

    if let Some(ref mesh) = object.mesh {
        let positions: Vec<Vector3<f32>> = mesh
            .vertices
            .iter()
            .map(|v| Vector3::new(v.x as f32, v.y as f32, v.z as f32))
            .collect();

        let mut indices = Vec::with_capacity(mesh.triangles.len() * 3);
        let mut normals = Vec::with_capacity(mesh.triangles.len());
        for tri in &mesh.triangles {
            indices.push(tri.v1 as u32);
            indices.push(tri.v2 as u32);
            indices.push(tri.v3 as u32);

            let p0 = &positions[tri.v1];
            let p1 = &positions[tri.v2];
            let p2 = &positions[tri.v3];
            let edge1 = p1 - p0;
            let edge2 = p2 - p0;
            normals.push(edge1.cross(edge2).normalize());
        }

        let (material_index, colors, uvs) =
            extract_material_info(object, mesh, model, ctx, &positions, &indices);

        let tri_mesh = TriMesh {
            positions: Positions::F32(positions),
            indices: Indices::U32(indices),
            normals: Some(normals),
            tangents: None,
            uvs,
            colors,
        };

        let name = object
            .name
            .clone()
            .unwrap_or_else(|| format!("object_{}", object.id));

        result.push(Node {
            name,
            geometry: Some(Geometry::Triangles(tri_mesh)),
            material_index,
            transformation: accumulated_transform,
            ..Default::default()
        });
    }

    // Recurse into components (assemblies referencing other objects)
    if !object.components.is_empty() {
        for component in &object.components {
            let component_transform = component
                .transform
                .as_ref()
                .map(transform_to_mat4)
                .unwrap_or_else(Mat4::identity);

            let combined = accumulated_transform * component_transform;

            if let Some(child_object) = objects.get(&component.objectid) {
                let mut children = resolve_object(child_object, objects, model, ctx, combined);
                result.append(&mut children);
            }
        }
    }

    result
}

/// Extract material index, per-vertex colors, and UV coordinates from triangle properties.
fn extract_material_info(
    object: &lib3mf::Object,
    mesh: &lib3mf::Mesh,
    model: &lib3mf::Model,
    ctx: &MaterialContext,
    positions: &[Vector3<f32>],
    _indices: &[u32],
) -> (Option<usize>, Option<Vec<Srgba>>, Option<Vec<Vec2>>) {
    let vertex_count = positions.len();

    let first_tri = match mesh.triangles.first() {
        Some(t) => t,
        None => return (None, None, None),
    };

    let pid = match first_tri.pid.or(object.pid) {
        Some(pid) => pid,
        None => return (None, None, None),
    };

    // --- BaseMaterialGroup: per-triangle material selection via pindex/p1 ---
    if let Some(bmg) = model
        .resources
        .base_material_groups
        .iter()
        .find(|g| g.id == pid)
    {
        // Use the pindex from the first triangle (or object default) to pick the material.
        // In 3MF, all triangles in an object typically reference the same base material group,
        // but individual triangles can select different materials via pindex.
        // For simplicity, we use the first triangle's selection.
        let pindex = first_tri
            .pindex
            .or(first_tri.p1)
            .or(object.pindex)
            .unwrap_or(0);
        let _ = bmg; // we only needed to confirm the group exists
        if let Some(&scene_idx) = ctx.base_material_map.get(&(pid, pindex)) {
            return (Some(scene_idx), None, None);
        }
    }

    // --- Texture2DGroup: per-vertex UV coordinates ---
    if let Some(tex_group) = model
        .resources
        .texture2d_groups
        .iter()
        .find(|g| g.id == pid)
    {
        if let Some(&scene_mat_idx) = ctx.texture_group_map.get(&pid) {
            // Extract per-vertex UVs from the triangle property indices
            let mut uvs = vec![Vec2::new(0.0, 0.0); vertex_count];
            for tri in &mesh.triangles {
                let p1 = tri.p1.unwrap_or(0);
                let p2 = tri.p2.unwrap_or(0);
                let p3 = tri.p3.unwrap_or(0);

                if p1 < tex_group.tex2coords.len() {
                    let tc = &tex_group.tex2coords[p1];
                    uvs[tri.v1] = Vec2::new(tc.u, tc.v);
                }
                if p2 < tex_group.tex2coords.len() {
                    let tc = &tex_group.tex2coords[p2];
                    uvs[tri.v2] = Vec2::new(tc.u, tc.v);
                }
                if p3 < tex_group.tex2coords.len() {
                    let tc = &tex_group.tex2coords[p3];
                    uvs[tri.v3] = Vec2::new(tc.u, tc.v);
                }
            }
            return (Some(scene_mat_idx), None, Some(uvs));
        }
    }

    // --- ColorGroup: per-vertex colors ---
    if let Some(color_group) = model.resources.color_groups.iter().find(|cg| cg.id == pid) {
        let mut colors = vec![Srgba::WHITE; vertex_count];
        for tri in &mesh.triangles {
            let p1 = tri.p1.unwrap_or(0);
            let p2 = tri.p2.unwrap_or(0);
            let p3 = tri.p3.unwrap_or(0);

            if p1 < color_group.colors.len() {
                let (r, g, b, a) = color_group.colors[p1];
                colors[tri.v1] = Srgba::new(r, g, b, a);
            }
            if p2 < color_group.colors.len() {
                let (r, g, b, a) = color_group.colors[p2];
                colors[tri.v2] = Srgba::new(r, g, b, a);
            }
            if p3 < color_group.colors.len() {
                let (r, g, b, a) = color_group.colors[p3];
                colors[tri.v3] = Srgba::new(r, g, b, a);
            }
        }
        return (None, Some(colors), None);
    }

    // --- Legacy Material resource ---
    if let Some(pos) = model.resources.materials.iter().position(|m| m.id == pid) {
        // Offset by the number of base materials already added
        let base_mat_count: usize = model
            .resources
            .base_material_groups
            .iter()
            .map(|g| g.materials.len())
            .sum();
        return (Some(base_mat_count + pos), None, None);
    }

    (None, None, None)
}

pub fn serialize_3mf(scene: &Scene) -> Result<Vec<u8>> {
    let mut model = lib3mf::Model::new();
    model.unit = "millimeter".to_string();

    // Convert materials to a BaseMaterialGroup (standard 3MF materials extension)
    let base_group_id: usize = 1;
    if !scene.materials.is_empty() {
        let mut group = lib3mf::BaseMaterialGroup::new(base_group_id);
        for mat in &scene.materials {
            group.materials.push(lib3mf::BaseMaterial::new(
                mat.name.clone(),
                (mat.albedo.r, mat.albedo.g, mat.albedo.b, mat.albedo.a),
            ));
        }
        model.resources.base_material_groups.push(group);
    }

    // Convert geometry nodes to 3MF objects + build items with transforms
    let mut object_id: usize = base_group_id + 1;
    for node in collect_geometry_nodes(&scene.children) {
        if let Some(Geometry::Triangles(ref tri_mesh)) = node.geometry {
            let mut mesh = lib3mf::Mesh::new();

            // Add vertices
            match &tri_mesh.positions {
                Positions::F32(positions) => {
                    for pos in positions {
                        mesh.vertices.push(lib3mf::Vertex::new(
                            pos.x as f64,
                            pos.y as f64,
                            pos.z as f64,
                        ));
                    }
                }
                Positions::F64(positions) => {
                    for pos in positions {
                        mesh.vertices.push(lib3mf::Vertex::new(pos.x, pos.y, pos.z));
                    }
                }
            }

            // Add triangles
            match &tri_mesh.indices {
                Indices::U8(indices) => {
                    for chunk in indices.chunks(3) {
                        mesh.triangles.push(lib3mf::Triangle::new(
                            chunk[0] as usize,
                            chunk[1] as usize,
                            chunk[2] as usize,
                        ));
                    }
                }
                Indices::U16(indices) => {
                    for chunk in indices.chunks(3) {
                        mesh.triangles.push(lib3mf::Triangle::new(
                            chunk[0] as usize,
                            chunk[1] as usize,
                            chunk[2] as usize,
                        ));
                    }
                }
                Indices::U32(indices) => {
                    for chunk in indices.chunks(3) {
                        mesh.triangles.push(lib3mf::Triangle::new(
                            chunk[0] as usize,
                            chunk[1] as usize,
                            chunk[2] as usize,
                        ));
                    }
                }
                Indices::None => {
                    let vertex_count = mesh.vertices.len();
                    for i in (0..vertex_count).step_by(3) {
                        mesh.triangles.push(lib3mf::Triangle::new(i, i + 1, i + 2));
                    }
                }
            }

            // Apply material reference via BaseMaterialGroup
            if let Some(mat_idx) = node.material_index {
                if mat_idx < scene.materials.len() {
                    for tri in &mut mesh.triangles {
                        tri.pid = Some(base_group_id);
                        tri.pindex = Some(mat_idx);
                        tri.p1 = Some(mat_idx);
                    }
                }
            }

            let mut object = lib3mf::Object::new(object_id);
            object.name = Some(node.name.clone());
            object.mesh = Some(mesh);
            model.resources.objects.push(object);

            // Build item with transform (if non-identity)
            let mut build_item = lib3mf::BuildItem::new(object_id);
            if node.transformation != Mat4::identity() {
                build_item.transform = Some(mat4_to_transform(&node.transformation));
            }
            model.build.items.push(build_item);
            object_id += 1;
        }
    }

    let mut buffer = Vec::new();
    let cursor = std::io::Cursor::new(&mut buffer);
    model.to_writer(cursor)?;
    Ok(buffer)
}

/// Recursively collect all nodes that contain geometry.
fn collect_geometry_nodes(nodes: &[Node]) -> Vec<&Node> {
    let mut result = Vec::new();
    for node in nodes {
        if node.geometry.is_some() {
            result.push(node);
        }
        result.extend(collect_geometry_nodes(&node.children));
    }
    result
}

#[cfg(test)]
mod test {
    use crate::{
        geometry::{Geometry, Indices, Positions},
        prelude::Srgba,
        Node, Scene,
    };
    use cgmath::Vector3;

    #[test]
    pub fn round_trip_3mf() {
        // Create a simple scene with a triangle
        let tri_mesh = crate::TriMesh {
            positions: Positions::F32(vec![
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(10.0, 0.0, 0.0),
                Vector3::new(5.0, 10.0, 0.0),
            ]),
            indices: Indices::U32(vec![0, 1, 2]),
            normals: None,
            tangents: None,
            uvs: None,
            colors: None,
        };

        let scene = Scene {
            name: "test".to_string(),
            children: vec![Node {
                name: "triangle".to_string(),
                geometry: Some(Geometry::Triangles(tri_mesh)),
                ..Default::default()
            }],
            materials: vec![crate::PbrMaterial {
                name: "red".to_string(),
                albedo: Srgba::new(255, 0, 0, 255),
                ..Default::default()
            }],
        };

        // Serialize
        let bytes = super::serialize_3mf(&scene).expect("Failed to serialize 3MF");
        assert!(!bytes.is_empty());

        // Deserialize
        let mut raw_assets = crate::io::RawAssets::new();
        raw_assets.insert("test.3mf", bytes);
        let loaded_scene: Scene = raw_assets
            .deserialize("test.3mf")
            .expect("Failed to deserialize 3MF");

        assert_eq!(loaded_scene.children.len(), 1);
        if let Some(Geometry::Triangles(ref mesh)) = loaded_scene.children[0].geometry {
            assert_eq!(mesh.positions.len(), 3);
            assert_eq!(mesh.triangle_count(), 1);
        } else {
            panic!("Expected triangle geometry");
        }
    }

    /// Tests loading a valid 3MF file with multiple mesh objects.
    /// Source: 3MF Consortium samples (examples/core/cube_gears.3mf).
    /// <https://github.com/3MFConsortium/3mf-samples>
    #[test]
    pub fn deserialize_multi_object_3mf() {
        let bytes = include_bytes!("../../test_data/cube_gears.3mf");
        let mut raw_assets = crate::io::RawAssets::new();
        raw_assets.insert("cube_gears.3mf", bytes.to_vec());
        let scene: Scene = raw_assets
            .deserialize("cube_gears.3mf")
            .expect("Failed to deserialize multi-object 3MF");

        // This file contains 17 distinct mesh objects (gears + cube)
        assert_eq!(scene.children.len(), 17);

        // Each child should have named triangle geometry
        for (i, node) in scene.children.iter().enumerate() {
            assert!(!node.name.is_empty(), "Object {} has no name", i);
            if let Some(Geometry::Triangles(ref mesh)) = node.geometry {
                assert!(
                    mesh.positions.len() > 0,
                    "Object '{}' has no vertices",
                    node.name
                );
                assert!(
                    mesh.triangle_count() > 0,
                    "Object '{}' has no triangles",
                    node.name
                );
            } else {
                panic!("Object '{}' has no triangle geometry", node.name);
            }
        }
    }

    /// Tests loading a valid 3MF file with per-vertex colors via a color group.
    /// Source: 3MF Consortium samples (examples/material/pyramid_vertexcolor.3mf).
    /// <https://github.com/3MFConsortium/3mf-samples>
    #[test]
    pub fn deserialize_vertex_color_3mf() {
        let bytes = include_bytes!("../../test_data/pyramid_vertexcolor.3mf");
        let mut raw_assets = crate::io::RawAssets::new();
        raw_assets.insert("pyramid_vertexcolor.3mf", bytes.to_vec());
        let scene: Scene = raw_assets
            .deserialize("pyramid_vertexcolor.3mf")
            .expect("Failed to deserialize vertex-color 3MF");

        // Single pyramid object
        assert_eq!(scene.children.len(), 1);

        if let Some(Geometry::Triangles(ref mesh)) = scene.children[0].geometry {
            // Pyramid: 4 vertices, 4 triangles
            assert_eq!(mesh.positions.len(), 4);
            assert_eq!(mesh.triangle_count(), 4);

            // Should have per-vertex colors extracted from the color group
            let colors = mesh.colors.as_ref().expect("Expected per-vertex colors");
            assert_eq!(colors.len(), 4);

            // Verify we got actual colors (not all white/default)
            let has_nonwhite = colors.iter().any(|c| *c != Srgba::WHITE);
            assert!(has_nonwhite, "Expected non-white vertex colors");
        } else {
            panic!("Expected triangle geometry");
        }
    }
}
