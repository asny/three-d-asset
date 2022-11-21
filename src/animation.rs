use crate::{prelude::*, Interpolation};

#[derive(Debug, Clone, Default)]
pub struct Animation {
    pub name: String,
    pub key_frames: Vec<KeyFrames>,
    pub loop_time: f32,
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
    pub fn rotation(&self, time: f32) -> Option<Quat> {
        self.rotations
            .as_ref()
            .map(|values| self.interpolate_rotation(time, values))
    }
    pub fn translation(&self, time: f32) -> Option<Vec3> {
        self.translations
            .as_ref()
            .map(|values| self.interpolate(time, values))
    }
    pub fn scale(&self, time: f32) -> Option<Vec3> {
        self.scales
            .as_ref()
            .map(|values| self.interpolate(time, values))
    }

    pub fn transformation(&self, time: f32) -> Mat4 {
        let mut transformation = Mat4::identity();
        if let Some(value) = self.scale(time) {
            transformation =
                Mat4::from_nonuniform_scale(value.x, value.y, value.z) * transformation;
        }
        if let Some(value) = self.rotation(time) {
            transformation = transformation * Mat4::from(value);
        }
        if let Some(value) = self.translation(time) {
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

    fn interpolate_rotation(&self, time: f32, values: &Vec<Quat>) -> Quat {
        if time < self.times[0] {
            values[0]
        } else {
            for i in 0..self.times.len() - 1 {
                if self.times[i] <= time && time < self.times[i + 1] {
                    let t = (time - self.times[i]) / (self.times[i + 1] - self.times[i]);
                    return values[i].slerp(values[i + 1], t);
                }
            }
            *values.last().unwrap()
        }
    }

    fn interpolate<T: Copy + std::ops::Mul<f32, Output = T> + std::ops::Add<T, Output = T>>(
        &self,
        time: f32,
        values: &Vec<T>,
    ) -> T {
        if time < self.times[0] {
            values[0]
        } else {
            for i in 0..self.times.len() - 1 {
                if self.times[i] <= time && time < self.times[i + 1] {
                    let t = (time - self.times[i]) / (self.times[i + 1] - self.times[i]);
                    return values[i] * (1.0 - t) + values[i + 1] * t;
                }
            }
            *values.last().unwrap()
        }
    }
}
