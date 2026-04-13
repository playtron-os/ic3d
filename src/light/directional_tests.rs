use super::*;
use crate::pipeline::gpu_types::LIGHT_TYPE_DIRECTIONAL;
use glam::Vec3;

#[test]
fn direction_normalized() {
    let light = DirectionalLight::new(Vec3::new(1.0, 2.0, 3.0), Vec3::ZERO, 10.0, 20.0);
    let d = light.direction();
    assert!((d.length() - 1.0).abs() < 1e-6, "direction not normalized");
}

#[test]
fn to_light_negated() {
    let light = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let toward = light.to_light();
    assert!((toward.y - 1.0).abs() < 1e-6);
}

#[test]
fn shadow_projection_non_zero() {
    let light = DirectionalLight::new(Vec3::new(-0.5, -1.0, -0.3), Vec3::ZERO, 15.0, 30.0);
    let sp = light.shadow_projection();
    assert!(sp.determinant().abs() > 0.0);
}

#[test]
fn gpu_light_type() {
    let light = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.light_type, LIGHT_TYPE_DIRECTIONAL);
}

#[test]
fn gpu_light_defaults() {
    let light = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.color, [1.0, 1.0, 1.0]);
    assert!((gpu.intensity - 1.0).abs() < 1e-6);
}

#[test]
fn with_color_and_intensity() {
    let light = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0)
        .with_color(Vec3::new(1.0, 0.5, 0.0))
        .with_intensity(2.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.color, [1.0, 0.5, 0.0]);
    assert!((gpu.intensity - 2.0).abs() < 1e-6);
}

#[test]
fn with_extents() {
    let light = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0)
        .with_extents(30.0, 20.0);
    let sp = light.shadow_projection();
    assert!(sp.determinant().abs() > 0.0);
}

#[test]
fn vertical_direction_uses_z_up() {
    let light = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let sp = light.shadow_projection();
    assert!(sp.determinant().abs() > 0.0, "degenerate shadow projection");
}
