use super::*;

#[test]
fn vertex_count() {
    let mesh = Mesh::cylinder(0.5, 1.0, 16);
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}

#[test]
fn min_segments_clamped() {
    let mesh = Mesh::cylinder(1.0, 1.0, 1);
    // Clamped to 3: wall (3×6=18) + top (3×3=9) + bottom (3×3=9) = 36
    assert_eq!(mesh.vertices().len(), 36);
}

#[test]
fn centered_at_origin() {
    let mesh = Mesh::cylinder(1.0, 4.0, 8);
    let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
    for v in mesh.vertices() {
        min_y = min_y.min(v.pos[1]);
        max_y = max_y.max(v.pos[1]);
    }
    assert!((min_y - (-2.0)).abs() < 1e-5);
    assert!((max_y - 2.0).abs() < 1e-5);
}

#[test]
fn radius_bounds() {
    let r = 1.5;
    let mesh = Mesh::cylinder(r, 1.0, 16);
    for v in mesh.vertices() {
        let xz = (v.pos[0].powi(2) + v.pos[2].powi(2)).sqrt();
        assert!(xz <= r + 1e-5, "vertex exceeded radius: {xz}");
    }
}

#[test]
fn normals_are_unit_length() {
    let mesh = Mesh::cylinder(0.5, 1.0, 8);
    for v in mesh.vertices() {
        let len = (v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2)).sqrt();
        assert!((len - 1.0).abs() < 1e-4, "normal length {len}");
    }
}
