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
        Mat4::identity()
    }

    pub fn weights(&self, time: f32) -> Vec4 {
        vec4(0.0, 0.0, 0.0, 0.0)
    }

    pub fn rotation(&self, time: f32) -> Quat {
        Quat::one()
    }

    fn index(&self, time: f32) -> usize {
        let time = time % self.times.last().unwrap();
        for i in 0..self.times.len() - 2 {
            if self.times[i] < time && time < self.times[i + 1] {
                return i;
            }
        }
        self.times.len() - 1
    }
}
