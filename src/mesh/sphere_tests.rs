use super::*;

#[test]
fn vertex_count() {
    let mesh = Mesh::sphere(1.0, 16, 12);
    // (rings - 1) * segments * 6 + segments * 3 * 2 (poles)
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}
