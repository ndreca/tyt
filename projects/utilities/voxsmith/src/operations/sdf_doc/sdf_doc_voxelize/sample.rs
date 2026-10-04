use crate::{
    Error, Result,
    operations::sdf_doc::{
        EntryPaths, SdfSampleOptions, SdfSampling, SdfShapes, check_document, check_shape_boxes,
        collect_places, sample_grid, sample_voxel_size,
    },
    utilities::VoxelFrame,
};
use sdfcore::SdfMain;
use std::collections::HashMap;
use ty_math::TyVector3F64;

/// Samples the model `main` into grids under `options` by model evaluation.
/// Errors on the first failed check.
pub fn sample(main: &SdfMain, options: &SdfSampleOptions) -> Result<SdfSampling> {
    let state = main.state();
    let mut places = collect_places(state);
    let paths = EntryPaths::new(state, &places);

    check_document(state, &places, &paths)?;

    let shapes = SdfShapes::new(main);
    check_shape_boxes(state, &shapes, &paths)?;

    let voxel_size = sample_voxel_size(state, &places, &shapes, options.resolution)?;
    let mut grids = Vec::new();
    let mut shared_grids = HashMap::new();

    for place in &mut places {
        for &object_id in &state.nodes[place.node_id.to_usize_id()].child_object_ids {
            let grid_index = match options.frame {
                VoxelFrame::World => {
                    grids.push(sample_grid(
                        state,
                        &shapes,
                        object_id,
                        &place.path,
                        voxel_size,
                        place.offset,
                    )?);
                    grids.len() - 1
                }

                VoxelFrame::Local => match shared_grids.get(&object_id) {
                    Some(grid_index) => *grid_index,

                    None => {
                        grids.push(sample_grid(
                            state,
                            &shapes,
                            object_id,
                            &place.path,
                            voxel_size,
                            TyVector3F64::ZERO,
                        )?);
                        shared_grids.insert(object_id, grids.len() - 1);
                        grids.len() - 1
                    }
                },
            };

            place.grid_indices.push(grid_index);
        }
    }

    let holds_live_cell = grids
        .iter()
        .any(|grid| grid.cells.iter().any(|cell| cell.material.is_some()));

    if !holds_live_cell {
        return Err(Error::invalid(
            "model must hold a live cell, not only empty grids",
        ));
    }

    Ok(SdfSampling {
        voxel_size,
        places,
        grids,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::sdf_doc::{SdfSampleOptions, SdfSampling, sample, single_part_main},
        utilities::{GridResolution, ResolutionReference, VoxelFrame},
    };
    use branded_id::{IdVec, U32Id};
    use sdfcore::{
        SdfMain, SdfMaterial, SdfNode, SdfObject, SdfShape3d, SdfSide, SdfState, SdfStep,
        SdfStepMaterial,
    };
    use std::f64::consts::PI;
    use ty_math::{TyAxis3, TyVector3F64, TyVector3I32};

    fn material(index: u32) -> SdfStepMaterial {
        SdfStepMaterial::Material(U32Id::from_u32(index))
    }

    fn cuboid(min: [f64; 3], max: [f64; 3]) -> SdfShape3d {
        SdfShape3d::Box {
            min: TyVector3F64::from_array(min),
            max: TyVector3F64::from_array(max),
            round: None,
        }
    }

    fn add(name: &str, shape: u32, material_index: u32) -> SdfStep {
        SdfStep::Add {
            name: name.to_string(),
            shape_id: U32Id::from_u32(shape),
            material: material(material_index),
        }
    }

    fn at_size(voxel_size: f64) -> SdfSampleOptions {
        SdfSampleOptions {
            resolution: GridResolution::VoxelSize(voxel_size),
            frame: VoxelFrame::World,
        }
    }

    fn live_cells(sampling: &SdfSampling) -> Vec<TyVector3I32> {
        sampling
            .grids
            .iter()
            .flat_map(|grid| {
                (0..grid.cells.len()).filter_map(move |index| {
                    let size = grid.size.as_ivec3();
                    let index = index as i32;
                    let cell = grid.min
                        + TyVector3I32::new(
                            index / (size.y * size.z),
                            index / size.z % size.y,
                            index % size.z,
                        );
                    grid.cell(cell).unwrap().material.map(|_| cell)
                })
            })
            .collect()
    }

    fn error(main: &SdfMain, options: SdfSampleOptions) -> String {
        sample(main, &options).unwrap_err().to_string()
    }

    #[test]
    fn a_box_with_corners_on_cell_corners_fills_exactly_its_cells() {
        for (min, max, first, last) in [
            ([0.0, 0.0, 0.0], [0.05, 0.25, 0.05], [0, 0, 0], [1, 9, 1]),
            (
                [-0.05, -0.025, 0.1],
                [0.025, 0.05, 0.175],
                [-2, -1, 4],
                [0, 1, 6],
            ),
        ] {
            let main = single_part_main(vec![cuboid(min, max)], vec![add("block", 0, 0)]);
            let cells = live_cells(&sample(&main, &at_size(0.025)).unwrap());

            let first = TyVector3I32::from_array(first);
            let last = TyVector3I32::from_array(last);
            let span = last - first + 1;

            assert_eq!(cells.len() as i32, span.x * span.y * span.z);
            assert!(
                cells
                    .iter()
                    .all(|cell| cell.cmpge(first).all() && cell.cmple(last).all())
            );
        }
    }

    #[test]
    fn each_step_writes_the_cells_model_evaluation_sets() {
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [4.0, 4.0, 4.0]),
                cuboid([0.0, 0.0, 2.0], [4.0, 4.0, 4.0]),
                cuboid([0.0, 0.0, 0.0], [2.0, 4.0, 2.0]),
            ],
            vec![
                add("block", 0, 0),
                SdfStep::Carve {
                    name: "cut".to_string(),
                    shape_id: U32Id::from_u32(1),
                },
                SdfStep::Paint {
                    name: "stain".to_string(),
                    shape_id: U32Id::from_u32(2),
                    material: material(1),
                },
                SdfStep::Coat {
                    name: "top".to_string(),
                    material: material(1),
                    sides: Some(vec![SdfSide::PositiveY]),
                    depth: None,
                    within_id: None,
                },
                SdfStep::Set {
                    name: "stud".to_string(),
                    points: vec![TyVector3F64::new(5.5, 0.5, 0.5)],
                    material: material(1),
                },
            ],
        );

        let sampling = sample(&main, &at_size(1.0)).unwrap();
        let grid = &sampling.grids[0];

        let written: Vec<u64> = grid.steps.iter().map(|record| record.written).collect();
        assert_eq!(written, [64, 32, 16, 8, 1]);
        assert_eq!(grid.size.to_array(), [6, 4, 4]);

        let cell = |x, y, z| *grid.cell(TyVector3I32::new(x, y, z)).unwrap();
        assert_eq!(
            (cell(3, 0, 0).material, cell(3, 0, 0).step),
            (Some(material(0)), Some(0))
        );
        assert_eq!(
            (cell(0, 0, 3).material, cell(0, 0, 3).step),
            (None, Some(1))
        );
        assert_eq!(
            (cell(0, 0, 0).material, cell(0, 0, 0).step),
            (Some(material(1)), Some(2))
        );
        assert_eq!(
            (cell(3, 3, 1).material, cell(3, 3, 1).step),
            (Some(material(1)), Some(3))
        );
        assert_eq!(
            (cell(5, 0, 0).material, cell(5, 0, 0).step),
            (Some(material(1)), Some(4))
        );
        assert_eq!((cell(4, 0, 0).material, cell(4, 0, 0).step), (None, None));
    }

    #[test]
    fn a_coat_reaches_its_depth_toward_every_side() {
        for (depth, written) in [(None, 26), (Some(2.0), 27)] {
            let main = single_part_main(
                vec![cuboid([0.0, 0.0, 0.0], [3.0, 3.0, 3.0])],
                vec![
                    add("block", 0, 0),
                    SdfStep::Coat {
                        name: "skin".to_string(),
                        material: material(1),
                        sides: None,
                        depth,
                        within_id: None,
                    },
                ],
            );

            let sampling = sample(&main, &at_size(1.0)).unwrap();
            assert_eq!(sampling.grids[0].steps[1].written, written);
        }
    }

    /// A model whose root part holds the parts `a` and `b`, which both place
    /// the part `leaf` and its unit box.
    fn shared_leaf_main() -> SdfMain {
        let node =
            |name: &str, offset: Option<[f64; 3]>, objects: Vec<u32>, nodes: Vec<u32>| SdfNode {
                name: name.to_string(),
                pivot: Some(TyVector3F64::new(0.5, 0.0, 0.0)),
                offset: offset.map(TyVector3F64::from_array),
                child_object_ids: objects.into_iter().map(U32Id::from_u32).collect(),
                child_node_ids: nodes.into_iter().map(U32Id::from_u32).collect(),
            };

        SdfMain::new(SdfState {
            shapes3d: IdVec::from_vec(vec![cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])]),
            materials: IdVec::from_vec(vec![SdfMaterial::Material {
                properties: Vec::new(),
            }]),
            steps: IdVec::from_vec(vec![add("cube", 0, 0)]),
            objects: IdVec::from_vec(vec![SdfObject {
                name: "leaf".to_string(),
                step_ids: vec![U32Id::from_u32(0)],
            }]),
            nodes: IdVec::from_vec(vec![
                node("leaf", None, vec![0], Vec::new()),
                node("a", Some([2.0, 0.0, 0.0]), Vec::new(), vec![0]),
                node("b", Some([0.0, 0.25, 0.0]), Vec::new(), vec![0]),
                node("model", None, Vec::new(), vec![1, 2]),
            ]),
            root_node_ids: vec![U32Id::from_u32(3)],
            ..SdfState::default()
        })
        .unwrap()
    }

    #[test]
    fn each_place_samples_moved_by_its_offsets_under_the_world_frame() {
        let sampling = sample(&shared_leaf_main(), &at_size(1.0)).unwrap();

        let paths: Vec<String> = sampling
            .places
            .iter()
            .map(|place| place.path.join("/"))
            .collect();
        assert_eq!(
            paths,
            [
                "model",
                "model/a",
                "model/a/leaf",
                "model/b",
                "model/b/leaf"
            ]
        );

        let leaf_under_a = &sampling.places[2];
        assert_eq!(leaf_under_a.parent, Some(1));
        assert_eq!(leaf_under_a.offset, TyVector3F64::new(2.0, 0.0, 0.0));
        assert_eq!(leaf_under_a.pivot, TyVector3F64::new(2.5, 0.0, 0.0));

        assert_eq!(
            live_cells(&sampling),
            [TyVector3I32::new(2, 0, 0), TyVector3I32::ZERO]
        );
    }

    #[test]
    fn each_part_samples_once_for_all_its_places_under_the_local_frame() {
        let options = SdfSampleOptions {
            frame: VoxelFrame::Local,
            ..at_size(1.0)
        };
        let sampling = sample(&shared_leaf_main(), &options).unwrap();

        assert_eq!(sampling.grids.len(), 1);
        assert_eq!(sampling.places[2].grid_indices, [0]);
        assert_eq!(sampling.places[4].grid_indices, [0]);
        assert_eq!(sampling.places[4].pivot, TyVector3F64::new(0.5, 0.25, 0.0));
    }

    #[test]
    fn a_resolution_divides_the_side_its_reference_measures() {
        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [2.0, 1.0, 1.0]),
                cuboid([0.0, 0.0, 0.0], [1.0, 0.5, 0.5]),
            ],
            vec![add("big", 0, 0), add("small", 1, 0)],
        );

        let voxel_size = |reference, count| {
            let options = SdfSampleOptions {
                resolution: GridResolution::ReferenceCount { reference, count },
                frame: VoxelFrame::World,
            };
            sample(&main, &options).unwrap().voxel_size
        };

        assert_eq!(voxel_size(ResolutionReference::LongestWorld, 4), 0.5);
        assert_eq!(voxel_size(ResolutionReference::ShortestObject, 4), 0.25);
        assert_eq!(voxel_size(ResolutionReference::WorldY, 2), 0.5);

        let world = sample(
            &shared_leaf_main(),
            &SdfSampleOptions {
                resolution: GridResolution::ReferenceCount {
                    reference: ResolutionReference::WorldX,
                    count: 3,
                },
                frame: VoxelFrame::World,
            },
        )
        .unwrap();
        assert_eq!(world.voxel_size, 1.0);
    }

    #[test]
    fn a_failed_check_reports_the_step_that_reaches_it() {
        let ball = |radius| SdfShape3d::Sphere {
            center: TyVector3F64::ZERO,
            radius,
        };

        assert_eq!(
            error(
                &single_part_main(vec![ball(-1.0)], vec![add("ball", 0, 0)]),
                at_size(1.0)
            ),
            "model/ball: sphere radius must be above zero, not -1"
        );
        assert_eq!(
            error(
                &single_part_main(vec![ball(1.0), ball(0.0)], vec![add("ball", 0, 0)]),
                at_size(1.0)
            ),
            "shapes3d[1]: sphere radius must be above zero, not 0"
        );
        assert_eq!(
            error(
                &single_part_main(vec![ball(1.0)], vec![add("ball", 0, 0), add("ball", 0, 1)]),
                at_size(1.0)
            ),
            "model: step name must be unique in its list, not \"ball\" twice"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![SdfShape3d::HalfSpace {
                        side: SdfSide::PositiveY,
                        at: 0.0,
                    }],
                    vec![add("floor", 0, 0)],
                ),
                at_size(1.0)
            ),
            "model/floor: add shape must be bounded, not a shape that reaches without end"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![
                        cuboid([0.0, 0.0, 0.0], [4.0, 0.1, 0.1]),
                        SdfShape3d::Bend {
                            shape_id: U32Id::from_u32(0),
                            along: TyAxis3::X,
                            toward: SdfSide::PositiveY,
                            radius: 1.0,
                            pivot: None,
                        }
                    ],
                    vec![add("hoop", 1, 0)],
                ),
                at_size(1.0)
            ),
            format!(
                "model/hoop: bend shape must be at most {} long along x to bend within half a turn, not 4",
                PI
            )
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![
                        ball(1.0),
                        SdfShape3d::Elongate {
                            shape_id: U32Id::from_u32(0),
                            lengths: TyVector3F64::new(0.0, 2.0, 0.0),
                            center: Some(TyVector3F64::new(0.0, 1.5, 0.0)),
                        }
                    ],
                    vec![add("pill", 1, 0)],
                ),
                at_size(1.0)
            ),
            "model/pill: elongate center must be inside its shape's box from -1 to 1 along y, not 1.5"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![cuboid([0.0, 0.0, 0.0], [1000.0, 1000.0, 1000.0])],
                    vec![add("slab", 0, 0)]
                ),
                at_size(1.0)
            ),
            "model: grid must hold at most 134217728 cells, not 1000000000"
        );
        assert_eq!(
            error(
                &single_part_main(vec![ball(1.0)], vec![add("ball", 0, 0)]),
                at_size(0.0)
            ),
            "voxel size must be above zero, not 0"
        );
        assert_eq!(
            error(
                &single_part_main(
                    vec![ball(1.0)],
                    vec![SdfStep::Carve {
                        name: "dig".to_string(),
                        shape_id: U32Id::from_u32(0),
                    }],
                ),
                at_size(1.0)
            ),
            "model must hold a live cell, not only empty grids"
        );
    }
}
