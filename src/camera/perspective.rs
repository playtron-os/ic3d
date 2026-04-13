//! Perspective camera — realistic projection with depth foreshortening.

use super::Camera;
use glam::{Mat4, Vec3};

/// Perspective camera — realistic projection with depth foreshortening.
#[derive(Debug, Clone)]
pub struct PerspectiveCamera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
    fov_y: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl PerspectiveCamera {
    /// Create a default perspective camera.
    #[must_use]
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_4, // 45 degrees
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
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

    /// Vertical field of view in radians.
    #[must_use]
    pub fn fov(mut self, fov_y: f32) -> Self {
        self.fov_y = fov_y;
        self
    }

    /// Aspect ratio (width / height).
    #[must_use]
    pub fn aspect(mut self, aspect: f32) -> Self {
        self.aspect = aspect;
        self
    }

    /// Near and far clip planes.
    #[must_use]
    pub fn clip(mut self, near: f32, far: f32) -> Self {
        self.near = near;
        self.far = far;
        self
    }

    // ──── Runtime mutators ────

    /// Set the camera position (for runtime updates).
    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    /// Set the look-at target (for runtime updates).
    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;
    }

    /// Set the vertical field of view in radians (for runtime updates).
    pub fn set_fov(&mut self, fov_y: f32) {
        self.fov_y = fov_y;
    }

    /// Set the aspect ratio (for runtime updates).
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Set the near and far clip planes (for runtime updates).
    pub fn set_clip(&mut self, near: f32, far: f32) {
        self.near = near;
        self.far = far;
    }

    /// The look-at target point.
    #[must_use]
    pub fn camera_target(&self) -> Vec3 {
        self.target
    }
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera for PerspectiveCamera {
    fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    fn camera_position(&self) -> Vec3 {
        self.position
    }

    fn camera_forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    fn fov_y(&self) -> Option<f32> {
        Some(self.fov_y)
    }

    fn camera_target(&self) -> Vec3 {
        self.target
    }

    fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

#[cfg(test)]
#[path = "perspective_tests.rs"]
mod tests;
