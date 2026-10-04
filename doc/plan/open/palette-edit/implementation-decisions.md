# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as
they land.

## S1. f64 language

- voxsmith checks the f32 range in `check_f32_range` and leaves the rounding
  to the glTF writer, which already narrows every float with `as f32`. The
  meshdoc keeps full `f64` values. Every material factor goes through the
  check, and only the unbounded `normalScale`, `emissiveStrength`,
  `alphaCutoff`, and `ior` can fail it. Custom float attributes go through it
  after the transfer. `COLOR_0` lies in `[0, 1]` and skips it.
- `CHROMA_FLOOR` stays at `1e-6`. With the `f32` narrowing gone, the rounded
  Oklab matrices leave at most about `1.7e-7` of chroma on a gray up to a
  linear `100`.
- `assert_close` keeps its `1e-5` relative tolerance because several tests
  compare against hand-typed reference values.
- Greedy merge class keys split an `f64` component's bits into two `u32`
  words.
- voxsurface's `corner_occlusion` and `mesh_occlusion` return `f64`, so the
  occlusion thirds reach the language and voxrender exact. Vertex positions
  stay `f32`.

## S2. Swatch count and mentioned names

- `Program::free_names` scans for the names the environment supplies, not
  every name the program mentions. It counts a name a binding reads before the
  program binds it, and a name an end-scope expression reads that the program
  never binds. A property the program rebinds before reading it stays unbound.
  Its value never blocks the run.
- `Destination::of_record` parses each expression and keeps it on the
  `Destination`. The run collects the free names from those expressions before
  it binds. A destination parse error now rises before the environment binds.
  `Destination` drops `Eq` because `Expression` does not implement it.
- Computed bindings bind even when nothing reads them because each comes from
  an explicit flag.
- `CheckedRecord` parses and checks a record once per object. A greedy mesh
  evaluates it twice, over the culled pre-pass and over the merged geometry.
  The property values bind once. The computed values and the groupings rebind
  for each geometry. `computed_type` gives a computed binding its type before
  any geometry exists. `Streams` now derives once per object.
- A destination check error now rises before an evaluation error.

## S3. voxcore setters

- `VoxValuePool` gains one `retain_<kind>_value` per kind, matching its
  constructors. `VoxMain` wraps each one under the same name. These replace
  the README's `retain_value_pool_value` and `VoxValuePoolValue`, so the
  caller picks the method for its value's kind. S5 compares a written value
  against a pool's values through `VoxValuePoolValueRef`.
- Each append checks the pool's kind first, then the value's domain.
- The checked constructors scan their own values before building and drop
  `checked`. `first_out_of_domain_value`, now only the audit `validate` runs,
  matches the kind once and walks the typed column. The domain checks take
  refs, so one function serves a constructor, an append, and the audit.
- The appends and `VoxPalette::set_value_id` are public, like `move_value` and
  `retain_property`. `VoxMain` adds the cross-reference checks.
- A rejected value has no id yet, so the two new errors carry none:
  `RetainedValueKind` and `MalformedRetainedValue`.
- An appended value can reuse a released id. It still lists last.

## S4. Palette selection

- One `IdSelector<TBrand>` in voxsmith's `utilities` replaces `IndexRange`,
  palette show's `PaletteRef`, and the planned `PaletteSelector`. It holds `*`,
  one id, or an inclusive id range, and never a reversed range.
- vxl parses every selector flag with `parse_id_selector`: `--select-index`,
  `--select-parent-index`, `--layer-index`, the `palette list` filters, the
  `palette show` palette field, and `palette quantize --index`. Each takes `*`.
- Selectors match ids. A loaded document's ids match its listing positions.
- `resolve_palette_selectors` returns palette ids in palette order. No
  selectors select nothing. vxl supplies `*` with clap's `default_value`, so
  the help shows the default.
- A named id the document lacks errors, whatever the entry kind. Palette,
  object, and node ids error with voxcore's `UnknownPalette`, `UnknownObject`,
  and `UnknownHierarchyNode`. A layer id errors with its object's id. Palette
  show resolves through `resolve_palette_selectors`.
- `quantize_palette` checks every id and every palette's sampling before it
  changes anything, then runs `gc` once after the last palette. Quantizing one
  palette never changes another's sampling.
- A palette counts as sampled when an object with a live voxel has a layer
  referencing it, since every live voxel samples each of its object's layers.

## S5. voxsmith `edit_palettes`

- `edit_palettes` takes the resolved palette ids, the program text, and one
  `PropertyWrite` per `--write-property`. vxl assembles the program in S6.
- The program and the writes parse once. Each palette checks and evaluates
  them. As in `object mesh`, every check error rises before an evaluation
  error.
- `Error::PaletteEdit` reports the element, the program or a write.
  `Error::InPalette` wraps any error raised over one palette with its id. A
  parse error or a repeated write rises before any palette and carries none.
- `WrittenValues` types a write's value for each material by its value pool
  kind. The domain, kind, and range checks all run before anything lands. The
  appends then `expect` because the evaluator rejects non-finite results and
  every unsigned value lies in the int domain.
- A landed value reuses the first equal value in the pool's listing order,
  found by a linear scan. `-0.0` reuses `0.0`.
- The range check mirrors `check_material_property_ranges`: a kind the name
  does not read passes.
- A materialless palette gains a written property on an empty value pool.
- Each overwritten cell's old value is a release candidate. After every
  palette lands, `release_undrawn_values` releases the candidates no palette
  draws. `quantize_palette` calls the same helper.
- `edit_palettes` leaves `gc` to `edit_document`. `quantize_palette` still
  runs it.
- `value_pool_kind_name` moves out of `choose_quantize_plan` for the kind
  error.
- An int outside `u32` still errors at its swatch, the material at that
  listing position.

## S6. vxl `palette edit` and profiles

- `ProgramFlag`, `ProgramFlags`, `parse_fragment`, and `check_expression` move
  into vxl's `internal` because `object mesh` and `palette edit` both use them.
  The `--value` and `--values-from` help no longer mentions mesh writers.
  `object mesh`'s command doc now says how its properties bind.
- `PaletteEditProgramBuilder` mirrors `ProgramBuilder` without the computed
  bindings.
- vxl checks that each write expression parses and that no property is written
  twice before it loads the document. Each error reports the flag or profile
  key. voxsmith repeats both checks.
- The profile stack merges `properties` by key. The flags' writes come first
  in line order. The stack's writes of every other property follow in property
  order. A write a flag replaces is never parsed.
- A property two stacked profiles write errors with `object mesh`'s claim
  wording.
- A run with no write still evaluates the program over each selected palette,
  then saves the document unchanged.
- A check or evaluation error of a lone expression now puts `does not check:`
  or `does not evaluate:` before its failure. The message reads after the
  element it rose from: "the write to `shadows` does not check: ...".
  `object mesh` errors read the same way.
- `vox-value-language` gains `check_expression_in_context`, which settles the
  bare literals no operand fixes as a given scalar. A write to an existing
  float property settles them as `f64`, and one to an int property as `u32`.
  A write adding a property has no context, so `1` there still errors.
- `object mesh` destinations that take one number type settle bare literals
  the same way. Each `Destination` carries the context its landing fixes. A
  slot, a PNG, `COLOR_0`, and every `srgb` value take `f64`. A select and a
  linear JSON entry or custom attribute take none. The value-language
  reference states the rule.
- The mesh writers' `srgb` encode evaluates the curve as `p + 0.055 * (p - 1)`
  instead of `palette`'s `1.055 * p - 0.055`. The two agree within an epsilon.
  Only the new form writes `1.0` as `1.0` instead of `0.9999999999999999`.
