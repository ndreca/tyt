use crate::operations::object::{MeshGeometry, Method, mesh_slices};
use voxcore::VoxObject;

/// Triangulates `object`'s live voxels into a [`MeshGeometry`] using `method`.
///
/// The mesh spans the object's build volume in grid units on the grid's axes:
/// a live voxel at grid `(x, y, z)` fills the unit cube
/// `[x, x+1] x [y, y+1] x [z, z+1]`.
/// `naive` emits all six faces of every live voxel, `culled` only the faces on
/// a solid-empty boundary, and `greedy` merges coplanar boundary faces into the
/// fewest quads. No hierarchy-node transform is applied; placement is the
/// caller's to add.
pub fn object_to_mesh_geometry(object: &VoxObject, method: Method) -> MeshGeometry {
    // A constant key merges every coplanar face regardless of material and
    // records no per-vertex material, the fewest-quads pure-geometry mesh.
    mesh_slices(object, method, &|_| 0, &|_| true, false)
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::{Method, object_to_mesh_geometry},
        test_utilities::live_object,
    };
    use ty_math::{TyVector3Ext, TyVector3F32};

    #[test]
    fn naive_emits_all_six_faces_per_voxel() {
        let object = live_object([1, 1, 1], &[[0, 0, 0]]);
        let mesh = object_to_mesh_geometry(&object, Method::Naive);
        assert_eq!(mesh.quad_count(), 6);
        assert_eq!(mesh.indices.len(), 36);
        assert_eq!(mesh.positions.len(), 24);
    }

    #[test]
    fn culled_drops_the_shared_interior_faces() {
        // Two adjacent voxels: 12 faces total, the shared pair is interior.
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Naive).quad_count(),
            12
        );
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Culled).quad_count(),
            10
        );
    }

    #[test]
    fn greedy_merges_a_two_voxel_bar_into_a_box() {
        // A 2x1x1 box exposes one rectangle per face.
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Greedy).quad_count(),
            6
        );
    }

    #[test]
    fn greedy_collapses_a_solid_slab() {
        // 3x3x1 solid: culled = 2*(3*3 + 3*1 + 1*3) = 30 quads; greedy = 6.
        let live: Vec<[u32; 3]> = (0..3)
            .flat_map(|x| (0..3).map(move |y| [x, y, 0]))
            .collect();
        let object = live_object([3, 3, 1], &live);
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Culled).quad_count(),
            30
        );
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Greedy).quad_count(),
            6
        );
    }

    #[test]
    fn single_voxel_spans_the_unit_cube() {
        let object = live_object([1, 1, 1], &[[0, 0, 0]]);
        let mesh = object_to_mesh_geometry(&object, Method::Culled);
        for point in &mesh.positions {
            assert!(
                point.to_array().iter().all(|&c| c == 0.0 || c == 1.0),
                "corner {point:?}"
            );
        }
    }

    #[test]
    fn every_triangle_winds_outward() {
        // A 2x2x2 solid cube exercises all six face directions under greedy.
        let live: Vec<[u32; 3]> = (0..2)
            .flat_map(|x| (0..2).flat_map(move |y| (0..2).map(move |z| [x, y, z])))
            .collect();
        let object = live_object([2, 2, 2], &live);
        let mesh = object_to_mesh_geometry(&object, Method::Greedy);
        assert_eq!(mesh.quad_count(), 6);

        for triangle in mesh.indices.chunks_exact(3) {
            let corner = |i: u32| mesh.positions[i as usize];
            let (p0, p1, p2) = (
                corner(triangle[0]),
                corner(triangle[1]),
                corner(triangle[2]),
            );
            let stored = mesh.normals[triangle[0] as usize];
            assert!(
                TyVector3F32::triangle_normal(p0, p1, p2).dot(stored) > 0.0,
                "triangle winds inward"
            );
        }
    }
}
