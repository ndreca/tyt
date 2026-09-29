use crate::{Components, Dimension, Domain, Value};

/// A value of `f32` components.
pub fn f32s(domain: Domain, dimension: Dimension, values: &[f32]) -> Value {
    Value::new(domain, dimension, Components::F32(values.to_vec())).unwrap()
}
