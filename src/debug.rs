//! Debug visualization for 3D scenes.
//!
//! Provides a drop-in replacement fragment shader with 6 visualization modes,
//! controlled via a custom uniform at `@group(1) @binding(0)`.
//!
//! # Modes
//!
//! | Mode | Visualization |
//! |------|---------------|
//! | 0    | Normal lit (Blinn-Phong + shadows) |
//! | 1    | Surface normals as RGB |
//! | 2    | NdotL grayscale (primary light) |
//! | 3    | Shadow factor (green=lit, red=shadow) |
//! | 4    | Lit without shadows |
//! | 5    | Flat base color (no lighting) |
//!
//! # Usage
//!
//! ```rust,ignore
//! use iced3d::debug;
//!
//! impl Scene3DProgram for MyScene {
//!     fn fragment_shader(&self) -> &str { debug::FRAGMENT_WGSL }
//!     fn custom_uniforms_size(&self) -> usize { debug::UNIFORM_SIZE }
//!     fn setup(&self, bounds: Rectangle) -> Scene3DSetup {
//!         Scene3DSetup {
//!             scene,
//!             draws,
//!             custom_uniforms: Some(debug::uniforms(self.mode)),
//!         }
//!     }
//! }
//! ```

/// Debug Blinn-Phong fragment shader source (WGSL).
pub const FRAGMENT_WGSL: &str = include_str!("../shaders/debug_blinn_phong.wgsl");

/// Size in bytes of the debug uniform buffer (`@group(1) @binding(0)`).
///
/// Pass this from [`Scene3DProgram::custom_uniforms_size`](crate::widget::Scene3DProgram::custom_uniforms_size).
pub const UNIFORM_SIZE: usize = 16;

/// Number of available debug visualization modes (0–5).
pub const MODE_COUNT: u32 = 6;

/// Build the raw bytes for the debug uniform buffer.
///
/// `mode` selects the visualization (0–5). Values ≥ [`MODE_COUNT`] wrap to mode 0 behavior.
#[must_use]
pub fn uniforms(mode: u32) -> Vec<u8> {
    let data: [f32; 4] = [mode as f32, 0.0, 0.0, 0.0];
    data.iter().flat_map(|f| f.to_le_bytes()).collect()
}
