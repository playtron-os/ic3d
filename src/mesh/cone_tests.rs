use super::*;

#[test]
fn vertex_count() {
    let mesh = Mesh::cone(0.5, 1.0, 16);
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}

#[test]
fn min_segments_clamped() {
    let mesh = Mesh::cone(1.0, 1.0, 1);
    // Clamped to 3: side (3×3=9) + cap (3×3=9) = 18
    assert_eq!(mesh.vertices().len(), 18);
}

#[test]
fn centered_at_origin() {
    let mesh = Mesh::cone(1.0, 2.0, 16);
    let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
    for v in mesh.vertices() {
        min_y = min_y.min(v.pos[1]);
        max_y = max_y.max(v.pos[1]);
    }
    assert!((min_y - (-1.0)).abs() < 1e-5);
    assert!((max_y - 1.0).abs() < 1e-5);
}

#[test]
fn normals_are_unit_length() {
    let mesh = Mesh::cone(0.5, 1.0, 8);
    for v in mesh.vertices() {
        let len = (v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-4, "normal length {len}");
    }
}

#[test]
fn radius_bounds() {
    let r = 2.0;
    let mesh = Mesh::cone(r, 1.0, 16);
    for v in mesh.vertices() {
        let xz = (v.pos[0].powi(2) + v.pos[2].powi(2)).sqrt();
        assert!(xz <= r + 1e-5, "vertex exceeded radius: {xz}");
    }
}
