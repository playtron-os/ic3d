//! Rotation math utilities for screen-space angular manipulation.

use glam::{Vec2, Vec3};

/// Compute the angle (radians) from `center` to `point` in screen space.
///
/// Returns the angle in `[−π, π]` using `atan2(dy, dx)`, where
/// the Y axis points downward (screen convention).
///
/// ```rust
/// use ic3d::math::screen_angle;
/// use ic3d::glam::Vec2;
///
/// let angle = screen_angle(Vec2::new(400.0, 300.0), Vec2::new(500.0, 300.0));
/// assert!(angle.abs() < 1e-3); // pointing right ≈ 0 radians
/// ```
#[must_use]
pub fn screen_angle(center: Vec2, point: Vec2) -> f32 {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    dy.atan2(dx)
}

/// Wrap an angle difference to `[−π, π]` for continuity.
///
/// When computing the difference between two angles, the raw subtraction
/// can jump by 2π when crossing the ±π boundary. This function wraps
/// the result back into the continuous range.
///
/// ```rust
/// use ic3d::math::wrap_angle_delta;
///
/// // Small forward step: unchanged
/// let d = wrap_angle_delta(0.1);
/// assert!((d - 0.1).abs() < 1e-6);
///
/// // Crossing the boundary: wraps instead of jumping
/// let d = wrap_angle_delta(std::f32::consts::PI + 0.5);
/// assert!(d < 0.0); // wrapped to negative
/// assert!(d.abs() < std::f32::consts::PI);
/// ```
#[must_use]
pub fn wrap_angle_delta(mut delta: f32) -> f32 {
    if delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    if delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

/// Compute the sign correction for rotation direction.
///
/// When the camera faces the same direction as the rotation axis,
/// screen-space clockwise rotation should map to positive world
/// rotation. When the camera faces the opposite direction, the
/// mapping inverts.
///
/// Returns `1.0` when the camera looks along the axis (dot ≥ 0),
/// `-1.0` when looking against it.
///
/// ```rust
/// use ic3d::math::rotation_sign;
/// use ic3d::glam::Vec3;
///
/// // Camera looking along +Z, rotating around Z → positive
/// assert_eq!(rotation_sign(Vec3::Z, Vec3::Z), 1.0);
///
/// // Camera looking against the axis → negative
/// assert_eq!(rotation_sign(Vec3::NEG_Z, Vec3::Z), -1.0);
/// ```
#[must_use]
pub fn rotation_sign(camera_forward: Vec3, axis: Vec3) -> f32 {
    if camera_forward.dot(axis) < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// Compute a rotation that orients the Y-up plane to face the camera.
///
/// Returns a quaternion that rotates `Vec3::Y` to point toward the camera
/// (i.e. opposite `cam_forward`). Useful for billboard discs, view-facing
/// circles, and any geometry that should always face the viewer.
///
/// ```rust
/// use ic3d::math::view_facing_rotation;
/// use ic3d::glam::Vec3;
///
/// let rot = view_facing_rotation(Vec3::NEG_Z);
/// let up = rot * Vec3::Y;
/// // Y axis now points toward the camera (along +Z)
/// assert!((up - Vec3::Z).length() < 1e-3);
/// ```
#[must_use]
pub fn view_facing_rotation(cam_forward: Vec3) -> glam::Quat {
    glam::Quat::from_rotation_arc(Vec3::Y, -cam_forward.normalize_or_zero())
}

/// Compute the start angle and sweep for the front-facing portion of a ring.
///
/// Given a ring with a certain axis direction and local-to-world rotation,
/// determines how much of the ring is visible from the camera and where
/// the visible arc starts. The torus mesh is assumed to lie in the local
/// XZ plane (Y-up normal).
///
/// Returns `(start_angle, sweep)` in radians:
/// - `sweep` ranges from π (edge-on) to τ (face-on)
/// - `start_angle` is centered on the front-facing portion
///
/// ```rust
/// use ic3d::math::front_arc_params;
/// use ic3d::glam::{Quat, Vec3};
///
/// // Y-axis ring seen from +Z camera: ring is edge-on → sweep ≈ π
/// let (start, sweep) = front_arc_params(Quat::IDENTITY, Vec3::Y, Vec3::NEG_Z);
/// assert!((sweep - std::f32::consts::PI).abs() < 0.1);
///
/// // Y-axis ring seen from +Y camera: ring is face-on → sweep ≈ τ
/// let (start, sweep) = front_arc_params(Quat::IDENTITY, Vec3::Y, Vec3::NEG_Y);
/// assert!((sweep - std::f32::consts::TAU).abs() < 0.1);
/// ```
#[must_use]
pub fn front_arc_params(
    axis_rotation: glam::Quat,
    axis_direction: Vec3,
    cam_forward: Vec3,
) -> (f32, f32) {
    // |dot| ≈ 1 → face-on (fully visible), 0 → edge-on (half visible).
    let face_factor = axis_direction.dot(cam_forward).abs();
    let sweep = std::f32::consts::PI + std::f32::consts::PI * face_factor;

    // Transform camera forward into the torus's local space and compute
    // the angle in the local XZ plane.
    let cam_local = axis_rotation.inverse() * cam_forward;
    let back_angle = cam_local.z.atan2(cam_local.x);
    let front_angle = back_angle + std::f32::consts::PI;
    let start = front_angle - sweep * 0.5;

    (start, sweep)
}

#[cfg(test)]
#[path = "rotation_tests.rs"]
mod tests;
