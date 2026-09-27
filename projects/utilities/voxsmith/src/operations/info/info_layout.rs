/// How [`info`](crate::operations::info::info()) lays out the report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfoLayout {
    /// Compact, single-line JSON.
    JsonCompact,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// A file-name title over `document`, `palettes`, and `objects` record
    /// tables.
    MdTables,
}
