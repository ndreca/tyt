use ty_math::TyVector3F64;

/// The transform a point light takes: a position read in a frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PositionTransform {
    /// The document's frame.
    World {
        /// The position, in meters.
        position: TyVector3F64,
    },

    /// World axes centered on the subject's bounds.
    Subject {
        /// The position from the subject's center, in meters.
        position: TyVector3F64,
    },

    /// The view being rendered, so the light follows every view.
    Camera {
        /// The position from the camera, on its axes, in meters.
        position: TyVector3F64,
    },

    /// A position on a sphere about the subject's center.
    Orbit {
        /// Degrees from +Z toward +X.
        azimuth: f64,

        /// Degrees from the azimuth's direction toward +Y.
        elevation: f64,

        /// The sphere's radius, in meters.
        distance: f64,
    },
}
