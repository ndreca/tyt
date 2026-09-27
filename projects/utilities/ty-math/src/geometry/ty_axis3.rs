use std::fmt::{self, Display};

/// One axis of a 3D vector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TyAxis3 {
    X,
    Y,
    Z,
}

impl TyAxis3 {
    /// The axis's component index into a vector, `0` through `2`.
    pub fn index(self) -> usize {
        match self {
            TyAxis3::X => 0,
            TyAxis3::Y => 1,
            TyAxis3::Z => 2,
        }
    }
}

impl Display for TyAxis3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            TyAxis3::X => "x",
            TyAxis3::Y => "y",
            TyAxis3::Z => "z",
        };

        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyAxis3, TyVector3U32};

    #[test]
    fn indexes_a_vector_and_prints_lowercase() {
        let vector = TyVector3U32::new(4, 5, 6);

        assert_eq!(vector[TyAxis3::Y.index()], 5);
        assert_eq!(TyAxis3::Z.to_string(), "z");
    }
}
