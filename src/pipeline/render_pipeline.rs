//! Main render pipeline — renders instanced 3D geometry with shadow mapping.

use crate::mesh::MeshBuffer;
use crate::pipeline::buffer::{BufferPool, DynBuffer};
use crate::pipeline::gpu_types::{GpuLight, InstanceData, SceneUniforms, Vertex, MAX_LIGHTS};
use crate::pipeline::post_process::PostProcessPass;
use crate::pipeline::shadow::{DrawCall, ShadowPass};
use crate::pipeline::utils::compose_overlay_shader;
use crate::scene::builder::SceneData;
use bytemuck::Zeroable;
use wgpu::util::DeviceExt;

/// Pipeline configuration: shadow map size, MSAA, and custom bind group.
pub struct PipelineConfig<'a> {
    /// Shadow map resolution (default: 2048). 0 disables shadows.
    pub shadow_map_size: u32,
    /// MSAA sample count (default: 4). Must be 1 or 4.
    pub msaa_samples: u32,
    /// Optional consumer bind group layout at `@group(1)`.
    pub custom_bind_group_layout: Option<&'a wgpu::BindGroupLayout>,
}

impl Default for PipelineConfig<'_> {
    fn default() -> Self {
        Self {
            shadow_map_size: 2048,
            msaa_samples: 4,
            custom_bind_group_layout: None,
        }
    }
}

impl<'a> PipelineConfig<'a> {
    #[must_use]
    pub fn shadow_map_size(mut self, size: u32) -> Self {
        self.shadow_map_size = size;
        self
    }

    #[must_use]
    pub fn msaa_samples(mut self, count: u32) -> Self {
        self.msaa_samples = count;
        self
    }

    #[must_use]
    pub fn custom_bind_group_layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.custom_bind_group_layout = Some(layout);
        self
    }
}

/// 3D render pipeline: shadow pass + MSAA main pass + optional post-processing.
///
/// Created once, updated per-frame via [`prepare`](Self::prepare).
pub struct RenderPipeline3D {
    // Main render
    main_pipeline: wgpu::RenderPipeline,
    // Overlay render (depth always passes, no depth write)
    overlay_pipeline: wgpu::RenderPipeline,
    // Kept alive — layout is referenced by the bind group
    _main_bind_group_layout: wgpu::BindGroupLayout,
    main_bind_group: wgpu::BindGroup,

    // Shared buffers
    uniform_buffer: wgpu::Buffer,
    light_buffer: wgpu::Buffer,
    instance_buffer: DynBuffer,
    overlay_instance_buffer: DynBuffer,

    // Shadow
    shadow: ShadowPass,

    // Buffer pool: frame-delayed recycling for GPU buffers
    buffer_pool: BufferPool,

    // Post-processing chain (applied in order after main render)
    post_process_passes: Vec<Box<dyn PostProcessPass + Send + Sync>>,
    pp_views: [Option<wgpu::TextureView>; 2],
    pp_size: (u32, u32),

    // MSAA render targets
    msaa_color_view: wgpu::TextureView,
    msaa_depth_view: wgpu::TextureView,
    msaa_size: (u32, u32),
    msaa_samples: u32,
    output_format: wgpu::TextureFormat,
    clear_color: wgpu::Color,
}

impl RenderPipeline3D {
    /// Create the pipeline with the given WGSL shader source and config.
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        main_shader_wgsl: &str,
        config: PipelineConfig<'_>,
    ) -> Self {
        let msaa_samples = config.msaa_samples.max(1);

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ic3d scene uniforms"),
            size: std::mem::size_of::<SceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Light storage buffer (fixed max size, zero-initialized)
        let light_buffer_size = (std::mem::size_of::<GpuLight>() * MAX_LIGHTS) as u64;
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ic3d lights"),
            size: light_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        // Zero all bytes so uninitialized light slots are safe to read
        light_buffer.slice(..).get_mapped_range_mut().fill(0);
        light_buffer.unmap();

        let instance_buffer = DynBuffer::new(
            device,
            "ic3d instances",
            std::mem::size_of::<InstanceData>() as u64 * 512,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        let overlay_instance_buffer = DynBuffer::new(
            device,
            "ic3d overlay instances",
            std::mem::size_of::<InstanceData>() as u64 * 64,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        // Shadow pass (reads light[0].shadow_projection from light buffer)
        let shadow = ShadowPass::new(device, &light_buffer, config.shadow_map_size);

        let (msaa_color_view, msaa_depth_view) =
            create_msaa_textures(device, output_format, 1, 1, msaa_samples);

        // Scene bind group layout (group 0): uniforms + lights + shadow map + sampler
        let main_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ic3d scene bind group layout"),
                entries: &[
                    // binding 0: SceneUniforms (uniform buffer)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 1: GpuLight[] (storage buffer, read-only)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 2: shadow map depth texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // binding 3: shadow comparison sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        let main_bind_group = Self::create_main_bind_group(
            device,
            &main_bind_group_layout,
            &uniform_buffer,
            &light_buffer,
            &shadow.shadow_texture_view,
            &shadow.shadow_sampler,
        );

        // Pipeline layout: group 0 (scene) + optional group 1 (custom)
        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> =
            if let Some(custom) = config.custom_bind_group_layout {
                vec![&main_bind_group_layout, custom]
            } else {
                vec![&main_bind_group_layout]
            };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ic3d main pipeline layout"),
            bind_group_layouts: &bind_group_layouts,
            immediate_size: 0,
        });

        let main_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ic3d main shader"),
            source: wgpu::ShaderSource::Wgsl(main_shader_wgsl.into()),
        });

        let main_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ic3d main pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &main_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout(), InstanceData::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &main_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        // Overlay pipeline — flat (unlit) shader, depth always passes (no depth write).
        // Used for gizmos, grid lines, and other helpers that render on top of geometry.
        let overlay_shader_src = compose_overlay_shader();
        let overlay_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ic3d overlay shader"),
            source: wgpu::ShaderSource::Wgsl(overlay_shader_src.into()),
        });

        // Overlay pipeline layout: group 0 only (scene uniforms + lights) — no custom bind group.
        // The lit overlay shader reads lights but does not sample shadows or need group 1.
        let overlay_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ic3d overlay pipeline layout"),
                bind_group_layouts: &[&main_bind_group_layout],
                immediate_size: 0,
            });

        let overlay_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ic3d overlay pipeline"),
            layout: Some(&overlay_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &overlay_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout(), InstanceData::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &overlay_shader,
                entry_point: Some("fs_main_flat"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Max,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            main_pipeline,
            overlay_pipeline,
            _main_bind_group_layout: main_bind_group_layout,
            main_bind_group,
            uniform_buffer,
            light_buffer,
            instance_buffer,
            overlay_instance_buffer,
            shadow,
            buffer_pool: BufferPool::new(),
            post_process_passes: Vec::new(),
            pp_views: [None, None],
            pp_size: (0, 0),
            msaa_color_view,
            msaa_depth_view,
            msaa_size: (1, 1),
            msaa_samples,
            output_format,
            clear_color: wgpu::Color::BLACK,
        }
    }

    /// Set the background clear color (default: opaque black).
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.clear_color = color;
    }

    fn create_main_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform_buffer: &wgpu::Buffer,
        light_buffer: &wgpu::Buffer,
        shadow_view: &wgpu::TextureView,
        shadow_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ic3d main bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(shadow_sampler),
                },
            ],
        })
    }

    /// The raw instance buffer for [`DrawCall`] construction.
    #[must_use]
    pub fn instance_buffer(&self) -> &wgpu::Buffer {
        self.instance_buffer.raw()
    }

    /// Build a [`DrawCall`] from an uploaded mesh and instance range.
    ///
    /// Automatically uses the pipeline's internal instance buffer, so consumers
    /// don't need to track it. Use with [`render()`](Self::render).
    ///
    /// ```rust,ignore
    /// let cube = Mesh::cube(1.0).upload(device);
    /// // in render():
    /// pipeline.render(encoder, target, bounds, &[
    ///     pipeline.draw(&cube, 0..100),
    /// ], None);
    /// ```
    #[must_use]
    pub fn draw<'a>(
        &'a self,
        mesh: &'a MeshBuffer,
        instance_range: std::ops::Range<u32>,
    ) -> DrawCall<'a> {
        DrawCall {
            vertex_buffer: mesh.buffer(),
            instance_buffer: self.instance_buffer.raw(),
            vertex_count: mesh.vertex_count(),
            instance_range,
        }
    }

    /// Add a post-processing pass to the chain.
    ///
    /// Passes execute in order after the main render, each reading the
    /// previous output and writing to the next (or final) target.
    pub fn add_post_process(&mut self, pass: Box<dyn PostProcessPass + Send + Sync>) {
        self.post_process_passes.push(pass);
    }

    /// Upload scene uniforms, lights, and instance data for this frame.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: (u32, u32),
        uniforms: &SceneUniforms,
        lights: &[GpuLight],
        instances: &[InstanceData],
    ) {
        // Advance buffer pool — recycle buffers the GPU is done with
        self.buffer_pool.advance_frame();

        // Resize MSAA textures if target size changed
        if self.msaa_size != target_size {
            let (color_view, depth_view) = create_msaa_textures(
                device,
                self.output_format,
                target_size.0,
                target_size.1,
                self.msaa_samples,
            );
            self.msaa_color_view = color_view;
            self.msaa_depth_view = depth_view;
            self.msaa_size = target_size;
        }

        // Upload uniforms
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));

        // Upload lights
        if !lights.is_empty() {
            queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(lights));
        }

        // Upload instances (pool-recycled growth)
        let needed = std::mem::size_of_val(instances) as u64;
        self.instance_buffer
            .ensure_capacity(device, &mut self.buffer_pool, needed);
        queue.write_buffer(
            self.instance_buffer.raw(),
            0,
            bytemuck::cast_slice(instances),
        );

        // Resize post-process ping-pong textures if needed
        if !self.post_process_passes.is_empty() && self.pp_size != target_size {
            let labels = ["ic3d post-process A", "ic3d post-process B"];
            for (i, label) in labels.iter().enumerate() {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: target_size.0.max(1),
                        height: target_size.1.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.output_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                self.pp_views[i] = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
            }
            self.pp_size = target_size;
        }

        // Prepare post-process passes
        for pp in &self.post_process_passes {
            pp.prepare(device, queue, target_size);
        }
    }

    /// Convenience: upload from [`SceneData`] (produced by [`Scene::build()`](crate::Scene::build)).
    pub fn prepare_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: (u32, u32),
        scene: &SceneData,
        instances: &[InstanceData],
    ) {
        self.prepare(
            device,
            queue,
            target_size,
            &scene.uniforms,
            &scene.lights,
            instances,
        );
    }

    /// Render: shadow pass → main pass → post-processing chain.
    ///
    /// If post-processing passes are registered, the main pass resolves to an
    /// intermediate texture. Each pass reads the previous output and writes to
    /// the next, with the final pass writing to `target`.
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: (u32, u32, u32, u32), // x, y, w, h
        draws: &[DrawCall<'_>],
        custom_bind_group: Option<&wgpu::BindGroup>,
    ) {
        // Pass 1: Shadow map
        self.shadow.render(encoder, draws);

        let has_post = !self.post_process_passes.is_empty();

        let resolve_target = if has_post {
            self.pp_views[0]
                .as_ref()
                .expect("ic3d: post-process textures not prepared")
        } else {
            target
        };

        // Pass 2: Main render (MSAA → resolved, or direct when samples == 1)
        {
            let use_msaa = self.msaa_samples > 1;
            let (view, resolve) = if use_msaa {
                (&self.msaa_color_view, Some(resolve_target))
            } else {
                (resolve_target, None)
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ic3d main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: resolve,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.msaa_depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let (x, y, w, h) = clip_bounds;
            pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
            pass.set_pipeline(&self.main_pipeline);
            pass.set_bind_group(0, &self.main_bind_group, &[]);

            if let Some(cbg) = custom_bind_group {
                pass.set_bind_group(1, cbg, &[]);
            }

            for draw in draws {
                pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, draw.instance_buffer.slice(..));
                pass.draw(0..draw.vertex_count, draw.instance_range.clone());
            }
        }

        // Pass 3+: Post-processing chain (ping-pong between cached textures)
        if has_post {
            let num_passes = self.post_process_passes.len();

            for (i, pp) in self.post_process_passes.iter().enumerate() {
                let source = self.pp_views[i % 2].as_ref().unwrap();
                let dest = if i == num_passes - 1 {
                    target
                } else {
                    self.pp_views[(i + 1) % 2].as_ref().unwrap()
                };
                pp.render(encoder, source, dest);
            }
        }
    }

    /// Upload overlay instance data for this frame.
    ///
    /// Overlays render on top of the scene with no depth testing (always visible).
    /// Call this after [`prepare()`](Self::prepare) and before [`render_overlay()`](Self::render_overlay).
    pub fn prepare_overlay(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[InstanceData],
    ) {
        let needed = std::mem::size_of_val(instances) as u64;
        self.overlay_instance_buffer
            .ensure_capacity(device, &mut self.buffer_pool, needed);
        queue.write_buffer(
            self.overlay_instance_buffer.raw(),
            0,
            bytemuck::cast_slice(instances),
        );
    }

    /// Build a [`DrawCall`] that uses the overlay instance buffer.
    ///
    /// Use with [`render_overlay()`](Self::render_overlay).
    #[must_use]
    pub fn draw_overlay<'a>(
        &'a self,
        mesh: &'a MeshBuffer,
        instance_range: std::ops::Range<u32>,
    ) -> DrawCall<'a> {
        DrawCall {
            vertex_buffer: mesh.buffer(),
            instance_buffer: self.overlay_instance_buffer.raw(),
            vertex_count: mesh.vertex_count(),
            instance_range,
        }
    }

    /// Render overlays on top of the already-rendered scene.
    ///
    /// Uses the overlay pipeline (depth compare = Always, no depth writes)
    /// so overlays are never occluded by scene geometry. No shadow pass is
    /// performed for overlays.
    ///
    /// Must be called **after** [`render()`](Self::render).
    pub fn render_overlay(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: (u32, u32, u32, u32),
        draws: &[DrawCall<'_>],
        custom_bind_group: Option<&wgpu::BindGroup>,
    ) {
        if draws.is_empty() {
            return;
        }

        let use_msaa = self.msaa_samples > 1;
        let (view, resolve): (&wgpu::TextureView, Option<&wgpu::TextureView>) = if use_msaa {
            (&self.msaa_color_view, Some(target))
        } else {
            (target, None)
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ic3d overlay pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: resolve,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.msaa_depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let (x, y, w, h) = clip_bounds;
        pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
        pass.set_pipeline(&self.overlay_pipeline);
        pass.set_bind_group(0, &self.main_bind_group, &[]);

        if let Some(cbg) = custom_bind_group {
            pass.set_bind_group(1, cbg, &[]);
        }

        for draw in draws {
            pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, draw.instance_buffer.slice(..));
            pass.draw(0..draw.vertex_count, draw.instance_range.clone());
        }
    }

    /// Force GPU shader compilation by issuing a real draw to a tiny offscreen target.
    ///
    /// Prevents the multi-second stall on first visible frame caused by NVIDIA's
    /// deferred compilation. Pass the custom bind group if the pipeline uses one.
    pub fn warmup(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertex_buffers: &[&wgpu::Buffer],
        custom_bind_group: Option<&wgpu::BindGroup>,
    ) {
        // Tiny offscreen targets
        let warmup_size = 8_u32;
        let (color_view, depth_view) = create_msaa_textures(
            device,
            self.output_format,
            warmup_size,
            warmup_size,
            self.msaa_samples,
        );
        let resolve_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ic3d warmup resolve"),
            size: wgpu::Extent3d {
                width: warmup_size,
                height: warmup_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.output_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let resolve_view = resolve_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Upload a single dummy instance + zeroed uniforms + dummy light
        let dummy_instance = InstanceData::zeroed();
        let dummy_uniforms = SceneUniforms::zeroed();
        let dummy_light = GpuLight::zeroed();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&dummy_uniforms));
        queue.write_buffer(&self.light_buffer, 0, bytemuck::bytes_of(&dummy_light));

        let dummy_instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ic3d warmup instance"),
            contents: bytemuck::bytes_of(&dummy_instance),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Build draw calls for each vertex buffer
        let draws: Vec<DrawCall<'_>> = vertex_buffers
            .iter()
            .map(|vb| DrawCall {
                vertex_buffer: vb,
                instance_buffer: &dummy_instance_buf,
                vertex_count: 3, // minimum triangle
                instance_range: 0..1,
            })
            .collect();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ic3d warmup encoder"),
        });

        // Shadow pass — forces shadow pipeline compilation
        self.shadow.render(&mut encoder, &draws);

        // Main pass — forces main pipeline + fragment shader compilation
        {
            let use_msaa = self.msaa_samples > 1;
            let (view, resolve): (&wgpu::TextureView, Option<&wgpu::TextureView>) = if use_msaa {
                (&color_view, Some(&resolve_view))
            } else {
                (&resolve_view, None)
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ic3d warmup main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: resolve,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_viewport(0.0, 0.0, warmup_size as f32, warmup_size as f32, 0.0, 1.0);
            pass.set_pipeline(&self.main_pipeline);
            pass.set_bind_group(0, &self.main_bind_group, &[]);

            if let Some(cbg) = custom_bind_group {
                pass.set_bind_group(1, cbg, &[]);
            }

            for draw in &draws {
                pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
                pass.set_vertex_buffer(1, draw.instance_buffer.slice(..));
                pass.draw(0..draw.vertex_count, draw.instance_range.clone());
            }
        }

        // Submit and wait for shader compilation
        let idx = queue.submit(std::iter::once(encoder.finish()));
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: Some(idx),
            timeout: Some(std::time::Duration::from_secs(5)),
        });
    }
}

/// Create multisampled color and depth textures for MSAA rendering.
fn create_msaa_textures(
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
    w: u32,
    h: u32,
    sample_count: u32,
) -> (wgpu::TextureView, wgpu::TextureView) {
    let extent = wgpu::Extent3d {
        width: w.max(1),
        height: h.max(1),
        depth_or_array_layers: 1,
    };

    let color_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ic3d MSAA color"),
        size: extent,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: color_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let depth_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ic3d MSAA depth"),
        size: extent,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());
    (color_view, depth_view)
}
