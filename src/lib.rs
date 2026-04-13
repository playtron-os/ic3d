//! Lightweight 3D instanced rendering for iced applications.
//!
//! Shadow mapping, configurable MSAA, camera abstractions, mesh primitives,
//! and WGSL shader preludes. Consumers write only a fragment shader.
//!
//! ## GPU bind groups
//!
//! - Group 0 (engine): `SceneUniforms` + light storage + shadow map + sampler
//! - Group 1 (consumer): optional custom uniforms

mod camera;
pub mod gizmo;
pub mod graph;
mod light;
pub mod math;
mod mesh;
mod overlay;
mod pipeline;
mod scene;
pub mod widget;

// ── Re-exports: camera ──
pub use camera::{Camera, CameraInfo, OrthographicCamera, PerspectiveCamera};

// ── Re-exports: light ──
pub use light::{DirectionalLight, Light, PointLight, SpotLight};

// ── Re-exports: mesh ──
pub use mesh::svg;
pub use mesh::{Mesh, MeshBuffer, MeshBuilder};

// ── Re-exports: transform ──
pub use scene::transform::Transform;

// ── Re-exports: scene ──
pub use scene::builder::{Scene, SceneData};
pub use scene::context::{SceneContext, SceneHandle};
pub use scene::object::SceneObjectId;

// ── Re-exports: overlay ──
pub use overlay::base::{Overlay, OverlayContext, OverlayEvent, OverlayInput};
pub use overlay::draggable::{DragState, Draggable, DraggableOverlay};
pub use overlay::interactive::{Interactive, InteractiveContext, InteractiveOverlay, ShapeHit};

// ── Re-exports: pipeline ──
pub use pipeline::buffer::{BufferPool, DynBuffer};
pub use pipeline::custom_uniforms::CustomUniformBuffer;
pub use pipeline::gpu_types::{
    GpuLight, InstanceData, SceneUniforms, Vertex, LIGHT_TYPE_DIRECTIONAL, LIGHT_TYPE_POINT,
    LIGHT_TYPE_SPOT, MAX_LIGHTS,
};
pub use pipeline::post_process::PostProcessPass;
pub use pipeline::render_pipeline::{PipelineConfig, RenderPipeline3D};
pub use pipeline::shaders::{
    BLINN_PHONG_WGSL, FLAT_COLOR_WGSL, SCENE_UNIFORMS_WGSL, SHADOW_PCF_WGSL, STANDARD_VS_WGSL,
    VERTEX_IO_WGSL,
};
pub use pipeline::shadow::{DrawCall, ShadowPass};
pub use pipeline::utils::compose_shader;

// ── Re-exports: debug (feature-gated) ──
#[cfg(feature = "debug")]
pub use pipeline::debug;

/// Convenience subscription that fires once per rendered frame.
///
/// Pair with [`graph::SceneGraph::tick`] for frame-accurate delta time:
///
/// ```rust,ignore
/// fn subscription(&self) -> Subscription<Message> {
///     ic3d::frames().map(|_| Message::Tick)
/// }
///
/// fn update(&mut self, message: Message) {
///     match message {
///         Message::Tick => {
///             let dt = self.graph.tick();
///             // animate using dt and self.graph.elapsed()
///         }
///     }
/// }
/// ```
pub fn frames() -> iced::Subscription<()> {
    iced::window::frames().map(|_| ())
}

/// Re-exported for consumer convenience — no need to add `glam` as a separate dependency.
pub use glam;

/// Re-exported for consumer convenience — matches the wgpu version used internally.
pub use wgpu;

/// Re-exported for consumer convenience — matches the iced version used internally.
pub use iced;
