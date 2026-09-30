use crate::{
    Error, Result,
    commands::{ViewElements, ViewEntry},
};
use branded_id::IdVec;
use std::collections::BTreeMap;
use voxsmith::operations::object::{BRenderView, ViewRecord};

/// The views the flags fill by name. A view's name suffixes its file, so an
/// empty name or one holding a path separator errors.
#[derive(Debug, Default)]
pub struct ViewTable {
    views: BTreeMap<String, ViewElements>,
}

impl ViewTable {
    /// The elements of the view `name`, which `flag` mentions, created empty
    /// on the first mention.
    pub(crate) fn view(&mut self, flag: &str, name: &str) -> Result<&mut ViewElements> {
        check_view_name(flag, name)?;

        Ok(self
            .views
            .entry(name.to_owned())
            .or_insert_with(|| ViewElements::new(name.to_owned())))
    }

    /// Whether no flag mentions a view.
    pub(crate) fn is_empty(&self) -> bool {
        self.views.is_empty()
    }

    /// The views by id in name order, each flag element standing and
    /// `entries`, the profile stack's views, filling the rest.
    pub(crate) fn finish(
        mut self,
        entries: &BTreeMap<String, ViewEntry>,
    ) -> Result<IdVec<BRenderView, ViewRecord>> {
        for name in entries.keys() {
            check_view_name("the profile stack", name)?;

            self.views
                .entry(name.clone())
                .or_insert_with(|| ViewElements::new(name.clone()));
        }

        self.views
            .into_iter()
            .map(|(name, elements)| elements.finish(entries.get(&name)))
            .collect::<Result<Vec<_>>>()
            .map(IdVec::from)
    }
}

/// Errors unless `name`, which `origin` gives, can suffix a file: non-empty
/// and free of path separators.
fn check_view_name(origin: &str, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::usage(format!(
            "{origin} names a view with an empty name, and the name suffixes the view's file"
        )));
    }

    if name.contains('/') || name.contains('\\') {
        return Err(Error::usage(format!(
            "{origin} names the view `{name}`, and the name suffixes the view's file, so it \
             cannot contain a path separator"
        )));
    }

    Ok(())
}
