//! UV sphere primitive.

use super::Mesh;
use crate::gpu_types::Vertex;
use std::f32::consts::PI;

impl Mesh {
    /// UV sphere centered at the origin.
    ///
    /// - `segments`: horizontal divisions (longitude, min 3)
    /// - `rings`: vertical divisions (latitude, min 2)
    #[must_use]
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let segments = segments.max(3);
        let rings = rings.max(2);
        let mut verts = Vec::new();

        // Generate vertex grid
        let mut grid: Vec<Vec<[f32; 3]>> = Vec::new();
        for j in 0..=rings {
            let v = j as f32 / rings as f32;
            let phi = v * PI;
            let mut row = Vec::new();
            for i in 0..=segments {
                let u = i as f32 / segments as f32;
                let theta = u * 2.0 * PI;
                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos();
                let z = radius * phi.sin() * theta.sin();
                row.push([x, y, z]);
            }
            grid.push(row);
        }

        // Build triangles
        for j in 0..rings {
            for i in 0..segments {
                let i1 = (i + 1) as usize;
                let j1 = (j + 1) as usize;
                let i0 = i as usize;
                let j0 = j as usize;

                let p00 = grid[j0][i0];
                let p10 = grid[j0][i1];
                let p01 = grid[j1][i0];
                let p11 = grid[j1][i1];

                let n = |p: [f32; 3]| -> [f32; 3] {
                    let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
                    if len > 0.0 {
                        [p[0] / len, p[1] / len, p[2] / len]
                    } else {
                        [0.0, 1.0, 0.0]
                    }
                };

                let uv = |gi: usize, gj: usize| -> [f32; 2] {
                    [gi as f32 / segments as f32, gj as f32 / rings as f32]
                };

                // First triangle (skip degenerate at top pole)
                if j > 0 {
                    verts.push(Vertex {
                        pos: p00,
                        normal: n(p00),
                        uv: uv(i0, j0),
                    });
                    verts.push(Vertex {
                        pos: p10,
                        normal: n(p10),
                        uv: uv(i1, j0),
                    });
                    verts.push(Vertex {
                        pos: p11,
                        normal: n(p11),
                        uv: uv(i1, j1),
                    });
                }

                // Second triangle (skip degenerate at bottom pole)
                if j < rings - 1 {
                    verts.push(Vertex {
                        pos: p00,
                        normal: n(p00),
                        uv: uv(i0, j0),
                    });
                    verts.push(Vertex {
                        pos: p11,
                        normal: n(p11),
                        uv: uv(i1, j1),
                    });
                    verts.push(Vertex {
                        pos: p01,
                        normal: n(p01),
                        uv: uv(i0, j1),
                    });
                }
            }
        }

        Self {
            vertices: verts,
            label: "iced3d sphere".into(),
        }
    }
}

#[cfg(test)]
#[path = "sphere_tests.rs"]
mod tests;
