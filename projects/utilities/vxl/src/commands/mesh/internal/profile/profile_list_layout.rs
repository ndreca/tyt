use crate::CliValue;
use voxsmith::operations::mesh::ProfileListLayout;

impl CliValue for ProfileListLayout {
    const VARIANTS: &'static [Self] = &[
        ProfileListLayout::Hierarchy,
        ProfileListLayout::JsonCompact,
        ProfileListLayout::JsonPretty,
        ProfileListLayout::MdLists,
        ProfileListLayout::MdTables,
        ProfileListLayout::Rows,
    ];

    fn name(self) -> &'static str {
        match self {
            ProfileListLayout::Hierarchy => "hierarchy",
            ProfileListLayout::JsonCompact => "json-compact",
            ProfileListLayout::JsonPretty => "json-pretty",
            ProfileListLayout::MdLists => "md-lists",
            ProfileListLayout::MdTables => "md-tables",
            ProfileListLayout::Rows => "rows",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ProfileListLayout::Hierarchy => {
                "A box-glyph tree, one branch per origin over its profiles"
            }
            ProfileListLayout::JsonCompact => "Compact, single-line JSON",
            ProfileListLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            ProfileListLayout::MdLists => {
                "A `# profiles` heading over one section per origin, each a numbered markdown \
                 list of its profiles"
            }
            ProfileListLayout::MdTables => {
                "One markdown table, a column per origin over its profiles"
            }
            ProfileListLayout::Rows => "One row per origin, its profiles beside it",
        }
    }
}
