/// How an SDF Json document's JSON is laid out.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SdfjSerialization {
    /// Compact JSON.
    #[default]
    Compact,

    /// Pretty-printed JSON.
    Pretty,
}
