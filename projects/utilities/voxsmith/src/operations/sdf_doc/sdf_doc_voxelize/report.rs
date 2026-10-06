use crate::{
    Error, Result,
    operations::sdf_doc::{FACE_OFFSETS, SdfCell, SdfGrid, SdfSampling, has_open_face, meters},
    utilities::{FillMode, VoxelFrame},
};
use sdfcore::{SdfMain, SdfState, SdfStep};
use std::{
    borrow::Cow,
    cmp::Reverse,
    collections::{HashMap, HashSet, VecDeque},
};
use ty_math::TyVector3I32;

/// The report on the grids of `sampling` for the voxj document `name` written
/// under `fill_mode` by model evaluation. Errors on grids sampled under the
/// local frame.
pub fn report(
    main: &SdfMain,
    sampling: &SdfSampling,
    fill_mode: FillMode,
    name: &str,
) -> Result<String> {
    if sampling.frame == VoxelFrame::Local {
        return Err(Error::invalid(
            "report must read grids sampled under the world frame, not the local frame",
        ));
    }

    let state = main.state();
    let voxel_size = sampling.voxel_size;

    let place_cells: Vec<HashSet<TyVector3I32>> = sampling
        .places
        .iter()
        .map(|place| {
            place
                .grid_indices
                .iter()
                .flat_map(|&grid_index| live_cells(&sampling.grids[grid_index]))
                .collect()
        })
        .collect();

    let covered: HashSet<TyVector3I32> = place_cells.iter().flatten().copied().collect();
    let pieces = face_pieces(&covered);

    // A model in one piece leaves no part detached and lists no pieces.
    let piece_of: HashMap<TyVector3I32, usize> = match pieces.len() {
        0 | 1 => HashMap::new(),
        _ => pieces
            .iter()
            .enumerate()
            .flat_map(|(index, piece)| piece.iter().map(move |cell| (*cell, index)))
            .collect(),
    };
    let detached = detached_places(sampling, &place_cells, &piece_of, pieces.len());

    // The voxel counts read the cells the document holds.
    let (place_voxels, voxels) = match fill_mode {
        FillMode::Solid => (Cow::Borrowed(&place_cells[..]), Cow::Borrowed(&covered)),

        FillMode::Surface => {
            let place_voxels: Vec<HashSet<TyVector3I32>> = sampling
                .places
                .iter()
                .map(|place| {
                    place
                        .grid_indices
                        .iter()
                        .flat_map(|&grid_index| {
                            let grid = &sampling.grids[grid_index];

                            live_cells(grid).filter(move |cell| has_open_face(grid, *cell))
                        })
                        .collect()
                })
                .collect();
            let voxels = place_voxels.iter().flatten().copied().collect();

            (Cow::Owned(place_voxels), Cow::Owned(voxels))
        }
    };

    let roots: Vec<usize> = (0..sampling.places.len())
        .filter(|&index| sampling.places[index].parent.is_none())
        .collect();
    let lone_root = roots.len() == 1;

    let mut lists_per_name: HashMap<&str, usize> = HashMap::new();

    for place in &sampling.places {
        for &grid_index in &place.grid_indices {
            let object = &state.objects[sampling.grids[grid_index].object_id.to_usize_id()];

            for step_id in &object.step_ids {
                *lists_per_name
                    .entry(state.steps[step_id.to_usize_id()].name())
                    .or_default() += 1;
            }
        }
    }

    let step_label = |place_index: usize, step: &SdfStep| {
        if lists_per_name[step.name()] < 2 {
            return step.name().to_owned();
        }

        let path = &sampling.places[place_index].path;
        let parts = if lone_root { &path[1..] } else { &path[..] };

        parts
            .iter()
            .map(String::as_str)
            .chain([step.name()])
            .collect::<Vec<_>>()
            .join("/")
    };

    let pieces_label = match pieces.len() {
        1 => "1 piece".to_owned(),
        count => format!("{count} pieces"),
    };

    let mut lines = vec![ReportLine {
        kind: LineKind::Model,
        depth: 0,
        fields: [
            vec![
                Some(Field::Text(name.to_owned())),
                Some(Field::Text(format!(
                    "{} of {} m",
                    count_text(voxels.len() as u64, "voxel", "voxels"),
                    meters(voxel_size)
                ))),
                Some(Field::Text(pieces_label)),
            ],
            extent_fields(covered.iter().copied(), voxel_size),
        ]
        .concat(),
    }];

    let mut writer = ReportWriter {
        state,
        sampling,
        place_cells: &place_cells,
        place_voxels: &place_voxels,
        detached: &detached,
        lines: &mut lines,
    };

    for &root in &roots {
        if lone_root {
            writer.contents(root, 1);
        } else {
            writer.place(root, 1);
        }
    }

    if pieces.len() > 1 {
        let mut sources: Vec<Vec<String>> = vec![Vec::new(); pieces.len()];

        for (place_index, place) in sampling.places.iter().enumerate() {
            for &grid_index in &place.grid_indices {
                let grid = &sampling.grids[grid_index];
                let object = &state.objects[grid.object_id.to_usize_id()];

                for (step_index, step_id) in object.step_ids.iter().enumerate() {
                    let step = &state.steps[step_id.to_usize_id()];
                    let mut reached: Vec<usize> = kept_cells(grid, step_index)
                        .filter(|(_, live)| *live)
                        .map(|(cell, _)| piece_of[&cell])
                        .collect();
                    reached.sort_unstable();
                    reached.dedup();

                    for piece_index in reached {
                        sources[piece_index].push(step_label(place_index, step));
                    }
                }
            }
        }

        let width = pieces.len().to_string().len();

        for (index, (piece, sources)) in pieces.iter().zip(sources).enumerate() {
            lines.push(ReportLine {
                kind: LineKind::Piece,
                depth: 1,
                fields: [
                    vec![
                        Some(Field::Text(format!("piece {:>width$}", index + 1))),
                        Some(count_field(
                            piece.iter().filter(|cell| voxels.contains(cell)).count() as u64,
                            "voxel",
                            "voxels",
                        )),
                    ],
                    extent_fields(piece.iter().copied(), voxel_size),
                    vec![Some(Field::Text(format!("from {}", sources.join(", "))))],
                ]
                .concat(),
            });
        }
    }

    Ok(layout(&lines))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineKind {
    Model,

    Part,

    Piece,

    Step,
}

#[derive(Clone)]
enum Field {
    /// A count beside its label, with the counts of one column aligned right.
    Count(u64, &'static str),

    /// Text aligned left.
    Text(String),
}

/// One line of the report before its columns line up.
struct ReportLine {
    kind: LineKind,

    depth: usize,

    /// The fields in column order, with `None` for a column the line leaves
    /// blank.
    fields: Vec<Option<Field>>,
}

/// Adds the part and step lines of the places to a report.
struct ReportWriter<'a> {
    state: &'a SdfState,

    sampling: &'a SdfSampling,

    place_cells: &'a [HashSet<TyVector3I32>],

    /// Each place's cells the document holds.
    place_voxels: &'a [HashSet<TyVector3I32>],

    /// Whether each place reads detached.
    detached: &'a [bool],

    lines: &'a mut Vec<ReportLine>,
}

impl ReportWriter<'_> {
    /// Adds the line of the place at `place_index` at `depth`, then the
    /// place's steps and child parts one level deeper.
    fn place(&mut self, place_index: usize, depth: usize) {
        let place = &self.sampling.places[place_index];
        let cells = &self.place_cells[place_index];

        self.lines.push(ReportLine {
            kind: LineKind::Part,
            depth,
            fields: [
                vec![
                    Some(Field::Text("part".to_owned())),
                    Some(Field::Text(
                        place.path.last().expect("a place has a part").clone(),
                    )),
                    Some(count_field(
                        self.place_voxels[place_index].len() as u64,
                        "voxel",
                        "voxels",
                    )),
                ],
                extent_fields(cells.iter().copied(), self.sampling.voxel_size),
                vec![self.detached[place_index].then(|| Field::Text("detached".to_owned()))],
            ]
            .concat(),
        });

        self.contents(place_index, depth + 1);
    }

    /// Adds the step lines of the place at `place_index` at `depth`, then the
    /// lines of its child parts.
    fn contents(&mut self, place_index: usize, depth: usize) {
        let place = &self.sampling.places[place_index];

        for &grid_index in &place.grid_indices {
            let grid = &self.sampling.grids[grid_index];
            let object = &self.state.objects[grid.object_id.to_usize_id()];

            for (step_index, step_id) in object.step_ids.iter().enumerate() {
                let step = &self.state.steps[step_id.to_usize_id()];
                let record = grid.steps[step_index];
                let kept: Vec<(TyVector3I32, bool)> = kept_cells(grid, step_index).collect();

                let exposed = match step {
                    SdfStep::Carve { .. } => None,

                    _ => Some(count_field(
                        kept.iter()
                            .filter(|(cell, live)| *live && has_open_face(grid, *cell))
                            .count() as u64,
                        "exposed",
                        "exposed",
                    )),
                };

                let written = record.written_bounds.map(|bounds| [bounds.min, bounds.max]);

                self.lines.push(ReportLine {
                    kind: LineKind::Step,
                    depth,
                    fields: [
                        vec![
                            Some(Field::Text(step_kind(step).to_owned())),
                            Some(Field::Text(step.name().to_owned())),
                            Some(count_field(record.written, "cell", "cells")),
                            Some(count_field(kept.len() as u64, "kept", "kept")),
                            exposed,
                        ],
                        extent_fields(written.into_iter().flatten(), self.sampling.voxel_size),
                    ]
                    .concat(),
                });
            }
        }

        for (child_index, child) in self.sampling.places.iter().enumerate() {
            if child.parent == Some(place_index) {
                self.place(child_index, depth);
            }
        }
    }
}

/// A count field labeled `singular` or `plural`.
fn count_field(count: u64, singular: &'static str, plural: &'static str) -> Field {
    Field::Count(count, if count == 1 { singular } else { plural })
}

/// A count beside its label, `singular` or `plural`.
fn count_text(count: u64, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

/// The size and bounds fields of the box around `cells`, or two blank fields
/// for no cells.
fn extent_fields(cells: impl Iterator<Item = TyVector3I32>, voxel_size: f64) -> Vec<Option<Field>> {
    let Some((min, max)) = cells.fold(None, |bounds, cell| {
        Some(
            bounds.map_or((cell, cell), |(min, max): (TyVector3I32, TyVector3I32)| {
                (min.min(cell), max.max(cell))
            }),
        )
    }) else {
        return vec![None, None];
    };

    let size = max - min + 1;
    let corner = |cell: TyVector3I32| {
        let position = cell.as_dvec3() * voxel_size;
        format!(
            "[{}, {}, {}]",
            meters(position.x),
            meters(position.y),
            meters(position.z)
        )
    };

    vec![
        Some(Field::Text(format!("{}x{}x{}", size.x, size.y, size.z))),
        Some(Field::Text(format!(
            "{} .. {}",
            corner(min),
            corner(max + 1)
        ))),
    ]
}

/// The live cells of `grid` by lattice index.
fn live_cells(grid: &SdfGrid) -> impl Iterator<Item = TyVector3I32> + '_ {
    grid_cells(grid)
        .filter(|(_, cell)| cell.material.is_some())
        .map(|(position, _)| position)
}

/// The cells of `grid` whose last change came from the step at `step_index` of
/// its list, each with whether it holds a material.
fn kept_cells(
    grid: &SdfGrid,
    step_index: usize,
) -> impl Iterator<Item = (TyVector3I32, bool)> + '_ {
    grid_cells(grid)
        .filter(move |(_, cell)| cell.step == Some(step_index as u32))
        .map(|(position, cell)| (position, cell.material.is_some()))
}

/// Each cell of `grid` beside its lattice index, in a raster with x outermost.
fn grid_cells(grid: &SdfGrid) -> impl Iterator<Item = (TyVector3I32, &SdfCell)> + '_ {
    let size = grid.size.as_ivec3();

    grid.cells.iter().enumerate().map(move |(index, cell)| {
        let index = index as i32;
        let offset = TyVector3I32::new(
            index / (size.y * size.z),
            index / size.z % size.y,
            index % size.z,
        );

        (grid.min + offset, cell)
    })
}

/// Whether each place of `sampling` reads detached: the place and its parent
/// have live cells, yet no piece holding the cells of the place or a part below
/// it holds a cell of the parent. `piece_of` holds the piece of every cell in
/// `place_cells` once the model splits into `piece_count` pieces.
fn detached_places(
    sampling: &SdfSampling,
    place_cells: &[HashSet<TyVector3I32>],
    piece_of: &HashMap<TyVector3I32, usize>,
    piece_count: usize,
) -> Vec<bool> {
    let place_count = sampling.places.len();

    if piece_count < 2 {
        return vec![false; place_count];
    }

    let mut children = vec![Vec::new(); place_count];

    for (index, place) in sampling.places.iter().enumerate() {
        if let Some(parent) = place.parent {
            children[parent].push(index);
        }
    }

    let place_pieces: Vec<HashSet<usize>> = place_cells
        .iter()
        .map(|cells| cells.iter().map(|cell| piece_of[cell]).collect())
        .collect();

    (0..place_count)
        .map(|index| {
            let Some(parent) = sampling.places[index].parent else {
                return false;
            };

            if place_cells[index].is_empty() || place_cells[parent].is_empty() {
                return false;
            }

            let mut below = vec![index];
            let mut stack = vec![index];

            while let Some(place) = stack.pop() {
                below.extend(&children[place]);
                stack.extend(&children[place]);
            }

            below
                .iter()
                .all(|&place| place_pieces[place].is_disjoint(&place_pieces[parent]))
        })
        .collect()
}

/// The face-connected groups of `cells`, from the largest. Among groups of one
/// size, the group whose first cell comes first in a raster with x outermost
/// comes first.
fn face_pieces(cells: &HashSet<TyVector3I32>) -> Vec<Vec<TyVector3I32>> {
    let mut raster: Vec<TyVector3I32> = cells.iter().copied().collect();
    raster.sort_by_key(|cell| cell.to_array());

    let mut seen = HashSet::with_capacity(cells.len());
    let mut pieces = Vec::new();

    for start in raster {
        if !seen.insert(start) {
            continue;
        }

        let mut piece = vec![start];
        let mut queue = VecDeque::from([start]);

        while let Some(cell) = queue.pop_front() {
            for face in FACE_OFFSETS {
                let neighbor = cell + face;

                if cells.contains(&neighbor) && seen.insert(neighbor) {
                    piece.push(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        pieces.push(piece);
    }

    pieces.sort_by_key(|piece| Reverse(piece.len()));
    pieces
}

/// The name a report gives the kind of `step`.
fn step_kind(step: &SdfStep) -> &'static str {
    match step {
        SdfStep::Add { .. } => "add",
        SdfStep::Carve { .. } => "carve",
        SdfStep::Coat { .. } => "coat",
        SdfStep::Paint { .. } => "paint",
        SdfStep::Set { .. } => "set",
    }
}

/// `lines` as text, with the fields of consecutive lines of one kind and depth
/// padded into columns.
fn layout(lines: &[ReportLine]) -> String {
    let mut text = String::new();
    let mut start = 0;

    while start < lines.len() {
        let end = start
            + lines[start..]
                .iter()
                .take_while(|line| {
                    line.kind == lines[start].kind && line.depth == lines[start].depth
                })
                .count();
        let group = &lines[start..end];
        let columns = group
            .iter()
            .map(|line| line.fields.len())
            .max()
            .unwrap_or(0);

        let widths: Vec<(usize, usize)> = (0..columns)
            .map(|column| {
                group
                    .iter()
                    .filter_map(|line| line.fields.get(column).and_then(Option::as_ref))
                    .fold((0, 0), |(first, second), field| match field {
                        Field::Count(count, label) => {
                            (first.max(count.to_string().len()), second.max(label.len()))
                        }

                        Field::Text(text) => (first.max(text.len()), second),
                    })
            })
            .collect();

        for line in group {
            let fields: Vec<String> = (0..columns)
                .map(|column| {
                    let (first, second) = widths[column];

                    match line.fields.get(column).and_then(Option::as_ref) {
                        Some(Field::Count(count, label)) => {
                            format!("{count:>first$} {label:<second$}")
                        }

                        Some(Field::Text(text)) => format!("{text:<first$}"),

                        None if second > 0 => " ".repeat(first + 1 + second),

                        None => " ".repeat(first),
                    }
                })
                .collect();

            text.push_str(&"  ".repeat(line.depth));
            text.push_str(fields.join("  ").trim_end());
            text.push('\n');
        }

        start = end;
    }

    text
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::sdf_doc::{
            SdfSampleOptions, parts_main, report, sample, shared_leaf_main, single_part_main,
        },
        utilities::{FillMode, GridResolution, VoxelFrame},
    };
    use branded_id::U32Id;
    use sdfcore::{SdfMain, SdfShape3d, SdfStep, SdfStepMaterial};
    use ty_math::TyVector3F64;

    /// The report on `main` sampled at `voxel_size` under the world frame.
    fn report_on(main: &SdfMain, voxel_size: f64, fill_mode: FillMode) -> String {
        let sampling = sample(
            main,
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(voxel_size),
                frame: VoxelFrame::World,
            },
        )
        .unwrap();

        report(main, &sampling, fill_mode, "model.voxj").unwrap()
    }

    #[test]
    fn a_step_line_counts_the_cells_the_step_wrote_kept_and_shows() {
        let cuboid = |min: [f64; 3], max: [f64; 3]| SdfShape3d::Box {
            min: TyVector3F64::from_array(min),
            max: TyVector3F64::from_array(max),
            round: None,
        };

        let main = single_part_main(
            vec![
                cuboid([0.0, 0.0, 0.0], [0.1, 0.075, 0.1]),
                cuboid([0.0, 0.05, 0.0], [0.1, 0.075, 0.1]),
                cuboid([0.0, 0.025, 0.0], [0.05, 0.05, 0.1]),
                cuboid([1.0, 1.0, 1.0], [1.025, 1.025, 1.025]),
            ],
            vec![
                SdfStep::Add {
                    name: "block".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
                SdfStep::Carve {
                    name: "top".to_string(),
                    shape_id: U32Id::from_u32(1),
                },
                SdfStep::Paint {
                    name: "band".to_string(),
                    shape_id: U32Id::from_u32(2),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                },
                SdfStep::Paint {
                    name: "miss".to_string(),
                    shape_id: U32Id::from_u32(3),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                },
            ],
        );

        assert_eq!(
            report_on(&main, 0.025, FillMode::Solid),
            "\
model.voxj  32 voxels of 0.025 m  1 piece  4x2x4  [0, 0, 0] .. [0.1, 0.05, 0.1]
  add    block  48 cells  24 kept  24 exposed  4x3x4  [0, 0, 0] .. [0.1, 0.075, 0.1]
  carve  top    16 cells  16 kept              4x1x4  [0, 0.05, 0] .. [0.1, 0.075, 0.1]
  paint  band    8 cells   8 kept   8 exposed  2x1x4  [0, 0.025, 0] .. [0.05, 0.05, 0.1]
  paint  miss    0 cells   0 kept   0 exposed
"
        );
    }

    #[test]
    fn parts_take_part_lines_and_separate_pieces_take_piece_lines() {
        assert_eq!(
            report_on(&parts_main(), 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  2 pieces  7x3x4  [0, 0, 0] .. [7, 3, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [6, 0, 0] .. [7, 1, 1]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [6, 0, 0] .. [7, 1, 1]
  piece 1  36 voxels  4x3x4  [0, 0, 0] .. [4, 3, 4]  from body, lid
  piece 2   1 voxel   1x1x1  [6, 0, 0] .. [7, 1, 1]  from pebble
"
        );
    }

    #[test]
    fn a_part_resting_on_a_sibling_reads_attached() {
        // The pebble sits on the lid and clears the body.
        let mut state = parts_main().state().clone();
        state.shapes3d[U32Id::from_u32(2).to_usize_id()] = SdfShape3d::Box {
            min: TyVector3F64::new(1.0, 3.0, 1.0),
            max: TyVector3F64::new(2.0, 4.0, 2.0),
            round: None,
        };
        state.nodes[U32Id::from_u32(1).to_usize_id()].pivot =
            Some(TyVector3F64::new(1.0, 3.0, 1.0));

        assert_eq!(
            report_on(&SdfMain::new(state).unwrap(), 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  1 piece  4x4x4  [0, 0, 0] .. [4, 4, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 2, 1] .. [3, 3, 3]
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 2, 1] .. [3, 3, 3]
  part  pebble  1 voxel  1x1x1  [1, 3, 1] .. [2, 4, 2]
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [1, 3, 1] .. [2, 4, 2]
"
        );
    }

    #[test]
    fn parts_floating_together_off_their_parent_read_detached() {
        // The lid hovers a cell above the body, and the pebble sits on the lid.
        let mut state = parts_main().state().clone();
        state.shapes3d[U32Id::from_u32(1).to_usize_id()] = SdfShape3d::Box {
            min: TyVector3F64::new(1.0, 3.0, 1.0),
            max: TyVector3F64::new(3.0, 4.0, 3.0),
            round: None,
        };
        state.shapes3d[U32Id::from_u32(2).to_usize_id()] = SdfShape3d::Box {
            min: TyVector3F64::new(1.0, 4.0, 1.0),
            max: TyVector3F64::new(2.0, 5.0, 2.0),
            round: None,
        };
        state.nodes[U32Id::from_u32(0).to_usize_id()].pivot =
            Some(TyVector3F64::new(2.0, 3.0, 2.0));
        state.nodes[U32Id::from_u32(1).to_usize_id()].pivot =
            Some(TyVector3F64::new(1.0, 4.0, 1.0));

        assert_eq!(
            report_on(&SdfMain::new(state).unwrap(), 1.0, FillMode::Solid),
            "\
model.voxj  37 voxels of 1 m  2 pieces  4x5x4  [0, 0, 0] .. [4, 5, 4]
  add  body  32 cells  32 kept  32 exposed  4x2x4  [0, 0, 0] .. [4, 2, 4]
  part  lid  4 voxels  2x1x2  [1, 3, 1] .. [3, 4, 3]  detached
    add  lid  4 cells  4 kept  4 exposed  2x1x2  [1, 3, 1] .. [3, 4, 3]
  part  pebble  1 voxel  1x1x1  [1, 4, 1] .. [2, 5, 2]  detached
    add  pebble  1 cell  1 kept  1 exposed  1x1x1  [1, 4, 1] .. [2, 5, 2]
  piece 1  32 voxels  4x2x4  [0, 0, 0] .. [4, 2, 4]  from body
  piece 2   5 voxels  2x2x2  [1, 3, 1] .. [3, 5, 3]  from lid, pebble
"
        );
    }

    #[test]
    fn the_surface_fill_thins_the_voxel_counts_to_the_shell() {
        let main = single_part_main(
            vec![SdfShape3d::Box {
                min: TyVector3F64::ZERO,
                max: TyVector3F64::splat(3.0),
                round: None,
            }],
            vec![SdfStep::Add {
                name: "block".to_string(),
                shape_id: U32Id::from_u32(0),
                material: SdfStepMaterial::Material(U32Id::from_u32(0)),
            }],
        );

        assert_eq!(
            report_on(&main, 1.0, FillMode::Surface),
            "\
model.voxj  26 voxels of 1 m  1 piece  3x3x3  [0, 0, 0] .. [3, 3, 3]
  add  block  27 cells  27 kept  26 exposed  3x3x3  [0, 0, 0] .. [3, 3, 3]
"
        );
    }

    #[test]
    fn a_step_name_in_several_lists_takes_its_path_in_the_piece_lines() {
        assert_eq!(
            report_on(&shared_leaf_main(), 1.0, FillMode::Solid),
            "\
model.voxj  2 voxels of 1 m  2 pieces  3x1x1  [0, 0, 0] .. [3, 1, 1]
  part  a  0 voxels
    part  leaf  1 voxel  1x1x1  [2, 0, 0] .. [3, 1, 1]
      add  cube  1 cell  1 kept  1 exposed  1x1x1  [2, 0, 0] .. [3, 1, 1]
  part  b  0 voxels
    part  leaf  1 voxel  1x1x1  [0, 0, 0] .. [1, 1, 1]
      add  cube  1 cell  1 kept  1 exposed  1x1x1  [0, 0, 0] .. [1, 1, 1]
  piece 1  1 voxel  1x1x1  [0, 0, 0] .. [1, 1, 1]  from b/leaf/cube
  piece 2  1 voxel  1x1x1  [2, 0, 0] .. [3, 1, 1]  from a/leaf/cube
"
        );
    }

    #[test]
    fn the_report_needs_the_world_frame() {
        let main = parts_main();
        let sampling = sample(
            &main,
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(1.0),
                frame: VoxelFrame::Local,
            },
        )
        .unwrap();

        assert_eq!(
            report(&main, &sampling, FillMode::Solid, "model.voxj")
                .unwrap_err()
                .to_string(),
            "report must read grids sampled under the world frame, not the local frame"
        );
    }
}
