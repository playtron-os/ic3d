//! High-level iced `Shader` widget that wraps `RenderPipeline3D`.
//!
//! Consumers implement [`Scene3DProgram`] with just scene setup logic.
//! The built-in Blinn-Phong shader handles lighting automatically.
//! All pipeline creation, buffer management, `Primitive`/`Pipeline`
//! trait wiring is handled internally.
//!
//! # Simple scene (built-in Blinn-Phong, no shader needed)
//!
//! ```rust,ignore
//! use ic3d::widget::{Scene3DProgram, Scene3DSetup, MeshDrawGroup};
//! use ic3d::{Mesh, Scene, PerspectiveCamera, DirectionalLight, Transform};
//!
//! #[derive(Debug)]
//! struct MyScene;
//!
//! impl Scene3DProgram for MyScene {
//!     fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
//!         let camera = PerspectiveCamera::default();
//!         let light = DirectionalLight::new(/* ... */);
//!         let scene = Scene::new(&camera).light(&light).build();
//!         let instances = vec![Transform::default().to_instance([1.0, 0.0, 0.0, 32.0])];
//!
//!         Scene3DSetup {
//!             scene,
//!             draws: vec![MeshDrawGroup::new(Mesh::cube(1.0), instances)],
//!             custom_uniforms: None,
//!         }
//!     }
//! }
//!
//! // In your view:
//! ic3d::widget::scene_3d(MyScene)
//!     .width(Length::Fill)
//!     .height(Length::Fill)
//! ```
//!
//! # Custom fragment shader (power-user)
//!
//! ```rust,ignore
//! impl Scene3DProgram for MyScene {
//!     fn fragment_shader(&self) -> &str {
//!         include_str!("my_custom_effect.wgsl")
//!     }
//!     // ...
//! }
//! ```
//!
//! # Custom uniforms at `@group(1)` (power-user)
//!
//! ```rust,ignore
//! #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
//! #[repr(C)]
//! struct MyUniforms { reveal_radius: f32, _pad: [f32; 3] }
//!
//! impl Scene3DProgram for MyScene {
//!     fn fragment_shader(&self) -> &str {
//!         include_str!("my_custom_effect.wgsl")
//!     }
//!
//!     fn custom_uniforms_size(&self) -> usize {
//!         std::mem::size_of::<MyUniforms>()
//!     }
//!
//!     fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
//!         // ...
//!         Scene3DSetup {
//!             scene,
//!             draws: vec![/* ... */],
//!             custom_uniforms: Some(bytemuck::bytes_of(&MyUniforms {
//!                 reveal_radius: 5.0,
//!                 _pad: [0.0; 3],
//!             }).to_vec()),
//!         }
//!     }
//! }
//! ```

use crate::gpu_types::InstanceData;
use crate::mesh::{Mesh, MeshBuffer};
use crate::pipeline::{PipelineConfig, RenderPipeline3D};
use crate::post_process::PostProcessPass;
use crate::scene::SceneData;
use crate::shaders::BLINN_PHONG_WGSL;
use crate::utils::compose_shader;

use iced::widget::shader::{self, Viewport};
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
pub struct MeshDrawGroup {
    /// The mesh to render.
    pub mesh: Mesh,
    /// Per-instance transform data for this mesh.
    pub instances: Vec<InstanceData>,
}

impl MeshDrawGroup {
    /// Create a draw group from a mesh and its instances.
    pub fn new(mesh: Mesh, instances: Vec<InstanceData>) -> Self {
        Self { mesh, instances }
    }
}

/// Per-frame scene setup returned by [`Scene3DProgram::setup`].
pub struct Scene3DSetup {
    /// Camera + lights + time + ambient → GPU uniforms.
    pub scene: SceneData,
    /// Meshes to draw, each with their own instances.
    pub draws: Vec<MeshDrawGroup>,
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
    /// shadow PCF) are prepended automatically via [`compose_shader`].
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

// ──────────── Internal iced wiring ────────────

/// Per-frame snapshot sent to the GPU. Created internally by [`scene_3d`].
pub struct Scene3DPrimitive {
    setup: Scene3DSetup,
    /// Type name of the program, for Debug impl.
    program_name: &'static str,
}

impl fmt::Debug for Scene3DPrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scene3DPrimitive")
            .field("program", &self.program_name)
            .field("draw_groups", &self.setup.draws.len())
            .finish()
    }
}

impl shader::Primitive for Scene3DPrimitive {
    type Pipeline = Scene3DPipeline;

    fn prepare(
        &self,
        pipeline: &mut Scene3DPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let size = viewport.physical_size();

        // Flatten all instances from all draw groups into one contiguous buffer.
        let total: usize = self.setup.draws.iter().map(|d| d.instances.len()).sum();
        let mut all_instances = Vec::with_capacity(total);
        for draw in &self.setup.draws {
            all_instances.extend_from_slice(&draw.instances);
        }

        pipeline.renderer.prepare(
            device,
            queue,
            (size.width, size.height),
            &self.setup.scene.uniforms,
            &self.setup.scene.lights,
            &all_instances,
        );

        // Upload mesh buffers if needed (first frame or mesh count changed).
        if pipeline.mesh_buffers.len() != self.setup.draws.len() {
            pipeline.mesh_buffers = self
                .setup
                .draws
                .iter()
                .map(|d| d.mesh.upload(device))
                .collect();
        }

        // Upload custom uniforms if provided.
        if let (Some(bytes), Some(buf)) =
            (&self.setup.custom_uniforms, &pipeline.custom_uniform_buffer)
        {
            queue.write_buffer(buf, 0, bytes);
        }
    }

    fn render(
        &self,
        pipeline: &Scene3DPipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let mut offset = 0_u32;
        let draws: Vec<_> = self
            .setup
            .draws
            .iter()
            .zip(&pipeline.mesh_buffers)
            .map(|(group, mesh_buf)| {
                let count = group.instances.len() as u32;
                let draw = pipeline.renderer.draw(mesh_buf, offset..offset + count);
                offset += count;
                draw
            })
            .collect();

        pipeline.renderer.render(
            encoder,
            target,
            (
                clip_bounds.x,
                clip_bounds.y,
                clip_bounds.width,
                clip_bounds.height,
            ),
            &draws,
            pipeline.custom_bind_group.as_ref(),
        );
    }
}

/// Persistent GPU resources. Created once by iced on first render.
pub struct Scene3DPipeline {
    renderer: RenderPipeline3D,
    mesh_buffers: Vec<MeshBuffer>,
    custom_uniform_buffer: Option<wgpu::Buffer>,
    custom_bind_group: Option<wgpu::BindGroup>,
}

impl shader::Pipeline for Scene3DPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader_src = SHADER_SOURCE
            .lock()
            .expect("ic3d: shader source lock")
            .take()
            .unwrap_or_default();
        let custom_size = CUSTOM_UNIFORM_SIZE
            .lock()
            .expect("ic3d: custom uniform size lock")
            .take()
            .unwrap_or(0);
        let warmup_meshes = WARMUP_MESHES
            .lock()
            .expect("ic3d: warmup meshes lock")
            .take()
            .unwrap_or_default();

        // Build custom bind group layout + buffer + bind group when custom uniforms are used.
        let (custom_uniform_buffer, custom_bind_group, custom_layout) = if custom_size > 0 {
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ic3d custom uniform layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ic3d custom uniform buffer"),
                size: custom_size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ic3d custom bind group"),
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

            (Some(buffer), Some(bind_group), Some(layout))
        } else {
            (None, None, None)
        };

        // Build pipeline config, injecting the custom layout if present.
        let mut config = PIPELINE_CONFIG
            .lock()
            .expect("ic3d: pipeline config lock")
            .take()
            .unwrap_or_default();
        if let Some(ref layout) = custom_layout {
            config.custom_bind_group_layout = Some(layout);
        }

        let mut renderer = RenderPipeline3D::new(device, format, &shader_src, config);

        // Register post-process passes if a factory was provided.
        if let Some(factory) = POST_PROCESS_FACTORY
            .lock()
            .expect("ic3d: post process lock")
            .take()
        {
            for pass in factory(device, queue) {
                renderer.add_post_process(pass);
            }
        }

        // Warmup if meshes were provided.
        if !warmup_meshes.is_empty() {
            let mesh_buffers: Vec<MeshBuffer> =
                warmup_meshes.iter().map(|m| m.upload(device)).collect();
            let vbs: Vec<&wgpu::Buffer> = mesh_buffers.iter().map(|mb| mb.buffer()).collect();
            renderer.warmup(device, queue, &vbs, custom_bind_group.as_ref());
        }

        Self {
            renderer,
            mesh_buffers: Vec::new(),
            custom_uniform_buffer,
            custom_bind_group,
        }
    }
}

// Static cells for passing program data to Pipeline::new().
// iced's Pipeline::new() takes no user state — only device/queue/format.
// We stash values before the Shader widget renders, then Pipeline::new() picks them up.
use std::sync::Mutex;
static SHADER_SOURCE: Mutex<Option<String>> = Mutex::new(None);
static PIPELINE_CONFIG: Mutex<Option<PipelineConfig<'static>>> = Mutex::new(None);
static CUSTOM_UNIFORM_SIZE: Mutex<Option<usize>> = Mutex::new(None);
static WARMUP_MESHES: Mutex<Option<Vec<Mesh>>> = Mutex::new(None);
static POST_PROCESS_FACTORY: Mutex<Option<PostProcessFactory>> = Mutex::new(None);

/// Create a [`Shader`](iced::widget::Shader) widget that renders a 3D scene.
///
/// This is the main entry point. Implement [`Scene3DProgram`] and pass it here.
///
/// ```rust,ignore
/// ic3d::widget::scene_3d(MyScene)
///     .width(Length::Fill)
///     .height(Length::Fill)
/// ```
#[must_use]
pub fn scene_3d<Message: 'static>(
    program: impl Scene3DProgram + 'static,
) -> iced::widget::Shader<Message, Scene3DWidget> {
    // Stash data for Pipeline::new() to pick up.
    *SHADER_SOURCE.lock().expect("ic3d: shader source lock") =
        Some(compose_shader(program.fragment_shader()));
    *PIPELINE_CONFIG.lock().expect("ic3d: pipeline config lock") = Some(program.pipeline_config());
    *CUSTOM_UNIFORM_SIZE
        .lock()
        .expect("ic3d: custom uniform size lock") = {
        let size = program.custom_uniforms_size();
        if size > 0 {
            Some(size)
        } else {
            None
        }
    };

    let warmup = program.warmup_meshes();
    *WARMUP_MESHES.lock().expect("ic3d: warmup meshes lock") = if warmup.is_empty() {
        None
    } else {
        Some(warmup)
    };

    // Stash post-process factory if the program provides one.
    *POST_PROCESS_FACTORY
        .lock()
        .expect("ic3d: post process lock") = program.post_process_factory();

    iced::widget::Shader::new(Scene3DWidget {
        program: Box::new(program),
    })
}

/// The iced `Program` wrapper. Not constructed directly — use [`scene_3d()`].
pub struct Scene3DWidget {
    program: Box<dyn Scene3DProgram>,
}

impl<Message: 'static> shader::Program<Message> for Scene3DWidget {
    type State = ();
    type Primitive = Scene3DPrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        let setup = self.program.setup(bounds);
        Scene3DPrimitive {
            setup,
            program_name: std::any::type_name_of_val(&*self.program),
        }
    }
}
