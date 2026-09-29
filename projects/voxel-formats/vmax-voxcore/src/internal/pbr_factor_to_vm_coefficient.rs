use crate::{COEFFICIENT_MIN, COEFFICIENT_SPAN, Error, Result};

/// Maps a 0 to 1 glTF metalness or roughness factor to the Voxel Max
/// coefficient whose slider reads the same percent. A `factor` outside 0 to 1
/// returns an error naming `key`.
pub fn pbr_factor_to_vm_coefficient(factor: f64, key: &str) -> Result<f64> {
    if !(0.0..=1.0).contains(&factor) {
        return Err(Error::invalid(format!(
            "`{key}` is {factor}, outside the glTF range 0 to 1, so no Voxel Max slider \
             coefficient reads it"
        )));
    }

    Ok(COEFFICIENT_MIN + factor * COEFFICIENT_SPAN)
}

#[cfg(test)]
mod tests {
    use crate::{pbr_factor_to_vm_coefficient, vm_coefficient_to_pbr_factor};
    use voxcore::material::METALLIC;

    /// Whether two coefficients agree within f64 rounding of the linear map.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// The coefficient for `factor`, read as metalness.
    fn coefficient(factor: f64) -> f64 {
        pbr_factor_to_vm_coefficient(factor, METALLIC).unwrap()
    }

    #[test]
    fn maps_factor_endpoints_onto_the_coefficient_range() {
        assert_eq!(coefficient(0.0), 0.1);
        assert_eq!(coefficient(0.5), 0.5);
        assert!(close(coefficient(1.0), 0.9));
    }

    #[test]
    fn rejects_a_factor_outside_the_gltf_range() {
        assert!(pbr_factor_to_vm_coefficient(-1.0, METALLIC).is_err());
        assert!(pbr_factor_to_vm_coefficient(2.0, METALLIC).is_err());
    }

    #[test]
    fn round_trips_a_factor_through_a_coefficient() {
        for percent in 0..=100 {
            let start = f64::from(percent) / 100.0;
            let back = vm_coefficient_to_pbr_factor(coefficient(start));
            assert!(close(back, start), "{start} -> {back}");
        }
    }
}
