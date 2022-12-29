use crate::{prelude::*, Interpolation};

/// A  set of key frames and transformations associated with a specific animation for a specific [Primitive](crate::Primitive).
#[derive(Debug, Clone, Default)]
pub struct KeyFrameAnimation {
    /// Optional name of the animation.
    ///
    /// Can be used to distinguish between different types of animations for the same [Primitive](crate::Primitive), for example between a "walk" and a "run" animation.
    pub name: Option<String>,
    /// A list of [KeyFrames] and transformations which are the result of traversing a [Scene](crate::Scene) from root to the leaves.
    pub key_frames: Vec<(Mat4, std::sync::Arc<KeyFrames>)>,
}

impl KeyFrameAnimation {
    /// Calculates a transformation for this animation at the given time.
    pub fn transformation(&self, time: f32) -> Mat4 {
        let mut transformation = Mat4::identity();
        for (t, animation) in self.key_frames.iter() {
            transformation = transformation * t * animation.transformation(time);
        }
        transformation
    }
}

///
/// Contains a set of key frames for rotations, translations, scales and morph weights.
///
#[derive(Debug, Clone, Default)]
pub struct KeyFrames {
    /// Optional time where the animation repeats itself.
    pub loop_time: Option<f32>,
    /// The type of interpolation used in between the key frames.
    pub interpolation: Interpolation,
    /// The time value for each key frame.
    pub times: Vec<f32>,
    /// The rotation for each key frame.
    pub rotations: Option<Vec<Quat>>,
    /// The rotation for each key frame.
    pub translations: Option<Vec<Vec3>>,
    /// The non uniform scale for each key frame.
    pub scales: Option<Vec<Vec3>>,
    /// The morph weights for each key frame.
    pub weights: Option<Vec<Vec<f32>>>,
}

impl KeyFrames {
    /// The rotation at the specified time.
    pub fn rotation(&self, time: f32) -> Option<Quat> {
        self.rotations
            .as_ref()
            .map(|values| self.interpolate_rotation(time, values))
    }
    /// The translation at the specified time.
    pub fn translation(&self, time: f32) -> Option<Vec3> {
        self.translations
            .as_ref()
            .map(|values| self.interpolate(time, values))
    }
    /// The scale at the specified time.
    pub fn scale(&self, time: f32) -> Option<Vec3> {
        self.scales
            .as_ref()
            .map(|values| self.interpolate(time, values))
    }

    /// The transformation at the specified time.
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

    /// The morph weights at the specified time.
    pub fn weights(&self, time: f32) -> Option<Vec<f32>> {
        self.weights
            .as_ref()
            .map(|values| self.interpolate_array(time, values))
    }

    fn interpolate_rotation(&self, time: f32, values: &[Quat]) -> Quat {
        let time = self.loop_time.map(|t| time % t).unwrap_or(time);
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

    fn interpolate_array(&self, time: f32, values: &[Vec<f32>]) -> Vec<f32> {
        let time = self.loop_time.map(|t| time % t).unwrap_or(time);
        if time < self.times[0] {
            values[0].clone()
        } else {
            for i in 0..self.times.len() - 1 {
                if self.times[i] <= time && time < self.times[i + 1] {
                    let t = (time - self.times[i]) / (self.times[i + 1] - self.times[i]);
                    let mut result = Vec::new();
                    for j in 0..values[i].len() {
                        result.push(values[i][j] * (1.0 - t) + values[i + 1][j] * t);
                    }
                    return result;
                }
            }
            values.last().unwrap().clone()
        }
    }

    fn interpolate<T: Copy + std::ops::Mul<f32, Output = T> + std::ops::Add<T, Output = T>>(
        &self,
        time: f32,
        values: &[T],
    ) -> T {
        let time = self.loop_time.map(|t| time % t).unwrap_or(time);
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
