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

// ──────────── torus_arc ────────────

#[test]
fn arc_half_has_half_verts() {
    let full = Mesh::torus(1.0, 0.3, 16, 8);
    let half = Mesh::torus_arc(1.0, 0.3, 0.0, std::f32::consts::PI, 8, 8);
    // Half arc with half the major segments should have half the vertices.
    assert_eq!(half.vertices().len(), full.vertices().len() / 2);
}

#[test]
fn arc_full_sweep_matches_torus() {
    let full = Mesh::torus(1.0, 0.3, 12, 6);
    let arc = Mesh::torus_arc(1.0, 0.3, 0.0, std::f32::consts::TAU, 12, 6);
    assert_eq!(arc.vertices().len(), full.vertices().len());
}

#[test]
fn arc_vertex_count_divisible_by_3() {
    let arc = Mesh::torus_arc(0.5, 0.1, 0.5, 2.0, 10, 5);
    assert_eq!(arc.vertices().len() % 3, 0);
}

#[test]
fn arc_min_segments_clamped() {
    let arc = Mesh::torus_arc(1.0, 0.3, 0.0, 1.0, 1, 1);
    assert_eq!(arc.vertices().len(), 3 * 3 * 6);
}

// ──────────── disc_arc ────────────

#[test]
fn disc_arc_vertex_count() {
    let disc = Mesh::disc_arc(1.0, 0.0, std::f32::consts::PI, 12);
    // 12 segments × 3 verts per triangle fan slice
    assert_eq!(disc.vertices().len(), 12 * 3);
}

#[test]
fn disc_arc_lies_in_xz_plane() {
    let disc = Mesh::disc_arc(1.0, 0.0, std::f32::consts::TAU, 16);
    for v in disc.vertices() {
        assert!(
            v.pos[1].abs() < 1e-6,
            "disc should be flat at y=0, got y={}",
            v.pos[1]
        );
    }
}

#[test]
fn disc_arc_within_radius() {
    let radius = 0.55;
    let disc = Mesh::disc_arc(radius, 0.0, std::f32::consts::TAU, 24);
    for v in disc.vertices() {
        let r = (v.pos[0].powi(2) + v.pos[2].powi(2)).sqrt();
        assert!(
            r <= radius + 1e-4,
            "vertex at r={r} exceeds radius={radius}"
        );
    }
}

#[test]
fn disc_arc_min_segments_clamped() {
    let disc = Mesh::disc_arc(1.0, 0.0, 1.0, 1);
    assert_eq!(disc.vertices().len(), 3 * 3);
}
