use crate::{GRID_GUARD_BITS, RenderGridRay};
use std::array;

/// A view's pixel rays in one placement's grid as integer steps, so every
/// renderer derives the same ray for a pixel. Made by
/// [`RenderViewRays::to_grid_rays`](crate::RenderViewRays::to_grid_rays).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderGridViewRays {
    pub(crate) width: u32,

    pub(crate) height: u32,

    // Pixel (0, 0)'s origin and direction and the steps along a row (`_x`)
    // and down a column (`_y`), with `GRID_GUARD_BITS` extra fraction bits.
    pub(crate) origin: [i64; 3],

    pub(crate) origin_x: [i64; 3],

    pub(crate) origin_y: [i64; 3],

    pub(crate) direction: [i64; 3],

    pub(crate) direction_x: [i64; 3],

    pub(crate) direction_y: [i64; 3],
}

impl RenderGridViewRays {
    /// The ray through the center of pixel `(x, y)`. Panics if the pixel is
    /// outside the image.
    pub fn ray(&self, x: u32, y: u32) -> RenderGridRay {
        assert!(
            x < self.width && y < self.height,
            "the pixel is within the image"
        );

        let x = i128::from(x);
        let y = i128::from(y);

        let step = |base: [i64; 3], along_x: [i64; 3], along_y: [i64; 3]| -> [i64; 3] {
            array::from_fn(|a| {
                round_guard(
                    i128::from(base[a]) + i128::from(along_x[a]) * x + i128::from(along_y[a]) * y,
                )
            })
        };

        let direction = step(self.direction, self.direction_x, self.direction_y).map(|component| {
            i32::try_from(component).expect("a quantized direction fits the direction bits")
        });

        RenderGridRay {
            origin: step(self.origin, self.origin_x, self.origin_y),
            direction,
            end: None,
        }
    }
}

/// `value` with its guard bits rounded off. Halves round away from zero.
fn round_guard(value: i128) -> i64 {
    let half = 1i128 << (GRID_GUARD_BITS - 1);
    let rounded = value.signum() * ((value.abs() + half) >> GRID_GUARD_BITS);

    i64::try_from(rounded).expect("a quantized coordinate fits i64")
}

#[cfg(test)]
mod tests {
    use crate::{
        RenderProjection, RenderView, RenderViewRays, grid_point, grid_vector, quantize_point,
    };
    use ty_math::{TyPoseF64, TyQuaternionF64, TyTransformF64, TyVector3F64};

    #[test]
    fn each_grid_ray_follows_its_float_ray() {
        let pose = TyPoseF64::new(
            TyVector3F64::new(3.0, 4.0, 20.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.2, -0.3, 0.1).normalize(), 0.4),
        );
        let transform = TyTransformF64::new(
            TyVector3F64::new(-1.0, 2.0, 0.5),
            TyQuaternionF64::from_rotation_y(0.7),
            TyVector3F64::new(0.5, 0.25, 0.5),
        );

        for projection in [
            RenderProjection::Perspective { fov: 0.8 },
            RenderProjection::Orthographic { scale: 6.0 },
        ] {
            let rays = RenderViewRays::new(&RenderView { pose, projection }, 37, 23);
            let grid = rays.to_grid_rays(&transform).unwrap();

            for y in 0..23 {
                for x in 0..37 {
                    let float = rays.ray(x, y);
                    let ray = grid.ray(x, y);

                    let origin = quantize_point(grid_point(&transform, float.origin)).unwrap();
                    assert!(
                        (0..3).all(|a| (ray.origin[a] - origin[a]).abs() <= 1),
                        "{ray:?}"
                    );

                    assert!(ray.direction.iter().all(|c| c.abs() <= 1 << 16));

                    let direction = TyVector3F64::from_array(ray.direction.map(f64::from));
                    let exact = grid_vector(&transform, float.direction);
                    assert!(
                        (direction.normalize() - exact.normalize()).length() < 1e-4,
                        "{ray:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_view_out_of_a_grids_range_has_no_grid_rays() {
        let view = RenderView {
            pose: TyPoseF64::new(TyVector3F64::new(0.0, 0.0, 5.0), TyQuaternionF64::IDENTITY),
            projection: RenderProjection::Perspective { fov: 0.8 },
        };
        let tiny = TyTransformF64::new(
            TyVector3F64::ZERO,
            TyQuaternionF64::IDENTITY,
            TyVector3F64::splat(1e-10),
        );

        assert_eq!(RenderViewRays::new(&view, 4, 4).to_grid_rays(&tiny), None);
    }
}
