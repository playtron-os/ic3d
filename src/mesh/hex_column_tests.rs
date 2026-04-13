use super::*;

#[test]
fn vertex_count_divisible_by_3() {
    let mesh = Mesh::hex_column(1.0);
    assert_eq!(mesh.vertex_count() % 3, 0, "must be triangle list");
}

#[test]
fn has_expected_geometry() {
    // 6 side quads × 2 triangles × 3 verts = 36
    // + top cap (4 triangles × 3 verts) = 12
    // Total = 48 (no bottom cap)
    let mesh = Mesh::hex_column(0.5);
    assert_eq!(
        mesh.vertex_count(),
        48,
        "expected 48 vertices (sides + top cap)"
    );
}

#[test]
fn radius_scales_geometry() {
    let small = Mesh::hex_column(0.5);
    let large = Mesh::hex_column(2.0);

    let max_extent = |m: &Mesh| -> f32 {
        m.vertices()
            .iter()
            .map(|v| (v.pos[0] * v.pos[0] + v.pos[1] * v.pos[1]).sqrt())
            .fold(0.0_f32, f32::max)
    };

    let small_r = max_extent(&small);
    let large_r = max_extent(&large);
    assert!(
        large_r > small_r * 3.0,
        "larger radius should produce larger geometry: {large_r} vs {small_r}"
    );
}

#[test]
fn label_is_hex_column() {
    let mesh = Mesh::hex_column(1.0);
    assert_eq!(mesh.label(), "hex_column");
}

#[test]
fn has_top_cap_with_upward_normals() {
    let mesh = Mesh::hex_column(1.0);
    // Top cap vertices sit at z=1.0 and should have normals pointing +Z
    let top_verts: Vec<_> = mesh
        .vertices()
        .iter()
        .filter(|v| (v.pos[2] - 1.0).abs() < 1e-6 && v.normal[2] > 0.5)
        .collect();
    assert!(
        top_verts.len() >= 12,
        "expected at least 12 top-cap vertices with +Z normals, got {}",
        top_verts.len()
    );
}

#[test]
fn no_bottom_cap() {
    let mesh = Mesh::hex_column(1.0);
    // No vertices at z=0 should have downward-facing normals (no bottom cap)
    let bottom_cap_verts = mesh
        .vertices()
        .iter()
        .filter(|v| v.pos[2].abs() < 1e-6 && v.normal[2] < -0.5)
        .count();
    assert_eq!(bottom_cap_verts, 0, "should have no bottom cap vertices");
}
