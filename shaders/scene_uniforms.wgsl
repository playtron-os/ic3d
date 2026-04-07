// ic3d: Scene uniforms + light array (group 0)

struct SceneUniforms {
    view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
    time: f32,
    screen_size: vec2<f32>,
    light_count: u32,
    ambient: f32,
    shadow_map_size: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

// 0 = directional, 1 = point, 2 = spot
const LIGHT_DIRECTIONAL: u32 = 0u;
const LIGHT_POINT: u32 = 1u;
const LIGHT_SPOT: u32 = 2u;

struct GpuLight {
    shadow_projection: mat4x4<f32>,
    direction: vec3<f32>,
    light_type: u32,
    color: vec3<f32>,
    intensity: f32,
    position: vec3<f32>,
    range: f32,
    inner_cone_cos: f32,
    outer_cone_cos: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var<uniform> scene: SceneUniforms;
@group(0) @binding(1) var<storage, read> lights: array<GpuLight>;
@group(0) @binding(2) var shadow_map: texture_depth_2d;
@group(0) @binding(3) var shadow_sampler: sampler_comparison;
