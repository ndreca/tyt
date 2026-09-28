# Implementation Checklist

Tracks building the commands in this plan. Start from the [README](README.md)
for the overview, the per-command pages under [reference/](reference/), and the
[design notes](reference/design-notes.md) for rationale. Code-level decisions
made while building are logged in
[implementation decisions](reference/implementation-decisions.md). Check items
off as they land.

## Ground rules

- `vxl` stays independent of `tyt-common` and `tyt-injection`. Use `std::fs` and
  the codec crates already wired in `Cargo.toml` (`voxcore`, `voxj-codec`,
  `voxsmith`, `vmax-codec`). The tyt FBX and material commands are behavioral
  models only, not dependencies. Mesh reading for `object mesh` and
  `mesh-doc voxelize` goes through voxsmith, which gains a `gltf` feature gating
  the `gltf` crate, rather than a vxl-level mesh dependency.
- Follow the existing command house style: one `clap` `Parser` struct per file
  in `src/commands/`, re-exported from `commands/mod.rs`, dispatched from
  `vxl.rs`; one `Dependencies` trait method per operation in `dependencies.rs`
  with the concrete impl behind the `impl` feature under `implementation/`. Use
  `vox-doc to voxj` (`commands/voxj.rs`, `implementation/to_voxj.rs`) as the
  template.
- Each `#[arg]` starts with `value_name`. Enumerated options are `ValueEnum`
  types, one per file under `utilities/`, re-exported. Outputs are
  `Option<PathBuf>` defaulted from the input stem. Booleans use the settable
  `--ext` style.

## Shared infrastructure

- [x] `MeshFormat` `ValueEnum` of `gltf` | `glb` with `from_path` extension
      inference, mirroring `Format::from_path`. glTF is the only mesh format for
      now, read via the `gltf` crate. (A prior scaffold held an `fbx` variant;
      retarget it to glTF.)
- [x] `--select-index` object selector parser: integer or `a-b` range,
      repeatable, union over all values. See
      [conventions](reference/conventions.md).
- [x] `--select` hierarchy-path glob object selector: a node path selects its
      subtree, repeatable, union over all values. See
      [conventions](reference/conventions.md).
- [x] `ObjectSelection`, the shared `--select` / `--select-index` clap group
      with the no-match usage error, flattened by `object mesh`, `vox-doc show`,
      and every `vox-doc to` target; voxsmith resolves it with
      `utilities::select_objects` and prunes for `vox-doc to` with
      `operations::to::keep_objects` over voxcore's `set_hierarchy_node`.
- [x] `--atlas` layout `ValueEnum`: `palette` shipped, one texel per distinct
      flattened material the object uses, its layers merged per property name
      by the format's layer-override resolution; `unwrap` (per-mesh UV) hidden
      until it lands. See [mesh](../../../ref/mesh/mesh.md#the-palette-atlas).
- [x] `--texture-map` channel parser: channel sources (`R`/`G`/`B`/`A` =
      `property` | `1-property` | `property.r`/`.g`/`.b`/`.a` color component |
      `0` | `1` | `computed-occlusion`) and the RGBA packing, sized by the
      highest channel named. See [mesh](../../../ref/mesh/value-language.md).
- [x] `--texture` preset packings (albedo, orm, metallic-roughness,
      metallic-smoothness, mse, emissive, occlusion, roughness, smoothness) and
      the `pbr` bundle; `computed-occlusion` hidden until the unwrap atlas
      lands. See [mesh](../../../ref/mesh/mesh.md).
- [x] `--define-property <property> <name>` binding, a pure rename alias giving
      a custom voxel-json key a name a packing reads. The type is not declared:
      `object mesh` reads it from the key's value pool in its winning layer's
      palette, a color pool exposing components and a scalar pool read whole,
      and a key no layer binds follows the format's unbound-default rule (a glTF
      built-in bakes its spec default, a custom key errors). See
      [mesh](../../../ref/mesh/value-language.md).
- [ ] `--vertex` / `--vertex-map` carrier: the vertex twins of the texture
      flags, writing `COLOR_0` and custom `_NAME` attributes, the
      `palette-index` / `palette-layers` index presets, and the `PaletteData`
      JSON per `--palette-storage`. See
      [mesh Deferred](../../../ref/mesh/mesh.md).
- [x] `ResourceStorage` `ValueEnum` (`embedded` | `external` | `both`) backing
      `--texture-storage` and `--palette-storage`, defaulting per target
      (`embedded` for `.glb`, `external` for `.gltf`): embed images in the glb
      chunk / gltf data URI and the palette JSON under `extras.vxl`, write
      external `.png` and `-palette.json` files, or both. See
      [mesh](../../../ref/mesh/mesh.md). (Image storage shipped with
      `object mesh`; the palette JSON reuse lands with the vertex carriers.)
- [x] Shared voxj encoding options (`--format`, `--encoding-preset`,
      `--position-encoding`, `--sample-encoding`) in `VoxjEncodingOptions`,
      flattened by `vox-doc to voxj` and `mesh-doc voxelize`;
      `--ext`/`--edit-state` stay on `vox-doc to voxj`.
- [x] `ValueEnum`s for the palette ops: quantize method, color space, dither,
      and `palette show` format (`auto` | `swatch` | `swatch-value` | `value`).
- [ ] Shared palette-reduction engine and a flattened options group (`--method`
      / `--space` / `--dither`), reused by `palette quantize` and
      `palette remap` (space/dither), on the one material-follows-color rule (a
      count bounds materials; a merged material takes its cluster
      representative's whole material). Landed: the `PaletteReductionOptions`
      group and voxsmith's `reduce_palette` with all three methods (`median-cut`
      / `octree` / `kmeans`) in oklab/lab/rgb via `remove_material`+`gc`, plus
      `--dither` (`floyd-steinberg` / `ordered`) as a per-voxel remap in 3D
      raster order. Pending: `quantize` / `remap` will reuse the engine.
- [ ] Split `reduce_palette` into two steps. The choose step maps a palette's
      candidate materials and their populations to representatives. The apply
      step rewrites the samples of the chosen object layers.
      `object voxels quantize` runs both steps. `palette quantize` runs them and
      then compacts the palette.
- [ ] `--partition <property>` in the choose step. Median-cut seeds its boxes
      from the partitions. Octree and kmeans give each partition one slot and
      split the rest of the cap by voxel count. The choose step errors when
      partitions outnumber the cap.
- [ ] Engine fixes, landing ahead of the commands:
      1. Error when the materials lacking the clustered property alone exceed
         the cap
      2. `--alpha partition | distance | ignore`, with `partition` keeping an
         opaque and a translucent material of one color apart
      3. Prune only the value-pool values the reduction orphaned
- [ ] Take the clustered property and its `--interpret-property` reading in
      place of the hardcoded `baseColor`. Points widen to 4D for vec4 `numeric`
      values and under `--alpha distance`. Octree errors on 4D points.

## Commands

### object mesh ([ref/mesh](../../../ref/mesh/mesh.md))

- [x] `Mesh` command struct, dispatch, and one mesh object per selected object
      under the hierarchy reaching it, or one mesh per object with
      `--split-files`. The mesher lives in
      voxsmith (`object_to_mesh_geometry` plus `object_to_glb_bytes` /
      `object_to_gltf_bytes`) behind the `gltf` feature; vxl stays a thin CLI.
- [x] `--to` / `--from` (`gltf` | `glb`), `--voxel-size` (meters per voxel,
      default `1.0`, baked into every vertex; glTF is meter-native), `--method`,
      and the `--select` / `--select-index` object selectors, `--select` matched
      through the shared `pathspec` gitignore engine like `node list`.
- [ ] `--atlas unwrap` and the `--computed-occlusion-*` tuning flags, landing
      with the computed-occlusion maps. See
      [mesh Deferred](../../../ref/mesh/mesh.md).
- [x] Material maps: `--texture <preset>` presets and the `pbr` bundle,
      repeatable; `--texture-name <preset> <file-name>` /
      `--texture-name-prefix <file-name>` naming; `--texture-map <file-name>
      <channels>`; `--define-property <property> <name>`; `--texture-shape`;
      and `--texture-storage`; default names from the output stem, unique per
      bake. The bake flattens the object's layers per property name by the
      format's layer-override resolution. The `--atlas palette` path;
      `computed-occlusion` errors until the unwrap atlas lands.
- [ ] Vertex attribute maps: `--vertex <preset>`, `--vertex-target <preset>
      <target>`, `--vertex-map <target> <channels>`, and the `PaletteData` JSON
      per `--palette-storage`. See
      [mesh Deferred](../../../ref/mesh/mesh.md).
- [x] `Dependencies::resolve_objects` and `mesh_object` and their impls, the
      flag-agnostic split that replaces the planned single `Dependencies::mesh`:
      the impl resolves the selectors to object indices and meshes by index,
      while the command owns the exactly-one policy and its flag-named errors.

### object material ([reference/object/material.md](reference/object/material.md))

- [ ] `Material` command sharing the mesh map flags, bake-only with no geometry.
- [ ] `--atlas` shared with `object mesh`; atlas derivation identical per mode;
      verify byte-for-byte parity.
- [ ] Require at least one map; otherwise list the maps and exit non-zero.

### mesh-doc voxelize ([reference/mesh-doc/voxelize.md](reference/mesh-doc/voxelize.md))

- [x] `Voxelize` command; `--from` (`gltf` | `glb`), mutually exclusive
      `--resolution` | `--voxel-size` (clap `ArgGroup`,
      `required = true`).
- [x] `--fill-mode` `solid` (default) | `surface`; `--fill-color` (default
      `white`, a `#RRGGBBAA` hex or name), used by `solid` only and rejected with
      `surface`. Surface is a hollow shell; per-voxel color sampling from the
      glTF material is deferred, so surface uses the flat color for now. (Shipped
      MVP; superseded by the material-mode work below.)
- [x] Reuse the voxj writer options; record `--voxel-size` (meters per
      voxel) as the node scale, leaving `--resolution` at scale `1`.
- [x] voxsmith `gltf` feature gating the `gltf` crate and a mesh-to-`VoxMain`
      voxelizer; `Dependencies::voxelize` and its impl call it, then write
      through `VoxjFileBuilder` like `vox-doc to voxj`. glTF Y-up is converted
      to the Z-up document convention; the inverse `object mesh` command must
      mirror it.

Material sampling (see [voxelize](reference/mesh-doc/voxelize.md) and
[design notes](reference/design-notes.md)):

- [x] `--material-mode auto | per-primitive | per-texel | flat` (default
      `auto`), replacing the shipped flat-only coloring and its
      `--fill-color`-with-`--fill-mode surface` guard. `per-texel` and
      texture-aware `auto` still fall back to `per-primitive` until the texel
      sampler lands below.
- [x] `per-primitive`: one material per source glTF material from the PBR
      factors (`baseColorFactor`, `metallicFactor`, `roughnessFactor`,
      `emissiveFactor`, `emissiveStrength`, `occlusionStrength`), matching what
      `object mesh` bakes.
- [x] `--fill-color #RRGGBBAA` (omitted for the default): the whole object under
      `flat`, the `solid`-fill interior under the sampling modes; a set color is
      rejected on a sampling-mode surface. An omitted interior adopts its nearest
      surface material.
- [ ] Remove the `--quantize-*` flags and `QuantizeOptions` so voxelize writes
      every distinct sampled material. `PaletteReductionOptions` moves out of
      `mesh_doc_voxelize` for the quantize commands.
- [x] `per-texel`: UV interpolation, image decode, area-average over the voxel
      footprint, epsilon-merge of near-identical tuples, and a `solid`-interior
      fallback to the nearest surface material. `auto` becomes texture-aware here.
      Base color, metallic, roughness, emissive, and occlusion all sample their
      own glTF textures on one scatter pass, each map reading the TEXCOORD set it
      declares (sRGB for base color and emissive, linear data for
      metallic-roughness and occlusion), area-averaged and merged; a map a
      material lacks keeps its flat factor.

### object voxels quantize ([reference/object/voxels/quantize.md](reference/object/voxels/quantize.md))

- [ ] `object voxels quantize`: `--max-materials`, `--select` /
      `--select-index`, `--property`, `--interpret-property`, `--alpha`,
      `--partition`, and the shared `--method` / `--space` / `--dither`. The
      command rewrites samples and leaves palettes and value pools untouched.
- [ ] `--layer-index`: an integer or `a-b` range into each object's `layers`.
      The flag repeats and defaults to every layer whose palette binds
      `--property`. A missing layer or a layer whose palette lacks the property
      errors.
- [ ] `--shared`: cluster every selected layer on one palette together instead
      of per object.
- [ ] Profiles, landing after the command: `object.voxels.quantize.profiles`,
      `--profile`, and `vxl profile object voxels quantize list`.

### palette ([reference/palette/](reference/palette/))

- [x] `palette list` (+ `--layout`): index, attributes, material count,
      referencing objects.
- [x] `palette show` (+ `--json`): `--index` / `--attribute` / `--format`. See
      the V2 follow-ups in [palette show](reference/palette/show.md) for the
      broader version.
- [ ] `palette quantize`: `--max-materials`, `--index`, `--property`,
      `--interpret-property`, `--alpha`, `--partition`, and the shared
      `--method` / `--space` / `--dither`, without `--keep-unused-values`. The
      candidates are the sampled materials, weighted by voxel count. Unsampled
      materials drop before the palette compacts. The input is a document.
      Every referencing object dithers.
- [ ] `palette quantize` profiles, landing after the command:
      `palette.quantize.profiles`, `--profile`, and
      `vxl profile palette quantize list`.
- [ ] `palette remap`: `--target` (JSON `palettes` array) or `--target-index`,
      `--target-property`, space, dither with `--select` / `--select-index`,
      and the same material-follows-color rule (a remapped voxel adopts the
      target material's whole set of property values). Accept a full document or
      a bare palette JSON input, the `--target` shape; dither only with a
      document.

### node list ([reference/node/list.md](reference/node/list.md))

- [x] Tree render with DAG / instancing and unplaced-node markers, plus
      `--collapse-instances`. Markdown tree only; no `--layout`.
- [x] `pattern` glob with `--collapse-ancestors` / `--collapse-descendants`.
- [x] `--show-transforms`, local and world.
- [x] `--show-{edit,runtime}-{origins,bounds,extents}`, the edit and runtime
      grids relative to the placing node, origins with a local/world space, an
      absent edit grid printing `null`, an empty object's runtime grid a
      zero-size box at its origin.
- [x] `--show-layers`, one child per layer labeled by its palette index with its
      material count.

### vox-doc validate ([reference/vox-doc/validate.md](reference/vox-doc/validate.md))

- [x] Implement the spec validation checklist; non-zero exit on failure;
      `--layout` report.

### vox-doc show ([reference/vox-doc/show.md](reference/vox-doc/show.md))

- [ ] Report version, per-object bounds / voxel count / encodings, palette
      property sets and material counts, `editState` and `ext` presence, and root /
      instanced / unplaced nodes; `--layout`. Landed: the command, dispatch,
      `Dependencies::info`, and a markdown / pretty-json / compact-json `--layout`
      report covering version, per-object bounds, voxel count, layer count,
      palette property sets and material counts, and `editState` / `ext`
      presence, with tests. Pending:
      per-object encodings and the root / instanced / unplaced node breakdown.
- [x] `--select` / `--select-index` narrow the objects section.

### vox-doc to ([reference/vox-doc/to/README.md](reference/vox-doc/to/README.md))

- [x] `--select` / `--select-index` write only the selected objects, the
      hierarchy pruned to what still places them.

## Finishing

- [x] `--layout` output on `list`, `vox-doc validate`, and `vox-doc show`;
      `palette show` keeps its own `--json`, and `node list` prints only its
      tree.
- [x] Help text and `clap_complete` completions cover the new commands.
- [ ] Tests per command, following the existing test style.

## Deferred (see [Future](reference/design-notes.md))

- [ ] Scene-assembly mode: hierarchy-node selection with baked transforms and
      instancing.
- [x] Single-object vs whole-document output nuance, and multi-object mesh
      layout in one file. Landed as one mesh object per selected object placed
      by the mirrored hierarchy, with `--split-files` for one mesh per object.
      See [implementation decisions](reference/implementation-decisions.md).
- [ ] stdin / stdout via `-`; dry-run for destructive palette ops.
- [x] Move the palette reduction from vxl into voxsmith as a general operation
      (public `reduce_palette` + plain enums; vxl maps its clap `ValueEnum`s like
      `FillMode` / `MaterialMode` do). See
      [implementation decisions](reference/implementation-decisions.md).
- [x] Adopt richer typed color types (one struct per space with `from_` / `to_`
      conversions) in `ty-math`, in place of the ad-hoc `[u8;4]` -> `[f64;3]`
      math, so spaces cannot be mixed. Landed with the generic `Mesh` (glTF is
      one reader into it). See
      [implementation decisions](reference/implementation-decisions.md).
