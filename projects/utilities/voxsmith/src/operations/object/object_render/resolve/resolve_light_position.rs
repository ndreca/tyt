use crate::{
    Error, Result,
    operations::object::{PositionTransform, RenderElement},
};
use ty_math::{TyAngleUnit, TyBoundsF64, TyPoseF64, TyVector3Ext, TyVector3F64};

/// Resolves a point light's `transform` to a world position. Errors if a
/// `subject` or `orbit` frame has no subject.
///
/// # Arguments
/// * `element` - the light, which errors report.
/// * `subject` - the world bounds of the view's subject, or `None` when it
///   has no voxel.
/// * `view` - the resolved pose of the view being rendered.
pub fn resolve_light_position(
    element: &RenderElement,
    transform: &PositionTransform,
    subject: Option<&TyBoundsF64>,
    view: &TyPoseF64,
) -> Result<TyVector3F64> {
    let subject = || {
        subject
            .ok_or_else(|| Error::render_record(element.clone(), "frames a subject with no voxel"))
    };

    Ok(match *transform {
        PositionTransform::World { position } => position,

        PositionTransform::Subject { position } => subject()?.center + position,

        PositionTransform::Camera { position } => view.position + view.rotation * position,

        PositionTransform::Orbit {
            azimuth,
            elevation,
            distance,
        } => {
            subject()?.center
                + TyVector3F64::from_azimuth_elevation(
                    TyAngleUnit::Degrees.to_radians(azimuth),
                    TyAngleUnit::Degrees.to_radians(elevation),
                ) * distance
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{PositionTransform, RenderElement, resolve_light_position};
    use branded_id::U32Id;
    use ty_math::{TyBoundsF64, TyPoseF64, TyQuaternionF64, TyVector3F64};

    fn element() -> RenderElement {
        RenderElement::LightTransform {
            light_id: U32Id::from_u32(0),
        }
    }

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-9
    }

    #[test]
    fn each_frame_places_the_light() {
        let subject = TyBoundsF64::new(TyVector3F64::new(1.0, 2.0, 3.0), TyVector3F64::ONE);
        let view = TyPoseF64::new(
            TyVector3F64::new(0.0, 0.0, 10.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 90f64.to_radians()),
        );

        let place =
            |transform| resolve_light_position(&element(), &transform, Some(&subject), &view);

        assert_eq!(
            place(PositionTransform::World {
                position: TyVector3F64::X
            })
            .unwrap(),
            TyVector3F64::X
        );
        assert_eq!(
            place(PositionTransform::Subject {
                position: TyVector3F64::X
            })
            .unwrap(),
            TyVector3F64::new(2.0, 2.0, 3.0)
        );
        // The camera's +X is world -Z after a quarter turn about Y.
        assert!(close(
            place(PositionTransform::Camera {
                position: TyVector3F64::X
            })
            .unwrap(),
            TyVector3F64::new(0.0, 0.0, 9.0)
        ));
        assert!(close(
            place(PositionTransform::Orbit {
                azimuth: 90.0,
                elevation: 0.0,
                distance: 4.0,
            })
            .unwrap(),
            TyVector3F64::new(5.0, 2.0, 3.0)
        ));

        assert!(
            resolve_light_position(
                &element(),
                &PositionTransform::Subject {
                    position: TyVector3F64::X
                },
                None,
                &view
            )
            .is_err()
        );
    }
}
