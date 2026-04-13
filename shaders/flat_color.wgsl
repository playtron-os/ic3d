// ic3d: Flat (unlit) fragment shader — material.rgb used directly, no lighting.
// Used for gizmos and other overlay geometry that should render as solid color.
@fragment
fn fs_main_flat(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.material.rgb, in.material.a);
}
