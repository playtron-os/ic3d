use super::*;

#[test]
fn zero_rings_single_cell() {
    let cells = hex_grid(1.0, 0.0, 0);
    assert_eq!(cells.len(), 1);
    assert_eq!(cells[0].q, 0);
    assert_eq!(cells[0].r, 0);
    assert!(cells[0].distance < 1e-6);
}

#[test]
fn one_ring_has_7_cells() {
    // Center + 6 neighbors
    let cells = hex_grid(1.0, 0.0, 1);
    assert_eq!(cells.len(), 7);
}

#[test]
fn two_rings_has_19_cells() {
    // 1 + 6 + 12
    let cells = hex_grid(1.0, 0.0, 2);
    assert_eq!(cells.len(), 19);
}

#[test]
fn cell_count_formula() {
    // Hex grid with n rings has 3n² + 3n + 1 cells
    for n in 0..=6 {
        let expected = (3 * n * n + 3 * n + 1) as usize;
        let cells = hex_grid(0.5, 0.04, n);
        assert_eq!(cells.len(), expected, "rings={n}");
    }
}

#[test]
fn center_cell_at_origin() {
    let cells = hex_grid(1.0, 0.1, 3);
    let center = cells.iter().find(|c| c.q == 0 && c.r == 0).unwrap();
    assert!(center.x.abs() < 1e-6);
    assert!(center.z.abs() < 1e-6);
    assert!(center.distance < 1e-6);
}

#[test]
fn sorted_by_distance() {
    let cells = hex_grid(0.55, 0.04, 5);
    for pair in cells.windows(2) {
        assert!(
            pair[0].distance <= pair[1].distance,
            "not sorted: {} > {}",
            pair[0].distance,
            pair[1].distance
        );
    }
}

#[test]
fn gap_increases_spacing() {
    let tight = hex_grid(1.0, 0.0, 1);
    let loose = hex_grid(1.0, 0.5, 1);

    let max_dist =
        |cells: &[HexCell]| -> f32 { cells.iter().map(|c| c.distance).fold(0.0_f32, f32::max) };

    assert!(
        max_dist(&loose) > max_dist(&tight),
        "larger gap should spread cells further"
    );
}

#[test]
fn radius_affects_spacing() {
    let small = hex_grid(0.5, 0.0, 1);
    let large = hex_grid(2.0, 0.0, 1);

    let max_dist =
        |cells: &[HexCell]| -> f32 { cells.iter().map(|c| c.distance).fold(0.0_f32, f32::max) };

    assert!(
        max_dist(&large) > max_dist(&small) * 3.0,
        "larger radius should produce larger grid"
    );
}
