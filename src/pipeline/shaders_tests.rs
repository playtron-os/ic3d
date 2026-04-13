use super::*;

#[test]
fn shaders_scene_uniforms_non_empty() {
    assert!(!SCENE_UNIFORMS_WGSL.is_empty());
}

#[test]
fn shaders_vertex_io_non_empty() {
    assert!(!VERTEX_IO_WGSL.is_empty());
}

#[test]
fn shaders_standard_vs_non_empty() {
    assert!(!STANDARD_VS_WGSL.is_empty());
}

#[test]
fn shaders_shadow_pcf_non_empty() {
    assert!(!SHADOW_PCF_WGSL.is_empty());
}

#[test]
fn shaders_blinn_phong_non_empty() {
    assert!(!BLINN_PHONG_WGSL.is_empty());
}

#[test]
fn shaders_scene_uniforms_has_struct() {
    assert!(SCENE_UNIFORMS_WGSL.contains("SceneUniforms"));
}

#[test]
fn shaders_vertex_io_has_structs() {
    assert!(VERTEX_IO_WGSL.contains("VertexIn"));
    assert!(VERTEX_IO_WGSL.contains("VertexOut"));
}

#[test]
fn shaders_standard_vs_has_entry() {
    assert!(STANDARD_VS_WGSL.contains("vs_main"));
}

#[test]
fn shaders_blinn_phong_has_entry() {
    assert!(BLINN_PHONG_WGSL.contains("fs_main"));
}

#[test]
fn shaders_shadow_pcf_has_function() {
    assert!(SHADOW_PCF_WGSL.contains("sample_shadow_pcf"));
}

#[test]
fn shaders_flat_color_non_empty() {
    assert!(!FLAT_COLOR_WGSL.is_empty());
}

#[test]
fn shaders_flat_color_has_entry() {
    assert!(FLAT_COLOR_WGSL.contains("fs_main_flat"));
}
