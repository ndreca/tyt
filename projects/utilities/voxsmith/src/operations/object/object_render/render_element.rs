use branded_id::U32Id;
use std::fmt::{Display, Formatter, Result as FmtResult};
use voxrender::BRenderLight;

/// The element of a render record an error rose from.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RenderElement {
    /// A light's transform.
    LightTransform {
        /// The light.
        light_id: U32Id<BRenderLight>,
    },

    /// A view's projection.
    ViewProjection {
        /// The view's name.
        name: String,
    },

    /// A view's transform.
    ViewTransform {
        /// The view's name.
        name: String,
    },
}

impl Display for RenderElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RenderElement::LightTransform { light_id } => {
                write!(f, "light {}'s transform", light_id.to_u32())
            }
            RenderElement::ViewProjection { name } => write!(f, "view {name}'s projection"),
            RenderElement::ViewTransform { name } => write!(f, "view {name}'s transform"),
        }
    }
}
