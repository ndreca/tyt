/// How [`palette_list`](crate::operations::palette_list::palette_list()) lays
/// out the listing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaletteListLayout {
    /// Indented tree, one palette per branch.
    Hierarchy,

    /// Compact, single-line JSON.
    JsonCompact,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// A `# palettes` heading over one aligned record table, one row per
    /// palette.
    MdTables,
}
