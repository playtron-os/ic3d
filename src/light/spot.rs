//! Spot light — directional cone of light from a position.

use super::Light;
use crate::gpu_types::{GpuLight, LIGHT_TYPE_SPOT};
use glam::Vec3;

/// Spot light — emits a cone of light from a position along a direction.
pub struct SpotLight {
    position: Vec3,
    direction: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
    inner_cone_angle: f32,
    outer_cone_angle: f32,
}

impl SpotLight {
    /// Create a spot light at `position` pointing along `direction`.
    ///
    /// - `inner_cone_angle`: half-angle (radians) of full-intensity cone
    /// - `outer_cone_angle`: half-angle (radians) of zero-intensity cone
    /// - `range`: attenuation distance (0 = infinite)
    pub fn new(
        position: Vec3,
        direction: Vec3,
        inner_cone_angle: f32,
        outer_cone_angle: f32,
        range: f32,
    ) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            color: Vec3::ONE,
            intensity: 1.0,
            range,
            inner_cone_angle,
            outer_cone_angle,
        }
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

    /// World-space position.
    #[must_use]
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Direction the light points (normalized).
    #[must_use]
    pub fn direction(&self) -> Vec3 {
        self.direction
    }
}

impl Light for SpotLight {
    fn to_gpu_light(&self) -> GpuLight {
        GpuLight {
            shadow_projection: [[0.0; 4]; 4],
            direction: self.direction.to_array(),
            light_type: LIGHT_TYPE_SPOT,
            color: self.color.to_array(),
            intensity: self.intensity,
            position: self.position.to_array(),
            range: self.range,
            inner_cone_cos: self.inner_cone_angle.cos(),
            outer_cone_cos: self.outer_cone_angle.cos(),
            _pad: [0.0; 2],
        }
    }
}

#[cfg(test)]
#[path = "spot_tests.rs"]
mod tests;
