use crate::{COEFFICIENT_MIN, COEFFICIENT_SPAN};

/// The highest coefficient a slider produces, its 100 percent point.
const COEFFICIENT_MAX: f64 = COEFFICIENT_MIN + COEFFICIENT_SPAN;

/// Maps a Voxel Max metalness or roughness coefficient onto the 0 to 1 glTF
/// factor its slider shows. A coefficient off the slider span has no glTF
/// factor and projects to the nearest end, while the ext block keeps the exact
/// coefficient the write-back needs.
pub fn vm_coefficient_to_pbr_factor(coefficient: f64) -> f64 {
    let clamped = coefficient.clamp(COEFFICIENT_MIN, COEFFICIENT_MAX);

    (clamped - COEFFICIENT_MIN) / COEFFICIENT_SPAN
}

#[cfg(test)]
mod tests {
    use crate::vm_coefficient_to_pbr_factor;

    /// Whether two factors agree within f64 rounding of the linear map.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn reads_coefficients_back_as_factors() {
        assert_eq!(vm_coefficient_to_pbr_factor(0.1), 0.0);
        assert_eq!(vm_coefficient_to_pbr_factor(0.5), 0.5);
        assert!(close(vm_coefficient_to_pbr_factor(0.9), 1.0));
    }

    #[test]
    fn projects_a_coefficient_off_the_slider_span_to_the_nearest_end() {
        // A file may carry either. The ext block keeps the exact coefficient.
        assert_eq!(vm_coefficient_to_pbr_factor(0.0), 0.0);
        assert_eq!(vm_coefficient_to_pbr_factor(1.0), 1.0);
        assert_eq!(vm_coefficient_to_pbr_factor(f64::INFINITY), 1.0);
    }
}
