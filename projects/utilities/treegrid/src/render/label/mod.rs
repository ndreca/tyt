// The path and grouping walks keep the narrower union because lists
// walk the tree directly.
#[cfg(any(
    feature = "render_columns",
    feature = "render_rows",
    feature = "render_md_tables"
))]
mod data_paths;
#[cfg(any(
    feature = "render_columns",
    feature = "render_rows",
    feature = "render_md_tables"
))]
mod group;
#[cfg(any(
    feature = "render_columns",
    feature = "render_rows",
    feature = "render_md_tables"
))]
mod groups;
mod heading;
mod leads_to_data;
// Tables pad through md_table, so pad_right keeps the narrower
// union.
#[cfg(any(feature = "render_columns", feature = "render_rows"))]
mod pad_right;

#[cfg(any(
    feature = "render_columns",
    feature = "render_rows",
    feature = "render_md_tables"
))]
pub(crate) use group::*;
pub(crate) use heading::*;
#[cfg(any(feature = "render_columns", feature = "render_rows"))]
pub(crate) use pad_right::*;
