use crate::CliValue;
use voxsmith::operations::vox_doc::VoxDocShowLayout;

impl CliValue for VoxDocShowLayout {
    const VARIANTS: &'static [Self] = &[
        VoxDocShowLayout::BoxTables,
        VoxDocShowLayout::JsonCompact,
        VoxDocShowLayout::JsonPretty,
        VoxDocShowLayout::MdTables,
    ];

    fn name(self) -> &'static str {
        match self {
            VoxDocShowLayout::BoxTables => "box-tables",
            VoxDocShowLayout::JsonCompact => "json-compact",
            VoxDocShowLayout::JsonPretty => "json-pretty",
            VoxDocShowLayout::MdTables => "md-tables",
        }
    }

    fn help(self) -> &'static str {
        match self {
            VoxDocShowLayout::BoxTables => {
                "A file-name line over `document`, `palettes`, and `objects` record tables drawn \
                 with box glyphs"
            }

            VoxDocShowLayout::JsonCompact => "Compact, single-line JSON",

            VoxDocShowLayout::JsonPretty => "Pretty-printed, multi-line JSON",

            VoxDocShowLayout::MdTables => {
                "A file-name title over `Document`, `Palettes`, and `Objects` record tables"
            }
        }
    }
}
