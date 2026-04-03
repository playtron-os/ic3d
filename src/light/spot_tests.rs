use super::*;
use crate::gpu_types::LIGHT_TYPE_SPOT;
use glam::Vec3;

#[test]
fn direction_normalized() {
    let light = SpotLight::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 0.0), 0.3, 0.5, 50.0);
    assert!((light.direction().length() - 1.0).abs() < 1e-6);
}

#[test]
fn basic() {
    let light = SpotLight::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y, 0.3, 0.5, 30.0);
    assert_eq!(light.position(), Vec3::new(0.0, 5.0, 0.0));
    assert!((light.direction().y - (-1.0)).abs() < 1e-6);
}

#[test]
fn gpu_light_type() {
    let light = SpotLight::new(Vec3::ZERO, Vec3::NEG_Y, 0.3, 0.5, 10.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.light_type, LIGHT_TYPE_SPOT);
}

#[test]
fn gpu_light_cone_cosines() {
    let inner = 0.3_f32;
    let outer = 0.5_f32;
    let light = SpotLight::new(Vec3::ZERO, Vec3::NEG_Y, inner, outer, 10.0);
    let gpu = light.to_gpu_light();
    assert!((gpu.inner_cone_cos - inner.cos()).abs() < 1e-6);
    assert!((gpu.outer_cone_cos - outer.cos()).abs() < 1e-6);
}

#[test]
fn with_color_and_intensity() {
    let light = SpotLight::new(Vec3::ZERO, Vec3::NEG_Y, 0.3, 0.5, 10.0)
        .with_color(Vec3::new(1.0, 0.0, 0.0))
        .with_intensity(5.0);
    let gpu = light.to_gpu_light();
    assert_eq!(gpu.color, [1.0, 0.0, 0.0]);
    assert!((gpu.intensity - 5.0).abs() < 1e-6);
}
