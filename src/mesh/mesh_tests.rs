use super::*;

/// Verify that every triangle in every built-in mesh has winding consistent
/// with its vertex normals. The geometric face normal (edge1 × edge2 via
/// the right-hand rule) must align with the average vertex normal (positive
/// dot product), confirming CCW winding for outward-facing triangles.
///
/// Degenerate triangles (zero-area, e.g. at sphere poles) are skipped.
#[test]
fn all_meshes_have_correct_winding() {
    let meshes = vec![
        ("cube", Mesh::cube(1.0)),
        ("sphere", Mesh::sphere(0.5, 16, 12)),
        ("cylinder", Mesh::cylinder(0.5, 1.0, 16)),
        ("cone", Mesh::cone(0.5, 1.0, 16)),
        ("torus", Mesh::torus(0.5, 0.2, 16, 8)),
        ("plane", Mesh::plane(1.0, 1.0)),
    ];

    for (name, mesh) in &meshes {
        let verts = mesh.vertices();
        assert!(
            verts.len() % 3 == 0,
            "{name}: vertex count {} not a multiple of 3",
            verts.len()
        );

        for i in (0..verts.len()).step_by(3) {
            let v0 = &verts[i];
            let v1 = &verts[i + 1];
            let v2 = &verts[i + 2];

            // Edge vectors
            let e1 = [
                v1.pos[0] - v0.pos[0],
                v1.pos[1] - v0.pos[1],
                v1.pos[2] - v0.pos[2],
            ];
            let e2 = [
                v2.pos[0] - v0.pos[0],
                v2.pos[1] - v0.pos[1],
                v2.pos[2] - v0.pos[2],
            ];

            // Geometric face normal (cross product)
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];

            let cross_len_sq =
                cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
            if cross_len_sq < 1e-12 {
                continue; // degenerate triangle (e.g. sphere pole)
            }

            // Average vertex normal
            let avg_n = [
                (v0.normal[0] + v1.normal[0] + v2.normal[0]) / 3.0,
                (v0.normal[1] + v1.normal[1] + v2.normal[1]) / 3.0,
                (v0.normal[2] + v1.normal[2] + v2.normal[2]) / 3.0,
            ];

            let dot =
                cross[0] * avg_n[0] + cross[1] * avg_n[1] + cross[2] * avg_n[2];

            assert!(
                dot > 0.0,
                "{name}: triangle {} has wrong winding (cross · avg_normal = {dot:.6})",
                i / 3
            );
        }
    }
}
