use super::*;

#[test]
fn vertex_count() {
    let mesh = Mesh::cylinder(0.5, 1.0, 16);
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}
