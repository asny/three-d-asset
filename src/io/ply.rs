use crate::geometry::{Geometry, PointCloud, Positions};
use crate::prelude::*;
use crate::{io::RawAssets, Node, Result, Scene};
use ply_rs_bw::parser::Parser;
use ply_rs_bw::ply::{DefaultElement, PropertyAccess};
use std::io::Cursor;
use std::path::PathBuf;

pub fn deserialize_ply(raw_assets: &mut RawAssets, path: &PathBuf) -> Result<Scene> {
    let name = path.to_str().unwrap().to_string();
    let bytes = raw_assets.get(path)?;
    let mut cursor = Cursor::new(bytes); // wrap with a position tracker

    let parser = Parser::<DefaultElement>::new();
    let ply = parser.read_ply(&mut cursor)?;

    let vertices = ply
        .payload
        .get("vertex")
        .ok_or_else(|| crate::Error::FailedDeserialize(name.clone()))?;

    let num_vertices = vertices.len();
    let mut positions = Vec::with_capacity(num_vertices);

    // check for traditional uchar color fields
    let first_vertex = vertices.first();
    let has_uchar_colors = first_vertex.is_some_and(|v| v.get_uchar("red").is_some());

    // check for gaussian splat fields, including color, scale, rotation, opacity, and spherical harmonic coefficients
    let has_gs_fields = first_vertex.is_some_and(|v| {
        v.get_float("scale_0").is_some()
            && v.get_float("rot_0").is_some()
            && v.get_float("opacity").is_some()
            && v.get_float("f_dc_0").is_some()
    });

    // check for spherical harmonic coefficients
    let mut sh_coeff_count = 0;
    while first_vertex.is_some_and(|v| v.get_float(&format!("f_rest_{}", sh_coeff_count)).is_some())
    {
        sh_coeff_count += 1;
    }

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
        Some(Vec::with_capacity(num_vertices))
    } else {
        None
    };

    for vertex in vertices {
        positions.push(vec3(
            vertex.get_float("x").unwrap_or(0.0),
            vertex.get_float("y").unwrap_or(0.0),
            vertex.get_float("z").unwrap_or(0.0),
        ));

        if has_uchar_colors {
            let r = vertex.get_uchar("red").unwrap_or(0);
            let g = vertex.get_uchar("green").unwrap_or(0);
            let b = vertex.get_uchar("blue").unwrap_or(0);
            if let Some(ref mut c) = colors {
                c.push(Srgba {
                    r,
                    g,
                    b,
                    ..Default::default()
                });
            }
        } else if has_gs_fields {
            // f_dc_0, f_dc_1, f_dc_2 for degree 0) spherical harmonic coefficients
            let f_dc_0 = vertex.get_float("f_dc_0").unwrap_or(0.0);
            let f_dc_1 = vertex.get_float("f_dc_1").unwrap_or(0.0);
            let f_dc_2 = vertex.get_float("f_dc_2").unwrap_or(0.0);

            // scale
            let scale_x = vertex.get_float("scale_0").unwrap_or(1.0);
            let scale_y = vertex.get_float("scale_1").unwrap_or(1.0);
            let scale_z = vertex.get_float("scale_2").unwrap_or(1.0);
            if let Some(ref mut s) = scale {
                s.push(vec3(scale_x, scale_y, scale_z));
            }

            // rotation
            let rot_w = vertex.get_float("rot_0").unwrap_or(1.0);
            let rot_x = vertex.get_float("rot_1").unwrap_or(0.0);
            let rot_y = vertex.get_float("rot_2").unwrap_or(0.0);
            let rot_z = vertex.get_float("rot_3").unwrap_or(0.0);
            if let Some(ref mut r) = rotation {
                r.push(Quat::from_sv(rot_w, vec3(rot_x, rot_y, rot_z)));
            }

            // opacity;
            if let Some(ref mut o) = opacity {
                o.push(vertex.get_float("opacity").unwrap_or(1.0));
            }

            // degree 0 spherical harmonic coefficients
            if let Some(ref mut dc0) = dc_spherical_harmonics {
                dc0.push(vec3(f_dc_0, f_dc_1, f_dc_2));
            }

            if let Some(ref mut s) = spherical_harmonics {
                let coeffs = (0..sh_coeff_count)
                    .map(|i| vertex.get_float(&format!("f_rest_{}", i)).unwrap_or(0.0))
                    .collect();
                s.push(coeffs);
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
