use super::*;

#[test]
fn has_6_vertices() {
    let mesh = Mesh::plane(1.0, 1.0);
    assert_eq!(mesh.vertices().len(), 6); // 2 triangles
}
