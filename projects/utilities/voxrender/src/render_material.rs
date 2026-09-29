use ty_math::TyLinSrgbF64;
use voxcore::material::{
    EMISSIVE_STRENGTH_DEFAULT, METALLIC_DEFAULT, OCCLUSION_STRENGTH_DEFAULT, ROUGHNESS_DEFAULT,
};

/// The shaded properties of one material, in linear light. Every voxel is
/// opaque, so the base color has no alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMaterial {
    /// The base color.
    pub base_color: TyLinSrgbF64,

    /// The metalness, `0..1`.
    pub metallic: f64,

    /// The roughness, `0..1`.
    pub roughness: f64,

    /// The emissive color.
    pub emissive_color: TyLinSrgbF64,

    /// The strength scaling the emissive color, `0..`.
    pub emissive_strength: f64,

    /// How far the occlusion darkens from full, `0..1`.
    pub occlusion_strength: f64,
}

/// The standard defaults: opaque white, fully metallic and rough, with no
/// emission.
impl Default for RenderMaterial {
    fn default() -> Self {
        RenderMaterial {
            base_color: TyLinSrgbF64::new(1.0, 1.0, 1.0),
            metallic: METALLIC_DEFAULT,
            roughness: ROUGHNESS_DEFAULT,
            emissive_color: TyLinSrgbF64::new(0.0, 0.0, 0.0),
            emissive_strength: EMISSIVE_STRENGTH_DEFAULT,
            occlusion_strength: OCCLUSION_STRENGTH_DEFAULT,
        }
    }
}
