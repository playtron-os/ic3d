// iced3d: Vertex input (matches Vertex + InstanceData layouts)
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

// iced3d: Standard vertex output
struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) material: vec4<f32>,
    @location(4) light_clip_pos: vec4<f32>,
}
