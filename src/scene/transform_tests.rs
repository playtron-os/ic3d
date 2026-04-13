use super::*;
use glam::{Quat, Vec3};

#[test]
fn default_is_identity() {
    let t = Transform::default();
    assert_eq!(t.position, Vec3::ZERO);
    assert_eq!(t.rotation, Quat::IDENTITY);
    assert_eq!(t.scale, Vec3::ONE);
}

#[test]
fn new_equals_default() {
    let a = Transform::new();
    let b = Transform::default();
    assert_eq!(a.position, b.position);
    assert_eq!(a.rotation, b.rotation);
    assert_eq!(a.scale, b.scale);
}

#[test]
fn position_builder() {
    let t = Transform::new().position(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(t.position, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn scale_builder() {
    let t = Transform::new().scale(Vec3::new(2.0, 3.0, 4.0));
    assert_eq!(t.scale, Vec3::new(2.0, 3.0, 4.0));
}

#[test]
fn uniform_scale_builder() {
    let t = Transform::new().uniform_scale(5.0);
    assert_eq!(t.scale, Vec3::splat(5.0));
}

#[test]
fn identity_matrix_is_identity() {
    let t = Transform::new();
    let m = t.matrix();
    let expected = glam::Mat4::IDENTITY;
    for i in 0..4 {
        for j in 0..4 {
            assert!(
                (m.col(i)[j] - expected.col(i)[j]).abs() < 1e-6,
                "matrix [{i},{j}] mismatch"
            );
        }
    }
}

#[test]
fn translation_in_matrix() {
    let t = Transform::new().position(Vec3::new(10.0, 20.0, 30.0));
    let m = t.matrix();
    // Translation is in column 3
    assert!((m.col(3).x - 10.0).abs() < 1e-6);
    assert!((m.col(3).y - 20.0).abs() < 1e-6);
    assert!((m.col(3).z - 30.0).abs() < 1e-6);
}

#[test]
fn scale_in_matrix() {
    let t = Transform::new().scale(Vec3::new(2.0, 3.0, 4.0));
    let m = t.matrix();
    assert!((m.col(0).x - 2.0).abs() < 1e-6);
    assert!((m.col(1).y - 3.0).abs() < 1e-6);
    assert!((m.col(2).z - 4.0).abs() < 1e-6);
}

#[test]
fn normal_matrix_identity() {
    let t = Transform::new();
    let n = t.normal_matrix();
    let expected = glam::Mat3::IDENTITY;
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                (n.col(i)[j] - expected.col(i)[j]).abs() < 1e-6,
                "normal_mat [{i},{j}] mismatch"
            );
        }
    }
}

#[test]
fn normal_matrix_with_non_uniform_scale() {
    let t = Transform::new().scale(Vec3::new(2.0, 1.0, 1.0));
    let n = t.normal_matrix();
    // Normal matrix should have 1/scale on diagonal for non-uniform scale
    assert!((n.col(0).x - 0.5).abs() < 1e-6);
    assert!((n.col(1).y - 1.0).abs() < 1e-6);
    assert!((n.col(2).z - 1.0).abs() < 1e-6);
}

#[test]
fn to_instance_identity_material() {
    let t = Transform::new();
    let inst = t.to_instance([1.0, 0.0, 0.0, 32.0]);
    assert_eq!(inst.material, [1.0, 0.0, 0.0, 32.0]);
    // Model should be identity
    assert!((inst.model[0][0] - 1.0).abs() < 1e-6);
    assert!((inst.model[3][3] - 1.0).abs() < 1e-6);
}

#[test]
fn chained_builders() {
    let t = Transform::new()
        .position(Vec3::new(1.0, 2.0, 3.0))
        .uniform_scale(2.0)
        .rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
    assert_eq!(t.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(t.scale, Vec3::splat(2.0));
    // Just verify it produces a valid matrix
    let m = t.matrix();
    assert!(m.determinant().abs() > 0.0);
}
