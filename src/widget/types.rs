//! Public API types for the 3D scene widget.
//!
//! These types are the consumer-facing interface: [`Scene3DProgram`] defines
//! a scene, [`Scene3DSetup`] carries per-frame data, [`MeshDrawGroup`]
//! bundles a mesh with its instances.

use crate::mesh::Mesh;
use crate::overlay::base::Overlay;
use crate::pipeline::gpu_types::InstanceData;
use crate::pipeline::post_process::PostProcessPass;
use crate::pipeline::render_pipeline::PipelineConfig;
use crate::pipeline::shaders::BLINN_PHONG_WGSL;
use crate::scene::builder::SceneData;
use crate::scene::object::SceneObjectId;
use iced::Rectangle;
use std::fmt;

/// Factory closure that creates post-processing passes at pipeline init time.
///
/// Returned by [`Scene3DProgram::post_process_factory`]. Receives `device` and
/// `queue` so passes can allocate GPU resources.
pub type PostProcessFactory = Box<
    dyn FnOnce(&wgpu::Device, &wgpu::Queue) -> Vec<Box<dyn PostProcessPass + Send + Sync>> + Send,
>;

/// One mesh + its per-instance transforms for a single draw group.
///
/// Each `MeshDrawGroup` becomes one draw call. Multiple groups allow
/// rendering different meshes with different instance sets.
///
/// Optionally assign an [`id`](Self::with_id) so overlays and gizmos
/// can reference this object by [`SceneObjectId`].
pub struct MeshDrawGroup {
    /// Optional identifier for scene object tracking.
    ///
    /// When set, the first instance's model matrix is stored in
    /// [`SceneContext::objects`](crate::scene::context::SceneContext::objects) so overlays can look up the object's
    /// transform (e.g. a gizmo following a mesh).
    pub id: Option<SceneObjectId>,
    /// The mesh to render.
    pub mesh: Mesh,
    /// Per-instance transform data for this mesh.
    pub instances: Vec<InstanceData>,
}

impl MeshDrawGroup {
    /// Create a draw group from a mesh and its instances.
    pub fn new(mesh: Mesh, instances: Vec<InstanceData>) -> Self {
        Self {
            id: None,
            mesh,
            instances,
        }
    }

    /// Assign a [`SceneObjectId`] to this draw group.
    ///
    /// The widget stores the first instance's model matrix in
    /// [`SceneContext`](crate::scene::context::SceneContext) so overlays can reference this object.
    #[must_use]
    pub fn with_id(mut self, id: SceneObjectId) -> Self {
        self.id = Some(id);
        self
    }
}

/// Per-frame scene setup returned by [`Scene3DProgram::setup`].
pub struct Scene3DSetup {
    /// Camera + lights + time + ambient → GPU uniforms.
    pub scene: SceneData,
    /// Meshes to draw, each with their own instances.
    pub draws: Vec<MeshDrawGroup>,
    /// Overlay objects rendered on top of the scene (no depth testing, no shadows).
    ///
    /// Overlays compute their own draw groups from scene context — no manual
    /// camera parameter plumbing needed. Use for gizmos, guides, and other
    /// always-visible elements.
    ///
    /// ```rust,ignore
    /// overlays: vec![Box::new(gizmo.clone())],
    /// ```
    pub overlays: Vec<Box<dyn Overlay>>,
    /// Optional raw bytes for the `@group(1) @binding(0)` custom uniform buffer.
    ///
    /// If your fragment shader uses `@group(1)`, return `Some(bytes)` here.
    /// The bytes must match your WGSL struct exactly (use `bytemuck::bytes_of`).
    /// Set to `None` if you don't use custom uniforms.
    pub custom_uniforms: Option<Vec<u8>>,
}

/// Implement this trait to define a 3D scene.
///
/// Only [`setup`](Self::setup) is required. Everything else has sensible defaults:
/// - Built-in Blinn-Phong fragment shader (override via [`fragment_shader`](Self::fragment_shader))
/// - Default pipeline config: 2048 shadow map, 4× MSAA
/// - No custom uniforms
///
/// All pipeline creation, GPU buffer management, shader composition,
/// `Primitive`/`Pipeline` trait implementations are handled by ic3d.
pub trait Scene3DProgram: fmt::Debug {
    /// Return the WGSL fragment shader source.
    ///
    /// Engine preludes (scene uniforms, vertex IO, standard vertex shader,
    /// shadow PCF) are prepended automatically via [`compose_shader`](crate::compose_shader).
    /// Just return your `@fragment fn fs_main(...)` and any helpers.
    ///
    /// Default: [`BLINN_PHONG_WGSL`] — standard Blinn-Phong with multi-light
    /// and shadow support. Material `vec4` is `(r, g, b, shininess)`.
    /// Override only if you need custom effects.
    fn fragment_shader(&self) -> &str {
        BLINN_PHONG_WGSL
    }

    /// Build the scene for this frame.
    ///
    /// Called every frame with the current widget bounds.
    /// Return the camera/light setup, mesh draw groups, and optional custom uniforms.
    fn setup(&self, bounds: Rectangle) -> Scene3DSetup;

    /// Override pipeline configuration (shadow map size, MSAA, etc.).
    ///
    /// Default: 2048 shadow map, 4× MSAA, no custom bind group.
    /// Note: if [`custom_uniforms_size`](Self::custom_uniforms_size) > 0,
    /// the custom bind group layout is created automatically — you do NOT
    /// need to set `custom_bind_group_layout` in the config.
    fn pipeline_config(&self) -> PipelineConfig<'static> {
        PipelineConfig::default()
    }

    /// Size in bytes of the custom uniform struct at `@group(1) @binding(0)`.
    ///
    /// Return 0 (default) if your shader doesn't use custom uniforms.
    /// If > 0, a uniform buffer and bind group are created automatically.
    fn custom_uniforms_size(&self) -> usize {
        0
    }

    /// Vertex buffers to use for GPU shader warmup.
    ///
    /// If provided, `warmup()` is called at pipeline creation to force shader
    /// compilation and avoid first-frame stalls on NVIDIA GPUs. Default: empty
    /// (no warmup). Return the meshes that will be drawn.
    fn warmup_meshes(&self) -> Vec<Mesh> {
        Vec::new()
    }

    /// Create post-processing passes to apply after the main render.
    ///
    /// Called once during pipeline creation with the GPU device and queue.
    /// Passes execute in order, ping-ponging between intermediate textures.
    /// Default: no post-processing.
    ///
    /// ```rust,ignore
    /// fn post_process_factory(&self) -> Option<PostProcessFactory> {
    ///     Some(Box::new(|device, _queue| {
    ///         vec![Box::new(MyBloomPass::new(device))]
    ///     }))
    /// }
    /// ```
    fn post_process_factory(&self) -> Option<PostProcessFactory> {
        None
    }
}
