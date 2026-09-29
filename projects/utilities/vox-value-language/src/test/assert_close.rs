use crate::{Components, Value};

/// Asserts two `f32` lists agree within a small tolerance.
pub fn assert_close(found: &Value, expected: &[f32]) {
    let Components::F32(found) = found.components() else {
        panic!("{found:?} is not f32");
    };

    assert_eq!(
        found.len(),
        expected.len(),
        "{found:?} against {expected:?}"
    );

    for (found, expected) in found.iter().zip(expected) {
        assert!(
            (found - expected).abs() <= 1e-5 * expected.abs().max(1.0),
            "{found} is not {expected}"
        );
    }
}
