use crate::geometry::{PointCloud, Positions};
use crate::prelude::*;
use crate::{io::RawAssets, Error, Result};
use pcd_rs::{DynReader, PcdDeserialize};
use std::mem;
use std::path::Path;

#[derive(PcdDeserialize)]
struct PcdPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(PcdDeserialize)]
struct PcdPointWithColor {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
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

fn min(point: &Vector3<f32>) -> f32 {
    f32::min(point.x, f32::min(point.y, point.z))
}

fn max(point: &Vector3<f32>) -> f32 {
    f32::max(point.x, f32::max(point.y, point.z))
}

fn normalize_value(value: f32, min: &f32, max: &f32) -> f32 {
    (2.0 * ((value - min) / (max - min))) - 1.0
}

fn normalize_point(point: &Vector3<f32>, min: f32, max: f32) -> Vector3<f32> {
    return Vector3 {
        x: normalize_value(point.x, &min, &max),
        y: normalize_value(point.y, &min, &max),
        z: normalize_value(point.z, &min, &max),
    };
}

pub fn deserialize_pcd(raw_assets: &mut RawAssets, path: impl AsRef<Path>) -> Result<PointCloud> {
    /*let reader = Reader::open(path)?;
    let points: pcd_rs::anyhow::Result<Vec<PcdPointWithColor>> = reader.collect();
    let points = points?;*/
    let reader = DynReader::from_bytes(raw_assets.get(path)?)?;
    let schema = reader.meta().field_defs.fields.clone();
    dbg!(&schema);
    let x_index = schema.iter().position(|f| f.name == "x").unwrap();
    let y_index = schema.iter().position(|f| f.name == "y").unwrap();
    let z_index = schema.iter().position(|f| f.name == "z").unwrap();
    let rgb_index = schema.iter().position(|f| f.name == "rgb");

    let points = reader.collect::<pcd_rs::anyhow::Result<Vec<_>>>()?;
    let positions = points
        .iter()
        .map(|p| {
            //dbg!(p);
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

    /*let meta = reader.meta();
    println!("{:?}", meta);

    let positions: Vec<_> = if meta.field_defs.fields.any(|f| f.name == "rgb") {
        reader
            .map(|p: PcdPointWithColor| vec3(0.0, 0.0, 0.0))
            .collect()
    } else {
        reader.map(|p: PcdPoint| vec3(0.0, 0.0, 0.0)).collect()
    };*/
    //let points = reader.collect::<pcd_rs::anyhow::Result<Vec<_>>>()?;
    /*let (mut positions, colors) = if color {
        let colored_points: Result<Vec<PcdPointWithColor>> = reader.collect();
        let colored_points = colored_points?;
        let positions: Vec<_> = colored_points
            .iter()
            .map(|p| Vec3 {
                x: p.x,
                y: p.y,
                z: p.z,
            })
            .collect();
        let colors: Option<Vec<_>> = Some(colored_points.iter().map(decode_color).collect());

        (positions, colors)
    } else {
        let reader = Reader::open(path)?;
        let points: Result<Vec<PcdPoint>> = reader.collect();
        let points = points?;
        let positions: Vec<_> = points
            .iter()
            .map(|p| Vec3 {
                x: p.x,
                y: p.y,
                z: p.z,
            })
            .collect();
        let colors = None;

        (positions, colors)
    };

    if normalize {
        let max = positions.iter().map(max).fold(0.0, f32::max);
        let min = positions.iter().map(min).fold(0.0, f32::min);
        for i in 0..positions.len() {
            positions[i] = normalize_point(&positions[i], min, max)
        }
    }*/
    Ok(PointCloud {
        positions: Positions::F32(positions),
        colors,
        ..Default::default()
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
}
