//! Axial hex grid layout generator.
//!
//! Produces world-space positions for a flat-top hexagonal grid, useful for
//! procedural environments, tile maps, and crystal fields.
//!
//! ```rust,ignore
//! use ic3d::math::hex_grid::hex_grid;
//!
//! let cells = hex_grid(0.55, 0.04, 12);
//! for cell in &cells {
//!     println!("({}, {}) at world ({:.1}, {:.1}), dist={:.1}", cell.q, cell.r, cell.x, cell.z, cell.distance);
//! }
//! ```

/// A single cell in an axial hex grid.
#[derive(Debug, Clone, Copy)]
pub struct HexCell {
    /// Axial coordinate q.
    pub q: i32,
    /// Axial coordinate r.
    pub r: i32,
    /// World-space X position.
    pub x: f32,
    /// World-space Z position.
    pub z: f32,
    /// Distance from the grid center.
    pub distance: f32,
}

/// Generate a flat-top axial hex grid centered at the origin.
///
/// - `radius` — hex cell outer radius (center to vertex)
/// - `gap` — spacing between adjacent hexagons
/// - `rings` — number of concentric rings from the center (0 = center cell only)
///
/// Returns cells sorted by distance from center.
#[must_use]
pub fn hex_grid(radius: f32, gap: f32, rings: i32) -> Vec<HexCell> {
    let spacing = (radius + gap) * 3.0_f32.sqrt();
    let mut cells = Vec::new();

    for q in -rings..=rings {
        let r_min = (-rings).max(-q - rings);
        let r_max = rings.min(-q + rings);
        for r in r_min..=r_max {
            let x = spacing * (q as f32 + r as f32 * 0.5);
            let z = spacing * r as f32 * 0.866_025_4;
            let distance = (x * x + z * z).sqrt();

            cells.push(HexCell {
                q,
                r,
                x,
                z,
                distance,
            });
        }
    }

    cells.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    cells
}

#[cfg(test)]
#[path = "hex_grid_tests.rs"]
mod tests;
