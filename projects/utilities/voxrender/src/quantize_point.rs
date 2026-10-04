use crate::{GRID_FRACTION_BITS, GRID_RANGE_BITS};
use ty_math::TyVector3F64;

/// `point` in grid units rounded to fixed point, or `None` when a coordinate
/// is not finite or lies out of [range](GRID_RANGE_BITS).
pub fn quantize_point(point: TyVector3F64) -> Option<[i64; 3]> {
    let range = 2f64.powi(GRID_RANGE_BITS as i32);
    let unit = 2f64.powi(GRID_FRACTION_BITS as i32);

    let mut fixed = [0; 3];

    for (fixed, coordinate) in fixed.iter_mut().zip(point.to_array()) {
        if !coordinate.is_finite() || coordinate.abs() >= range {
            return None;
        }

        *fixed = (coordinate * unit).round() as i64;
    }

    Some(fixed)
}

#[cfg(test)]
mod tests {
    use crate::quantize_point;
    use ty_math::TyVector3F64;

    #[test]
    fn a_point_rounds_to_fixed_point_within_the_range() {
        assert_eq!(
            quantize_point(TyVector3F64::new(1.0, -0.5, 0.3)),
            Some([8192, -4096, 2458])
        );

        let range = 2f64.powi(32);
        assert!(quantize_point(TyVector3F64::new(range - 1.0, 0.0, 0.0)).is_some());
        assert_eq!(quantize_point(TyVector3F64::new(0.0, -range, 0.0)), None);
        assert_eq!(quantize_point(TyVector3F64::new(0.0, 0.0, f64::NAN)), None);
    }
}
