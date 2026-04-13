use super::*;

#[test]
fn has_36_vertices() {
    let mesh = Mesh::cube(1.0);
    assert_eq!(mesh.vertices().len(), 36); // 6 faces × 2 triangles × 3 verts
}

#[test]
fn centered_at_origin() {
    let mesh = Mesh::cube(2.0);
    let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in mesh.vertices() {
        for i in 0..3 {
            min[i] = min[i].min(v.pos[i]);
            max[i] = max[i].max(v.pos[i]);
        }
    }
    for i in 0..3 {
        assert!((min[i] - (-1.0)).abs() < 1e-5);
        assert!((max[i] - 1.0).abs() < 1e-5);
    }
}

#[test]
fn normals_are_axis_aligned() {
    let mesh = Mesh::cube(1.0);
    for v in mesh.vertices() {
        let non_zero = v.normal.iter().filter(|&&n| n.abs() > 0.5).count();
        assert_eq!(
            non_zero, 1,
            "cube normal should have exactly one non-zero component: {:?}",
            v.normal
        );
    }
}

#[test]
fn scales_with_size() {
    let small = Mesh::cube(1.0);
    let big = Mesh::cube(4.0);
    assert_eq!(small.vertices().len(), big.vertices().len());
    // Big cube should have vertices at ±2.0
    let max_coord = big
        .vertices()
        .iter()
        .map(|v| v.pos[0])
        .fold(f32::MIN, f32::max);
    assert!((max_coord - 2.0).abs() < 1e-5);
}
