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

// ──────────── screen_to_ground ────────────

#[test]
fn screen_to_ground_center_looking_down() {
    // Camera at (0, 10, 0) looking straight down at the origin.
    let view = glam::Mat4::look_at_rh(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::ZERO,
        Vec3::NEG_Z, // "up" is -Z when looking down Y
    );
    let proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
    let inv_vp = (proj * view).inverse();
    let viewport = Vec2::new(800.0, 800.0);
    // Screen center should map to world origin on ground
    let hit = screen_to_ground(Vec2::new(400.0, 400.0), viewport, inv_vp, 0.0).unwrap();
    assert!(hit.x.abs() < 0.5, "x should be near 0, got {}", hit.x);
    assert!(hit.y.abs() < 0.5, "z should be near 0, got {}", hit.y);
}

#[test]
fn screen_to_ground_offset_cursor() {
    // Camera at (0, 10, 10) looking at origin — angled view.
    let view = glam::Mat4::look_at_rh(Vec3::new(0.0, 10.0, 10.0), Vec3::ZERO, Vec3::Y);
    let proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
    let inv_vp = (proj * view).inverse();
    let viewport = Vec2::new(800.0, 600.0);
    // Moving cursor left should shift world X
    let center = screen_to_ground(Vec2::new(400.0, 300.0), viewport, inv_vp, 0.0).unwrap();
    let left = screen_to_ground(Vec2::new(200.0, 300.0), viewport, inv_vp, 0.0).unwrap();
    assert!(left.x < center.x, "left cursor should give smaller world X");
}

#[test]
fn screen_to_ground_returns_none_for_parallel_ray() {
    // Camera at (0, 0, 5) looking along -Z (parallel to ground plane)
    let view = glam::Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    let proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
    let inv_vp = (proj * view).inverse();
    let viewport = Vec2::new(800.0, 600.0);
    // Center of screen should look straight along -Z, parallel to Y=0 plane
    let result = screen_to_ground(Vec2::new(400.0, 300.0), viewport, inv_vp, 0.0);
    // Ray is nearly parallel to ground — may return None or a very distant point
    // Either outcome is acceptable for a parallel ray
    if let Some(hit) = result {
        // If it hits, it should be very far away
        assert!(
            hit.length() > 50.0,
            "parallel ray hit should be very distant, got {}",
            hit.length()
        );
    }
}

#[test]
fn screen_to_ground_elevated_plane() {
    // Camera at (0, 10, 0) looking down
    let view = glam::Mat4::look_at_rh(Vec3::new(0.0, 10.0, 0.0), Vec3::ZERO, Vec3::NEG_Z);
    let proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
    let inv_vp = (proj * view).inverse();
    let viewport = Vec2::new(800.0, 800.0);
    // Ground at Y=2 should still work, but hit closer to camera
    let hit_0 = screen_to_ground(Vec2::new(400.0, 400.0), viewport, inv_vp, 0.0).unwrap();
    let hit_2 = screen_to_ground(Vec2::new(400.0, 400.0), viewport, inv_vp, 2.0).unwrap();
    // Both should be near center, but elevated plane hit is valid
    assert!(hit_0.x.abs() < 0.5);
    assert!(hit_2.x.abs() < 0.5);
}

// ──────────── world_radius_to_screen ────────────

fn test_camera(pos: Vec3, target: Vec3) -> crate::CameraInfo {
    let view = glam::Mat4::look_at_rh(pos, target, Vec3::Y);
    let proj = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
    crate::CameraInfo {
        position: pos,
        forward: (target - pos).normalize(),
        fov_y: Some(std::f32::consts::FRAC_PI_4),
        view_projection: proj * view,
    }
}

#[test]
fn world_radius_to_screen_positive() {
    let cam = test_camera(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO);
    let px = world_radius_to_screen(Vec3::ZERO, 1.0, &cam, Vec2::new(800.0, 600.0));
    assert!(px.is_some());
    assert!(
        px.unwrap() > 0.0,
        "radius should project to positive pixels"
    );
}

#[test]
fn world_radius_to_screen_proportional() {
    let cam = test_camera(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO);
    let vp = Vec2::new(800.0, 600.0);
    let px1 = world_radius_to_screen(Vec3::ZERO, 1.0, &cam, vp).unwrap();
    let px2 = world_radius_to_screen(Vec3::ZERO, 2.0, &cam, vp).unwrap();
    assert!(
        (px2 / px1 - 2.0).abs() < 0.1,
        "double world radius → double screen px: ratio = {}",
        px2 / px1
    );
}

#[test]
fn world_radius_to_screen_farther_is_smaller() {
    let vp = Vec2::new(800.0, 600.0);
    let cam_near = test_camera(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO);
    let cam_far = test_camera(Vec3::new(0.0, 0.0, 20.0), Vec3::ZERO);
    let px_near = world_radius_to_screen(Vec3::ZERO, 1.0, &cam_near, vp).unwrap();
    let px_far = world_radius_to_screen(Vec3::ZERO, 1.0, &cam_far, vp).unwrap();
    assert!(
        px_near > px_far,
        "closer camera should give larger screen radius"
    );
}

#[test]
fn world_radius_to_screen_behind_camera() {
    let cam = test_camera(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO);
    // Point behind the camera
    let result = world_radius_to_screen(
        Vec3::new(0.0, 0.0, 20.0),
        1.0,
        &cam,
        Vec2::new(800.0, 600.0),
    );
    assert!(result.is_none(), "point behind camera should return None");
}

#[test]
fn world_radius_to_screen_camera_looking_along_x() {
    // Camera along X axis — ensures the view-aligned edge computation is correct
    let cam = test_camera(Vec3::new(10.0, 0.0, 0.0), Vec3::ZERO);
    let px = world_radius_to_screen(Vec3::ZERO, 1.0, &cam, Vec2::new(800.0, 600.0));
    assert!(px.is_some());
    assert!(px.unwrap() > 0.0, "should work from any camera direction");
}

// ──────────── screen_hit_test (Arc) ────────────

#[test]
fn screen_hit_test_arc_hit() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    // Full circle arc at origin, radius 0.5. In NDC, 0.5 maps to 200px.
    // Cursor at (600, 300) = NDC (0.5, 0) = exactly on the arc.
    let shape = HitShape::arc(
        Vec3::ZERO,
        0.5,
        glam::Quat::IDENTITY,
        0.0,
        std::f32::consts::TAU,
        24,
        20.0,
    );
    let cursor = Vec2::new(600.0, 300.0);
    let result = screen_hit_test(&shape, cursor, vp, viewport);
    assert!(result.is_some(), "cursor on arc should hit");
    assert!(result.unwrap() < 20.0);
}

#[test]
fn screen_hit_test_arc_miss() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    // Arc at origin, radius 0.5. Cursor far away at (100, 100).
    let shape = HitShape::arc(
        Vec3::ZERO,
        0.5,
        glam::Quat::IDENTITY,
        0.0,
        std::f32::consts::TAU,
        24,
        10.0,
    );
    let cursor = Vec2::new(100.0, 100.0);
    assert!(screen_hit_test(&shape, cursor, vp, viewport).is_none());
}

#[test]
fn screen_hit_test_arc_partial_sweep() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    // Quarter-circle arc from angle 0 to π/4 at radius 0.5.
    // With identity VP, this covers screen x from ~600 to ~540 (right side only).
    let shape = HitShape::arc(
        Vec3::ZERO,
        0.5,
        glam::Quat::IDENTITY,
        0.0,
        std::f32::consts::FRAC_PI_4,
        12,
        10.0,
    );
    // Left side of screen should miss (far from the short arc).
    let cursor_left = Vec2::new(200.0, 300.0);
    assert!(
        screen_hit_test(&shape, cursor_left, vp, viewport).is_none(),
        "cursor on the non-swept side should miss"
    );
    // Right side should hit (near the arc start at angle 0).
    let cursor_right = Vec2::new(600.0, 300.0);
    assert!(
        screen_hit_test(&shape, cursor_right, vp, viewport).is_some(),
        "cursor on the swept side should hit"
    );
}

#[test]
fn screen_hit_test_arc_rotated() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    // Arc in the XY plane (rotated 90° around X). The arc is at Z=0 in
    // local XZ, which after rot_x(π/2) becomes the XY plane.
    let rot = glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let shape = HitShape::arc(Vec3::ZERO, 0.5, rot, 0.0, std::f32::consts::TAU, 24, 20.0);
    // With identity VP, point at (0.5, 0, 0) in world projects to (600, 300).
    let cursor = Vec2::new(600.0, 300.0);
    assert!(
        screen_hit_test(&shape, cursor, vp, viewport).is_some(),
        "rotated arc should still be hittable"
    );
}

// ──────────── screen_hit_test_closest ────────────

#[test]
fn closest_returns_nearest() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    // Two points: A at origin (screen center 400, 300) and B at (0.5, 0, 0) → (600, 300).
    // Cursor at (410, 300) → 10px from A, 190px from B.
    let shapes = [
        ("A", HitShape::point(Vec3::ZERO, 20.0)),
        ("B", HitShape::point(Vec3::new(0.5, 0.0, 0.0), 20.0)),
    ];
    let result = screen_hit_test_closest(shapes, Vec2::new(410.0, 300.0), vp, viewport);
    assert!(result.is_some());
    let (label, dist) = result.unwrap();
    assert_eq!(label, "A");
    assert!((dist - 10.0).abs() < 1e-3);
}

#[test]
fn closest_returns_none_when_all_miss() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    let shapes = [
        (0, HitShape::point(Vec3::ZERO, 5.0)),
        (1, HitShape::point(Vec3::new(0.5, 0.0, 0.0), 5.0)),
    ];
    // Cursor far from both.
    let result = screen_hit_test_closest(shapes, Vec2::new(100.0, 100.0), vp, viewport);
    assert!(result.is_none());
}

#[test]
fn closest_single_shape() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    let shapes = [(42_u32, HitShape::point(Vec3::ZERO, 20.0))];
    let result = screen_hit_test_closest(shapes, Vec2::new(405.0, 300.0), vp, viewport);
    assert_eq!(result.unwrap().0, 42);
}

#[test]
fn closest_empty_iterator() {
    let vp = glam::Mat4::IDENTITY;
    let viewport = Vec2::new(800.0, 600.0);
    let shapes: [(i32, HitShape); 0] = [];
    assert!(screen_hit_test_closest(shapes, Vec2::new(400.0, 300.0), vp, viewport).is_none());
}
