//! GPU data types: vertex, instance, and uniform structs (`#[repr(C)]` + `Pod`).

use bytemuck::{Pod, Zeroable};

// ──────────────────── Vertex ────────────────────

/// Per-vertex data: position + normal + UV.
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x3,  // normal
        2 => Float32x2,  // uv
    ];

    /// Vertex buffer layout for slot 0 (per-vertex, step per vertex).
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ──────────────────── Instance ────────────────────

/// Per-instance GPU data (128 bytes): model mat4 + normal mat3x3 + material vec4.
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
    pub normal_mat: [[f32; 3]; 3],
    pub _pad: [f32; 3],
    pub material: [f32; 4],
}

impl InstanceData {
    // Explicit offsets needed due to padding between normal_mat and material.
    const ATTRIBS: [wgpu::VertexAttribute; 8] = [
        // Model matrix (4 × vec4 columns)
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 3,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 16,
            shader_location: 4,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 32,
            shader_location: 5,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 48,
            shader_location: 6,
        },
        // Normal matrix (3 × vec3 columns)
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 64,
            shader_location: 7,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 76,
            shader_location: 8,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x3,
            offset: 88,
            shader_location: 9,
        },
        // Material params (vec4) — after padding
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 112,
            shader_location: 10,
        },
    ];

    /// Instance buffer layout for slot 1 (per-instance, step per instance).
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// ──────────────────── Scene Uniforms ────────────────────

/// Maximum number of lights supported in a single frame.
pub const MAX_LIGHTS: usize = 16;

/// Engine-managed GPU uniforms. Must match WGSL `SceneUniforms` exactly.
///
/// Contains camera, screen, and time data. Light data is in a separate
/// storage buffer ([`GpuLight`] array).
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct SceneUniforms {
    /// Camera view-projection matrix.
    pub view_projection: [[f32; 4]; 4],
    /// Camera world-space position (for specular/Fresnel).
    pub camera_position: [f32; 3],
    /// Elapsed time in seconds.
    pub time: f32,
    /// Screen resolution in pixels (width, height).
    pub screen_size: [f32; 2],
    /// Number of active lights in the light buffer.
    pub light_count: u32,
    /// Ambient light level.
    pub ambient: f32,
    /// Shadow map resolution in texels (both width and height).
    pub shadow_map_size: f32,
    /// Padding for 16-byte alignment.
    pub _pad: [f32; 3],
}

// ──────────────────── Light ────────────────────

/// Light type discriminant for GPU.
///
/// 0 = directional, 1 = point, 2 = spot.
pub const LIGHT_TYPE_DIRECTIONAL: u32 = 0;
pub const LIGHT_TYPE_POINT: u32 = 1;
pub const LIGHT_TYPE_SPOT: u32 = 2;

/// Per-light GPU data (128 bytes). Must match WGSL `GpuLight` exactly.
///
/// All light types share the same struct. Unused fields are zeroed.
/// - Directional: uses `direction`, ignores `position`/`range`/`inner_cone`/`outer_cone`
/// - Point: uses `position`/`range`, ignores `direction`/`inner_cone`/`outer_cone`
/// - Spot: uses all fields
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct GpuLight {
    /// Light-space view-projection for shadow mapping (only light 0 casts shadows currently).
    pub shadow_projection: [[f32; 4]; 4],
    /// Direction light travels (directional/spot). Normalized.
    pub direction: [f32; 3],
    /// 0 = directional, 1 = point, 2 = spot.
    pub light_type: u32,
    /// Light color (linear RGB).
    pub color: [f32; 3],
    /// Light intensity multiplier.
    pub intensity: f32,
    /// World-space position (point/spot lights).
    pub position: [f32; 3],
    /// Attenuation range (point/spot lights). 0 = infinite.
    pub range: f32,
    /// Spot light inner cone cosine (full intensity inside).
    pub inner_cone_cos: f32,
    /// Spot light outer cone cosine (zero intensity outside).
    pub outer_cone_cos: f32,
    pub _pad: [f32; 2],
}

#[cfg(test)]
#[path = "gpu_types_tests.rs"]
mod tests;
