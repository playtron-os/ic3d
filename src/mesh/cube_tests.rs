use super::*;

#[test]
fn has_36_vertices() {
    let mesh = Mesh::cube(1.0);
    assert_eq!(mesh.vertices().len(), 36); // 6 faces × 2 triangles × 3 verts
}
