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
