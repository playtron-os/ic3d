//! Helper for `@group(1) @binding(0)` custom uniform buffers.

/// GPU buffer + bind group for consumer-defined uniforms at `@group(1) @binding(0)`.
///
/// Encapsulates the bind group layout, buffer, and bind group creation that
/// every custom-uniform consumer would otherwise duplicate.
///
/// ```ignore
/// let custom = CustomUniformBuffer::new(device, std::mem::size_of::<MyUniforms>());
/// let config = PipelineConfig { custom_bind_group_layout: Some(custom.layout()), .. };
/// // per frame:
/// custom.write(queue, bytemuck::bytes_of(&my_uniforms));
/// pipeline.render(encoder, target, bounds, &draws, Some(custom.bind_group()));
/// ```
pub struct CustomUniformBuffer {
    layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl CustomUniformBuffer {
    /// Create a new custom uniform buffer of `size` bytes.
    pub fn new(device: &wgpu::Device, size: usize) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("iced3d custom uniform layout"),
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
            label: Some("iced3d custom uniform buffer"),
            size: size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("iced3d custom bind group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            layout,
            buffer,
            bind_group,
        }
    }

    /// Bind group layout — pass to [`PipelineConfig::custom_bind_group_layout`](crate::PipelineConfig).
    #[must_use]
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// Bind group — pass to [`RenderPipeline3D::render()`](crate::RenderPipeline3D::render).
    #[must_use]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Raw GPU buffer (for direct access if needed).
    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Upload uniform data. `data` must be `size` bytes or fewer.
    pub fn write(&self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(&self.buffer, 0, data);
    }
}
