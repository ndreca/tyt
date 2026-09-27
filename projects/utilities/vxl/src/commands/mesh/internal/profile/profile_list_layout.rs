use crate::CliValue;
use voxsmith::operations::mesh::ProfileListLayout;

impl CliValue for ProfileListLayout {
    const VARIANTS: &'static [Self] = &[
        ProfileListLayout::Hierarchy,
        ProfileListLayout::Lists,
        ProfileListLayout::Rows,
        ProfileListLayout::Tables,
        ProfileListLayout::JsonPretty,
        ProfileListLayout::JsonCompact,
    ];

    fn name(self) -> &'static str {
        match self {
            ProfileListLayout::Hierarchy => "hierarchy",
            ProfileListLayout::Lists => "lists",
            ProfileListLayout::Rows => "rows",
            ProfileListLayout::Tables => "tables",
            ProfileListLayout::JsonPretty => "json-pretty",
            ProfileListLayout::JsonCompact => "json-compact",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ProfileListLayout::Hierarchy => {
                "A box-glyph tree, one branch per origin over its profiles"
            }
            ProfileListLayout::Lists => {
                "A `# profiles` heading over one section per origin, each a numbered list of its \
                 profiles"
            }
            ProfileListLayout::Rows => "One row per origin, its profiles beside it",
            ProfileListLayout::Tables => "One table, a column per origin over its profiles",
            ProfileListLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            ProfileListLayout::JsonCompact => "Compact, single-line JSON",
        }
    }
}
