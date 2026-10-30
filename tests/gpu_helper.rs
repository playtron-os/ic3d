//! Shared GPU test helper — creates a wgpu Device via Vulkan (lavapipe in CI).
//!
//! All GPU integration tests use this helper. Requires the `vulkan` feature.

use std::sync::OnceLock;

use ic3d::wgpu;

/// One device per test binary, shared by its tests. Vulkan loaders before
/// 1.4.345 crash when one thread destroys a device while another names an object.
static GPU: OnceLock<Option<(wgpu::Device, wgpu::Queue)>> = OnceLock::new();

/// Request a wgpu device and queue without a surface (headless).
///
/// Uses `Backends::VULKAN` — on CI this hits lavapipe (Mesa software renderer).
/// Returns `None` if no Vulkan adapter is available.
pub fn try_gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
    GPU.get_or_init(request_gpu).clone()
}

fn request_gpu() -> Option<(wgpu::Device, wgpu::Queue)> {
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
