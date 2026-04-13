//! Ray casting: screen-to-world unprojection and geometric intersection.

use glam::{Mat4, Vec2, Vec3};

/// A ray in 3D space defined by an origin and a normalized direction.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// World-space origin of the ray.
    pub origin: Vec3,
    /// Normalized direction vector.
    pub direction: Vec3,
}

impl Ray {
    /// Construct a ray from origin and direction (direction is normalized).
    #[must_use]
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize_or_zero(),
        }
    }

    /// Unproject a screen-space cursor position into a world-space ray.
    ///
    /// - `screen_pos`: cursor position in pixels (origin at top-left)
    /// - `viewport_size`: width and height of the viewport in pixels
    /// - `inv_view_proj`: inverse of the combined view-projection matrix
    #[must_use]
    pub fn from_screen(screen_pos: Vec2, viewport_size: Vec2, inv_view_proj: Mat4) -> Self {
        // Convert screen coordinates to NDC [-1, 1]
        let ndc_x = (2.0 * screen_pos.x / viewport_size.x) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y / viewport_size.y);

        // Unproject near and far points
        let near_ndc = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_ndc = glam::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let near_world = inv_view_proj * near_ndc;
        let far_world = inv_view_proj * far_ndc;

        // Perspective divide
        let near = near_world.truncate() / near_world.w;
        let far = far_world.truncate() / far_world.w;

        let direction = (far - near).normalize_or_zero();

        Self {
            origin: near,
            direction,
        }
    }

    /// Point along the ray at parameter `t`.
    #[must_use]
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Intersect with a plane defined by a normal and a point on the plane.
    ///
    /// Returns the `t` parameter along the ray, or `None` if the ray is
    /// parallel to the plane (within epsilon).
    #[must_use]
    pub fn intersect_plane(&self, plane_normal: Vec3, plane_point: Vec3) -> Option<f32> {
        let denom = plane_normal.dot(self.direction);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = plane_normal.dot(plane_point - self.origin) / denom;
        Some(t)
    }

    /// Find the closest approach between this ray and an infinite line
    /// defined by a point and direction.
    ///
    /// Returns `(t_ray, t_line)` — the parameters along each line at the
    /// closest point. The actual closest points are `ray.point_at(t_ray)`
    /// and `line_point + line_dir * t_line`.
    #[must_use]
    pub fn closest_to_line(&self, line_point: Vec3, line_dir: Vec3) -> (f32, f32) {
        let w = self.origin - line_point;
        let a = self.direction.dot(self.direction);
        let b = self.direction.dot(line_dir);
        let c = line_dir.dot(line_dir);
        let d = self.direction.dot(w);
        let e = line_dir.dot(w);

        let denom = a * c - b * b;
        if denom.abs() < 1e-10 {
            // Lines are parallel
            return (0.0, e / c);
        }

        let t_ray = (b * e - c * d) / denom;
        let t_line = (a * e - b * d) / denom;

        (t_ray, t_line)
    }

    /// Minimum distance from the ray to a line segment `(a, b)`.
    ///
    /// Only considers the forward portion of the ray (`t >= 0`).
    #[must_use]
    pub fn distance_to_segment(&self, a: Vec3, b: Vec3) -> f32 {
        let seg_dir = b - a;
        let seg_len = seg_dir.length();
        if seg_len < 1e-10 {
            // Degenerate segment: distance to point
            let t = (a - self.origin).dot(self.direction).max(0.0);
            return (self.point_at(t) - a).length();
        }
        let seg_dir_norm = seg_dir / seg_len;

        // Check if lines are nearly parallel
        let cross = self.direction.cross(seg_dir_norm);
        if cross.length_squared() < 1e-6 {
            // Parallel: perpendicular distance is constant along the overlap.
            // Project segment endpoints onto the ray to find closest approach.
            let t_a = (a - self.origin).dot(self.direction).max(0.0);
            let t_b = (b - self.origin).dot(self.direction).max(0.0);
            // Pick the endpoint that gets us closest
            let p_a = self.point_at(t_a);
            let p_b = self.point_at(t_b);
            let d_a = (p_a - a).length();
            let d_b = (p_b - b).length();
            return d_a.min(d_b);
        }

        let (t_ray, t_seg) = self.closest_to_line(a, seg_dir_norm);

        let t_ray = t_ray.max(0.0);
        let t_seg = t_seg.clamp(0.0, seg_len);

        let p_ray = self.point_at(t_ray);
        let p_seg = a + seg_dir_norm * t_seg;

        (p_ray - p_seg).length()
    }
}

#[cfg(test)]
#[path = "ray_tests.rs"]
mod tests;
