// ic3d: Standard vertex shader
@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    let model = mat4x4<f32>(in.model_0, in.model_1, in.model_2, in.model_3);
    let normal_mat = mat3x3<f32>(in.normal_mat_0, in.normal_mat_1, in.normal_mat_2);

    let world = model * vec4<f32>(in.position, 1.0);
    let wn = normalize(normal_mat * in.normal);

    var out: VertexOut;
    out.clip_pos = scene.view_projection * world;
    out.world_pos = world.xyz;
    out.world_normal = wn;
    out.uv = in.uv;
    out.material = in.material;
    // Shadow map lookup uses primary light (index 0)
    out.light_clip_pos = lights[0].shadow_projection * world;
    return out;
}
