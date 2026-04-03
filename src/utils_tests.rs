use super::compose_shader;

#[test]
fn compose_shader_includes_preludes() {
    let result =
        compose_shader("@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4(1.0); }");
    assert!(result.contains("SceneUniforms"));
    assert!(result.contains("VertexIn"));
    assert!(result.contains("vs_main"));
    assert!(result.contains("sample_shadow_pcf"));
    assert!(result.contains("fs_main"));
}

#[test]
fn compose_shader_fragment_at_end() {
    let fragment = "@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4(1.0); }";
    let result = compose_shader(fragment);
    assert!(result.ends_with(fragment));
}

#[test]
fn compose_shader_non_empty() {
    let result = compose_shader("");
    // Even with empty fragment, preludes should be present
    assert!(result.len() > 100);
}
