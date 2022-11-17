use crate::{prelude::*, Interpolation};

#[derive(Debug, Clone, Default)]
pub struct Animation {
    pub name: String,
    pub key_frames: Vec<KeyFrames>,
}

#[derive(Debug, Clone, Default)]
pub struct KeyFrames {
    pub target_node: usize,
    pub interpolation: Interpolation,
    pub times: Vec<f32>,
    pub rotations: Option<Vec<Quat>>,
    pub translations: Option<Vec<Vec3>>,
    pub scales: Option<Vec<Vec3>>,
    pub weights: Option<Vec<Vec4>>,
}

impl KeyFrames {
    pub fn transformation(&self, time: f32) -> Mat4 {
        let (index, t) = self.interpolate(time);
        let mut transformation = Mat4::identity();
        /*if let Some(values) = &self.rotations {
            let value = (1.0 - t) * values[index] + t * values[index + 1];
            transformation *= value.to_mat4();
        }*/
        // TODO
        if let Some(values) = &self.scales {
            let value = (1.0 - t) * values[index] + t * values[index + 1];
            transformation =
                Mat4::from_nonuniform_scale(value.x, value.y, value.z) * transformation;
        }
        if let Some(values) = &self.translations {
            let value = (1.0 - t) * values[index] + t * values[index + 1];
            transformation = Mat4::from_translation(value) * transformation;
        }
        transformation
    }

    pub fn weights(&self, time: f32) -> Vec4 {
        if let Some(values) = &self.weights {
            let (index, t) = self.interpolate(time);
            (1.0 - t) * values[index] + t * values[index + 1]
        } else {
            vec4(0.0, 0.0, 0.0, 0.0)
        }
    }

    fn interpolate(&self, time: f32) -> (usize, f32) {
        let time = time % self.times.last().unwrap();
        for i in 0..self.times.len() - 2 {
            if self.times[i] < time && time < self.times[i + 1] {
                return (
                    i,
                    (time - self.times[i]) / self.times[i + 1] / self.times[i],
                );
            }
        }
        (self.times.len() - 1, 0.0)
    }
}
