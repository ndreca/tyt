use crate::{
    GRID_DIRECTION_BITS, GRID_FRACTION_BITS, RenderRay, grid_point, grid_vector,
    quantize_direction, quantize_point,
};
use std::array;
use ty_math::TyTransformF64;

/// A ray in one placement's grid, quantized for the integer walk. Positions
/// are fixed point with [`GRID_FRACTION_BITS`] fraction bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderGridRay {
    /// Where the ray starts.
    pub origin: [i64; 3],

    /// The direction. Only its proportions matter.
    pub direction: [i32; 3],

    /// The last cell the walk visits, or `None` to run out of the grid. The
    /// walk stops at the first step past it on any axis.
    pub end: Option<[i64; 3]>,
}

impl RenderGridRay {
    /// `ray` carried into the grid of a placement under `transform`, or
    /// `None` when the ray's origin lies out of the grid's
    /// [range](crate::GRID_RANGE_BITS).
    pub fn from_ray(transform: &TyTransformF64, ray: &RenderRay) -> Option<Self> {
        Some(RenderGridRay {
            origin: quantize_point(grid_point(transform, ray.origin))?,
            direction: quantize_direction(grid_vector(transform, ray.direction)),
            end: None,
        })
    }

    /// The ray from `origin` toward `target`, their offset shifted right to
    /// fit [`GRID_DIRECTION_BITS`]. It ends in the cell it reaches at
    /// `target`'s coordinate on its longest axis.
    pub fn toward(origin: [i64; 3], target: [i64; 3]) -> Self {
        let offset: [i128; 3] = array::from_fn(|a| i128::from(target[a]) - i128::from(origin[a]));

        let major = (1..3).fold(0, |major, a| {
            if offset[a].abs() > offset[major].abs() {
                a
            } else {
                major
            }
        });

        let reach = offset[major].abs();
        let shift = (i128::BITS - reach.leading_zeros()).saturating_sub(GRID_DIRECTION_BITS);

        let direction = offset.map(|component| {
            i32::try_from(component.signum() * (component.abs() >> shift))
                .expect("a shifted offset fits the direction bits")
        });

        if reach == 0 {
            return RenderGridRay {
                origin,
                direction,
                end: Some(origin.map(|coordinate| coordinate >> GRID_FRACTION_BITS)),
            };
        }

        // `scaled / span` is the end's coordinate in cells. A point on a
        // boundary belongs to the cell the ray moves into.
        let run = i128::from(direction[major]).abs();
        let span = run << GRID_FRACTION_BITS;

        let end = array::from_fn(|a| {
            let scaled = i128::from(origin[a]) * run + reach * i128::from(direction[a]);

            let cell = if direction[a] < 0 {
                -(-scaled).div_euclid(span) - 1
            } else {
                scaled.div_euclid(span)
            };

            i64::try_from(cell).expect("a cell of an i64 coordinate fits i64")
        });

        RenderGridRay {
            origin,
            direction,
            end: Some(end),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{GRID_FRACTION_BITS, RenderGridRay, RenderRay};
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};

    const ONE: i64 = 1 << GRID_FRACTION_BITS;

    #[test]
    fn a_world_ray_lands_in_the_grid_of_its_placement() {
        let transform = TyTransformF64::new(
            TyVector3F64::new(10.0, 0.0, 0.0),
            TyQuaternionF64::IDENTITY,
            TyVector3F64::splat(2.0),
        );
        let ray = |x| RenderRay {
            origin: TyVector3F64::new(x, 1.0, 0.5),
            direction: -TyVector3F64::Z,
        };

        assert_eq!(
            RenderGridRay::from_ray(&transform, &ray(12.0)),
            Some(RenderGridRay {
                origin: [ONE, ONE / 2, ONE / 4],
                direction: [0, 0, -65536],
                end: None,
            })
        );
        assert_eq!(RenderGridRay::from_ray(&transform, &ray(1e10)), None);
    }

    #[test]
    fn a_ray_toward_a_point_shifts_its_offset_to_fit_and_ends_in_its_cell() {
        let half = ONE / 2;

        let near = RenderGridRay::toward([half; 3], [7 * half, 3 * half, half]);
        assert_eq!(near.direction, [3 * 8192, 8192, 0]);
        assert_eq!(near.end, Some([3, 1, 0]));

        // 100 cells takes 20 bits, a shift of 4. The end's boundaries belong
        // to the cells the ray moves into.
        let far = RenderGridRay::toward([0; 3], [100 * ONE, -ONE, 0]);
        assert_eq!(far.direction, [51200, -512, 0]);
        assert_eq!(far.end, Some([100, -2, 0]));

        assert_eq!(
            RenderGridRay::toward([ONE; 3], [ONE; 3]),
            RenderGridRay {
                origin: [ONE; 3],
                direction: [0; 3],
                end: Some([1; 3]),
            }
        );
    }
}
