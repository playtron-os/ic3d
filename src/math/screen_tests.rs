use super::*;
use glam::{Vec2, Vec3};

// ──────────── world_to_screen ────────────

#[test]
fn world_to_screen_origin_identity() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    let s = world_to_screen(Vec3::ZERO, vp, viewport).unwrap();
    assert!((s.x - 400.0).abs() < 1e-3);
    assert!((s.y - 300.0).abs() < 1e-3);
}

#[test]
fn world_to_screen_behind_camera() {
    // A point behind the camera has w <= 0 when projected.
    // With identity VP, a point at z=2 clips to w=1 (visible),
    // but we can craft a VP that puts it behind.
    let vp = glam::Mat4::look_at_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y)
        * glam::Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0);
    // Point far behind the camera
    let result = world_to_screen(Vec3::new(0.0, 0.0, 100.0), vp, Vec2::new(800.0, 600.0));
    assert!(result.is_none());
}

#[test]
fn world_to_screen_top_left_corner() {
    // NDC (-1, 1) should map to screen (0, 0)
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(100.0, 100.0);
    let s = world_to_screen(Vec3::new(-1.0, 1.0, 0.0), vp, viewport).unwrap();
    assert!((s.x - 0.0).abs() < 1e-3);
    assert!((s.y - 0.0).abs() < 1e-3);
}

// ──────────── point_to_segment_distance ────────────

#[test]
fn segment_distance_perpendicular() {
    let d = point_to_segment_distance(
        Vec2::new(5.0, 5.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
    );
    assert!((d - 5.0).abs() < 1e-3);
}

#[test]
fn segment_distance_at_endpoint_a() {
    let d = point_to_segment_distance(Vec2::new(-3.0, 4.0), Vec2::ZERO, Vec2::new(10.0, 0.0));
    assert!((d - 5.0).abs() < 1e-3);
}

#[test]
fn segment_distance_at_endpoint_b() {
    let d = point_to_segment_distance(Vec2::new(13.0, 4.0), Vec2::ZERO, Vec2::new(10.0, 0.0));
    assert!((d - 5.0).abs() < 1e-3);
}

#[test]
fn segment_distance_degenerate() {
    // Zero-length segment: distance is just point-to-point.
    let d = point_to_segment_distance(Vec2::new(3.0, 4.0), Vec2::ZERO, Vec2::ZERO);
    assert!((d - 5.0).abs() < 1e-3);
}

#[test]
fn segment_distance_on_segment() {
    let d = point_to_segment_distance(Vec2::new(5.0, 0.0), Vec2::ZERO, Vec2::new(10.0, 0.0));
    assert!(d < 1e-6);
}

// ──────────── screen_constant_scale ────────────

#[test]
fn screen_constant_scale_perspective() {
    let camera = crate::CameraInfo {
        position: Vec3::new(0.0, 0.0, 10.0),
        forward: Vec3::NEG_Z,
        fov_y: Some(std::f32::consts::FRAC_PI_4),
        view_projection: glam::Mat4::IDENTITY,
    };
    let s = screen_constant_scale(Vec3::ZERO, &camera, 600.0, 80.0);
    assert!(s > 0.0, "scale should be positive");
}

#[test]
fn screen_constant_scale_orthographic_fallback() {
    let camera = crate::CameraInfo {
        position: Vec3::ZERO,
        forward: Vec3::NEG_Z,
        fov_y: None,
        view_projection: glam::Mat4::IDENTITY,
    };
    let s = screen_constant_scale(Vec3::new(0.0, 0.0, -5.0), &camera, 600.0, 80.0);
    assert!((s - 1.0).abs() < 1e-6, "orthographic should return 1.0");
}

#[test]
fn screen_constant_scale_behind_camera() {
    let camera = crate::CameraInfo {
        position: Vec3::ZERO,
        forward: Vec3::NEG_Z,
        fov_y: Some(1.0),
        view_projection: glam::Mat4::IDENTITY,
    };
    // Point behind the camera (positive Z when forward is -Z)
    let s = screen_constant_scale(Vec3::new(0.0, 0.0, 5.0), &camera, 600.0, 80.0);
    assert!((s - 1.0).abs() < 1e-6, "behind camera should return 1.0");
}

#[test]
fn screen_constant_scale_proportional() {
    let camera = crate::CameraInfo {
        position: Vec3::new(0.0, 0.0, 10.0),
        forward: Vec3::NEG_Z,
        fov_y: Some(std::f32::consts::FRAC_PI_4),
        view_projection: glam::Mat4::IDENTITY,
    };
    let s80 = screen_constant_scale(Vec3::ZERO, &camera, 600.0, 80.0);
    let s160 = screen_constant_scale(Vec3::ZERO, &camera, 600.0, 160.0);
    assert!(
        (s160 / s80 - 2.0).abs() < 1e-3,
        "double screen_px → double scale"
    );
}

// ──────────── screen_hit_test ────────────

#[test]
fn screen_hit_test_point_hit() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    // Origin projects to screen center (400, 300).
    let shape = HitShape::point(Vec3::ZERO, 10.0);
    let cursor = Vec2::new(405.0, 300.0);
    let result = screen_hit_test(&shape, cursor, vp, viewport);
    assert!(result.is_some());
    assert!((result.unwrap() - 5.0).abs() < 1e-3);
}

#[test]
fn screen_hit_test_point_miss() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    let shape = HitShape::point(Vec3::ZERO, 10.0);
    let cursor = Vec2::new(420.0, 300.0); // 20px away, radius is 10
    assert!(screen_hit_test(&shape, cursor, vp, viewport).is_none());
}

#[test]
fn screen_hit_test_point_behind_camera() {
    let vp = glam::Mat4::look_at_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y)
        * glam::Mat4::perspective_rh(1.0, 1.0, 0.1, 100.0);
    let viewport = Vec2::new(800.0, 600.0);
    let shape = HitShape::point(Vec3::new(0.0, 0.0, 100.0), 50.0);
    assert!(screen_hit_test(&shape, Vec2::new(400.0, 300.0), vp, viewport).is_none());
}

#[test]
fn screen_hit_test_segment_hit() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    // Segment from (-0.5, 0, 0) to (0.5, 0, 0) projects to (200, 300)→(600, 300).
    let shape = HitShape::segment(Vec3::new(-0.5, 0.0, 0.0), Vec3::new(0.5, 0.0, 0.0), 10.0);
    let cursor = Vec2::new(400.0, 305.0); // 5px perpendicular, threshold 10
    let result = screen_hit_test(&shape, cursor, vp, viewport);
    assert!(result.is_some());
    assert!((result.unwrap() - 5.0).abs() < 1e-3);
}

#[test]
fn screen_hit_test_segment_miss() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    let shape = HitShape::segment(Vec3::new(-0.5, 0.0, 0.0), Vec3::new(0.5, 0.0, 0.0), 10.0);
    let cursor = Vec2::new(400.0, 320.0); // 20px perpendicular, threshold 10
    assert!(screen_hit_test(&shape, cursor, vp, viewport).is_none());
}
