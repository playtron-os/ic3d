//! Perspective camera — realistic projection with depth foreshortening.

use super::Camera;
use glam::{Mat4, Vec3};

/// Perspective camera — realistic projection with depth foreshortening.
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
}
