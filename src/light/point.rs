//! Point light — omnidirectional light emitting from a position.

use super::Light;
use crate::gpu_types::{GpuLight, LIGHT_TYPE_POINT};
use glam::Vec3;

/// Point light — emits light equally in all directions from a position.
pub struct PointLight {
    position: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
}

impl PointLight {
    /// Create a point light at `position` with the given `range`.
    ///
    /// Range controls attenuation distance. 0 = infinite (no falloff).
    pub fn new(position: Vec3, range: f32) -> Self {
        Self {
            position,
            color: Vec3::ONE,
            intensity: 1.0,
            range,
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

    /// Attenuation range.
    #[must_use]
    pub fn range(&self) -> f32 {
        self.range
    }
}

impl Light for PointLight {
    fn to_gpu_light(&self) -> GpuLight {
        GpuLight {
            shadow_projection: [[0.0; 4]; 4],
            direction: [0.0; 3],
            light_type: LIGHT_TYPE_POINT,
            color: self.color.to_array(),
            intensity: self.intensity,
            position: self.position.to_array(),
            range: self.range,
            inner_cone_cos: 0.0,
            outer_cone_cos: 0.0,
            _pad: [0.0; 2],
        }
    }
}
