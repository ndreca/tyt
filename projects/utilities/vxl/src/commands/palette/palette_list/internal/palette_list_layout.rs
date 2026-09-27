use crate::CliValue;
use voxsmith::operations::palette_list::PaletteListLayout;

impl CliValue for PaletteListLayout {
    const VARIANTS: &'static [Self] = &[
        PaletteListLayout::BoxHierarchy,
        PaletteListLayout::BoxTables,
        PaletteListLayout::JsonCompact,
        PaletteListLayout::JsonPretty,
        PaletteListLayout::MdTables,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteListLayout::BoxHierarchy => "box-hierarchy",
            PaletteListLayout::BoxTables => "box-tables",
            PaletteListLayout::JsonCompact => "json-compact",
            PaletteListLayout::JsonPretty => "json-pretty",
            PaletteListLayout::MdTables => "md-tables",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteListLayout::BoxHierarchy => {
                "Indented tree, one palette per branch, like `hierarchy show`"
            }
            PaletteListLayout::BoxTables => {
                "A `palettes` line over one aligned record table drawn with box glyphs, one row \
                 per palette"
            }
            PaletteListLayout::JsonCompact => "Compact, single-line JSON",
            PaletteListLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            PaletteListLayout::MdTables => {
                "A `# palettes` heading over one aligned record table, one row per palette"
            }
        }
    }
}
