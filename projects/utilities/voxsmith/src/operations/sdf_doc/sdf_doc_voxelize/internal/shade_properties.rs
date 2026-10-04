use crate::operations::sdf_doc::{ArgumentCheck, SdfMaterialProperties};
use sdfcore::SdfShades;
use std::result::Result as StdResult;
use ty_math::TyLinSrgbaF64;

/// The lightness step between neighboring shades when the model leaves it out.
const DEFAULT_SPREAD: f64 = 0.08;

/// The shade at `index` of `shades`, whose base material holds `base`. The
/// shade steps the base color's Oklab lightness, and every other property
/// carries over.
pub fn shade_properties(
    base: &SdfMaterialProperties,
    shades: &SdfShades,
    index: u32,
) -> StdResult<SdfMaterialProperties, String> {
    let spread = shades.spread.unwrap_or(DEFAULT_SPREAD);
    let step = (f64::from(index) - (shades.count - 1.0) / 2.0) * spread;

    if step == 0.0 {
        return Ok(base.clone());
    }

    let color = base.base_color;
    let [lightness, a, b] = oklab([color.red, color.green, color.blue]);
    let rgb = lin_srgb([lightness + step, a, b]);

    if rgb.iter().any(|component| !(0.0..=1.0).contains(component)) {
        return Err(ArgumentCheck::new("shades").fail(
            &format!("shade {index}"),
            "within [0, 1] in linear light",
            format!("[{}, {}, {}]", rgb[0], rgb[1], rgb[2]),
        ));
    }

    Ok(SdfMaterialProperties {
        base_color: TyLinSrgbaF64::new(rgb[0], rgb[1], rgb[2], color.alpha),
        ..base.clone()
    })
}

/// The linear sRGB color `rgb` in Oklab, by Bjorn Ottosson's matrices.
fn oklab([r, g, b]: [f64; 3]) -> [f64; 3] {
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();

    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

/// The Oklab color `lab` in linear sRGB, by Bjorn Ottosson's matrices.
fn lin_srgb([lightness, a, b]: [f64; 3]) -> [f64; 3] {
    let l = (lightness + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m = (lightness - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s = (lightness - 0.089_484_177_5 * a - 1.291_485_548_0 * b).powi(3);

    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    ]
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        material_properties,
        sdf_doc_voxelize::internal::shade_properties::{lin_srgb, oklab},
        shade_properties,
    };
    use branded_id::U32Id;
    use sdfcore::{SdfProperty, SdfPropertyValue, SdfShades};

    /// A `shades` call of `count` shades `spread` apart.
    fn shades(count: f64, spread: Option<f64>) -> SdfShades {
        SdfShades {
            base_id: U32Id::from_u32(0),
            count,
            spread,
        }
    }

    /// The Oklab lightness of the shade at `index` of `call` over `color`.
    fn lightness(color: &str, call: &SdfShades, index: u32) -> f64 {
        let base = material_properties(&[SdfProperty {
            name: "baseColor".to_owned(),
            value: SdfPropertyValue::Text(color.to_owned()),
        }])
        .unwrap();
        let shade = shade_properties(&base, call, index).unwrap().base_color;
        oklab([shade.red, shade.green, shade.blue])[0]
    }

    #[test]
    fn oklab_round_trips_and_reads_white_as_full_lightness() {
        let white = oklab([1.0, 1.0, 1.0]);
        assert!((white[0] - 1.0).abs() < 1e-6 && white[1].abs() < 1e-6);

        let color = [0.2, 0.5, 0.7];
        let back = lin_srgb(oklab(color));
        for (component, expected) in back.iter().zip(color) {
            assert!((component - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn the_shades_step_lightness_around_the_original_in_the_middle() {
        let call = shades(3.0, None);
        let middle = lightness("#8A5A2B", &call, 1);

        assert!((lightness("#8A5A2B", &call, 0) - (middle - 0.08)).abs() < 1e-6);
        assert!((lightness("#8A5A2B", &call, 2) - (middle + 0.08)).abs() < 1e-6);

        let even = shades(2.0, Some(0.1));
        assert!((lightness("#8A5A2B", &even, 1) - (middle + 0.05)).abs() < 1e-6);
    }

    #[test]
    fn the_middle_shade_keeps_the_base_and_every_other_property_carries_over() {
        let base = material_properties(&[
            SdfProperty {
                name: "baseColor".to_owned(),
                value: SdfPropertyValue::Text("#8A5A2B80".to_owned()),
            },
            SdfProperty {
                name: "roughness".to_owned(),
                value: SdfPropertyValue::Number(0.25),
            },
        ])
        .unwrap();

        assert_eq!(
            shade_properties(&base, &shades(3.0, None), 1).unwrap(),
            base
        );

        let darker = shade_properties(&base, &shades(3.0, Some(0.01)), 0).unwrap();
        assert_eq!(darker.base_color.alpha, base.base_color.alpha);
        assert_eq!(darker.roughness, 0.25);
    }

    #[test]
    fn a_shade_past_white_errors() {
        let white = material_properties(&[SdfProperty {
            name: "baseColor".to_owned(),
            value: SdfPropertyValue::Text("#FFFFFF".to_owned()),
        }])
        .unwrap();

        let error = shade_properties(&white, &shades(3.0, None), 2).unwrap_err();
        assert!(
            error.starts_with("shades shade 2 must be within [0, 1] in linear light, not ["),
            "{error}"
        );
        assert!(shade_properties(&white, &shades(3.0, None), 0).is_ok());
    }
}
