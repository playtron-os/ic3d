//! Flat-top hexagonal prism primitive.

use super::{Mesh, MeshBuilder};
use std::f32::consts::PI;

impl Mesh {
    /// Flat-top hexagonal column (prism) with unit height along Z.
    ///
    /// The hex is centered at the origin in the XY plane with the given
    /// outer `radius` (center to vertex). Height is 1.0 along Z — scale
    /// via [`Transform`](crate::Transform) to the desired column height,
    /// and rotate to stand upright (Z → Y).
    ///
    /// Generates top cap and six side-wall quads. No bottom cap — the
    /// base is typically hidden by a ground plane.
    #[must_use]
    pub fn hex_column(radius: f32) -> Self {
        let mut points = Vec::with_capacity(6);
        for i in 0..6 {
            let angle = PI / 3.0 * i as f32;
            points.push([radius * angle.cos(), radius * angle.sin()]);
        }
        MeshBuilder::new("hex_column").extrude(&points, 1.0).build()
    }
}

#[cfg(test)]
#[path = "hex_column_tests.rs"]
mod tests;
