//! Utility functions.

use crate::shaders::{SCENE_UNIFORMS_WGSL, SHADOW_PCF_WGSL, STANDARD_VS_WGSL, VERTEX_IO_WGSL};

/// Compose a complete WGSL shader from a consumer fragment shader.
///
/// Prepends all engine preludes (scene uniforms, vertex IO, standard vertex
/// shader, shadow PCF) so the consumer only writes a `@fragment fn fs_main`.
///
/// # Example
///
/// ```rust,ignore
/// let shader = iced3d::compose_shader(include_str!("my_fragment.wgsl"));
/// let pipeline = RenderPipeline3D::new(device, format, &shader, config);
/// ```
#[must_use]
pub fn compose_shader(fragment_wgsl: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        SCENE_UNIFORMS_WGSL, VERTEX_IO_WGSL, STANDARD_VS_WGSL, SHADOW_PCF_WGSL, fragment_wgsl,
    )
}
