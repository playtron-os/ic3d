use super::*;

#[test]
fn vertex_count() {
    let mesh = Mesh::torus(0.5, 0.2, 16, 8);
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}

#[test]
fn min_segments_clamped() {
    let mesh = Mesh::torus(1.0, 0.3, 1, 1);
    // Clamped to 3×3: 3 major × 3 minor × 6 verts/quad = 54
    assert_eq!(mesh.vertices().len(), 54);
}

#[test]
fn expected_vertex_count() {
    // major × minor × 6 (two triangles per quad)
    let mesh = Mesh::torus(1.0, 0.3, 8, 4);
    assert_eq!(mesh.vertices().len(), 8 * 4 * 6);
}

#[test]
fn lies_in_xz_plane() {
    let major = 2.0;
    let minor = 0.5;
    let mesh = Mesh::torus(major, minor, 16, 8);
    for v in mesh.vertices() {
        let y_max = minor + 1e-4;
        assert!(
            v.pos[1].abs() <= y_max,
            "y={} exceeded minor radius",
            v.pos[1]
        );
    }
}

#[test]
fn normals_are_unit_length() {
    let mesh = Mesh::torus(1.0, 0.3, 8, 6);
    for v in mesh.vertices() {
        let len = (v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-4, "normal length {len}");
    }
}
