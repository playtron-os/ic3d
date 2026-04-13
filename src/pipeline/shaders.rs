//! WGSL shader preludes — embedded from `shaders/*.wgsl` via `include_str!`.
//!
//! Consumers concatenate these with their own fragment shader to avoid
//! duplicating struct definitions and alignment-sensitive layouts.

/// `SceneUniforms` struct + group 0 bindings. Matches [`SceneUniforms`](crate::SceneUniforms).
pub const SCENE_UNIFORMS_WGSL: &str = include_str!("../../shaders/scene_uniforms.wgsl");

/// `VertexIn` / `VertexOut` structs matching [`Vertex`](crate::Vertex) and [`InstanceData`](crate::InstanceData).
pub const VERTEX_IO_WGSL: &str = include_str!("../../shaders/vertex_io.wgsl");

/// Standard `vs_main` vertex shader. Requires [`SCENE_UNIFORMS_WGSL`] and [`VERTEX_IO_WGSL`].
pub const STANDARD_VS_WGSL: &str = include_str!("../../shaders/standard_vs.wgsl");

/// 9-tap PCF shadow sampling: `sample_shadow_pcf(in.light_clip_pos, 2048.0)`.
pub const SHADOW_PCF_WGSL: &str = include_str!("../../shaders/shadow_pcf.wgsl");

/// Standard Blinn-Phong fragment shader with multi-light + shadow support.
///
/// This is the default fragment shader used by [`Scene3DProgram`](crate::widget::Scene3DProgram)
/// when `fragment_shader()` is not overridden. Material `vec4` is interpreted as `(r, g, b, shininess)`.
pub const BLINN_PHONG_WGSL: &str = include_str!("../../shaders/blinn_phong.wgsl");

/// Flat (unlit) fragment shader — outputs `material.rgba` directly, no lighting.
///
/// Used by the overlay pipeline for gizmos and other helpers that should
/// render as solid color without Blinn-Phong shading.
pub const FLAT_COLOR_WGSL: &str = include_str!("../../shaders/flat_color.wgsl");

/// Internal shadow depth pass shader. Used by [`ShadowPass`](crate::ShadowPass).
pub(crate) const SHADOW_WGSL: &str = include_str!("../../shaders/shadow_pass.wgsl");

#[cfg(test)]
#[path = "shaders_tests.rs"]
mod tests;
