//! Machinery shared by two or more layout renders, grouped by feature
//! gate so each `cfg` sits once, on the module that rides it.

mod annotated_label;
mod cell;
#[cfg(any(
    feature = "render_box_hierarchy",
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod cell_render;
#[cfg(any(
    feature = "render_box_hierarchy",
    feature = "render_box_tables",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod cell_separator;
#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
mod label;
#[cfg(any(feature = "render_box_tables", feature = "render_md_tables"))]
mod table;
mod visible_width;

pub(crate) use cell::*;
#[cfg(any(
    feature = "render_box_tables",
    feature = "render_text_columns",
    feature = "render_md_lists",
    feature = "render_md_tables",
    feature = "render_text_rows"
))]
pub(crate) use label::*;
#[cfg(any(feature = "render_box_tables", feature = "render_md_tables"))]
pub(crate) use table::*;
pub(crate) use visible_width::*;
