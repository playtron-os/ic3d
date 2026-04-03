//! Lightweight 3D instanced rendering for iced applications.
//!
//! Shadow mapping, configurable MSAA, camera abstractions, mesh primitives,
//! and WGSL shader preludes. Consumers write only a fragment shader.
//!
//! ## GPU bind groups
//!
//! - Group 0 (engine): `SceneUniforms` + light storage + shadow map + sampler
//! - Group 1 (consumer): optional custom uniforms

mod buffer;
mod camera;
mod custom_uniforms;
#[cfg(feature = "debug")]
pub mod debug;
mod gpu_types;
mod light;
pub mod math;
mod mesh;
mod pipeline;
mod post_process;
mod scene;
mod shaders;
mod shadow;
pub mod svg;
mod transform;
mod utils;
pub mod widget;

pub use buffer::{BufferPool, DynBuffer};
pub use camera::{Camera, OrthographicCamera, PerspectiveCamera};
pub use custom_uniforms::CustomUniformBuffer;
pub use gpu_types::{
    GpuLight, InstanceData, SceneUniforms, Vertex, LIGHT_TYPE_DIRECTIONAL, LIGHT_TYPE_POINT,
    LIGHT_TYPE_SPOT, MAX_LIGHTS,
};
pub use light::{DirectionalLight, Light, PointLight, SpotLight};
pub use mesh::{Mesh, MeshBuffer, MeshBuilder};
pub use pipeline::{PipelineConfig, RenderPipeline3D};
pub use post_process::PostProcessPass;
pub use scene::{Scene, SceneData};
pub use shaders::{
    BLINN_PHONG_WGSL, SCENE_UNIFORMS_WGSL, SHADOW_PCF_WGSL, STANDARD_VS_WGSL, VERTEX_IO_WGSL,
};
pub use shadow::{DrawCall, ShadowPass};
pub use transform::Transform;
pub use utils::compose_shader;

/// Re-exported for consumer convenience — no need to add `glam` as a separate dependency.
pub use glam;

/// Re-exported for consumer convenience — matches the wgpu version used internally.
pub use wgpu;

/// Re-exported for consumer convenience — matches the iced version used internally.
pub use iced;
