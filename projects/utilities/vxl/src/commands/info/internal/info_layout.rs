use crate::CliValue;
use voxsmith::operations::info::InfoLayout;

impl CliValue for InfoLayout {
    const VARIANTS: &'static [Self] = &[
        InfoLayout::BoxTables,
        InfoLayout::JsonCompact,
        InfoLayout::JsonPretty,
        InfoLayout::MdTables,
    ];

    fn name(self) -> &'static str {
        match self {
            InfoLayout::BoxTables => "box-tables",
            InfoLayout::JsonCompact => "json-compact",
            InfoLayout::JsonPretty => "json-pretty",
            InfoLayout::MdTables => "md-tables",
        }
    }

    fn help(self) -> &'static str {
        match self {
            InfoLayout::BoxTables => {
                "A file-name line over `document`, `palettes`, and `objects` record tables drawn \
                 with box glyphs"
            }
            InfoLayout::JsonCompact => "Compact, single-line JSON",
            InfoLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            InfoLayout::MdTables => {
                "A file-name title over `Document`, `Palettes`, and `Objects` record tables"
            }
        }
    }
}
