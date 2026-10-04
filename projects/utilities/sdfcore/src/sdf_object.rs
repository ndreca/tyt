use crate::BSdfStep;
use branded_id::U32Id;

/// A part's grid and the steps that run over it: an entry of
/// [`SdfState::objects`](crate::SdfState::objects).
#[derive(Clone, Debug, PartialEq)]
pub struct SdfObject {
    /// The part's name.
    pub name: String,

    /// The part's steps, in list order.
    pub step_ids: Vec<U32Id<BSdfStep>>,
}
