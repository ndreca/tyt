/// How [`profile_list`](crate::operations::mesh::profile_list()) lays out the
/// listing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileListLayout {
    /// A box-glyph tree, one branch per origin over its profiles.
    Hierarchy,

    /// A `# profiles` heading over one section per origin, each a numbered
    /// list of its profiles.
    Lists,

    /// One row per origin, its profiles beside it.
    Rows,

    /// One table, a column per origin over its profiles.
    Tables,

    /// Pretty-printed, multi-line JSON.
    JsonPretty,

    /// Compact, single-line JSON.
    JsonCompact,
}
