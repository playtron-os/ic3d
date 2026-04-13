use super::*;
use crate::camera::{OrthographicCamera, PerspectiveCamera};
use crate::light::DirectionalLight;
use glam::Vec3;

#[test]
fn scene_defaults() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).build();
    assert!((data.uniforms.ambient - 0.1).abs() < 1e-6);
    assert_eq!(data.uniforms.light_count, 0);
    assert_eq!(data.uniforms.screen_size, [1.0, 1.0]);
    assert!((data.uniforms.shadow_map_size - 2048.0).abs() < 1e-6);
    assert!(data.lights.is_empty());
}

#[test]
fn scene_with_time() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).time(1.5).build();
    assert!((data.uniforms.time - 1.5).abs() < 1e-6);
}

#[test]
fn scene_with_ambient() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).ambient(0.5).build();
    assert!((data.uniforms.ambient - 0.5).abs() < 1e-6);
}

#[test]
fn scene_with_screen_size() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).screen_size(1920.0, 1080.0).build();
    assert_eq!(data.uniforms.screen_size, [1920.0, 1080.0]);
}

#[test]
fn scene_with_shadow_map_size() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).shadow_map_size(4096.0).build();
    assert!((data.uniforms.shadow_map_size - 4096.0).abs() < 1e-6);
}

#[test]
fn scene_with_camera_position() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).camera_position([1.0, 2.0, 3.0]).build();
    assert_eq!(data.uniforms.camera_position, [1.0, 2.0, 3.0]);
}

#[test]
fn scene_with_one_light() {
    let cam = OrthographicCamera::new();
    let sun = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let data = Scene::new(&cam).light(&sun).build();
    assert_eq!(data.uniforms.light_count, 1);
    assert_eq!(data.lights.len(), 1);
}

#[test]
fn scene_with_multiple_lights() {
    let cam = OrthographicCamera::new();
    let sun = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let fill = DirectionalLight::new(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO, 5.0, 10.0);
    let data = Scene::new(&cam).light(&sun).light(&fill).build();
    assert_eq!(data.uniforms.light_count, 2);
    assert_eq!(data.lights.len(), 2);
}

#[test]
fn scene_view_projection_is_set() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).build();
    // View-projection should not be all zeros
    let flat: Vec<f32> = data
        .uniforms
        .view_projection
        .iter()
        .flat_map(|col| col.iter())
        .copied()
        .collect();
    assert!(flat.iter().any(|&v| v != 0.0));
}

#[test]
fn scene_chaining() {
    let cam = OrthographicCamera::new();
    let sun = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let data = Scene::new(&cam)
        .light(&sun)
        .time(2.0)
        .ambient(0.3)
        .screen_size(800.0, 600.0)
        .camera_position([0.0, 5.0, 0.0])
        .shadow_map_size(1024.0)
        .build();
    assert_eq!(data.uniforms.light_count, 1);
    assert!((data.uniforms.time - 2.0).abs() < 1e-6);
    assert!((data.uniforms.ambient - 0.3).abs() < 1e-6);
    assert_eq!(data.uniforms.screen_size, [800.0, 600.0]);
    assert_eq!(data.uniforms.camera_position, [0.0, 5.0, 0.0]);
    assert!((data.uniforms.shadow_map_size - 1024.0).abs() < 1e-6);
}

#[test]
#[should_panic(expected = "exceeded MAX_LIGHTS")]
fn scene_panics_on_too_many_lights() {
    let cam = OrthographicCamera::new();
    let sun = DirectionalLight::new(Vec3::new(0.0, -1.0, 0.0), Vec3::ZERO, 10.0, 20.0);
    let mut scene = Scene::new(&cam);
    for _ in 0..=crate::pipeline::gpu_types::MAX_LIGHTS {
        scene = scene.light(&sun);
    }
}

#[test]
fn scene_camera_info_auto_derived() {
    let cam = PerspectiveCamera::new().position(Vec3::new(0.0, 5.0, 10.0));
    let data = Scene::new(&cam).build();
    let pos = data.camera.position;
    assert!((pos.x - 0.0).abs() < 1e-4);
    assert!((pos.y - 5.0).abs() < 1e-4);
    assert!((pos.z - 10.0).abs() < 1e-4);
    assert!(data.camera.fov_y.is_some());
}

#[test]
fn scene_camera_position_auto_derived_from_camera() {
    let cam = PerspectiveCamera::new().position(Vec3::new(1.0, 2.0, 3.0));
    let data = Scene::new(&cam).build();
    assert_eq!(data.uniforms.camera_position, [1.0, 2.0, 3.0]);
}

#[test]
fn scene_camera_position_override_takes_precedence() {
    let cam = PerspectiveCamera::new().position(Vec3::new(1.0, 2.0, 3.0));
    let data = Scene::new(&cam).camera_position([9.0, 8.0, 7.0]).build();
    assert_eq!(data.uniforms.camera_position, [9.0, 8.0, 7.0]);
}

#[test]
fn scene_ortho_camera_has_no_fov() {
    let cam = OrthographicCamera::new();
    let data = Scene::new(&cam).build();
    assert!(data.camera.fov_y.is_none());
}
