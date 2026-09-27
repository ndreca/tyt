// The path and grouping walks keep the narrower union because lists
// walk the tree directly.
#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod data_paths;
#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod group;
#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod groups;
// Box tables head with bare lines, so heading keeps the markdown
// union.
#[cfg(any(
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod heading;
mod leads_to_data;
// Tables pad through their frames, so pad_right keeps the narrower
// union.
#[cfg(any(feature = "render_text_columns", feature = "render_text_rows"))]
mod pad_right;

#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
pub(crate) use group::*;
#[cfg(any(
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
pub(crate) use heading::*;
#[cfg(any(feature = "render_text_columns", feature = "render_text_rows"))]
pub(crate) use pad_right::*;
