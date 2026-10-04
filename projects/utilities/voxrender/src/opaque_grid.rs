use crate::{RenderObject, RenderScene, material_pass};
use branded_id::U32Id;
use ty_math::{TyLinSrgbF64, TyVector3U32};
use voxcore::BVoxVoxel;
use voxsurface::SurfaceGrid;

const BLACK: TyLinSrgbF64 = TyLinSrgbF64::new(0.0, 0.0, 0.0);

/// A render object's grid as the corner occlusion reads it: a cell is solid
/// only when its material's pass is zero, so glass darkens nothing it
/// encloses.
pub struct OpaqueGrid<'a> {
    scene: &'a RenderScene,

    object: &'a RenderObject,
}

impl<'a> OpaqueGrid<'a> {
    pub fn new(scene: &'a RenderScene, object: &'a RenderObject) -> Self {
        OpaqueGrid { scene, object }
    }
}

impl SurfaceGrid for OpaqueGrid<'_> {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        SurfaceGrid::bounds(self.object)
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.object.cell(position).filter(|&voxel_id| {
            let material_id = self
                .object
                .voxel_material(voxel_id)
                .expect("a solid cell holds a material");

            let material = self
                .scene
                .material(material_id)
                .expect("a voxel samples one of the scene's materials");

            material_pass(material) == BLACK
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{OpaqueGrid, RenderMaterial, RenderObject, RenderScene};
    use branded_id::U32Id;
    use ty_math::{TyLinSrgbaF64, TyVector3U32};
    use voxsurface::SurfaceGrid;

    #[test]
    fn only_a_cell_that_passes_no_light_is_solid() {
        let mut scene = RenderScene::default();
        let opaque_id = scene.retain_material(RenderMaterial::default()).unwrap();
        let glass_id = scene
            .retain_material(RenderMaterial {
                metallic: 0.0,
                transmission: 1.0,
                ..RenderMaterial::default()
            })
            .unwrap();
        let half_id = scene
            .retain_material(RenderMaterial {
                base_color: TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5),
                ..RenderMaterial::default()
            })
            .unwrap();

        let mut object = RenderObject::new("o".to_owned(), TyVector3U32::new(4, 1, 1)).unwrap();
        for (x, material_id) in [(0, opaque_id), (1, glass_id), (2, half_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .set_voxel_material(voxel_id, Some(material_id))
                .unwrap();
        }
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();
        let object = scene.object(object_id).unwrap();

        let grid = OpaqueGrid::new(&scene, object);

        assert_eq!(SurfaceGrid::bounds(&grid), TyVector3U32::new(4, 1, 1));
        assert_eq!(
            grid.cell(TyVector3U32::new(0, 0, 0)),
            object.voxel_id(TyVector3U32::new(0, 0, 0))
        );
        assert!(grid.is_solid([0, 0, 0]));
        assert!(!grid.is_solid([1, 0, 0]));
        assert!(!grid.is_solid([2, 0, 0]));
        assert!(!grid.is_solid([3, 0, 0]));
        assert!(!grid.is_solid([4, 0, 0]));
    }
}
