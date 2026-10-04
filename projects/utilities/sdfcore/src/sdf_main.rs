use crate::{Result, SdfState};

/// A model whose state passed [`SdfState::validate`]. The model never edits
/// the state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfMain {
    state: SdfState,
}

impl SdfMain {
    /// The model over `state`. Errors when `state` fails validation.
    pub fn new(state: SdfState) -> Result<Self> {
        state.validate()?;

        Ok(Self { state })
    }

    /// The model's tables.
    pub fn state(&self) -> &SdfState {
        &self.state
    }

    /// The model's tables, taken out of the model.
    pub fn into_state(self) -> SdfState {
        self.state
    }
}
