use crate::geometry::{Geometry, PointCloud, Positions};
use crate::prelude::*;
use crate::{io::RawAssets, Node, Result, Scene};
use ply_rs_bw::parser::{Parser, Reader};
use ply_rs_bw::ply::{ PropertyAccess, PropertyAccessResult};
use std::io::Cursor;
use std::path::PathBuf;

const MAX_SH:usize = 45;

struct GsSplat {
    position: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
    opacity: f32,
    f_dc: [f32; 3],
    f_rest: [f32; MAX_SH],
    red: u8,
    green: u8,
    blue: u8,
}

impl PropertyAccess for GsSplat {
    fn new() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [1.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            opacity: 1.0,
            f_dc: [0.0, 0.0, 0.0],
            f_rest: [0.0; MAX_SH],
            red: 0,
            green: 0,
            blue: 0,
        }
    }

    fn set_float(&mut self, key: &str, value: f32) -> PropertyAccessResult {
        match key {
            "x" => self.position[0] = value,
            "y" => self.position[1] = value,
            "z" => self.position[2] = value,
            "rot_0" => self.rotation[0] = value,
            "rot_1" => self.rotation[1] = value,
            "rot_2" => self.rotation[2] = value,
            "rot_3" => self.rotation[3] = value,
            "scale_0" => self.scale[0] = value,
            "scale_1" => self.scale[1] = value,
            "scale_2" => self.scale[2] = value,
            "opacity" => self.opacity = value,
            "f_dc_0" => self.f_dc[0] = value,
            "f_dc_1" => self.f_dc[1] = value,
            "f_dc_2" => self.f_dc[2] = value,
            _ => {
                match key 
                .strip_prefix("f_rest_")
                .and_then(|s| s.parse::<usize>().ok())
                {
                    Some(i) if i < MAX_SH => self.f_rest[i] = value,
                    _ => return PropertyAccessResult::Ignored
                }
            }
        }
        PropertyAccessResult::Set
    }

    fn set_uchar(&mut self, key: &str, value: u8) -> PropertyAccessResult {
        match key {
            "red" => self.red = value,
            "green" => self.green = value,
            "blue" => self.blue = value,
            _ => return PropertyAccessResult::Ignored
        }
        PropertyAccessResult::Set
    }
}

pub fn deserialize_ply(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let name = path.to_str().unwrap().to_string();
    let bytes = raw_assets.get(path)?;
    let mut cursor = Cursor::new(bytes); // wrap with a position tracker

    let parser = Parser::<GsSplat>::new();
    let ply = parser.read_ply(&mut cursor)?;

    // read header
    let mut reader = Reader::new(Cursor::new(bytes));
    let header = parser.read_header(&mut reader)?;
    let vertex_def = header.elements.get("vertex").ok_or_else(|| crate::Error::FailedDeserialize(name.clone()))?;

    // checking ply type
    let has_uchar_colors = vertex_def.properties.contains_key("red");
    let has_gs_fields = vertex_def.properties.contains_key("scale_0")
        && vertex_def.properties.contains_key("rot_0")
        && vertex_def.properties.contains_key("opacity")
        && vertex_def.properties.contains_key("f_dc_0");

    let mut sh_coeff_count = 0;
    while vertex_def.properties.contains_key(&format!("f_rest_{}", sh_coeff_count)) {
        sh_coeff_count += 1;
    }
    sh_coeff_count = sh_coeff_count.min(MAX_SH);

    let vertices = ply
        .payload
        .get("vertex")
        .ok_or_else(|| crate::Error::FailedDeserialize(name.clone()))?;

    let num_vertices = vertices.len();
    let mut positions = Vec::with_capacity(num_vertices);


    let mut colors = if has_uchar_colors {
        Some(Vec::with_capacity(num_vertices))
    } else {
        None
    };

    let mut scale = if has_gs_fields {
        Some(Vec::with_capacity(num_vertices))
    } else {
        None
    };

    let mut rotation = if has_gs_fields {
        Some(Vec::with_capacity(num_vertices))
    } else {
        None
    };

    let mut opacity = if has_gs_fields {
        Some(Vec::with_capacity(num_vertices))
    } else {
        None
    };

    let mut dc_spherical_harmonics = if has_gs_fields {
        Some(Vec::with_capacity(num_vertices))
    } else {
        None
    };

    // some ply files may not have spherical harmonic coefficients ( only f_dc_* )
    let mut spherical_harmonics = if has_gs_fields && sh_coeff_count > 0 {
        Some(Vec::with_capacity(num_vertices * sh_coeff_count))
    } else {
        None
    };

    for vertex in vertices {
        positions.push(vec3(
            vertex.position[0],
            vertex.position[1],
            vertex.position[2],
        ));

        if has_uchar_colors {
            let r = vertex.red;
            let g = vertex.green;
            let b = vertex.blue;
            if let Some(ref mut c) = colors {
                c.push(Srgba {
                    r,
                    g,
                    b,
                    ..Default::default()
                });
            }
        } else if has_gs_fields {
            // scale
            let scale_x = vertex.scale[0];
            let scale_y = vertex.scale[1];
            let scale_z = vertex.scale[2];
            if let Some(ref mut s) = scale {
                s.push(vec3(scale_x, scale_y, scale_z));
            }

            // rotation
            let rot_w = vertex.rotation[0];
            let rot_x = vertex.rotation[1];
            let rot_y = vertex.rotation[2];
            let rot_z = vertex.rotation[3];
            if let Some(ref mut r) = rotation {
                r.push(Quat::from_sv(rot_w, vec3(rot_x, rot_y, rot_z)));
            }

            // opacity;
            if let Some(ref mut o) = opacity {
                o.push(vertex.opacity);
            }

            // f_dc_0, f_dc_1, f_dc_2 for degree 0) spherical harmonic coefficients
            let f_dc_0 = vertex.f_dc[0];
            let f_dc_1 = vertex.f_dc[1];
            let f_dc_2 = vertex.f_dc[2];
            // degree 0 spherical harmonic coefficients
            if let Some(ref mut dc0) = dc_spherical_harmonics {
                dc0.push(vec3(f_dc_0, f_dc_1, f_dc_2));
            }

            // spherical harmonic coefficients
            if let Some(ref mut sh) = spherical_harmonics {
                sh.extend_from_slice(&vertex.f_rest[0..sh_coeff_count]);
            }

            
        }
    }

    Ok(Scene {
        name,
        children: vec![Node {
            geometry: Some(Geometry::Points(PointCloud {
                positions: Positions::F32(positions),
                colors,
                scale,
                rotation,
                opacity,
                dc_spherical_harmonics,
                spherical_harmonics,
            })),
            ..Default::default()
        }],
        ..Default::default()
    })
}

#[cfg(test)]
mod test {
    /// Gaussian splat file, with colors, scale, rotation, opacity, dc_spherical_harmonics, and spherical_harmonics
    #[test]
    pub fn deserialize_gaussian_ply() {
        let pc: crate::PointCloud = crate::io::RawAssets::new()
            .insert(
                "test_data/grape_small.ply",
                include_bytes!("../../test_data/grape_small.ply").to_vec(),
            )
            .deserialize("grape_small.ply")
            .unwrap();
        assert_eq!(pc.positions.len(), 5000);
        assert!(pc.colors.is_none(), "expected no colors");
        assert!(pc.scale.is_some(), "expected scale");
        assert!(pc.rotation.is_some(), "expected rotation");
        assert!(pc.opacity.is_some(), "expected opacity");
        assert!(
            pc.dc_spherical_harmonics.is_some(),
            "expected dc_spherical_harmonics"
        );
        assert!(
            pc.spherical_harmonics.is_some(),
            "expected spherical_harmonics"
        );
    }

    /// Simple ply file, Positions only
    #[test]
    pub fn deserialize_positions_only_ply() {
        let pc: crate::PointCloud = crate::io::RawAssets::new()
            .insert(
                "test_data/positions_only.ply",
                include_bytes!("../../test_data/positions_only.ply").to_vec(),
            )
            .deserialize("positions_only.ply")
            .unwrap();
        assert_eq!(pc.positions.len(), 100);
        assert!(pc.colors.is_none(), "expected no colors");
        assert!(pc.scale.is_none(), "expected no scale");
        assert!(pc.rotation.is_none(), "expected no rotation");
        assert!(pc.opacity.is_none(), "expected no opacity");
        assert!(
            pc.dc_spherical_harmonics.is_none(),
            "expected no dc_spherical_harmonics"
        );
        assert!(
            pc.spherical_harmonics.is_none(),
            "expected no spherical_harmonics"
        );
    }

    /// Simple ply file, Positions and colors
    #[test]
    pub fn deserialize_positions_colors_ply() {
        let pc: crate::PointCloud = crate::io::RawAssets::new()
            .insert(
                "test_data/positions_colors.ply",
                include_bytes!("../../test_data/positions_colors.ply").to_vec(),
            )
            .deserialize("positions_colors.ply")
            .unwrap();
        assert_eq!(pc.positions.len(), 100);
        assert!(
            pc.colors.is_some(),
            "expected colors from uchar red/green/blue"
        );
        assert!(pc.scale.is_none(), "expected no scale");
        assert!(pc.rotation.is_none(), "expected no rotation");
        assert!(pc.opacity.is_none(), "expected no opacity");
        assert!(
            pc.dc_spherical_harmonics.is_none(),
            "expected no dc_spherical_harmonics"
        );
        assert!(
            pc.spherical_harmonics.is_none(),
            "expected no spherical_harmonics"
        );
    }
}
