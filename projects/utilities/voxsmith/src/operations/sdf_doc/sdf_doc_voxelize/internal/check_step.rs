use crate::operations::sdf_doc::ArgumentCheck;
use sdfcore::SdfStep;
use std::{collections::HashSet, result::Result as StdResult};

/// Checks the arguments of the step `step` other than its shapes, material,
/// and pattern.
pub fn check_step(step: &SdfStep) -> StdResult<(), String> {
    match step {
        SdfStep::Add { .. } | SdfStep::Carve { .. } | SdfStep::Paint { .. } => Ok(()),

        SdfStep::Coat { depth, .. } => match depth {
            Some(depth) => ArgumentCheck::new("coat").whole_above_zero("depth", *depth),
            None => Ok(()),
        },

        SdfStep::Set { points, .. } => {
            let mut seen = HashSet::with_capacity(points.len());

            match points
                .iter()
                .find(|point| !seen.insert(point.to_array().map(f64::to_bits)))
            {
                Some(point) => Err(ArgumentCheck::new("set").fail(
                    "points",
                    "distinct",
                    format!("[{}, {}, {}] twice", point.x, point.y, point.z),
                )),

                None => Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::check_step;
    use branded_id::U32Id;
    use sdfcore::{SdfStep, SdfStepMaterial};
    use ty_math::TyVector3F64;

    #[test]
    fn a_set_lists_each_point_once_and_a_coat_reaches_whole_cells() {
        let set = |points| SdfStep::Set {
            name: "studs".to_string(),
            points,
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
        };

        assert!(check_step(&set(vec![TyVector3F64::ZERO, TyVector3F64::X])).is_ok());
        assert_eq!(
            check_step(&set(vec![
                TyVector3F64::X,
                TyVector3F64::ZERO,
                TyVector3F64::X
            ])),
            Err("set points must be distinct, not [1, 0, 0] twice".to_string())
        );

        let coat = SdfStep::Coat {
            name: "skin".to_string(),
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
            sides: None,
            depth: Some(1.5),
            within_id: None,
        };
        assert_eq!(
            check_step(&coat),
            Err("coat depth must be a whole number above zero, not 1.5".to_string())
        );
    }
}
