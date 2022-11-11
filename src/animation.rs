use crate::prelude::*;

#[derive(Debug)]
pub struct KeyFrames {
    pub times: Vec<f32>,
    pub rotations: Vec<Quat>,
}
