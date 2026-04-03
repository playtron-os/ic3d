//! Cone primitive.

use super::Mesh;
use crate::gpu_types::Vertex;
use std::f32::consts::PI;

impl Mesh {
    /// Cone along the Y axis with tip at top, centered at the origin.
    ///
    /// - `segments`: radial divisions (min 3)
    ///
    /// Produces a cone with a flat bottom disc.
    #[must_use]
    pub fn cone(radius: f32, height: f32, segments: u32) -> Self {
        let segments = segments.max(3);
        let half_h = height * 0.5;
        let mut verts = Vec::new();

        let tip = [0.0_f32, half_h, 0.0];
        let slope_len = (radius * radius + height * height).sqrt();
        let ny = radius / slope_len;
        let nr = height / slope_len;

        // Side wall
        for i in 0..segments {
            let a0 = 2.0 * PI * i as f32 / segments as f32;
            let a1 = 2.0 * PI * (i + 1) as f32 / segments as f32;
            let (c0, s0) = (a0.cos(), a0.sin());
            let (c1, s1) = (a1.cos(), a1.sin());

            let base0 = [radius * c0, -half_h, radius * s0];
            let base1 = [radius * c1, -half_h, radius * s1];

            let n0 = [nr * c0, ny, nr * s0];
            let n1 = [nr * c1, ny, nr * s1];
            // Tip normal = average of the two edge normals
            let mid_a = (a0 + a1) * 0.5;
            let n_tip = [nr * mid_a.cos(), ny, nr * mid_a.sin()];

            verts.push(Vertex {
                pos: tip,
                normal: n_tip,
                uv: [0.5, 0.0],
            });
            verts.push(Vertex {
                pos: base1,
                normal: n1,
                uv: [(i + 1) as f32 / segments as f32, 1.0],
            });
            verts.push(Vertex {
                pos: base0,
                normal: n0,
                uv: [i as f32 / segments as f32, 1.0],
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
            label: "iced3d cone".into(),
        }
    }
}
