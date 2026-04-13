use super::*;
use glam::Vec3;

#[test]
fn camera_info_from_perspective() {
    let cam = PerspectiveCamera::new()
        .position(Vec3::new(0.0, 0.0, 5.0))
        .target(Vec3::ZERO)
        .fov(std::f32::consts::FRAC_PI_4);
    let info = CameraInfo::from_camera(&cam);
    assert_eq!(info.position, Vec3::new(0.0, 0.0, 5.0));
    assert!(info.fov_y.is_some());
    assert!((info.fov_y.unwrap() - std::f32::consts::FRAC_PI_4).abs() < 1e-6);
    // Forward should be -Z (towards target at origin from z=5)
    assert!(
        info.forward.z < -0.9,
        "forward should be -Z, got {:?}",
        info.forward
    );
}

#[test]
fn camera_info_from_orthographic() {
    let cam = OrthographicCamera::new()
        .position(Vec3::new(0.0, 0.0, 3.0))
        .target(Vec3::ZERO);
    let info = CameraInfo::from_camera(&cam);
    assert_eq!(info.position, Vec3::new(0.0, 0.0, 3.0));
    assert!(info.fov_y.is_none(), "orthographic should have no FOV");
    assert!(info.forward.z < -0.9, "forward should be -Z");
}

#[test]
fn perspective_camera_position_accessor() {
    let cam = PerspectiveCamera::new().position(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(cam.camera_position(), Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn perspective_camera_forward_accessor() {
    let cam = PerspectiveCamera::new()
        .position(Vec3::new(0.0, 0.0, 5.0))
        .target(Vec3::ZERO);
    let fwd = cam.camera_forward();
    assert!(fwd.z < -0.9, "forward should be -Z, got {:?}", fwd);
    assert!(
        (fwd.length() - 1.0).abs() < 1e-6,
        "forward should be normalized"
    );
}

#[test]
fn orthographic_camera_position_accessor() {
    let cam = OrthographicCamera::new().position(Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(cam.camera_position(), Vec3::new(4.0, 5.0, 6.0));
}

#[test]
fn orthographic_camera_forward_accessor() {
    let cam = OrthographicCamera::new()
        .position(Vec3::new(0.0, 0.0, 3.0))
        .target(Vec3::ZERO);
    let fwd = cam.camera_forward();
    assert!(fwd.z < -0.9, "forward should be -Z, got {:?}", fwd);
}

#[test]
fn orthographic_fov_y_is_none() {
    let cam = OrthographicCamera::new();
    assert!(cam.fov_y().is_none());
}

#[test]
fn perspective_fov_y_matches_builder() {
    let fov = std::f32::consts::FRAC_PI_3;
    let cam = PerspectiveCamera::new().fov(fov);
    assert_eq!(cam.fov_y(), Some(fov));
}

#[test]
fn camera_info_view_projection_matches() {
    let cam = PerspectiveCamera::new()
        .position(Vec3::new(0.0, 5.0, 10.0))
        .target(Vec3::ZERO);
    let info = CameraInfo::from_camera(&cam);
    let vp = cam.view_projection();
    for c in 0..4 {
        for r in 0..4 {
            assert!(
                (info.view_projection.col(c)[r] - vp.col(c)[r]).abs() < 1e-6,
                "VP mismatch at [{c},{r}]"
            );
        }
    }
}
