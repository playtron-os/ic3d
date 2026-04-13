use super::*;

#[test]
fn lerp_endpoints() {
    assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
}

#[test]
fn lerp_midpoint() {
    assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < f32::EPSILON);
}

#[test]
fn lerp_extrapolates() {
    assert!((lerp(0.0, 10.0, 2.0) - 20.0).abs() < f32::EPSILON);
}

#[test]
fn inverse_lerp_endpoints() {
    assert_eq!(inverse_lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(inverse_lerp(0.0, 10.0, 10.0), 1.0);
}

#[test]
fn inverse_lerp_midpoint() {
    assert!((inverse_lerp(0.0, 10.0, 5.0) - 0.5).abs() < f32::EPSILON);
}

#[test]
fn remap_basic() {
    let result = remap(5.0, 0.0, 10.0, 100.0, 200.0);
    assert!((result - 150.0).abs() < f32::EPSILON);
}

#[test]
fn remap_identity() {
    let result = remap(3.0, 0.0, 10.0, 0.0, 10.0);
    assert!((result - 3.0).abs() < f32::EPSILON);
}
