# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. voxcore resample

- `resample_voxels` pulls: per cell of the new grid, in raster order, the
  closure names the old cell it copies, or `None` for an empty cell. A dead
  source also leaves the cell empty, so an upsample's closure is plain
  arithmetic. An old position outside the old grid is the
  `ResampleSourceOutsideGrid` error.
- The setter returns the old grid's live ids, and the hook carries them with
  the old bounds, the pair vmax rebuilds the old content center from.
- `voxel_position` moves onto a static `raster_position(bounds, id)` beside
  `raster_id`, so the new grid's positions come from the same arithmetic as
  the old grid's.
- vmax and goxl each fold their remap refresh into a method, named
  `follow_content_center` and `follow_bounds`, that both hooks call.

## S2. voxsmith operations

- `ResampleFactor` is a voxsmith newtype of at least 2, and vxl parses it
  through `parse_resample_factor` as it parses `IndexRange` through
  `parse_index_range`. `KeepRule` is a voxsmith enum that vxl names on the
  command line through `CliValue`, as it names `QuarterTurns`.
- `upsample_objects` scales bounds and origin with `checked_mul` and checks
  the cell cap itself, so each error names the object.
- `downsample_objects` computes every coarse cell's source into a
  raster-ordered table before calling the setter, since the setter's closure
  cannot read the object the main is mutating. The coarse box is computed in
  `i64` with `div_euclid`, which rounds toward negative infinity for the
  positive factor.
- The tally keys on a voxel's whole sample vector, so a block's layers stay
  consistent with one another. `max_by_key` over the count and the reversed
  first id picks the most common vector and, on a tie, the earliest.
- Both operations fire the resample hook, then the origin hook, as `trim`
  does.
- The tests share `two_material_scene` and `live_cells` from
  `test_utilities`.

## S3. vxl commands

- `--keep` defaults to `majority` through clap's `default_value`, so the
  `CliValue` name doubles as the default's spelling.
