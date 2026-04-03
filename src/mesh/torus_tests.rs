use super::*;

#[test]
fn vertex_count() {
    let mesh = Mesh::torus(0.5, 0.2, 16, 8);
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}
