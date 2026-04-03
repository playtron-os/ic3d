//! Scene composition: camera + lights → [`SceneUniforms`] + [`GpuLight`] array.

use crate::camera::Camera;
use crate::gpu_types::{GpuLight, SceneUniforms, MAX_LIGHTS};
use crate::light::Light;
use bytemuck::Zeroable;

/// Scene output: uniforms + light array ready for GPU upload.
#[derive(Debug)]
pub struct SceneData {
    pub uniforms: SceneUniforms,
    pub lights: Vec<GpuLight>,
}

/// Composes camera and lights into GPU-ready data.
pub struct Scene<'a> {
    camera: &'a dyn Camera,
    camera_position: [f32; 3],
    lights: [GpuLight; MAX_LIGHTS],
    light_count: usize,
    time: f32,
    ambient: f32,
    screen_size: [f32; 2],
    shadow_map_size: f32,
}

impl<'a> Scene<'a> {
    /// Create a scene from a camera (no lights yet — add with [`light`](Self::light)).
    pub fn new(camera: &'a dyn Camera) -> Self {
        Self {
            camera,
            camera_position: [0.0; 3],
            lights: [GpuLight::zeroed(); MAX_LIGHTS],
            light_count: 0,
            time: 0.0,
            ambient: 0.1,
            screen_size: [1.0, 1.0],
            shadow_map_size: 2048.0,
        }
    }

    /// Set camera world-space position (for specular/Fresnel).
    #[must_use]
    pub fn camera_position(mut self, pos: [f32; 3]) -> Self {
        self.camera_position = pos;
        self
    }

    /// Add a light. First light added is the shadow caster.
    ///
    /// Panics if more than [`MAX_LIGHTS`] are added.
    #[must_use]
    pub fn light(mut self, light: &dyn Light) -> Self {
        assert!(
            self.light_count < MAX_LIGHTS,
            "iced3d: exceeded MAX_LIGHTS ({MAX_LIGHTS})"
        );
        self.lights[self.light_count] = light.to_gpu_light();
        self.light_count += 1;
        self
    }

    /// Set elapsed time in seconds.
    #[must_use]
    pub fn time(mut self, t: f32) -> Self {
        self.time = t;
        self
    }

    /// Set ambient light level (0.0–1.0).
    #[must_use]
    pub fn ambient(mut self, a: f32) -> Self {
        self.ambient = a;
        self
    }

    /// Set screen resolution in pixels.
    #[must_use]
    pub fn screen_size(mut self, width: f32, height: f32) -> Self {
        self.screen_size = [width, height];
        self
    }

    /// Set shadow map resolution in texels (default: 2048).
    ///
    /// Should match [`PipelineConfig::shadow_map_size`](crate::PipelineConfig::shadow_map_size).
    #[must_use]
    pub fn shadow_map_size(mut self, size: f32) -> Self {
        self.shadow_map_size = size;
        self
    }

    /// Produce GPU-ready scene data (uniforms + light array).
    #[must_use]
    pub fn build(&self) -> SceneData {
        let uniforms = SceneUniforms {
            view_projection: self.camera.view_projection().to_cols_array_2d(),
            camera_position: self.camera_position,
            time: self.time,
            screen_size: self.screen_size,
            light_count: self.light_count as u32,
            ambient: self.ambient,
            shadow_map_size: self.shadow_map_size,
            _pad: [0.0; 3],
        };

        SceneData {
            uniforms,
            lights: self.lights[..self.light_count].to_vec(),
        }
    }
}

#[cfg(test)]
#[path = "scene_tests.rs"]
mod tests;
