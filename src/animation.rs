use crate::{prelude::*, Interpolation};

#[derive(Debug, Clone, Default)]
pub struct Animation {
    pub name: String,
    pub key_frames: Vec<KeyFrames>,
}

#[derive(Debug, Clone, Default)]
pub struct KeyFrames {
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub rotations: Option<Vec<Quat>>,
    pub translations: Option<Vec<Vec3>>,
    pub scales: Option<Vec<Vec3>>,
    pub weights: Option<Vec<f32>>,
}

impl KeyFrames {
    pub fn transformation(&self, time: f32) -> Mat4 {
        let (index, t) = self.interpolate(time);
        let mut transformation = Mat4::identity();
        if let Some(values) = &self.rotations {
            let v0 = index.map(|i| values[i]).unwrap_or(Quat::one());
            let v1 = values[index.map(|i| i + 1).unwrap_or(0)];
            let value = v0.slerp(v1, t);
            transformation = transformation * Mat4::from(value);
        }
        if let Some(values) = &self.scales {
            let v0 = index.map(|i| values[i]).unwrap_or(vec3(1.0, 1.0, 1.0));
            let v1 = values[index.map(|i| i + 1).unwrap_or(0)];
            let value = v0.lerp(v1, t);
            transformation =
                Mat4::from_nonuniform_scale(value.x, value.y, value.z) * transformation;
        }
        if let Some(values) = &self.translations {
            let v0 = index.map(|i| values[i]).unwrap_or(Vec3::zero());
            let v1 = values[index.map(|i| i + 1).unwrap_or(0)];
            let value = v0.lerp(v1, t);
            transformation = Mat4::from_translation(value) * transformation;
        }
        transformation
    }

    /*pub fn weights(&self, time: f32) -> Vec<f32> {
        if let Some(values) = &self.weights {
            let (index, t) = self.interpolate(time);
            let index = index.unwrap(); // TODO
            let count = values.len() / self.times.len();
            let v0 = &values[count * index..count * (index + 1)];
            let v1 = &values[count * (index + 1)..count * (index + 2)];
            (0..count).map(|i| (1.0 - t) * v0[i] + t * v1[i]).collect()
        } else {
            Vec::new()
        }
    }*/

    fn interpolate(&self, time: f32) -> (Option<usize>, f32) {
        let time = time % self.times.last().unwrap();
        for i in 0..self.times.len() - 1 {
            if self.times[i] <= time && time < self.times[i + 1] {
                return (
                    Some(i),
                    (time - self.times[i]) / (self.times[i + 1] - self.times[i]),
                );
            }
        }
        (None, time / self.times[0])
    }
}
