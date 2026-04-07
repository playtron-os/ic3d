//! Shared GPU test helper — creates a wgpu Device via Vulkan (lavapipe in CI).
//!
//! All GPU integration tests use this helper. Requires the `vulkan` feature.

use ic3d::wgpu;

/// Request a wgpu device and queue without a surface (headless).
///
/// Uses `Backends::VULKAN` — on CI this hits lavapipe (Mesa software renderer).
/// Returns `None` if no Vulkan adapter is available.
pub fn try_gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ic3d test device"),
        ..Default::default()
    }))
    .ok()?;

    Some((device, queue))
}

/// Like [`try_gpu`] but panics with a descriptive message if unavailable.
pub fn gpu() -> (wgpu::Device, wgpu::Queue) {
    try_gpu().expect(
        "No Vulkan adapter available. Install mesa-vulkan-drivers (lavapipe) or run on a system with a GPU.",
    )
}
