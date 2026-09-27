use crate::CliValue;
use voxsmith::operations::validate::ValidateLayout;

impl CliValue for ValidateLayout {
    const VARIANTS: &'static [Self] = &[
        ValidateLayout::JsonCompact,
        ValidateLayout::JsonPretty,
        ValidateLayout::MdTables,
    ];

    fn name(self) -> &'static str {
        match self {
            ValidateLayout::JsonCompact => "json-compact",
            ValidateLayout::JsonPretty => "json-pretty",
            ValidateLayout::MdTables => "md-tables",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ValidateLayout::JsonCompact => "Compact, single-line JSON",
            ValidateLayout::JsonPretty => "Pretty-printed, multi-line JSON",
            ValidateLayout::MdTables => {
                "A file-name heading over one line per check and a closing pass/fail summary"
            }
        }
    }
}
