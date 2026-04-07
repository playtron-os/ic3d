// ic3d shadow pass: depth-only render from primary light's perspective.

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

@group(0) @binding(0) var<storage, read> lights: array<GpuLight>;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) model_0: vec4<f32>,
    @location(4) model_1: vec4<f32>,
    @location(5) model_2: vec4<f32>,
    @location(6) model_3: vec4<f32>,

    @location(7) normal_mat_0: vec3<f32>,
    @location(8) normal_mat_1: vec3<f32>,
    @location(9) normal_mat_2: vec3<f32>,

    @location(10) material: vec4<f32>,
}

@vertex
fn vs_shadow(in: VertexIn) -> @builtin(position) vec4<f32> {
    let model = mat4x4<f32>(in.model_0, in.model_1, in.model_2, in.model_3);
    let world = model * vec4<f32>(in.position, 1.0);
    return lights[0].shadow_projection * world;
}
