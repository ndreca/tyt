use crate::CliValue;
use voxsmith::operations::palette_show::PaletteShowLayout;

impl CliValue for PaletteShowLayout {
    const VARIANTS: &'static [Self] = &[
        PaletteShowLayout::Columns,
        PaletteShowLayout::Hierarchy,
        PaletteShowLayout::JsonCompact,
        PaletteShowLayout::JsonPretty,
        PaletteShowLayout::MdTables,
        PaletteShowLayout::Rows,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteShowLayout::Columns => "columns",
            PaletteShowLayout::Hierarchy => "hierarchy",
            PaletteShowLayout::JsonCompact => "json-compact",
            PaletteShowLayout::JsonPretty => "json-pretty",
            PaletteShowLayout::MdTables => "md-tables",
            PaletteShowLayout::Rows => "rows",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteShowLayout::Columns => {
                "Each value collection as its own column beneath its label, padded to a common \
                 width"
            }
            PaletteShowLayout::Hierarchy => {
                "The value collections as a box-glyph tree of palettes, properties, and \
                 components, each value collection's values inline on its node"
            }
            PaletteShowLayout::JsonCompact => {
                "The value collection tree as single-line JSON records"
            }
            PaletteShowLayout::JsonPretty => "The value collection tree as indented JSON records",
            PaletteShowLayout::MdTables => {
                "The value collections as aligned markdown tables led by a `#` column of 0-based \
                 material indices; `--table-shape` picks per-palette tables under headings or one \
                 flat comparison table"
            }
            PaletteShowLayout::Rows => {
                "Each value collection on one row, separated by a blank line, under labels padded \
                 to the longest so each row's first value aligns. Swatch cells abut into a strip; \
                 other formats put one space between cells"
            }
        }
    }
}
