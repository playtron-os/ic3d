//! Utility functions.

use crate::pipeline::shaders::{
    FLAT_COLOR_WGSL, SCENE_UNIFORMS_WGSL, SHADOW_PCF_WGSL, STANDARD_VS_WGSL, VERTEX_IO_WGSL,
};

/// Compose a complete WGSL shader from a consumer fragment shader.
///
/// Prepends all engine preludes (scene uniforms, vertex IO, standard vertex
/// shader, shadow PCF) so the consumer only writes a `@fragment fn fs_main`.
///
/// # Example
///
/// ```rust,ignore
/// let shader = ic3d::compose_shader(include_str!("my_fragment.wgsl"));
/// let pipeline = RenderPipeline3D::new(device, format, &shader, config);
/// ```
#[must_use]
pub fn compose_shader(fragment_wgsl: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        SCENE_UNIFORMS_WGSL, VERTEX_IO_WGSL, STANDARD_VS_WGSL, SHADOW_PCF_WGSL, fragment_wgsl,
    )
}

/// Compose the overlay (flat/unlit) shader.
///
/// Prepends scene uniforms, vertex IO, and standard vertex shader, then
/// appends the built-in flat-color fragment shader. The resulting shader
/// outputs `material.rgba` directly with no lighting or shadow sampling.
#[must_use]
pub(crate) fn compose_overlay_shader() -> String {
    format!(
        "{}\n{}\n{}\n{}",
        SCENE_UNIFORMS_WGSL, VERTEX_IO_WGSL, STANDARD_VS_WGSL, FLAT_COLOR_WGSL,
    )
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;
