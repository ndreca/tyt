use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::{BVoxVoxel, VoxObject};

/// A dense grid of cells the surface algorithms read.
pub trait SurfaceGrid {
    /// A solid cell's handle. The mesher keys faces by it and records it per
    /// face.
    type Cell: Copy;

    /// The grid's size in cells along each axis.
    fn bounds(&self) -> TyVector3U32;

    /// The handle of the solid cell at `position`, or `None` where the cell
    /// is empty. `position` is within [`bounds`](Self::bounds).
    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell>;

    /// Whether `position` lies within the grid and holds a solid cell.
    fn is_solid(&self, position: [i64; 3]) -> bool {
        let bounds = self.bounds().to_array();

        if position
            .iter()
            .zip(bounds)
            .any(|(&component, bound)| component < 0 || component >= i64::from(bound))
        {
            return false;
        }

        let position = TyVector3U32::from_array(position.map(|component| {
            u32::try_from(component).expect("the check above keeps the position within the grid")
        }));

        self.cell(position).is_some()
    }
}

/// A cell is solid where its voxel is live.
impl SurfaceGrid for VoxObject {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.bounds()
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.voxel_id(position)
            .filter(|&voxel_id| self.is_live(voxel_id))
    }
}

#[cfg(test)]
mod tests {
    use crate::{SurfaceGrid, test_utilities::live_object};
    use ty_math::TyVector3U32;

    #[test]
    fn a_live_voxel_is_a_solid_cell_and_the_outside_is_empty() {
        let object = live_object([2, 1, 1], &[[1, 0, 0]]);

        let voxel_id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();

        assert_eq!(SurfaceGrid::bounds(&object), TyVector3U32::new(2, 1, 1));
        assert_eq!(object.cell(TyVector3U32::new(1, 0, 0)), Some(voxel_id));
        assert_eq!(object.cell(TyVector3U32::new(0, 0, 0)), None);

        assert!(object.is_solid([1, 0, 0]));
        assert!(!object.is_solid([0, 0, 0]));
        assert!(!object.is_solid([2, 0, 0]));
        assert!(!object.is_solid([1, -1, 0]));
    }
}
