use crate::{BRenderPlacement, RenderHit, RenderObject, RenderPlacement, RenderRay, RenderScene};
use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxsurface::{SurfaceGrid, SurfaceSpan};

/// The nearest voxel that `ray` meets within `max_distance` across every
/// placement of `scene`. The ray enters each placement through the inverse
/// of its transform, clips to the grid, and steps cell to cell. A ray that
/// starts inside a cell leaves that cell before it can hit anything.
pub fn cast_ray(scene: &RenderScene, ray: &RenderRay, max_distance: f64) -> Option<RenderHit> {
    let mut nearest: Option<RenderHit> = None;

    for (placement_id, placement) in scene.iter_placements() {
        let limit = nearest.map_or(max_distance, |hit| hit.distance);

        let object = scene
            .object(placement.object_id)
            .expect("a placement's object is one of the scene's");

        if let Some(hit) = cast_placement(placement_id, placement, object, ray, limit) {
            nearest = Some(hit);
        }
    }

    nearest
}

/// The nearest cell of `object` under `placement` that `ray` meets within
/// `max_distance`.
fn cast_placement(
    placement_id: U32Id<BRenderPlacement>,
    placement: &RenderPlacement,
    object: &RenderObject,
    ray: &RenderRay,
    max_distance: f64,
) -> Option<RenderHit> {
    let transform = &placement.transform;
    let inverse = transform.rotation.inverse();

    // The ray in grid units. The parameter stays the world distance.
    let origin = (inverse * (ray.origin - transform.position)) / transform.scale;
    let direction = (inverse * ray.direction) / transform.scale;

    let bounds = object.bounds().as_dvec3();

    // Clip to the grid's slab, keeping the axis the ray enters through.
    let mut t_enter = f64::NEG_INFINITY;
    let mut t_exit = f64::INFINITY;
    let mut entry_axis = 0;

    for axis in 0..3 {
        if direction[axis] == 0.0 {
            if origin[axis] < 0.0 || origin[axis] >= bounds[axis] {
                return None;
            }

            continue;
        }

        let t0 = -origin[axis] / direction[axis];
        let t1 = (bounds[axis] - origin[axis]) / direction[axis];

        if t0.min(t1) > t_enter {
            t_enter = t0.min(t1);
            entry_axis = axis;
        }

        t_exit = t_exit.min(t0.max(t1));
    }

    if t_exit < t_enter || t_exit < 0.0 || t_enter > max_distance {
        return None;
    }

    let start = t_enter.max(0.0);
    let point = origin + direction * start;

    let mut cell = [0i64; 3];
    let mut step = [0i64; 3];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];

    for axis in 0..3 {
        cell[axis] = (point[axis].floor() as i64).clamp(0, bounds[axis] as i64 - 1);

        if direction[axis] > 0.0 {
            step[axis] = 1;
            t_max[axis] = ((cell[axis] + 1) as f64 - origin[axis]) / direction[axis];
            t_delta[axis] = 1.0 / direction[axis];
        } else if direction[axis] < 0.0 {
            step[axis] = -1;
            t_max[axis] = (cell[axis] as f64 - origin[axis]) / direction[axis];
            t_delta[axis] = -1.0 / direction[axis];
        }
    }

    let hit = |cell: [i64; 3], axis: usize, distance: f64| {
        let position = TyVector3U32::new(cell[0] as u32, cell[1] as u32, cell[2] as u32);

        let voxel_id = object.cell(position)?;

        let face = SurfaceSpan {
            d: axis,
            sign: -step[axis] as i32,
            s: position[axis],
            u0: position[(axis + 1) % 3] as usize,
            u1: position[(axis + 1) % 3] as usize + 1,
            v0: position[(axis + 2) % 3] as usize,
            v1: position[(axis + 2) % 3] as usize + 1,
        };

        let point = origin + direction * distance;
        let along = [face.u(), face.v()]
            .map(|tangent| (point[tangent] - cell[tangent] as f64).clamp(0.0, 1.0));

        Some(RenderHit {
            placement_id,
            voxel_id,
            face,
            along,
            distance,
        })
    };

    // A ray from outside may meet a solid cell on the slab's face.
    if t_enter >= 0.0
        && let Some(hit) = hit(cell, entry_axis, t_enter)
    {
        return Some(hit);
    }

    loop {
        let axis = (0..3)
            .min_by(|&a, &b| t_max[a].total_cmp(&t_max[b]))
            .expect("three axes");

        let distance = t_max[axis];

        if distance > max_distance {
            return None;
        }

        cell[axis] += step[axis];

        if cell[axis] < 0 || cell[axis] >= bounds[axis] as i64 {
            return None;
        }

        t_max[axis] += t_delta[axis];

        if let Some(hit) = hit(cell, axis, distance) {
            return Some(hit);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{RenderMaterial, RenderObject, RenderPlacement, RenderRay, RenderScene, cast_ray};
    use branded_id::U32Id;
    use std::f64::consts::PI;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};

    /// A scene with one object of `bounds` live at `live`, under each of
    /// `transforms`.
    fn scene(bounds: [u32; 3], live: &[[u32; 3]], transforms: &[TyTransformF64]) -> RenderScene {
        let mut scene = RenderScene::default();
        let material_id = scene.retain_material(RenderMaterial::default()).unwrap();

        let mut object =
            RenderObject::new("o".to_owned(), TyVector3U32::from_array(bounds)).unwrap();
        for &position in live {
            let voxel_id = object.voxel_id(TyVector3U32::from_array(position)).unwrap();
            object
                .set_voxel_material(voxel_id, Some(material_id))
                .unwrap();
        }
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();

        for &transform in transforms {
            scene
                .retain_placement(RenderPlacement {
                    object_id,
                    transform,
                })
                .unwrap();
        }

        scene
    }

    fn ray(origin: [f64; 3], direction: [f64; 3]) -> RenderRay {
        RenderRay {
            origin: TyVector3F64::from_array(origin),
            direction: TyVector3F64::from_array(direction).normalize(),
        }
    }

    #[test]
    fn a_ray_meets_each_face_of_a_cube_where_it_lands() {
        let scene = scene([1, 1, 1], &[[0, 0, 0]], &[TyTransformF64::IDENTITY]);

        // Rays come from three units out along each axis, both ways. The
        // hit's face faces the ray, and the hit lands a quarter along each
        // tangent.
        for axis in 0..3 {
            for sign in [-1i32, 1] {
                let mut origin = [0.25; 3];
                origin[axis] = if sign > 0 { 4.0 } else { -3.0 };
                let mut direction = [0.0; 3];
                direction[axis] = -f64::from(sign);

                let hit = cast_ray(&scene, &ray(origin, direction), f64::INFINITY).unwrap();

                assert_eq!(hit.face.d, axis);
                assert_eq!(hit.face.sign, sign);
                assert_eq!(hit.face.s, 0);
                assert_eq!(hit.along, [0.25, 0.25]);
                assert_eq!(hit.distance, 3.0);
            }
        }
    }

    #[test]
    fn a_ray_beside_the_cube_misses() {
        let scene = scene([1, 1, 1], &[[0, 0, 0]], &[TyTransformF64::IDENTITY]);

        assert_eq!(
            cast_ray(
                &scene,
                &ray([0.5, 1.5, 4.0], [0.0, 0.0, -1.0]),
                f64::INFINITY
            ),
            None
        );
        assert_eq!(
            cast_ray(
                &scene,
                &ray([0.5, 0.5, 4.0], [0.0, 0.0, 1.0]),
                f64::INFINITY
            ),
            None
        );
        assert_eq!(
            cast_ray(&scene, &ray([0.5, 0.5, 4.0], [0.0, 0.0, -1.0]), 2.0),
            None
        );
    }

    #[test]
    fn a_ray_from_inside_a_voxel_leaves_it_first() {
        let scene = scene(
            [2, 1, 1],
            &[[0, 0, 0], [1, 0, 0]],
            &[TyTransformF64::IDENTITY],
        );

        let hit = cast_ray(
            &scene,
            &ray([0.5, 0.5, 0.5], [1.0, 0.0, 0.0]),
            f64::INFINITY,
        )
        .unwrap();
        assert_eq!(hit.face.d, 0);
        assert_eq!(hit.face.sign, -1);
        assert_eq!(hit.face.s, 1);
        assert_eq!(hit.distance, 0.5);

        assert_eq!(
            cast_ray(
                &scene,
                &ray([0.5, 0.5, 0.5], [-1.0, 0.0, 0.0]),
                f64::INFINITY
            ),
            None
        );
    }

    #[test]
    fn the_nearer_placement_wins_and_the_transform_applies() {
        let far = TyTransformF64::from_translation(TyVector3F64::new(0.0, 0.0, -5.0));
        let stacked = scene([1, 1, 1], &[[0, 0, 0]], &[far, TyTransformF64::IDENTITY]);

        let hit = cast_ray(
            &stacked,
            &ray([0.5, 0.5, 10.0], [0.0, 0.0, -1.0]),
            f64::INFINITY,
        )
        .unwrap();
        assert_eq!(hit.distance, 9.0);
        assert_eq!(
            hit.placement_id,
            stacked.iter_placements().nth(1).unwrap().0
        );

        // Scaled by two and turned a quarter about y at x = 10, the cube
        // spans 10..12 on x, 0..2 on y, and -2..0 on z. A ray down -Z at
        // (11, 1) meets its +z side at z = 0.
        let turned = TyTransformF64::new(
            TyVector3F64::new(10.0, 0.0, 0.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::Y, PI / 2.0),
            TyVector3F64::splat(2.0),
        );
        let turned_scene = scene([1, 1, 1], &[[0, 0, 0]], &[turned]);

        let hit = cast_ray(
            &turned_scene,
            &ray([11.0, 1.0, 5.0], [0.0, 0.0, -1.0]),
            f64::INFINITY,
        )
        .unwrap();
        assert!((hit.distance - 5.0).abs() < 1e-9);
        // Grid +X turns onto world -Z, so the ray runs along grid +X and
        // meets the grid's -x face.
        assert_eq!(hit.face.d, 0);
        assert_eq!(hit.face.sign, -1);
    }
}
