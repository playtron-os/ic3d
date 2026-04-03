use super::*;
use glam::Vec3;

#[test]
fn default_non_zero() {
    let cam = PerspectiveCamera::new();
    let vp = cam.view_projection();
    let flat: Vec<f32> = (0..4)
        .flat_map(|c| (0..4).map(move |r| vp.col(c)[r]))
        .collect();
    assert!(flat.iter().any(|&v| v != 0.0));
}

#[test]
fn default_equals_new() {
    let a = PerspectiveCamera::new();
    let b = PerspectiveCamera::default();
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
fn builder_chaining() {
    let cam = PerspectiveCamera::new()
        .position(Vec3::new(0.0, 5.0, 10.0))
        .target(Vec3::ZERO)
        .up(Vec3::Y)
        .fov(std::f32::consts::FRAC_PI_3)
        .aspect(16.0 / 9.0)
        .clip(0.01, 1000.0);
    let vp = cam.view_projection();
    assert!(vp.determinant().abs() > 0.0);
}

#[test]
fn view_matrix_differs_from_projection() {
    let cam = PerspectiveCamera::new();
    let view = cam.view_matrix();
    let proj = cam.projection_matrix();
    let mut same = true;
    for c in 0..4 {
        for r in 0..4 {
            if (view.col(c)[r] - proj.col(c)[r]).abs() > 1e-6 {
                same = false;
            }
        }
    }
    assert!(!same, "view and projection should differ");
}
