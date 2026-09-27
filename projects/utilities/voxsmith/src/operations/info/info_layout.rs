/// How [`info`](crate::operations::info::info()) lays out the report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfoLayout {
    /// A file-name line over `document`, `palettes`, and `objects` record
    /// tables drawn with box glyphs.
    BoxTables,

    /// Compact, single-line JSON.
    JsonCompact,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// A file-name title over `document`, `palettes`, and `objects` record
    /// tables.
    MdTables,
}
