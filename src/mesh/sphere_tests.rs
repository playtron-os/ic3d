use super::*;

#[test]
fn vertex_count() {
    let mesh = Mesh::sphere(1.0, 16, 12);
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}

#[test]
fn min_segments_clamped() {
    let mesh = Mesh::sphere(1.0, 1, 1);
    // Clamped to segments=3, rings=2
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}

#[test]
fn vertices_on_sphere_surface() {
    let r = 2.0;
    let mesh = Mesh::sphere(r, 16, 12);
    for v in mesh.vertices() {
        let dist = (v.pos[0].powi(2) + v.pos[1].powi(2) + v.pos[2].powi(2)).sqrt();
        assert!(
            (dist - r).abs() < 1e-4,
            "vertex at distance {dist}, expected {r}"
        );
    }
}

#[test]
fn normals_are_unit_length() {
    let mesh = Mesh::sphere(1.0, 8, 6);
    for v in mesh.vertices() {
        let len = (v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-4, "normal length {len}");
    }
}

#[test]
fn normals_point_outward() {
    let mesh = Mesh::sphere(1.0, 8, 6);
    for v in mesh.vertices() {
        let dot = v.pos[0] * v.normal[0] + v.pos[1] * v.normal[1] + v.pos[2] * v.normal[2];
        assert!(dot > 0.0, "normal should point outward, dot={dot}");
    }
}

#[test]
fn more_segments_more_vertices() {
    let lo = Mesh::sphere(1.0, 4, 3);
    let hi = Mesh::sphere(1.0, 16, 12);
    assert!(hi.vertices().len() > lo.vertices().len());
}
