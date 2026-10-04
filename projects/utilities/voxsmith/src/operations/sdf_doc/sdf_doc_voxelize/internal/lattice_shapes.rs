use crate::operations::sdf_doc::{Bounds3d, SdfEvaluation, SdfShapes, ShapeSource, box_distance};
use branded_id::{IdVec, U32Id};
use sdfcore::{BSdfShape2d, BSdfShape3d};
use ty_math::{TyVector2F64, TyVector3F64};

/// A model's shapes moved by a shift onto the lattice of cells `voxel_size`
/// across. Each 3D shape's distance takes the larger of its formula's value and
/// the signed distance to its box rounded out to the lattice's cell corners.
#[derive(Clone, Debug)]
pub struct LatticeShapes<'a> {
    shapes: &'a SdfShapes,

    voxel_size: f64,

    shift: TyVector3F64,

    /// Each box's corners as whole numbers of cells in the lattice.
    cells: IdVec<BSdfShape3d, Option<Bounds3d>>,
}

impl<'a> LatticeShapes<'a> {
    /// The shapes of `shapes` moved by `shift` onto the lattice of cells
    /// `voxel_size` across.
    pub fn new(shapes: &'a SdfShapes, voxel_size: f64, shift: TyVector3F64) -> Self {
        let cells = shapes
            .all_bounds3d()
            .iter()
            .map(|bounds| {
                bounds.map(|bounds| Bounds3d {
                    min: ((bounds.min + shift) / voxel_size).floor(),
                    max: ((bounds.max + shift) / voxel_size).ceil(),
                })
            })
            .collect();

        Self {
            shapes,
            voxel_size,
            shift,
            cells,
        }
    }

    /// The rounded box of the shape at `shape3d_id`, with its corners as whole
    /// numbers of cells in the lattice, or `None` for a shape that reaches
    /// without end.
    pub fn cells(&self, shape3d_id: U32Id<BSdfShape3d>) -> Option<Bounds3d> {
        self.cells[shape3d_id.to_usize_id()]
    }
}

impl ShapeSource for LatticeShapes<'_> {
    fn evaluate(&self, shape3d_id: U32Id<BSdfShape3d>, point: TyVector3F64) -> SdfEvaluation {
        let evaluation = self.shapes.field3d(shape3d_id).evaluate(self, point);

        match self.cells(shape3d_id) {
            Some(cells) => {
                let min = cells.min * self.voxel_size - self.shift;
                let max = cells.max * self.voxel_size - self.shift;
                let to_box = box_distance(point - (min + max) / 2.0, (max - min) / 2.0);

                SdfEvaluation {
                    distance: evaluation.distance.max(to_box),
                    ..evaluation
                }
            }

            None => evaluation,
        }
    }

    fn distance2d(&self, shape2d_id: U32Id<BSdfShape2d>, point: TyVector2F64) -> f64 {
        self.shapes.field2d(shape2d_id).distance(self, point)
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        Bounds3d, LatticeShapes, ShapeSource, box_distance, grid3d, shapes_of,
    };
    use branded_id::U32Id;
    use sdfcore::SdfShape3d;
    use ty_math::TyVector3F64;

    #[test]
    fn each_box_rounds_out_to_the_cells_and_clamps_the_distance() {
        let shapes = shapes_of(
            Vec::new(),
            vec![SdfShape3d::Ellipsoid {
                center: TyVector3F64::ZERO,
                radii: TyVector3F64::new(1.0, 0.1, 0.3),
            }],
        );
        let shift = TyVector3F64::new(0.1, 0.0, -0.1);
        let lattice = LatticeShapes::new(&shapes, 0.25, shift);

        let cells = lattice.cells(U32Id::from_u32(0)).unwrap();
        assert_eq!(
            cells,
            Bounds3d {
                min: TyVector3F64::new(-4.0, -1.0, -2.0),
                max: TyVector3F64::new(5.0, 1.0, 1.0),
            }
        );

        let min = cells.min * 0.25 - shift;
        let max = cells.max * 0.25 - shift;
        let mut clamped = false;

        for point in grid3d(TyVector3F64::splat(-2.0), TyVector3F64::splat(2.0), 9) {
            let raw = shapes.evaluate(U32Id::from_u32(0), point).distance;
            let to_box = box_distance(point - (min + max) / 2.0, (max - min) / 2.0);
            let distance = lattice.evaluate(U32Id::from_u32(0), point).distance;

            assert_eq!(distance, raw.max(to_box));
            clamped |= to_box > raw;
        }

        assert!(clamped);
    }
}
