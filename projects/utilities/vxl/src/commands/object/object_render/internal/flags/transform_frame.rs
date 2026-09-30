use crate::CliValue;

/// The frame a view's or light's transform is read in, before the shape
/// checks which frames its entity takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransformFrame {
    /// The document's frame.
    World,

    /// World axes centered on the subject's bounds.
    Subject,

    /// The view being rendered, so a light follows every view.
    Camera,
}

impl CliValue for TransformFrame {
    const VARIANTS: &'static [Self] = &[
        TransformFrame::World,
        TransformFrame::Subject,
        TransformFrame::Camera,
    ];

    fn name(self) -> &'static str {
        match self {
            TransformFrame::World => "world",
            TransformFrame::Subject => "subject",
            TransformFrame::Camera => "camera",
        }
    }

    fn help(self) -> &'static str {
        match self {
            TransformFrame::World => "The document's frame",
            TransformFrame::Subject => "World axes centered on the subject's bounds",
            TransformFrame::Camera => "The view being rendered, so a light follows every view",
        }
    }
}
