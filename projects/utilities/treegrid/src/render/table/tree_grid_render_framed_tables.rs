use crate::{
    TreeGrid, TreeGridCells, TreeGridTableLabelMode, TreeGridTableShapeKind, render::TableFrame,
};

/// The table walk the table layouts share: the blocks a shape yields,
/// each heading and table drawn by a frame.
pub(crate) trait TreeGridRenderFramedTables {
    /// Renders the headings and tables of `shape` through `frame`, the
    /// nested headings spending the ancestor path per `label`. Blocks
    /// join with a blank line; an empty grid renders as an empty string.
    fn render_framed_tables<F: TableFrame>(
        &self,
        shape: TreeGridTableShapeKind,
        label: TreeGridTableLabelMode,
        frame: &F,
    ) -> String;
}

impl<C: TreeGridCells> TreeGridRenderFramedTables for TreeGrid<C> {
    fn render_framed_tables<F: TableFrame>(
        &self,
        shape: TreeGridTableShapeKind,
        label: TreeGridTableLabelMode,
        frame: &F,
    ) -> String {
        let blocks = match shape {
            TreeGridTableShapeKind::Flat => self.flat_blocks(frame),
            TreeGridTableShapeKind::Nested => self.nested_blocks(label, frame),
            TreeGridTableShapeKind::Records => self.records_blocks(frame),
        };
        if blocks.is_empty() {
            String::new()
        } else {
            format!("{}\n", blocks.join("\n\n"))
        }
    }
}
