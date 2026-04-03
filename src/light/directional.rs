//! Directional light with orthographic shadow projection.

use super::Light;
use crate::gpu_types::{GpuLight, LIGHT_TYPE_DIRECTIONAL};
use glam::{Mat4, Vec3};

/// Directional light. `direction` points FROM source TOWARD the scene.
pub struct DirectionalLight {
    direction: Vec3,
    color: Vec3,
    intensity: f32,
    scene_center: Vec3,
    half_w: f32,
    half_h: f32,
    near: f32,
    far: f32,
}

impl DirectionalLight {
    /// Create a directional light aimed at `scene_center` with the given shadow volume.
    pub fn new(direction: Vec3, scene_center: Vec3, extent: f32, depth: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color: Vec3::ONE,
            intensity: 1.0,
            scene_center,
            half_w: extent,
            half_h: extent,
            near: 0.1,
            far: depth,
        }
    }

    /// Set non-square shadow extents.
    #[must_use]
    pub fn with_extents(mut self, half_w: f32, half_h: f32) -> Self {
        self.half_w = half_w;
        self.half_h = half_h;
        self
    }

    /// Set light color (linear RGB, default: white).
    #[must_use]
    pub fn with_color(mut self, color: Vec3) -> Self {
        self.color = color;
        self
    }

    /// Set intensity multiplier (default: 1.0).
    #[must_use]
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// Direction the light travels (from source toward scene).
    #[must_use]
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Direction toward the light source (for shading).
    #[must_use]
    pub fn to_light(&self) -> Vec3 {
        -self.direction
    }

    /// Light-space view-projection matrix for shadow mapping.
    #[must_use]
    pub fn shadow_projection(&self) -> Mat4 {
        let light_eye = self.scene_center - self.direction * (self.far * 0.5);
        let light_target = self.scene_center;

        let up = if self.direction.y.abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };

        let light_view = Mat4::look_at_rh(light_eye, light_target, up);
        let light_proj = Mat4::orthographic_rh(
            -self.half_w,
            self.half_w,
            -self.half_h,
            self.half_h,
            self.near,
            self.far,
        );

        light_proj * light_view
    }
}

impl Light for DirectionalLight {
    fn to_gpu_light(&self) -> GpuLight {
        GpuLight {
            shadow_projection: self.shadow_projection().to_cols_array_2d(),
            direction: self.direction.to_array(),
            light_type: LIGHT_TYPE_DIRECTIONAL,
            color: self.color.to_array(),
            intensity: self.intensity,
            position: [0.0; 3],
            range: 0.0,
            inner_cone_cos: 0.0,
            outer_cone_cos: 0.0,
            _pad: [0.0; 2],
        }
    }
}

#[cfg(test)]
#[path = "directional_tests.rs"]
mod tests;
