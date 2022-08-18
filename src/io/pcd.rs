use crate::geometry::{Points, Positions};
use crate::prelude::*;
use crate::{io::RawAssets, Result};
use pcd_rs::DynReader;
use std::mem;
use std::path::Path;

pub fn deserialize_pcd(raw_assets: &mut RawAssets, path: impl AsRef<Path>) -> Result<Points> {
    let reader = DynReader::from_bytes(raw_assets.get(path)?)?;
    let schema = reader.meta().field_defs.fields.clone();
    let x_index = schema.iter().position(|f| f.name == "x").unwrap();
    let y_index = schema.iter().position(|f| f.name == "y").unwrap();
    let z_index = schema.iter().position(|f| f.name == "z").unwrap();
    let rgb_index = schema.iter().position(|f| f.name == "rgb");

    let points = reader.collect::<pcd_rs::anyhow::Result<Vec<_>>>()?;
    let positions = points
        .iter()
        .map(|p| {
            vec3(
                p.0[x_index].to_value::<f32>().unwrap(),
                p.0[y_index].to_value::<f32>().unwrap(),
                p.0[z_index].to_value::<f32>().unwrap(),
            )
        })
        .collect();

    let colors = rgb_index.map(|i| {
        points
            .iter()
            .map(|p| {
                let rgb = p.0[i].to_value::<f32>().unwrap();
                decode_color(rgb)
            })
            .collect()
    });
    Ok(Points {
        positions: Positions::F32(positions),
        colors,
        ..Default::default()
    })
}

fn decode_color(rgb: f32) -> Color {
    unsafe {
        let rgb: u32 = mem::transmute_copy(&rgb);
        let r = ((rgb >> 16) & 255).try_into().unwrap();
        let g = ((rgb >> 8) & 255).try_into().unwrap();
        let b = (rgb & 255).try_into().unwrap();
        Color { r, g, b, a: 1 }
    }
}

#[cfg(test)]
mod test {

    #[test]
    pub fn deserialize_pcd() {
        let point_cloud: crate::Points = crate::io::RawAssets::new()
            .insert(
                "test_data/hand.pcd",
                include_bytes!("../../test_data/hand.pcd").to_vec(),
            )
            .deserialize("pcd")
            .unwrap();
        assert_eq!(point_cloud.positions.len(), 9199);
    }
}
