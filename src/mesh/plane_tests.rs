use super::*;

#[test]
fn has_6_vertices() {
    let mesh = Mesh::plane(1.0, 1.0);
    assert_eq!(mesh.vertices().len(), 6); // 2 triangles
}

#[test]
fn normals_point_up() {
    let mesh = Mesh::plane(2.0, 3.0);
    for v in mesh.vertices() {
        assert!((v.normal[0]).abs() < 1e-6);
        assert!((v.normal[1] - 1.0).abs() < 1e-6);
        assert!((v.normal[2]).abs() < 1e-6);
    }
}

#[test]
fn centered_at_origin() {
    let mesh = Mesh::plane(4.0, 6.0);
    let (mut min_x, mut max_x, mut min_z, mut max_z) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for v in mesh.vertices() {
        min_x = min_x.min(v.pos[0]);
        max_x = max_x.max(v.pos[0]);
        min_z = min_z.min(v.pos[2]);
        max_z = max_z.max(v.pos[2]);
    }
    assert!((min_x - (-2.0)).abs() < 1e-5);
    assert!((max_x - 2.0).abs() < 1e-5);
    assert!((min_z - (-3.0)).abs() < 1e-5);
    assert!((max_z - 3.0).abs() < 1e-5);
}

#[test]
fn y_is_zero() {
    let mesh = Mesh::plane(1.0, 1.0);
    for v in mesh.vertices() {
        assert!((v.pos[1]).abs() < 1e-6);
    }
}
