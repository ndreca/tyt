# Checklist

The [README](README.md) holds the design. Each step is checked off as it lands.
Code-level choices go in an `implementation-decisions.md` beside this file.

## Ground rules

- The builder only records the model. Every check the `.sdfj` document can carry
  runs in vxl. Every voxj document vxl writes passes `vxl vox-doc validate`.
- The [modeling API](reference/modeling-api.md) and
  [model evaluation](reference/model-evaluation.md) lead. A change to the
  commands' surface or behavior lands in them in the same step.
- The builder has no dependencies and no install step. The builder runs under
  Node 24, Bun, and Deno and imports only `node:` modules and the builder's
  `.ts` files.
- `deno check`, `deno lint`, and `deno fmt --check` pass over the builder before
  a step stages. The builder's tests sit in `*.test.ts` files beside the code
  they cover and run under `deno test`. The pre-commit hook runs all four on a
  commit that touches the builder.
- A new crate starts at `0.1.0` with
  `[package.metadata.workspaces] independent = true` and joins the workspace
  members and `[patch.crates-io]`.
- Trial notes go in `trials.md` beside this file. A trial model that turns out
  well joins the builder's `examples/`. Renders stay out of git.

## Phase 1: pipeline

The builder records models as `.sdfj`, and vxl voxelizes and renders them. A
trial of about ten prompts then shows which operations Claude reaches for and
where the loop fails.

- [x] **S0. Open questions.** Settle the README's open questions and fold the
      answers into the plan and the reference pages.
- [x] **S1. Document.** A reference page for the `.sdfj` document beside the
      other two, with the entry the builder writes for each call in the modeling
      API. The document follows voxj: tables whose entries reference each other
      by index hold every value a model can share, including steps, objects, and
      nodes. The crates the [crate layout](README.md#decisions) sets read and
      write the document. `sdfcore` holds the tables as branded-id tables.
- [x] **S2. Builder.** The builder's `.ts` files:
  1. An entry point that puts the API in scope, imports the model, and writes
     the `.sdfj` document
  2. Type definitions declaring the API as globals, matching the modeling API's
     declaration blocks
  3. The builder's checks from model evaluation
  4. A test per call pinning the entry the call writes
- [x] **S3. Build command.** `vxl sdf-doc build` with `--runtime` and profiles
      at `sdfDoc.build.profiles`. vxl embeds the builder and writes it to a
      temporary directory for each run. `vxl profile sdf-doc build list` lists
      the profiles.
- [x] **S4. Shapes.** A voxsmith operation reads the `.sdfj` document and
      evaluates every shape, each with a test over a small grid against its
      distance in model evaluation.
- [x] **S5. Steps.** The sampling, the boxes' rounding and clamp, the grid, the
      five steps, the document checks, and parts with their grids, hierarchy,
      places, offsets, and frames. A test pins a box with corners on cell
      corners filling exactly its cells.
- [x] **S6. Materials.** The named and custom properties with their checks,
      `shades`, and the patterns.
- [ ] **S7. Libraries.** Libraries as [decision 17](README.md#decisions) sets
      them:
  1. The `names` table in sdfj, sdfcore, and the bridge
  2. The builder writing names, reading the libraries vxl hands it, and copying
     the entries a model uses
  3. `sdfDoc.build.libraries`, the build profiles' `libraries`, and `--library`
  4. vxl's built-in `materials` library with properties for every name the
     modeling API lists. A test checks every material with its default `shades`
- [ ] **S8. Voxelize command.** `vxl sdf-doc voxelize` with the flags and
      profiles the modeling API lists, the voxj document, and the report under
      `--report`. `vxl profile sdf-doc voxelize list` lists the profiles.
- [ ] **S9. Review profile.** A built-in `review` render profile imports the
      built-in `hero` view and adds orthographic `front`, `right`, and `top`
      views under the default `studio` rig. The render profile language's
      built-ins list `review`. The brightest library material keeps its shading
      short of white.
- [ ] **S10. Integration commands.** Move every workspace binary's `completion`
      command to `integration print-completions`. Update the tyt-meta templates
      that scaffold new crates and the READMEs that show the command.
- [ ] **S11. Skill.** Write the skill's workflow as the README lays it out. Move
      the workflow and the modeling API into vxl. The modeling API links to
      other pages by their repository URLs.
      `vxl integration print-skill voxel-modeling` prints the workflow and then
      the modeling API as one `SKILL.md`. Move model evaluation to
      `doc/ref/sdf-doc/` and the sdfj format into the sdfj crate's `docs/`.
- [ ] **S12. Trials.** Run about ten prompts through the skill, each in a fresh
      session. Log each prompt in `trials.md` with the passes it took, the
      failures, and the operations and report data Claude wanted and lacked. The
      log also notes where a model's colors read flat.
- [ ] **S13. Phase 2 design.** Write the phase 2 steps from the trials and
      revise the README.

## Phase 2: follow-ups

S13 writes phase 2's steps from the trials. Known so far:

1. The operators and patterns Claude reached for and lacked
2. The `.sdfj` changes those operators need
3. The contact sheet from the
   [rendering follow-ups](../voxel-rendering-followups/README.md#waiting-for-a-reason)
4. Coloring candidates for the trials to confirm:
   1. Patterns that take patterns as entries, for knots over grain or moss over
      cobbles. A pick that lands on a pattern lets that pattern decide the cell.
      `speckle` over `grain` cannot layer today because `speckle` writes its
      base into every cell it leaves unaccented
   2. A `hueShift` option on `shades` that turns the darks' hue one way and the
      lights' hue the other
   3. A step that reads the finished grid as `coat` does and picks darker shades
      in crevices and lighter ones on edges. Review renders shade crevices
      already, but an exported mesh keeps only the colors
5. Part candidates for the trials to confirm:
   1. A step that runs a part's steps on another part's grid. vxl samples the
      part's shapes at that grid's cells. Later steps can then carve, paint, or
      coat across both parts
   2. A rotation on parts that turns the part's shapes at any angle before
      sampling
6. A `typescript` library group whose sources vxl builds before the model
