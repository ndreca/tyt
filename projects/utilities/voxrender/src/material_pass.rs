use crate::RenderMaterial;
use ty_math::TyLinSrgbF64;
use voxcore::material;

/// The pass of `material`: the share of the light behind its surface that
/// continues, per channel.
pub fn material_pass(material: &RenderMaterial) -> TyLinSrgbF64 {
    material::pass(
        material.base_color,
        material.metallic,
        material.transmission,
        material.ior,
    )
}
