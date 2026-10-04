use crate::{Dependencies, ReadFile, ReadFormat, Result, read};
use sdfcore::SdfMain;
use std::path::Path;

/// Reads the document at `input` into an [`SdfMain`]: the file's bytes, then
/// [`read()`].
pub fn load<D: Dependencies + ReadFile>(
    dependencies: &D,
    format: ReadFormat,
    input: &Path,
) -> Result<SdfMain> {
    read(dependencies, format, &dependencies.read_file(input)?)
}
