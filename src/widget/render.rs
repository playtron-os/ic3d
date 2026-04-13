//! Internal GPU pipeline and primitive types for the 3D scene widget.
//!
//! These implement iced's `shader::Primitive` and `shader::Pipeline` traits.
//! Not part of the public API — consumers use [`Scene3DProgram`](super::Scene3DProgram).

use super::types::MeshDrawGroup;
use crate::mesh::{Mesh, MeshBuffer};
use crate::pipeline::render_pipeline::{PipelineConfig, RenderPipeline3D};
use crate::pipeline::shaders::BLINN_PHONG_WGSL;
use crate::pipeline::utils::compose_shader;
use crate::scene::builder::SceneData;
use iced::widget::shader::{self, Viewport};
use iced::Rectangle;
use std::fmt;

use parking_lot::Mutex;

use super::types::PostProcessFactory;

// ──────────── Static cells ────────────

// Static cells for passing program data to Pipeline::new().
// iced's Pipeline::new() takes no user state — only device/queue/format.
// We stash values before the Shader widget renders, then Pipeline::new() picks them up.
pub(super) static SHADER_SOURCE: Mutex<Option<String>> = Mutex::new(None);
pub(super) static PIPELINE_CONFIG: Mutex<Option<PipelineConfig<'static>>> = Mutex::new(None);
pub(super) static CUSTOM_UNIFORM_SIZE: Mutex<Option<usize>> = Mutex::new(None);
pub(super) static WARMUP_MESHES: Mutex<Option<Vec<Mesh>>> = Mutex::new(None);
pub(super) static POST_PROCESS_FACTORY: Mutex<Option<PostProcessFactory>> = Mutex::new(None);

// ──────────── Scene3DPrimitive ────────────

/// Per-frame snapshot sent to the GPU. Created internally by [`scene_3d`](super::scene_3d).
pub(crate) struct Scene3DPrimitive {
    pub(super) scene: SceneData,
    pub(super) draws: Vec<MeshDrawGroup>,
    /// Overlay groups resolved from `Overlay::draw()` during widget `draw()`.
    pub(super) overlay_groups: Vec<MeshDrawGroup>,
    pub(super) custom_uniforms: Option<Vec<u8>>,
    /// Pipeline clear color for this frame.
    pub(super) clear_color: wgpu::Color,
    /// Type name of the program, for Debug impl.
    pub(super) program_name: &'static str,
}

impl fmt::Debug for Scene3DPrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scene3DPrimitive")
            .field("program", &self.program_name)
            .field("draw_groups", &self.draws.len())
            .field("overlay_groups", &self.overlay_groups.len())
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
        let total: usize = self.draws.iter().map(|d| d.instances.len()).sum();
        let mut all_instances = Vec::with_capacity(total);
        for draw in &self.draws {
            all_instances.extend_from_slice(&draw.instances);
        }

        pipeline.renderer.set_clear_color(self.clear_color);
        pipeline.renderer.prepare(
            device,
            queue,
            (size.width, size.height),
            &self.scene.uniforms,
            &self.scene.lights,
            &all_instances,
        );

        // Upload mesh buffers if needed (first frame or mesh count changed).
        if pipeline.mesh_buffers.len() != self.draws.len() {
            pipeline.mesh_buffers = self.draws.iter().map(|d| d.mesh.upload(device)).collect();
        }

        // Upload custom uniforms if provided.
        if let (Some(bytes), Some(buf)) = (&self.custom_uniforms, &pipeline.custom_uniform_buffer) {
            queue.write_buffer(buf, 0, bytes);
        }

        // Upload overlay mesh buffers if needed.
        if pipeline.overlay_mesh_buffers.len() != self.overlay_groups.len() {
            pipeline.overlay_mesh_buffers = self
                .overlay_groups
                .iter()
                .map(|d| d.mesh.upload(device))
                .collect();
        }

        // Flatten overlay instances and upload.
        if !self.overlay_groups.is_empty() {
            let total: usize = self.overlay_groups.iter().map(|d| d.instances.len()).sum();
            let mut overlay_instances = Vec::with_capacity(total);
            for draw in &self.overlay_groups {
                overlay_instances.extend_from_slice(&draw.instances);
            }
            pipeline
                .renderer
                .prepare_overlay(device, queue, &overlay_instances);
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

        // Overlay pass (no depth test, no shadows)
        if !self.overlay_groups.is_empty() {
            let mut overlay_offset = 0_u32;
            let overlay_draws: Vec<_> = self
                .overlay_groups
                .iter()
                .zip(&pipeline.overlay_mesh_buffers)
                .map(|(group, mesh_buf)| {
                    let count = group.instances.len() as u32;
                    let draw = pipeline
                        .renderer
                        .draw_overlay(mesh_buf, overlay_offset..overlay_offset + count);
                    overlay_offset += count;
                    draw
                })
                .collect();

            pipeline.renderer.render_overlay(
                encoder,
                target,
                (
                    clip_bounds.x,
                    clip_bounds.y,
                    clip_bounds.width,
                    clip_bounds.height,
                ),
                &overlay_draws,
                pipeline.custom_bind_group.as_ref(),
            );
        }
    }
}

// ──────────── Scene3DPipeline ────────────

/// Persistent GPU resources. Created once by iced on first render.
pub(crate) struct Scene3DPipeline {
    pub(super) renderer: RenderPipeline3D,
    pub(super) mesh_buffers: Vec<MeshBuffer>,
    pub(super) overlay_mesh_buffers: Vec<MeshBuffer>,
    pub(super) custom_uniform_buffer: Option<wgpu::Buffer>,
    pub(super) custom_bind_group: Option<wgpu::BindGroup>,
}

impl shader::Pipeline for Scene3DPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader_src = SHADER_SOURCE
            .lock()
            .take()
            .unwrap_or_else(|| compose_shader(BLINN_PHONG_WGSL));
        let custom_size = CUSTOM_UNIFORM_SIZE.lock().take().unwrap_or(0);
        let warmup_meshes = WARMUP_MESHES.lock().take().unwrap_or_default();

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
        let mut config = PIPELINE_CONFIG.lock().take().unwrap_or_default();
        if let Some(ref layout) = custom_layout {
            config.custom_bind_group_layout = Some(layout);
        }

        let mut renderer = RenderPipeline3D::new(device, format, &shader_src, config);

        // Register post-process passes if a factory was provided.
        if let Some(factory) = POST_PROCESS_FACTORY.lock().take() {
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
            overlay_mesh_buffers: Vec::new(),
            custom_uniform_buffer,
            custom_bind_group,
        }
    }
}
