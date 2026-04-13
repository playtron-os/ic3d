use super::*;
use crate::pipeline::gpu_types::LIGHT_TYPE_POINT;
use glam::Vec3;

#[test]
fn basic() {
    let light = PointLight::new(Vec3::new(1.0, 2.0, 3.0), 50.0);
    assert_eq!(light.position(), Vec3::new(1.0, 2.0, 3.0));
    assert!((light.range() - 50.0).abs() < 1e-6);
}

#[test]
fn gpu_light_type() {
    let light = PointLight::new(Vec3::ZERO, 10.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.light_type, LIGHT_TYPE_POINT);
}

#[test]
fn gpu_light_defaults() {
    let light = PointLight::new(Vec3::new(5.0, 0.0, 0.0), 25.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.color, [1.0, 1.0, 1.0]);
    assert!((gpu.intensity - 1.0).abs() < 1e-6);
    assert_eq!(gpu.position, [5.0, 0.0, 0.0]);
    assert!((gpu.range - 25.0).abs() < 1e-6);
}

#[test]
fn with_color_and_intensity() {
    let light = PointLight::new(Vec3::ZERO, 10.0)
        .with_color(Vec3::new(0.0, 1.0, 0.0))
        .with_intensity(3.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.color, [0.0, 1.0, 0.0]);
    assert!((gpu.intensity - 3.0).abs() < 1e-6);
}
