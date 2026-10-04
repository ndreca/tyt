use crate::RenderMaterial;
use ty_math::TyLinSrgbF64;

/// The reflectance of `material` at normal incidence. The dielectric share
/// follows from the index of refraction as `((ior - 1) / (ior + 1))^2`.
pub fn normal_reflectance(material: &RenderMaterial) -> TyLinSrgbF64 {
    let ior = material.ior;
    let dielectric = ((ior - 1.0) / (ior + 1.0)).powi(2);
    let dielectric = TyLinSrgbF64::new(dielectric, dielectric, dielectric);

    dielectric * (1.0 - material.metallic) + material.base_color.color * material.metallic
}

#[cfg(test)]
mod tests {
    use crate::{RenderMaterial, normal_reflectance};
    use ty_math::TyLinSrgbF64;

    #[test]
    fn the_index_of_refraction_sets_the_dielectric_reflectance() {
        let glass = RenderMaterial {
            metallic: 0.0,
            ..RenderMaterial::default()
        };
        let f0 = normal_reflectance(&glass);
        assert!((f0.red - 0.04).abs() < 1e-12);
        assert_eq!(f0.green, f0.red);
        assert_eq!(f0.blue, f0.red);

        let mirror = RenderMaterial {
            metallic: 0.0,
            ior: 0.0,
            ..RenderMaterial::default()
        };
        assert_eq!(
            normal_reflectance(&mirror),
            TyLinSrgbF64::new(1.0, 1.0, 1.0)
        );
    }
}
