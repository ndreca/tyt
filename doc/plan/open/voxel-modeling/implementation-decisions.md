# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as they
land.

## S1. Document

1. sdfcore holds a model whole and never edits it. `SdfState` holds public
   `IdVec` tables, and `SdfMain::new` validates a state once. sdfcore has no
   retain, release, gc, or ext because nothing in Rust edits a model and one
   format leaves no foreign state to carry. A shade and its `shades` entry
   reference each other's tables, and a retain per entry would have to order
   the two. Validating whole tables avoids the order
2. Validation covers the format's rules, finite numbers, and unique property
   names and `json` keys. A state built in Rust can hold a NaN, and serde_json
   would write the NaN as `null`. The arguments' values wait for S5's checks
3. Numbers stay `f64`, counts and seeds included, because S5's checks test
   whether a count is whole. A shade's `index` reads as a whole number in the
   format, and sdfcore holds the index as a `u32`
4. Entries reference each other by `U32Id`. The bridge errors on a wire index
   past the `u32` id space, as voxj-voxcore does
5. A step's material or pattern is an `SdfStepMaterial` in sdfcore. sdfj
   mirrors the wire with two optional keys, and the bridge errors unless
   exactly one holds an index
6. `SdfEntryId` displays as `table[index]` with the wire table names, so
   sdfcore's errors point into the document. The bridge starts its own errors
   about one entry the same way
7. sdfj's variants mirror sdfcore's field for field. A reference field drops
   its `_id` or `_ids` on the wire, and `shape_id` reads `shape`. An optional
   key holding `null` fails to read
8. `SdfjMap` takes its value type. Material properties and `json` objects share
   one ordered map that rejects a repeated key
9. An `int` or `json` property value writes as an adjacently tagged
   `{ kind, value }`. The plain forms parse untagged
10. sdfj's types error on writing a NaN, an infinity, or a repeated object key.
    serde_json would otherwise write a non-finite number as `null` and keep both
    keys. sdfj-codec's encode returns a `Result` because a hand-built `SdfjFile`
    can hold any of the three. sdfj-sdfcore's byte writers `expect` success
    because `SdfMain` validation rules all three out
11. `SDFJ_VERSION` lives in sdfj. sdfj's parse takes any `u32`, and the bridge
    checks the version on read
12. sdfconv keeps meshconv's format markers and visitors but moves bytes instead
    of a file list and has no ext module. An `.sdfj` document is one file, and
    its state carries nothing past sdfcore's. `SdfjSerialization` picks compact
    JSON, the default, or pretty-printed JSON
13. The bridge's tests round-trip the chair, the forest, and a fixture holding
    one entry of every kind with every optional key

## S2. Builder

1. The builder is the TypeScript package `sdfj-builder` in
   `projects/utilities`, apart from vxl for a later npm release. The
   TypeScript sits in `ts/`. `main.ts` takes the model path and the output
   path and reads `library.json` from its own directory
2. The folder is also a crate with its Rust in `src/`. `SDFJ_BUILDER_FILES`
   embeds every file a run needs because a published vxl can include only
   files inside its own directory. A test checks the list against the folder
3. `deno.json` sets `nodeModulesDir` to `none` because Deno otherwise expects a
   `node_modules` beside the `package.json`
4. Each module holds one section of the modeling API. One export per file would
   split the API into about sixty files of a few lines each
5. The builder checks every argument against the type the modeling API
   declares, because a runtime strips a model's types without checking them. A
   value the document cannot hold then errors at the call that made it. Ranges
   and whole numbers stay vxl's checks. `shades` checks its count because it
   returns that many materials, and a boolean checks for a first shape because
   that shape picks the table
6. A value holds its entry in a private field, which keeps each class distinct
   to the type checker. Vectors, lists, properties, and `json` values copy at
   the call, and no value changes after its call returns
7. `globals.d.ts` declares each global as the type of its export from `api.ts`.
   `putApiInScope` assigns the globals through `Pick<typeof globalThis, ...>`,
   and a name missing from either file fails the type check
8. `deno.json` turns on `verbatimModuleSyntax` and `erasableSyntaxOnly`. With
   both on, every file Deno checks also runs under Node's type stripping
9. The builder writes compact JSON and a trailing newline, the form sdfj-codec
   writes by default. `jsonText` keeps the sign of `-0` and errors on a value
   without a JSON form instead of writing `null`
10. An error escapes uncaught. Each runtime prints the message and a stack whose
    frames include the model file's line
11. The tests run on `Deno.test` and `node:assert/strict`, and the builder stays
    free of dependencies. `test/chair.ts` and `test/forest.ts` hold the sdfj
    format's examples, and their tests compare the written documents with the
    format page's
12. The builder holds no material names. `mat` types every name as a
    `Material`, and the library vxl passes in decides which names it reads

## S3. Build command

1. sdfj-builder holds each runtime's program and arguments in
   `JavaScriptRuntime` and the library's path in `SDFJ_BUILDER_LIBRARY_PATH`.
   vxl's code holds no path inside the builder's directory
2. `--config` points Deno at the builder's `deno.json`, which keeps a config
   beside the model out of the run. Deno's write access covers every path
   because Deno's permission lists split a path on commas with no escape
3. vxl's `Dependencies` gains `CreateTempDir` and `RunProgram`. The runtime
   inherits vxl's standard streams, which carry the builder's messages and the
   model's stack unchanged
4. The builder writes the document inside the temporary directory. vxl then
   writes the output through `WriteFile`, which creates missing directories for
   every command's output
5. vxl passes an empty library until S6 embeds one
6. A stand-in for `node` tests the run because `cargo test` runs without a
   JavaScript runtime installed. At S3, Node 24.15 and Deno 2.9 ran the builder
   through vxl by hand. Bun was not installed
