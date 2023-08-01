//!
//! Contain material asset definitions.
//!

#[doc(inline)]
pub use crate::{prelude::Srgba, texture::texture2d::*};

/// Lighting models which specify how the lighting is computed when rendering a material.
/// This is a trade-off between how fast the computations are versus how physically correct they look.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightingModel {
    /// Phong lighting model.
    /// The fastest lighting model to calculate.
    Phong,
    /// Blinn lighting model.
    /// Almost as fast as Phong and has less artifacts.
    Blinn,
    /// Cook-Torrance lighting model with the given normal distribution and geometry functions.
    /// The most physically correct lighting model but also the most expensive.
    Cook(NormalDistributionFunction, GeometryFunction),
}

/// The geometry function used in a Cook-Torrance lighting model.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(missing_docs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GeometryFunction {
    SmithSchlickGGX,
}

/// The normal distribution function used in a Cook-Torrance lighting model.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(missing_docs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NormalDistributionFunction {
    Blinn,
    Beckmann,
    TrowbridgeReitzGGX,
}

///
/// A CPU-side version of a material used for physically based rendering (PBR).
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PbrMaterial {
    /// Name. Used for matching geometry and material.
    pub name: String,
    /// Albedo base color, also called diffuse color.
    pub albedo: Srgba,
    /// Texture with albedo base colors, also called diffuse colors.
    /// The colors are assumed to be in sRGB (`RgbU8`), sRGB with an alpha channel (`RgbaU8`) or HDR color space.
    pub albedo_texture: Option<Texture2D>,
    /// A value in the range `[0..1]` specifying how metallic the material is.
    pub metallic: f32,
    /// A value in the range `[0..1]` specifying how rough the material surface is.
    pub roughness: f32,
    /// Texture containing the occlusion, metallic and roughness parameters.
    /// The occlusion values are sampled from the red channel, metallic from the blue channel and the roughness from the green channel.
    /// Is sometimes in two textures, see [Self::occlusion_texture] and [Self::metallic_roughness_texture].
    pub occlusion_metallic_roughness_texture: Option<Texture2D>,
    /// Texture containing the metallic and roughness parameters which are multiplied with the [Self::metallic] and [Self::roughness] to get the final parameter.
    /// The metallic values are sampled from the blue channel and the roughness from the green channel.
    /// Can be combined with occlusion into one texture, see [Self::occlusion_metallic_roughness_texture].
    pub metallic_roughness_texture: Option<Texture2D>,
    /// A scalar multiplier controlling the amount of occlusion applied from the [Self::occlusion_texture]. A value of 0.0 means no occlusion. A value of 1.0 means full occlusion.
    pub occlusion_strength: f32,
    /// An occlusion map. Higher values indicate areas that should receive full indirect lighting and lower values indicate no indirect lighting.
    /// The occlusion values are sampled from the red channel.
    /// Can be combined with metallic and roughness into one texture, see [Self::occlusion_metallic_roughness_texture].
    pub occlusion_texture: Option<Texture2D>,
    /// A scalar multiplier applied to each normal vector of the [Self::normal_texture].
    pub normal_scale: f32,
    /// A tangent space normal map, also known as bump map.
    pub normal_texture: Option<Texture2D>,
    /// Color of light shining from an object.
    pub emissive: Srgba,
    /// Texture with color of light shining from an object.
    /// The colors are assumed to be in sRGB (`RgbU8`), sRGB with an alpha channel (`RgbaU8`) or HDR color space.
    pub emissive_texture: Option<Texture2D>,
    /// Alpha cutout value for transparency in deferred rendering pipeline.
    pub alpha_cutout: Option<f32>,
    /// The lighting model used when rendering this material
    pub lighting_model: LightingModel,
    /// The index of refraction for this material    
    pub index_of_refraction: f32,
    /// A value in the range `[0..1]` specifying how transmissive the material surface is.
    pub transmission: f32,
    /// Texture containing the transmission parameter which are multiplied with the [Self::transmission] to get the final parameter.
    pub transmission_texture: Option<Texture2D>,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            albedo: Srgba::WHITE,
            albedo_texture: None,
            occlusion_metallic_roughness_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            occlusion_strength: 1.0,
            normal_texture: None,
            normal_scale: 1.0,
            emissive: Srgba::BLACK,
            emissive_texture: None,
            index_of_refraction: 1.5,
            transmission: 0.0,
            transmission_texture: None,
            alpha_cutout: None,
            lighting_model: LightingModel::Blinn,
        }
    }
}
