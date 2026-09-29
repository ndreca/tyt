use crate::{
    Error, Result,
    operations::object::{RenderElement, Rotation, RotationTransform, resolve_rotation},
};
use ty_math::{TyPoseF64, TyQuaternionF64, TyVector3F64};

/// Resolves a directional light's `transform` to a world rotation. Errors if
/// a look-at has no target, because the light sits at its frame's origin.
///
/// # Arguments
/// * `element` - the light, which errors report.
/// * `view` - the resolved pose of the view being rendered.
pub fn resolve_light_rotation(
    element: &RenderElement,
    transform: &RotationTransform,
    view: &TyPoseF64,
) -> Result<TyQuaternionF64> {
    let (frame, rotation): (TyQuaternionF64, Rotation) = match *transform {
        RotationTransform::World { rotation } => (TyQuaternionF64::IDENTITY, rotation),
        RotationTransform::Camera { rotation } => (view.rotation, rotation),
    };

    let local = resolve_rotation(&rotation, TyVector3F64::ZERO)
        .ok_or_else(|| Error::render_record(element.clone(), "looks at its own position"))?;

    Ok(frame * local)
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{
        RenderElement, Rotation, RotationTransform, resolve_light_rotation,
    };
    use branded_id::U32Id;
    use ty_math::{TyPoseF64, TyQuaternionF64, TyVector3Ext, TyVector3F64};

    fn element() -> RenderElement {
        RenderElement::LightTransform {
            light_id: U32Id::from_u32(0),
        }
    }

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-9
    }

    #[test]
    fn a_camera_frame_light_turns_with_the_view() {
        // The view looks along +X, which turns the view's -X into world -Z.
        // A light at the view's upper left comes from world -Z and +Y.
        let view = TyPoseF64::new(
            TyVector3F64::ZERO,
            TyQuaternionF64::from_axis_angle(TyVector3F64::Y, -90f64.to_radians()),
        );
        assert!(close(view.rotation * -TyVector3F64::Z, TyVector3F64::X));

        let rotation = resolve_light_rotation(
            &element(),
            &RotationTransform::Camera {
                rotation: Rotation::Angles {
                    azimuth: -30.0,
                    elevation: 30.0,
                },
            },
            &view,
        )
        .unwrap();

        let from = view.rotation
            * TyVector3F64::from_azimuth_elevation(-30f64.to_radians(), 30f64.to_radians());
        assert!(close(rotation * -TyVector3F64::Z, -from));
        assert!(from.y > 0.0 && from.z < 0.0);

        let world = resolve_light_rotation(
            &element(),
            &RotationTransform::World {
                rotation: Rotation::Angles {
                    azimuth: 0.0,
                    elevation: 90.0,
                },
            },
            &view,
        )
        .unwrap();
        assert!(close(world * -TyVector3F64::Z, -TyVector3F64::Y));

        assert!(
            resolve_light_rotation(
                &element(),
                &RotationTransform::World {
                    rotation: Rotation::LookAt { target: None },
                },
                &view,
            )
            .is_err()
        );
    }
}
