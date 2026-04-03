//! GPU integration tests for `ShadowPass`.

#[path = "gpu_helper.rs"]
mod gpu_helper;

use iced3d::wgpu;
use iced3d::{GpuLight, ShadowPass, MAX_LIGHTS};

#[test]
fn creation() {
    let (device, _queue) = gpu_helper::gpu();

    let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test lights"),
        size: (std::mem::size_of::<GpuLight>() * MAX_LIGHTS) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shadow = ShadowPass::new(&device, &light_buffer, 1024);
    assert_eq!(shadow.size, 1024);
}

#[test]
fn small_shadow_map() {
    let (device, _queue) = gpu_helper::gpu();

    let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test lights"),
        size: (std::mem::size_of::<GpuLight>() * MAX_LIGHTS) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shadow = ShadowPass::new(&device, &light_buffer, 256);
    assert_eq!(shadow.size, 256);
}

#[test]
fn render_empty_draws() {
    let (device, _queue) = gpu_helper::gpu();

    let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test lights"),
        size: (std::mem::size_of::<GpuLight>() * MAX_LIGHTS) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shadow = ShadowPass::new(&device, &light_buffer, 512);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("test encoder"),
    });
    shadow.render(&mut encoder, &[]);
}
