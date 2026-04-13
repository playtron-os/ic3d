//! Tests for arrow mesh primitive.

use super::Mesh;

#[test]
fn arrow_has_vertices() {
    let mesh = Mesh::arrow(1.0);
    assert!(mesh.vertex_count() > 0, "arrow mesh should have vertices");
    assert!(
        mesh.vertex_count().is_multiple_of(3),
        "vertex count should be divisible by 3 (triangle list)"
    );
}

#[test]
fn arrow_scales_with_length() {
    let short = Mesh::arrow(1.0);
    let long = Mesh::arrow(2.0);
    // Same topology, same vertex count
    assert_eq!(short.vertex_count(), long.vertex_count());
}

#[test]
fn arrow_along_positive_y() {
    let mesh = Mesh::arrow(5.0);
    let max_y = mesh
        .vertices()
        .iter()
        .map(|v| v.pos[1])
        .fold(f32::MIN, f32::max);
    let min_y = mesh
        .vertices()
        .iter()
        .map(|v| v.pos[1])
        .fold(f32::MAX, f32::min);
    assert!(
        (max_y - 5.0).abs() < 1e-4,
        "tip should be at length=5.0, got {max_y}"
    );
    assert!(min_y.abs() < 1e-4, "base should be at y=0, got {min_y}");
}

#[test]
fn normals_are_unit_length() {
    let mesh = Mesh::arrow(1.0);
    for v in mesh.vertices() {
        let len = (v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-4, "normal length {len}");
    }
}
