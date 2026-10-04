use crate::{RenderMaterial, material_pass};
use ty_math::TyLinSrgbF64;

/// Whether `material` passes no light.
pub fn is_opaque(material: &RenderMaterial) -> bool {
    material_pass(material) == TyLinSrgbF64::new(0.0, 0.0, 0.0)
}
