use crate::{
    BRenderMaterial, BRenderPlacement, RenderHit, RenderObject, RenderPlacement, RenderRay,
    RenderScene,
};
use branded_id::U32Id;
use ty_math::{TyVector3F64, TyVector3U32};
use voxsurface::SurfaceSpan;

/// The surface hits a ray meets within a distance across every placement
/// of a scene, nearest first. Each placement walks its grid on its own,
/// remembering the material of the cell the ray is inside: nothing before
/// the ray enters the grid and nothing after it enters an empty cell.
/// Entering a live cell whose material differs is a hit through the face
/// the ray came in by. Entering a cell of the same material or an empty
/// cell yields nothing, so a slab of one material shows one face and no
/// back face. A ray that starts inside a cell leaves that cell's material
/// before it can hit anything.
#[derive(Clone, Debug)]
pub struct RenderRayWalk<'a> {
    walks: Vec<PlacementWalk<'a>>,

    max_distance: f64,
}

impl<'a> RenderRayWalk<'a> {
    /// The walk of `ray` through `scene` out to `max_distance`.
    pub fn new(scene: &'a RenderScene, ray: &RenderRay, max_distance: f64) -> Self {
        let walks = scene
            .iter_placements()
            .filter_map(|(placement_id, placement)| {
                let object = scene
                    .object(placement.object_id)
                    .expect("a placement's object is one of the scene's");

                PlacementWalk::new(placement_id, placement, object, ray)
            })
            .collect();

        RenderRayWalk {
            walks,
            max_distance,
        }
    }
}

/// Each placement advances only as far as the nearest hit another placement
/// has already found, so a placement behind a near hit never walks past it.
impl Iterator for RenderRayWalk<'_> {
    type Item = RenderHit;

    fn next(&mut self) -> Option<RenderHit> {
        let mut nearest: Option<(usize, f64)> = None;

        for (index, walk) in self.walks.iter_mut().enumerate() {
            let limit = nearest.map_or(self.max_distance, |(_, distance)| distance);

            if walk.peeked.is_none() {
                walk.peeked = walk.advance(limit);
            }

            if let Some(hit) = walk.peeked
                && nearest.is_none_or(|(_, distance)| hit.distance < distance)
            {
                nearest = Some((index, hit.distance));
            }
        }

        let (index, _) = nearest?;

        self.walks[index].peeked.take()
    }
}

/// One placement's walk: a DDA through its grid in grid units, with the
/// world distance as the parameter.
#[derive(Clone, Debug)]
struct PlacementWalk<'a> {
    placement_id: U32Id<BRenderPlacement>,

    object: &'a RenderObject,

    /// The ray's origin in grid units.
    origin: TyVector3F64,

    /// The ray's direction in grid units per meter.
    direction: TyVector3F64,

    bounds: [i64; 3],

    cell: [i64; 3],

    step: [i64; 3],

    /// The distance to the next cell boundary along each axis.
    t_max: [f64; 3],

    /// The distance between cell boundaries along each axis.
    t_delta: [f64; 3],

    /// The slab face the ray enters the grid through and the distance there,
    /// until the walk takes it.
    entry: Option<(usize, f64)>,

    /// The material of the cell the ray is inside.
    inside: Option<U32Id<BRenderMaterial>>,

    /// Whether the ray has left the grid.
    done: bool,

    /// The next hit, found but not yet yielded.
    peeked: Option<RenderHit>,
}

impl<'a> PlacementWalk<'a> {
    /// The walk of `ray` through `object` under `placement`, or `None` when
    /// the ray misses the grid.
    fn new(
        placement_id: U32Id<BRenderPlacement>,
        placement: &RenderPlacement,
        object: &'a RenderObject,
        ray: &RenderRay,
    ) -> Option<Self> {
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

        if t_exit < t_enter || t_exit < 0.0 {
            return None;
        }

        let point = origin + direction * t_enter.max(0.0);

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

        // A ray from outside enters through the slab's face. One from inside
        // starts inside its cell's material.
        let (entry, inside) = if t_enter >= 0.0 {
            (Some((entry_axis, t_enter)), None)
        } else {
            let voxel_id = object
                .voxel_id(position_of(cell))
                .expect("the start cell is within the grid");

            (None, object.voxel_material(voxel_id))
        };

        Some(PlacementWalk {
            placement_id,
            object,
            origin,
            direction,
            bounds: bounds.to_array().map(|side| side as i64),
            cell,
            step,
            t_max,
            t_delta,
            entry,
            inside,
            done: false,
            peeked: None,
        })
    }

    /// The next hit within `limit`, or `None` when the ray has left the
    /// grid or its next cell lies past `limit`. A walk stopped at `limit`
    /// stays where it is and resumes on the next call.
    fn advance(&mut self, limit: f64) -> Option<RenderHit> {
        if self.done {
            return None;
        }

        if let Some((axis, distance)) = self.entry {
            if distance > limit {
                return None;
            }

            self.entry = None;

            if let Some(hit) = self.enter(axis, distance) {
                return Some(hit);
            }
        }

        loop {
            let axis = (0..3)
                .min_by(|&a, &b| self.t_max[a].total_cmp(&self.t_max[b]))
                .expect("three axes");

            let distance = self.t_max[axis];

            if distance > limit {
                return None;
            }

            self.cell[axis] += self.step[axis];

            if self.cell[axis] < 0 || self.cell[axis] >= self.bounds[axis] {
                self.done = true;

                return None;
            }

            self.t_max[axis] += self.t_delta[axis];

            if let Some(hit) = self.enter(axis, distance) {
                return Some(hit);
            }
        }
    }

    /// Enters the current cell through its face on `axis` at `distance`: a
    /// hit when the cell is live and its material differs from the one the
    /// ray was inside.
    fn enter(&mut self, axis: usize, distance: f64) -> Option<RenderHit> {
        let position = position_of(self.cell);

        let voxel_id = self
            .object
            .voxel_id(position)
            .expect("the walk stays within the grid");

        let material_id = self.object.voxel_material(voxel_id);

        if material_id == self.inside {
            return None;
        }

        self.inside = material_id;

        // Entering an empty cell is not a hit.
        material_id?;

        let face = SurfaceSpan {
            d: axis,
            sign: -self.step[axis] as i32,
            s: position[axis],
            u0: position[(axis + 1) % 3] as usize,
            u1: position[(axis + 1) % 3] as usize + 1,
            v0: position[(axis + 2) % 3] as usize,
            v1: position[(axis + 2) % 3] as usize + 1,
        };

        let point = self.origin + self.direction * distance;
        let along = [face.u(), face.v()]
            .map(|tangent| (point[tangent] - self.cell[tangent] as f64).clamp(0.0, 1.0));

        Some(RenderHit {
            placement_id: self.placement_id,
            voxel_id,
            face,
            along,
            distance,
        })
    }
}

/// The grid position of `cell`, which lies within the grid.
fn position_of(cell: [i64; 3]) -> TyVector3U32 {
    TyVector3U32::new(cell[0] as u32, cell[1] as u32, cell[2] as u32)
}

#[cfg(test)]
mod tests {
    use crate::{RenderRay, RenderRayWalk, RenderScene, test_utilities::cells_scene};
    use ty_math::{TyTransformF64, TyVector3F64};

    fn ray(origin: [f64; 3], direction: [f64; 3]) -> RenderRay {
        RenderRay {
            origin: TyVector3F64::from_array(origin),
            direction: TyVector3F64::from_array(direction).normalize(),
        }
    }

    /// The face layer and distance of each hit along +X from `x`.
    fn layers_along_x(scene: &RenderScene, x: f64) -> Vec<(u32, f64)> {
        RenderRayWalk::new(scene, &ray([x, 0.5, 0.5], [1.0, 0.0, 0.0]), f64::INFINITY)
            .map(|hit| {
                assert_eq!(hit.face.d, 0);
                assert_eq!(hit.face.sign, -1);

                (hit.face.s, hit.distance)
            })
            .collect()
    }

    #[test]
    fn one_material_shows_one_face_and_two_show_their_seam() {
        let one = cells_scene(
            [2, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 0)],
            &[TyTransformF64::IDENTITY],
        );
        assert_eq!(layers_along_x(&one, -1.0), [(0, 1.0)]);

        let two = cells_scene(
            [2, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
            &[TyTransformF64::IDENTITY],
        );
        assert_eq!(layers_along_x(&two, -1.0), [(0, 1.0), (1, 2.0)]);
    }

    #[test]
    fn a_hollow_box_shows_its_near_wall_and_its_far_wall() {
        let shell: Vec<_> = (0..27u32)
            .map(|index| [index / 9, index / 3 % 3, index % 3])
            .filter(|&position| position != [1, 1, 1])
            .map(|position| (position, 0))
            .collect();
        let scene = cells_scene([3, 3, 3], &shell, &[TyTransformF64::IDENTITY]);

        let hits: Vec<_> = RenderRayWalk::new(
            &scene,
            &ray([-1.0, 1.5, 1.5], [1.0, 0.0, 0.0]),
            f64::INFINITY,
        )
        .map(|hit| (hit.face.s, hit.distance))
        .collect();
        assert_eq!(hits, [(0, 1.0), (2, 3.0)]);
    }

    #[test]
    fn a_ray_from_inside_leaves_its_material_first() {
        let scene = cells_scene(
            [3, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 0), ([2, 0, 0], 1)],
            &[TyTransformF64::IDENTITY],
        );

        assert_eq!(layers_along_x(&scene, 0.5), [(2, 1.5)]);
        assert_eq!(layers_along_x(&scene, 1.5), [(2, 0.5)]);
        assert_eq!(layers_along_x(&scene, 2.5), []);
    }

    #[test]
    fn placements_interleave_by_distance_and_the_cap_ends_the_walk() {
        // A bar live at both ends, placed at the origin and again two and
        // a half units down -Z, so the hits alternate between the two.
        let scene = cells_scene(
            [1, 1, 5],
            &[([0, 0, 0], 0), ([0, 0, 4], 0)],
            &[
                TyTransformF64::IDENTITY,
                TyTransformF64::from_translation(TyVector3F64::new(0.0, 0.0, -2.5)),
            ],
        );
        let ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();
        let down = ray([0.5, 0.5, 10.0], [0.0, 0.0, -1.0]);

        let hits: Vec<_> = RenderRayWalk::new(&scene, &down, f64::INFINITY)
            .map(|hit| (hit.placement_id, hit.distance))
            .collect();
        assert_eq!(
            hits,
            [(ids[0], 5.0), (ids[1], 7.5), (ids[0], 9.0), (ids[1], 11.5)]
        );

        let capped: Vec<_> = RenderRayWalk::new(&scene, &down, 9.0)
            .map(|hit| hit.distance)
            .collect();
        assert_eq!(capped, [5.0, 7.5, 9.0]);
    }
}
