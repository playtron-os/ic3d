use super::*;

#[test]
fn gpu_types_vertex_size() {
    // pos(3) + normal(3) + uv(2) = 8 floats = 32 bytes
    assert_eq!(std::mem::size_of::<Vertex>(), 32);
}

#[test]
fn gpu_types_instance_data_size() {
    // 128 bytes (model 64 + normal_mat 36 + pad 12 + material 16)
    assert_eq!(std::mem::size_of::<InstanceData>(), 128);
}

#[test]
fn gpu_types_scene_uniforms_size() {
    // Must be a multiple of 16 for uniform buffer alignment
    let size = std::mem::size_of::<SceneUniforms>();
    assert_eq!(size % 16, 0, "SceneUniforms size {size} not 16-aligned");
}

#[test]
fn gpu_types_gpu_light_size() {
    // 128 bytes
    assert_eq!(std::mem::size_of::<GpuLight>(), 128);
}

#[test]
fn gpu_types_max_lights() {
    assert_eq!(MAX_LIGHTS, 16);
}

#[test]
fn gpu_types_light_type_constants() {
    assert_eq!(LIGHT_TYPE_DIRECTIONAL, 0);
    assert_eq!(LIGHT_TYPE_POINT, 1);
    assert_eq!(LIGHT_TYPE_SPOT, 2);
}

#[test]
fn gpu_types_vertex_zeroed() {
    let v = Vertex::zeroed();
    assert_eq!(v.pos, [0.0, 0.0, 0.0]);
    assert_eq!(v.normal, [0.0, 0.0, 0.0]);
    assert_eq!(v.uv, [0.0, 0.0]);
}

#[test]
fn gpu_types_instance_data_zeroed() {
    let i = InstanceData::zeroed();
    assert_eq!(i.material, [0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn gpu_types_gpu_light_zeroed() {
    let l = GpuLight::zeroed();
    assert_eq!(l.light_type, 0);
    assert_eq!(l.intensity, 0.0);
}

#[test]
fn gpu_types_vertex_layout_attributes() {
    let layout = Vertex::layout();
    assert_eq!(layout.array_stride, 32);
    assert_eq!(layout.attributes.len(), 3);
}

#[test]
fn gpu_types_instance_data_layout_attributes() {
    let layout = InstanceData::layout();
    assert_eq!(layout.array_stride, 128);
    assert_eq!(layout.attributes.len(), 8);
}
