use super::*;
use glam::Vec3;

#[test]
fn default_position() {
    let cam = OrthographicCamera::new();
    let view = cam.view_matrix();
    assert!(view != glam::Mat4::IDENTITY);
}

#[test]
fn view_projection_non_zero() {
    let cam = OrthographicCamera::new();
    let vp = cam.view_projection();
    let flat: Vec<f32> = (0..4)
        .flat_map(|c| (0..4).map(move |r| vp.col(c)[r]))
        .collect();
    assert!(flat.iter().any(|&v| v != 0.0));
}

#[test]
fn builder_chaining() {
    let cam = OrthographicCamera::new()
        .position(Vec3::new(0.0, 10.0, 0.0))
        .target(Vec3::ZERO)
        .up(Vec3::Z)
        .extents(20.0, 15.0)
        .depth(-50.0, 50.0);
    let vp = cam.view_projection();
    let flat: Vec<f32> = (0..4)
        .flat_map(|c| (0..4).map(move |r| vp.col(c)[r]))
        .collect();
    assert!(flat.iter().any(|&v| v != 0.0));
}

#[test]
fn default_equals_new() {
    let a = OrthographicCamera::new();
    let b = OrthographicCamera::default();
    let va = a.view_projection();
    let vb = b.view_projection();
    for c in 0..4 {
        for r in 0..4 {
            assert!(
                (va.col(c)[r] - vb.col(c)[r]).abs() < 1e-6,
                "mismatch at [{c},{r}]"
            );
        }
    }
}

#[test]
fn screen_to_world_center() {
    let cam = OrthographicCamera::new()
        .position(Vec3::new(0.0, 0.0, 3.0))
        .extents(5.0, 5.0);
    let world = cam.screen_to_world([400.0, 300.0], [800.0, 600.0]);
    assert!((world.x - 0.0).abs() < 1e-4);
    assert!((world.y - 0.0).abs() < 1e-4);
    assert!((world.z - 3.0).abs() < 1e-4);
}

#[test]
fn screen_to_world_corners() {
    let cam = OrthographicCamera::new()
        .position(Vec3::new(0.0, 0.0, 3.0))
        .extents(5.0, 5.0);
    let tl = cam.screen_to_world([0.0, 0.0], [800.0, 600.0]);
    assert!((tl.x - (-5.0)).abs() < 1e-4);
    assert!((tl.y - 5.0).abs() < 1e-4);
    let br = cam.screen_to_world([800.0, 600.0], [800.0, 600.0]);
    assert!((br.x - 5.0).abs() < 1e-4);
    assert!((br.y - (-5.0)).abs() < 1e-4);
}
