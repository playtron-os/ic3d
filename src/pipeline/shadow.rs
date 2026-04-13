//! Shadow map pass — depth-only render from the light's perspective.

use crate::pipeline::gpu_types::{InstanceData, Vertex};

/// Shadow map resolution and GPU resources.
pub struct ShadowPass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    depth_view: wgpu::TextureView,
    /// The shadow map texture view, bindable as a texture in the main pass.
    pub shadow_texture_view: wgpu::TextureView,
    pub shadow_sampler: wgpu::Sampler,
    pub size: u32,
}

impl ShadowPass {
    /// Create the shadow pass with the given map resolution.
    ///
    /// `light_buffer` is the storage buffer containing `GpuLight` array —
    /// the shadow vertex shader reads `lights[0].shadow_projection`.
    pub fn new(device: &wgpu::Device, light_buffer: &wgpu::Buffer, size: u32) -> Self {
        // Shadow map depth texture
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ic3d shadow map"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_texture_view =
            shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Depth-only render target (same texture)
        let depth_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Comparison sampler for PCF
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ic3d shadow sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // Bind group: light storage buffer (vertex stage only)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ic3d shadow bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ic3d shadow bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ic3d shadow pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ic3d shadow shader"),
            source: wgpu::ShaderSource::Wgsl(crate::pipeline::shaders::SHADOW_WGSL.into()),
        });

        // Depth-only pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ic3d shadow pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_shadow"),
                buffers: &[Vertex::layout(), InstanceData::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                // No culling — all faces write depth so thin geometry
                // (caps, planes) is always represented in the shadow map.
                // Self-shadow acne is handled by depth bias + normal offset.
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                // Hardware depth bias pushes stored depth slightly away from
                // the light, preventing self-shadow acne on front faces.
                // Slope bias adapts to grazing-angle surfaces automatically.
                bias: wgpu::DepthBiasState {
                    constant: 4,
                    slope_scale: 4.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: None, // depth-only
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            depth_view,
            shadow_texture_view,
            shadow_sampler,
            size,
        }
    }

    /// Record the shadow depth pass.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, draws: &[DrawCall<'_>]) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ic3d shadow pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
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

        pass.set_viewport(0.0, 0.0, self.size as f32, self.size as f32, 0.0, 1.0);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        for draw in draws {
            pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, draw.instance_buffer.slice(..));
            pass.draw(0..draw.vertex_count, draw.instance_range.clone());
        }
    }
}

/// A single instanced draw call.
pub struct DrawCall<'a> {
    pub vertex_buffer: &'a wgpu::Buffer,
    pub instance_buffer: &'a wgpu::Buffer,
    pub vertex_count: u32,
    pub instance_range: std::ops::Range<u32>,
}
