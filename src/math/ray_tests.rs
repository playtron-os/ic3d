//! Tests for the ray module.

use super::*;
use glam::{Mat4, Vec2, Vec3};

#[test]
fn ray_point_at() {
    let ray = Ray::new(Vec3::ZERO, Vec3::X);
    assert_eq!(ray.point_at(0.0), Vec3::ZERO);
    assert_eq!(ray.point_at(1.0), Vec3::X);
    assert_eq!(ray.point_at(5.0), Vec3::new(5.0, 0.0, 0.0));
}

#[test]
fn ray_direction_is_normalized() {
    let ray = Ray::new(Vec3::ZERO, Vec3::new(3.0, 4.0, 0.0));
    let len = ray.direction.length();
    assert!(
        (len - 1.0).abs() < 1e-6,
        "direction should be normalized, got length {len}"
    );
}

#[test]
fn intersect_plane_perpendicular() {
    // Ray along +Z hitting the XY plane at z=5
    let ray = Ray::new(Vec3::ZERO, Vec3::Z);
    let t = ray.intersect_plane(Vec3::Z, Vec3::new(0.0, 0.0, 5.0));
    assert!((t.unwrap() - 5.0).abs() < 1e-6);
}

#[test]
fn intersect_plane_parallel() {
    // Ray along +X, plane normal is also +X-direction but ray is parallel
    let ray = Ray::new(Vec3::ZERO, Vec3::Y);
    let t = ray.intersect_plane(Vec3::X, Vec3::new(5.0, 0.0, 0.0));
    assert!(t.is_none(), "parallel ray should not intersect");
}

#[test]
fn closest_to_line_perpendicular() {
    // Ray along +X, line along +Y, crossing at origin
    let ray = Ray::new(Vec3::ZERO, Vec3::X);
    let (t_ray, t_line) = ray.closest_to_line(Vec3::ZERO, Vec3::Y);
    assert!(t_ray.abs() < 1e-6);
    assert!(t_line.abs() < 1e-6);
}

#[test]
fn closest_to_line_offset() {
    // Ray along +X starting at y=1, line along +Y at origin
    let ray = Ray::new(Vec3::new(0.0, 1.0, 0.0), Vec3::X);
    let (t_ray, t_line) = ray.closest_to_line(Vec3::ZERO, Vec3::Y);
    // Closest point on line to the ray start is (0, 1, 0)
    assert!(t_ray.abs() < 1e-6);
    assert!((t_line - 1.0).abs() < 1e-6);
}

#[test]
fn distance_to_segment() {
    // Ray along +Z at origin, segment from (1,0,5) to (1,0,10)
    let ray = Ray::new(Vec3::ZERO, Vec3::Z);
    let dist = ray.distance_to_segment(Vec3::new(1.0, 0.0, 5.0), Vec3::new(1.0, 0.0, 10.0));
    // Closest approach should be at z=5, distance = 1.0 (the x offset)
    assert!((dist - 1.0).abs() < 1e-4, "expected ~1.0, got {dist}");
}

#[test]
fn from_screen_center() {
    // Identity VP → screen center should produce a ray along -Z
    let vp = Mat4::IDENTITY;
    let inv_vp = vp.inverse();
    let ray = Ray::from_screen(Vec2::new(400.0, 300.0), Vec2::new(800.0, 600.0), inv_vp);
    // Center of screen → NDC (0,0) → should go roughly along -Z
    assert!(
        ray.direction.z.abs() > 0.9,
        "center ray should point along Z, got {:?}",
        ray.direction
    );
}

// ── intersect_disk ──

#[test]
fn intersect_disk_hit() {
    // Ray along -Y hitting a horizontal disk at y=0, center (0,0,0), radius 1
    let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y);
    let t = ray.intersect_disk(Vec3::ZERO, Vec3::Y, 1.0);
    assert!((t.unwrap() - 5.0).abs() < 1e-6);
}

#[test]
fn intersect_disk_miss_outside_radius() {
    // Ray along -Y but offset so it hits the plane outside the disk
    let ray = Ray::new(Vec3::new(2.0, 5.0, 0.0), Vec3::NEG_Y);
    let t = ray.intersect_disk(Vec3::ZERO, Vec3::Y, 1.0);
    assert!(t.is_none(), "should miss disk when outside radius");
}

#[test]
fn intersect_disk_edge() {
    // Ray hits exactly at the edge of the disk
    let ray = Ray::new(Vec3::new(1.0, 5.0, 0.0), Vec3::NEG_Y);
    let t = ray.intersect_disk(Vec3::ZERO, Vec3::Y, 1.0);
    assert!(t.is_some(), "should hit at edge of disk");
}

#[test]
fn intersect_disk_behind_ray() {
    // Disk is behind the ray origin
    let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::Y); // pointing away
    let t = ray.intersect_disk(Vec3::ZERO, Vec3::Y, 1.0);
    assert!(t.is_none(), "should not hit disk behind ray");
}

#[test]
fn intersect_disk_parallel() {
    // Ray parallel to disk plane
    let ray = Ray::new(Vec3::new(0.0, 1.0, 0.0), Vec3::X);
    let t = ray.intersect_disk(Vec3::ZERO, Vec3::Y, 1.0);
    assert!(t.is_none(), "parallel ray should miss");
}

#[test]
fn intersect_disk_at_height() {
    // Disk at y=3 — simulating a column top
    let ray = Ray::new(Vec3::new(0.5, 10.0, 0.5), Vec3::NEG_Y);
    let center = Vec3::new(0.5, 3.0, 0.5);
    let t = ray.intersect_disk(center, Vec3::Y, 0.55);
    assert!(t.is_some(), "should hit column top disk");
    assert!((t.unwrap() - 7.0).abs() < 1e-6);
}

// ── intersect_sphere ──

#[test]
fn intersect_sphere_hit() {
    // Ray along +X hitting unit sphere at origin
    let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::X);
    let t = ray.intersect_sphere(Vec3::ZERO, 1.0);
    assert!((t.unwrap() - 4.0).abs() < 1e-6, "should hit at t=4 (enter)");
}

#[test]
fn intersect_sphere_miss() {
    // Ray above sphere
    let ray = Ray::new(Vec3::new(-5.0, 2.0, 0.0), Vec3::X);
    let t = ray.intersect_sphere(Vec3::ZERO, 1.0);
    assert!(t.is_none(), "should miss sphere");
}

#[test]
fn intersect_sphere_inside() {
    // Ray origin inside sphere — should return far intersection
    let ray = Ray::new(Vec3::ZERO, Vec3::X);
    let t = ray.intersect_sphere(Vec3::ZERO, 1.0);
    assert!(t.is_some(), "should hit from inside");
    assert!((t.unwrap() - 1.0).abs() < 1e-6, "far intersection at t=1");
}

#[test]
fn intersect_sphere_behind() {
    // Sphere is behind the ray
    let ray = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::X);
    let t = ray.intersect_sphere(Vec3::ZERO, 1.0);
    assert!(t.is_none(), "sphere behind ray");
}

#[test]
fn intersect_sphere_tangent() {
    // Ray tangent to unit sphere at (0, 1, 0)
    let ray = Ray::new(Vec3::new(-5.0, 1.0, 0.0), Vec3::X);
    let t = ray.intersect_sphere(Vec3::ZERO, 1.0);
    // Tangent → discriminant ~ 0, should still count as a hit
    assert!(t.is_some(), "tangent should hit");
    assert!((t.unwrap() - 5.0).abs() < 1e-4);
}
