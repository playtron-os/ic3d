//! Capped cylinder primitive.

use super::Mesh;
use crate::gpu_types::Vertex;
use std::f32::consts::PI;

impl Mesh {
    /// Cylinder along the Y axis, centered at the origin.
    ///
    /// - `segments`: radial divisions (min 3)
    ///
    /// Produces a capped cylinder (top + bottom discs + side wall).
    #[must_use]
    pub fn cylinder(radius: f32, height: f32, segments: u32) -> Self {
        let segments = segments.max(3);
        let half_h = height * 0.5;
        let mut verts = Vec::new();

        // Side wall
        for i in 0..segments {
            let a0 = 2.0 * PI * i as f32 / segments as f32;
            let a1 = 2.0 * PI * (i + 1) as f32 / segments as f32;

            let (c0, s0) = (a0.cos(), a0.sin());
            let (c1, s1) = (a1.cos(), a1.sin());

            let n0 = [c0, 0.0, s0];
            let n1 = [c1, 0.0, s1];

            let u0 = i as f32 / segments as f32;
            let u1 = (i + 1) as f32 / segments as f32;

            let p0t = [radius * c0, half_h, radius * s0];
            let p1t = [radius * c1, half_h, radius * s1];
            let p0b = [radius * c0, -half_h, radius * s0];
            let p1b = [radius * c1, -half_h, radius * s1];

            // Two triangles per segment
            verts.push(Vertex {
                pos: p0t,
                normal: n0,
                uv: [u0, 0.0],
            });
            verts.push(Vertex {
                pos: p1t,
                normal: n1,
                uv: [u1, 0.0],
            });
            verts.push(Vertex {
                pos: p1b,
                normal: n1,
                uv: [u1, 1.0],
            });
            verts.push(Vertex {
                pos: p0t,
                normal: n0,
                uv: [u0, 0.0],
            });
            verts.push(Vertex {
                pos: p1b,
                normal: n1,
                uv: [u1, 1.0],
            });
            verts.push(Vertex {
                pos: p0b,
                normal: n0,
                uv: [u0, 1.0],
            });
        }

        // Top cap
        let top_n = [0.0, 1.0, 0.0];
        let top_center = [0.0, half_h, 0.0];
        for i in 0..segments {
            let a0 = 2.0 * PI * i as f32 / segments as f32;
            let a1 = 2.0 * PI * (i + 1) as f32 / segments as f32;
            let p0 = [radius * a0.cos(), half_h, radius * a0.sin()];
            let p1 = [radius * a1.cos(), half_h, radius * a1.sin()];
            verts.push(Vertex {
                pos: top_center,
                normal: top_n,
                uv: [0.5, 0.5],
            });
            verts.push(Vertex {
                pos: p1,
                normal: top_n,
                uv: [a1.cos() * 0.5 + 0.5, a1.sin() * 0.5 + 0.5],
            });
            verts.push(Vertex {
                pos: p0,
                normal: top_n,
                uv: [a0.cos() * 0.5 + 0.5, a0.sin() * 0.5 + 0.5],
            });
        }

        // Bottom cap
        let bot_n = [0.0, -1.0, 0.0];
        let bot_center = [0.0, -half_h, 0.0];
        for i in 0..segments {
            let a0 = 2.0 * PI * i as f32 / segments as f32;
            let a1 = 2.0 * PI * (i + 1) as f32 / segments as f32;
            let p0 = [radius * a0.cos(), -half_h, radius * a0.sin()];
            let p1 = [radius * a1.cos(), -half_h, radius * a1.sin()];
            verts.push(Vertex {
                pos: bot_center,
                normal: bot_n,
                uv: [0.5, 0.5],
            });
            verts.push(Vertex {
                pos: p0,
                normal: bot_n,
                uv: [a0.cos() * 0.5 + 0.5, a0.sin() * 0.5 + 0.5],
            });
            verts.push(Vertex {
                pos: p1,
                normal: bot_n,
                uv: [a1.cos() * 0.5 + 0.5, a1.sin() * 0.5 + 0.5],
            });
        }

        Self {
            vertices: verts,
            label: "iced3d cylinder".into(),
        }
    }
}

#[cfg(test)]
#[path = "cylinder_tests.rs"]
mod tests;
