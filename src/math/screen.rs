//! Screen-space projection and hit-testing utilities.

use glam::{Mat4, Quat, Vec2, Vec3};

use super::ray::Ray;

/// Project a world-space point to screen-space pixels.
///
/// Returns `None` if the point is behind the camera (clip `w ≤ 0`).
///
/// - `point`: world-space position
/// - `view_proj`: combined view-projection matrix
/// - `viewport`: viewport size in logical pixels `(width, height)`
///
/// Screen origin is top-left, X goes right, Y goes down.
///
/// ```rust
/// use ic3d::math::world_to_screen;
/// use ic3d::glam::{Mat4, Vec2, Vec3};
///
/// let vp = Mat4::IDENTITY;
/// // NDC (0,0) maps to screen center
/// let screen = world_to_screen(Vec3::ZERO, vp, Vec2::new(800.0, 600.0)).unwrap();
/// assert!((screen.x - 400.0).abs() < 1e-3);
/// assert!((screen.y - 300.0).abs() < 1e-3);
/// ```
#[must_use]
pub fn world_to_screen(point: Vec3, view_proj: Mat4, viewport: Vec2) -> Option<Vec2> {
    let clip = view_proj * point.extend(1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    Some(Vec2::new(
        (ndc.x + 1.0) * 0.5 * viewport.x,
        (1.0 - ndc.y) * 0.5 * viewport.y,
    ))
}

/// 2D distance from a point to a line segment.
///
/// Useful for screen-space hit testing against projected line handles.
///
/// - `p`: the test point
/// - `a`, `b`: segment endpoints
///
/// ```rust
/// use ic3d::math::point_to_segment_distance;
/// use ic3d::glam::Vec2;
///
/// let d = point_to_segment_distance(Vec2::new(0.0, 5.0), Vec2::ZERO, Vec2::new(10.0, 0.0));
/// assert!((d - 5.0).abs() < 1e-3);
/// ```
#[must_use]
pub fn point_to_segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-10 {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    (p - (a + ab * t)).length()
}

/// Compute a world-space scale that maintains a constant screen-pixel size.
///
/// Returns a world-space size such that an object at `position` will appear
/// `screen_px` pixels tall on screen, regardless of camera distance.
///
/// - `position`: world-space position of the object
/// - `camera`: camera metadata (position, forward, FOV)
/// - `viewport_height`: viewport height in logical pixels
/// - `screen_px`: desired on-screen size in pixels
///
/// Returns `fallback` (1.0) for orthographic cameras or when the
/// position is behind the camera.
///
/// ```rust
/// use ic3d::math::screen_constant_scale;
/// use ic3d::CameraInfo;
/// use ic3d::glam::{Mat4, Vec3};
///
/// let camera = CameraInfo {
///     position: Vec3::new(0.0, 0.0, 10.0),
///     forward: Vec3::NEG_Z,
///     fov_y: Some(std::f32::consts::FRAC_PI_4),
///     view_projection: Mat4::IDENTITY,
/// };
/// let scale = screen_constant_scale(Vec3::ZERO, &camera, 600.0, 80.0);
/// assert!(scale > 0.0);
/// ```
#[must_use]
pub fn screen_constant_scale(
    position: Vec3,
    camera: &crate::CameraInfo,
    viewport_height: f32,
    screen_px: f32,
) -> f32 {
    let Some(fov_y) = camera.fov_y else {
        return 1.0;
    };
    let depth = camera.forward.dot(position - camera.position);
    if depth < 1e-6 || viewport_height < 1.0 {
        return 1.0;
    }
    let px_world = 2.0 * depth * (fov_y * 0.5).tan() / viewport_height;
    screen_px * px_world
}

/// Project a world-space radius to screen-space pixels at a given position.
///
/// Computes the screen-pixel length of `world_radius` at `position` by
/// sampling a point on the silhouette edge perpendicular to the camera's
/// forward direction and measuring the screen-space distance.
///
/// Returns `None` if the center or edge point is behind the camera.
///
/// - `position`: world-space center of the radius
/// - `world_radius`: radius in world units
/// - `camera`: camera metadata (position, forward, FOV)
/// - `viewport`: viewport size in logical pixels `(width, height)`
///
/// ```rust
/// use ic3d::math::world_radius_to_screen;
/// use ic3d::CameraInfo;
/// use ic3d::glam::{Mat4, Vec2, Vec3};
///
/// let view = Mat4::look_at_rh(
///     Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y,
/// );
/// let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
/// let camera = CameraInfo {
///     position: Vec3::new(0.0, 0.0, 10.0),
///     forward: Vec3::NEG_Z,
///     fov_y: Some(std::f32::consts::FRAC_PI_4),
///     view_projection: proj * view,
/// };
/// let px = world_radius_to_screen(
///     Vec3::ZERO, 1.0, &camera, Vec2::new(800.0, 600.0),
/// ).unwrap();
/// assert!(px > 0.0);
/// ```
#[must_use]
pub fn world_radius_to_screen(
    position: Vec3,
    world_radius: f32,
    camera: &crate::CameraInfo,
    viewport: Vec2,
) -> Option<f32> {
    let vp = camera.view_projection;
    let center_screen = world_to_screen(position, vp, viewport)?;

    // Compute an edge point on the silhouette disc perpendicular to the camera.
    let view_rot = super::rotation::view_facing_rotation(camera.forward);
    let edge_point = position + view_rot * Vec3::new(world_radius, 0.0, 0.0);
    let edge_screen = world_to_screen(edge_point, vp, viewport)?;

    Some((edge_screen - center_screen).length())
}

/// A hittable shape in world space for screen-space hit testing.
///
/// Combine with [`screen_hit_test`] to check if a cursor is within
/// range of a projected shape. This is the building block for overlay
/// and gizmo interaction — all shapes project to screen space and test
/// pixel distance.
///
/// ```rust,ignore
/// use ic3d::math::{HitShape, screen_hit_test};
///
/// // Point hit (e.g. draggable handle):
/// let shape = HitShape::point(object_position, 24.0);
/// if let Some(dist) = screen_hit_test(&shape, cursor, view_proj, viewport) {
///     // dist is screen-space px distance (guaranteed < radius)
/// }
///
/// // Segment hit (e.g. gizmo axis):
/// let shape = HitShape::segment(arrow_start, arrow_end, 20.0);
/// if let Some(dist) = screen_hit_test(&shape, cursor, view_proj, viewport) {
///     // cursor is within 20px of the projected segment
/// }
///
/// // Arc hit (e.g. rotation ring):
/// let shape = HitShape::arc(
///     center, radius, rotation, start_angle, sweep, 24, 20.0,
/// );
/// if let Some(dist) = screen_hit_test(&shape, cursor, view_proj, viewport) {
///     // cursor is within 20px of the projected arc polyline
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub enum HitShape {
    /// A world-space point with a screen-pixel hit radius.
    Point {
        /// World-space position.
        position: Vec3,
        /// Hit radius in screen pixels.
        radius_px: f32,
    },
    /// A world-space line segment with a screen-pixel threshold.
    Segment {
        /// Segment start in world space.
        start: Vec3,
        /// Segment end in world space.
        end: Vec3,
        /// Hit threshold in screen pixels.
        threshold_px: f32,
    },
    /// A world-space circular arc, tested as a projected polyline.
    ///
    /// The arc lies in the local XZ plane (Y-up normal) and is transformed
    /// to world space by `rotation`. The polyline is sampled at `segments`
    /// intervals and each projected segment is tested against the cursor.
    Arc {
        /// World-space center of the arc.
        center: Vec3,
        /// Arc radius in world units.
        radius: f32,
        /// Local-to-world rotation for the arc plane.
        rotation: Quat,
        /// Start angle in radians (in local XZ space).
        start_angle: f32,
        /// Sweep angle in radians.
        sweep: f32,
        /// Number of polyline segments to sample.
        segments: u32,
        /// Hit threshold in screen pixels.
        threshold_px: f32,
    },
}

impl HitShape {
    /// Create a point hit shape.
    #[must_use]
    pub fn point(position: Vec3, radius_px: f32) -> Self {
        Self::Point {
            position,
            radius_px,
        }
    }

    /// Create a segment hit shape.
    #[must_use]
    pub fn segment(start: Vec3, end: Vec3, threshold_px: f32) -> Self {
        Self::Segment {
            start,
            end,
            threshold_px,
        }
    }

    /// Create an arc hit shape.
    ///
    /// The arc lies in the local XZ plane and is rotated to world space
    /// by `rotation`. It is sampled as a polyline with `segments` intervals.
    #[must_use]
    pub fn arc(
        center: Vec3,
        radius: f32,
        rotation: Quat,
        start_angle: f32,
        sweep: f32,
        segments: u32,
        threshold_px: f32,
    ) -> Self {
        Self::Arc {
            center,
            radius,
            rotation,
            start_angle,
            sweep,
            segments: segments.max(1),
            threshold_px,
        }
    }
}

/// Test if a cursor position hits a world-space [`HitShape`].
///
/// Projects the shape to screen space using the view-projection matrix
/// and returns the pixel distance if within the shape's threshold.
/// Returns `None` if the shape is behind the camera or the cursor is
/// too far away.
///
/// - `shape`: the world-space hit shape to test
/// - `cursor`: screen-space cursor position in pixels (top-left origin)
/// - `view_proj`: combined view-projection matrix
/// - `viewport`: viewport size in logical pixels `(width, height)`
#[must_use]
pub fn screen_hit_test(
    shape: &HitShape,
    cursor: Vec2,
    view_proj: Mat4,
    viewport: Vec2,
) -> Option<f32> {
    match *shape {
        HitShape::Point {
            position,
            radius_px,
        } => {
            let sp = world_to_screen(position, view_proj, viewport)?;
            let dist = (cursor - sp).length();
            (dist < radius_px).then_some(dist)
        }
        HitShape::Segment {
            start,
            end,
            threshold_px,
        } => {
            let s = world_to_screen(start, view_proj, viewport)?;
            let e = world_to_screen(end, view_proj, viewport)?;
            let dist = point_to_segment_distance(cursor, s, e);
            (dist < threshold_px).then_some(dist)
        }
        HitShape::Arc {
            center,
            radius,
            rotation,
            start_angle,
            sweep,
            segments,
            threshold_px,
        } => {
            let mut min_dist = f32::MAX;
            for i in 0..segments {
                let a0 = start_angle + sweep * i as f32 / segments as f32;
                let a1 = start_angle + sweep * (i + 1) as f32 / segments as f32;
                let p0 = center + rotation * Vec3::new(a0.cos(), 0.0, a0.sin()) * radius;
                let p1 = center + rotation * Vec3::new(a1.cos(), 0.0, a1.sin()) * radius;
                if let (Some(s0), Some(s1)) = (
                    world_to_screen(p0, view_proj, viewport),
                    world_to_screen(p1, view_proj, viewport),
                ) {
                    min_dist = min_dist.min(point_to_segment_distance(cursor, s0, s1));
                }
            }
            (min_dist < threshold_px).then_some(min_dist)
        }
    }
}

/// Find the closest hit among multiple labeled shapes.
///
/// Tests each `(label, shape)` pair against the cursor and returns the
/// label and distance of the closest hit, or `None` if nothing was hit.
///
/// This eliminates the common "iterate shapes, keep best" boilerplate:
///
/// ```rust,ignore
/// use ic3d::math::{HitShape, screen_hit_test_closest};
///
/// let shapes = [
///     ("x", HitShape::segment(pos, pos + Vec3::X, 20.0)),
///     ("y", HitShape::segment(pos, pos + Vec3::Y, 20.0)),
/// ];
/// if let Some((axis, dist)) = screen_hit_test_closest(shapes, cursor, vp, viewport) {
///     println!("Hit {axis} at {dist}px");
/// }
/// ```
#[must_use]
pub fn screen_hit_test_closest<I, T>(
    shapes: I,
    cursor: Vec2,
    view_proj: Mat4,
    viewport: Vec2,
) -> Option<(T, f32)>
where
    I: IntoIterator<Item = (T, HitShape)>,
{
    let mut best: Option<(T, f32)> = None;
    for (label, shape) in shapes {
        if let Some(dist) = screen_hit_test(&shape, cursor, view_proj, viewport) {
            if best.as_ref().is_none_or(|(_, d)| dist < *d) {
                best = Some((label, dist));
            }
        }
    }
    best
}

/// Unproject a screen-space cursor position onto the XZ ground plane (Y = `height`).
///
/// Casts a ray from the camera through the cursor and intersects with the
/// horizontal plane at the given Y height. Returns the world-space XZ
/// coordinates, or `None` if the camera is looking away from the plane
/// (ray parallel or pointing upward).
///
/// - `cursor`: screen-space cursor position in pixels (top-left origin)
/// - `viewport`: viewport size in logical pixels `(width, height)`
/// - `inv_view_proj`: inverse of the combined view-projection matrix
/// - `height`: Y-level of the ground plane (typically `0.0`)
///
/// ```rust,ignore
/// use ic3d::math::screen_to_ground;
/// use ic3d::glam::{Mat4, Vec2};
///
/// let inv_vp = (proj * view).inverse();
/// if let Some(pos) = screen_to_ground(cursor, viewport, inv_vp, 0.0) {
///     println!("cursor at world ({:.1}, {:.1})", pos.x, pos.y);
/// }
/// ```
#[must_use]
pub fn screen_to_ground(
    cursor: Vec2,
    viewport: Vec2,
    inv_view_proj: Mat4,
    height: f32,
) -> Option<Vec2> {
    let ray = Ray::from_screen(cursor, viewport, inv_view_proj);
    let t = ray.intersect_plane(Vec3::Y, Vec3::new(0.0, height, 0.0))?;
    // Only accept forward intersections
    if t < 0.0 {
        return None;
    }
    let hit = ray.point_at(t);
    Some(Vec2::new(hit.x, hit.z))
}

#[cfg(test)]
#[path = "screen_tests.rs"]
mod tests;
