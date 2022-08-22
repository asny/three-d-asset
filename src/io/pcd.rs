use crate::geometry::{PointCloud, Positions};
use crate::prelude::*;
use crate::{io::RawAssets, Result};
use pcd_rs::DynReader;
use std::path::Path;

pub fn deserialize_pcd(raw_assets: &mut RawAssets, path: impl AsRef<Path>) -> Result<PointCloud> {
    let name = path.as_ref().to_str().unwrap().to_string();
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
                let t = match p.0[i] {
                    pcd_rs::Field::U32(ref v) => v[0].to_ne_bytes(),
                    pcd_rs::Field::F32(ref v) => v[0].to_ne_bytes(),
                    _ => unimplemented!(),
                };
                Color {
                    r: t[2],
                    g: t[1],
                    b: t[0],
                    ..Default::default()
                }
            })
            .collect()
    });
    Ok(PointCloud {
        positions: Positions::F32(positions),
        colors,
        name,
    })
}

#[cfg(test)]
mod test {

    #[test]
    pub fn deserialize_pcd() {
        let point_cloud: crate::PointCloud = crate::io::RawAssets::new()
            .insert(
                "test_data/hand.pcd",
                include_bytes!("../../test_data/hand.pcd").to_vec(),
            )
            .deserialize("pcd")
            .unwrap();
        assert_eq!(point_cloud.positions.len(), 9199);
    }

    #[test]
    pub fn deserialize_binary_pcd() {
        let point_cloud: crate::PointCloud = crate::io::RawAssets::new()
            .insert(
                "test_data/binary.pcd",
                include_bytes!("../../test_data/binary.pcd").to_vec(),
            )
            .deserialize("pcd")
            .unwrap();
        assert_eq!(point_cloud.positions.len(), 28944);
    }
}
