use crate::{RenderMaterial, normal_reflectance};
use ty_math::TyLinSrgbF64;

const WHITE: TyLinSrgbF64 = TyLinSrgbF64::new(1.0, 1.0, 1.0);

/// The pass of `material`: the share of the light behind its surface that
/// continues, per channel.
pub fn material_pass(material: &RenderMaterial) -> TyLinSrgbF64 {
    let alpha = material.base_color.alpha;
    let transmitted = (WHITE - normal_reflectance(material))
        * material.base_color.color
        * (material.transmission * (1.0 - material.metallic));

    WHITE * (1.0 - alpha) + transmitted * alpha
}

#[cfg(test)]
mod tests {
    use crate::{RenderMaterial, material_pass, material_pass::WHITE};
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};

    const BLACK: TyLinSrgbF64 = TyLinSrgbF64::new(0.0, 0.0, 0.0);

    fn glass(base_color: TyLinSrgbaF64) -> RenderMaterial {
        RenderMaterial {
            base_color,
            metallic: 0.0,
            roughness: 0.2,
            transmission: 1.0,
            ..RenderMaterial::default()
        }
    }

    fn close(a: TyLinSrgbF64, b: TyLinSrgbF64) -> bool {
        (a.red - b.red).abs() < 1e-9
            && (a.green - b.green).abs() < 1e-9
            && (a.blue - b.blue).abs() < 1e-9
    }

    #[test]
    fn the_pass_is_the_uncovered_part_plus_what_the_covered_part_transmits() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);

        assert_eq!(material_pass(&RenderMaterial::default()), BLACK);
        assert_eq!(
            material_pass(&RenderMaterial {
                base_color: TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.0),
                metallic: 0.0,
                ..RenderMaterial::default()
            }),
            WHITE
        );
        assert!(close(
            material_pass(&glass(white)),
            TyLinSrgbF64::new(0.96, 0.96, 0.96)
        ));
        assert!(close(
            material_pass(&glass(TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0))),
            TyLinSrgbF64::new(0.96, 0.0, 0.0)
        ));
        assert!(close(
            material_pass(&glass(TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5))),
            TyLinSrgbF64::new(0.98, 0.98, 0.98)
        ));

        // A metal and a mirror reflect everything they cover.
        assert_eq!(
            material_pass(&RenderMaterial {
                metallic: 1.0,
                ..glass(white)
            }),
            BLACK
        );
        assert_eq!(
            material_pass(&RenderMaterial {
                ior: 0.0,
                ..glass(white)
            }),
            BLACK
        );
    }
}
