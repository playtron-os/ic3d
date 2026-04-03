//! Orthographic camera — parallel projection, no foreshortening.

use super::Camera;
use glam::{Mat4, Vec3};

/// Orthographic camera — parallel projection, no foreshortening.
pub struct OrthographicCamera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
    half_w: f32,
    half_h: f32,
    near: f32,
    far: f32,
}

impl OrthographicCamera {
    /// Create a default orthographic camera at the origin looking along -Z.
    #[must_use]
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 3.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            half_w: 5.0,
            half_h: 5.0,
            near: -5.0,
            far: 5.0,
        }
    }

    #[must_use]
    pub fn position(mut self, pos: Vec3) -> Self {
        self.position = pos;
        self
    }

    #[must_use]
    pub fn target(mut self, target: Vec3) -> Self {
        self.target = target;
        self
    }

    #[must_use]
    pub fn up(mut self, up: Vec3) -> Self {
        self.up = up;
        self
    }

    /// Set orthographic half-extents (half-width, half-height).
    #[must_use]
    pub fn extents(mut self, half_w: f32, half_h: f32) -> Self {
        self.half_w = half_w;
        self.half_h = half_h;
        self
    }

    /// Set near and far clip planes.
    #[must_use]
    pub fn depth(mut self, near: f32, far: f32) -> Self {
        self.near = near;
        self.far = far;
        self
    }

    /// Convert a screen-space pixel position to a world-space point on the
    /// camera's near plane.
    ///
    /// `screen_pos` is in pixels (origin top-left), `screen_size` is
    /// `(width, height)` in pixels. Y is flipped automatically (screen Y
    /// is top-down, world Y is bottom-up).
    #[must_use]
    pub fn screen_to_world(&self, screen_pos: [f32; 2], screen_size: [f32; 2]) -> Vec3 {
        let nx = screen_pos[0] / screen_size[0];
        let ny = 1.0 - screen_pos[1] / screen_size[1];
        Vec3::new(
            self.position.x - self.half_w + nx * 2.0 * self.half_w,
            self.position.y - self.half_h + ny * 2.0 * self.half_h,
            self.position.z,
        )
    }
}

impl Default for OrthographicCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera for OrthographicCamera {
    fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    fn projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            -self.half_w,
            self.half_w,
            -self.half_h,
            self.half_h,
            self.near,
            self.far,
        )
    }
}
