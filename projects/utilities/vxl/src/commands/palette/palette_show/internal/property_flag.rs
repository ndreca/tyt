/// One `--properties-from` or `--property` occurrence. A `--property` holds its
/// four fields unparsed until the run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PropertyFlag {
    PropertiesFrom(String),
    Property([String; 4]),
}
