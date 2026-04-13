//! Tests for top-level math utilities (distance_xz, distance_xz_squared).

use super::*;
use glam::Vec3;

#[test]
fn distance_xz_squared_same_point() {
    let p = Vec3::new(3.0, 7.0, 2.0);
    assert!((distance_xz_squared(p, p)).abs() < 1e-10);
}

#[test]
fn distance_xz_squared_ignores_y() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 100.0, 0.0);
    assert!((distance_xz_squared(a, b)).abs() < 1e-10);
}

#[test]
fn distance_xz_squared_3_4_triangle() {
    let a = Vec3::new(0.0, 5.0, 0.0);
    let b = Vec3::new(3.0, 99.0, 4.0);
    assert!((distance_xz_squared(a, b) - 25.0).abs() < 1e-6);
}

#[test]
fn distance_xz_3_4_triangle() {
    let a = Vec3::new(0.0, 5.0, 0.0);
    let b = Vec3::new(3.0, 99.0, 4.0);
    assert!((distance_xz(a, b) - 5.0).abs() < 1e-6);
}

#[test]
fn distance_xz_symmetric() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 7.0);
    assert!((distance_xz(a, b) - distance_xz(b, a)).abs() < 1e-10);
}
