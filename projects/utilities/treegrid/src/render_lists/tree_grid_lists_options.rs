use crate::TreeGridListsLabelMode;

/// Options the `lists` layout consumes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridListsOptions {
    /// How lists spend the ancestor path.
    pub label: TreeGridListsLabelMode,
}

impl TreeGridListsOptions {
    /// Sets the label mode.
    pub fn with_label(mut self, label: TreeGridListsLabelMode) -> Self {
        self.label = label;
        self
    }
}
