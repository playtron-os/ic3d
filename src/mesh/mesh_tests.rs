use super::*;

// ────────────────── Mesh accessors ──────────────────

#[test]
fn mesh_label() {
    let mesh = Mesh::cube(1.0);
    assert!(mesh.label().contains("cube"));
}

#[test]
fn mesh_custom() {
    let v = crate::gpu_types::Vertex {
        pos: [0.0, 0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    };
    let mesh = Mesh::custom(vec![v, v, v], "test");
    assert_eq!(mesh.vertices().len(), 3);
    assert_eq!(mesh.label(), "test");
}

// ────────────────── Mirror Y ──────────────────

#[test]
fn mirror_y_flips_y() {
    let mesh = Mesh::plane(1.0, 1.0);
    let mirrored = mesh.mirror_y();
    for (orig, mir) in mesh.vertices().iter().zip(mirrored.vertices()) {
        assert!((orig.pos[1] - (-mir.pos[1])).abs() < 1e-6);
    }
}

#[test]
fn mirror_y_negates_normals() {
    let mesh = Mesh::plane(1.0, 1.0);
    let mirrored = mesh.mirror_y();
    for (orig, mir) in mesh.vertices().iter().zip(mirrored.vertices()) {
        assert!((orig.normal[0] - (-mir.normal[0])).abs() < 1e-6);
        assert!((orig.normal[1] - (-mir.normal[1])).abs() < 1e-6);
        assert!((orig.normal[2] - (-mir.normal[2])).abs() < 1e-6);
    }
}

#[test]
fn mirror_y_preserves_vertex_count() {
    let mesh = Mesh::cube(1.0);
    let mirrored = mesh.mirror_y();
    assert_eq!(mesh.vertices().len(), mirrored.vertices().len());
}

// ────────────────── MeshBuilder ──────────────────

#[test]
fn builder_triangle_auto_normal() {
    let mesh = MeshBuilder::new("test")
        .triangle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
        .build();
    assert_eq!(mesh.vertices().len(), 3);
    // Auto-normal for this triangle should point in +Z
    let n = mesh.vertices()[0].normal;
    assert!(n[2] > 0.0, "expected +Z normal, got {n:?}");
}

#[test]
fn builder_triangle_with_normal() {
    let mesh = MeshBuilder::new("test")
        .triangle_with_normal(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, -1.0],
        )
        .build();
    let n = mesh.vertices()[0].normal;
    assert!((n[2] - (-1.0)).abs() < 1e-6);
}

#[test]
fn builder_quad_makes_6_vertices() {
    let mesh = MeshBuilder::new("test")
        .quad(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        )
        .build();
    assert_eq!(mesh.vertices().len(), 6);
}

#[test]
fn builder_extrude_pentagon() {
    let points: Vec<[f32; 2]> = (0..5)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / 5.0;
            [angle.cos(), angle.sin()]
        })
        .collect();
    let mesh = MeshBuilder::new("pentagon").extrude(&points, 1.0).build();
    // Should have top face + sides
    assert!(!mesh.vertices().is_empty());
    assert_eq!(mesh.vertices().len() % 3, 0);
}

#[test]
fn builder_extrude_triangle_shape() {
    let mesh = MeshBuilder::new("tri")
        .extrude(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]], 2.0)
        .build();
    // Top face: 1 triangle = 3 verts, sides: 3 edges × 2 tris × 3 verts = 18
    assert_eq!(mesh.vertices().len(), 3 + 18);
}

#[test]
fn builder_chaining() {
    let mesh = MeshBuilder::new("multi")
        .triangle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
        .quad(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        )
        .build();
    assert_eq!(mesh.vertices().len(), 3 + 6);
}

// ────────────────── Winding correctness ──────────────────

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

            let cross_len_sq = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
            if cross_len_sq < 1e-12 {
                continue; // degenerate triangle (e.g. sphere pole)
            }

            // Average vertex normal
            let avg_n = [
                (v0.normal[0] + v1.normal[0] + v2.normal[0]) / 3.0,
                (v0.normal[1] + v1.normal[1] + v2.normal[1]) / 3.0,
                (v0.normal[2] + v1.normal[2] + v2.normal[2]) / 3.0,
            ];

            let dot = cross[0] * avg_n[0] + cross[1] * avg_n[1] + cross[2] * avg_n[2];

            assert!(
                dot > 0.0,
                "{name}: triangle {} has wrong winding (cross · avg_normal = {dot:.6})",
                i / 3
            );
        }
    }
}
