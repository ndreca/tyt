use crate::TreeGridHeaderOptions;

/// How `lists` headings spend the ancestor path.
///
/// A data node's label heads its list or is dropped because a list has
/// no inline slot for the `Concat` path of
/// [`TreeGridLabelMode`](crate::TreeGridLabelMode).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridListsLabelMode {
    /// No headings anywhere.
    None,

    /// Every node on the way to data heads its subtree with its leaf
    /// segment at a nested level.
    Header(TreeGridHeaderOptions),
}

impl Default for TreeGridListsLabelMode {
    fn default() -> Self {
        TreeGridListsLabelMode::Header(TreeGridHeaderOptions::default())
    }
}
