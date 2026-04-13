//! Arrow primitive: cylindrical shaft capped by a conical head.

use super::Mesh;
use crate::pipeline::gpu_types::Vertex;
use std::f32::consts::PI;

/// Shaft radius as a fraction of the total arrow length.
const SHAFT_RADIUS_RATIO: f32 = 0.006;
/// Head (cone) radius as a fraction of the total arrow length.
const HEAD_RADIUS_RATIO: f32 = 0.037;
/// Head (cone) length as a fraction of the total arrow length.
const HEAD_LENGTH_RATIO: f32 = 0.20;
/// Number of radial segments for arrow geometry.
const SEGMENTS: u32 = 16;

impl Mesh {
    /// Arrow along +Y (from origin to `length`): cylindrical shaft capped
    /// by a conical head.
    ///
    /// Use a [`Transform`](crate::Transform) to orient it along any axis.
    #[must_use]
    pub fn arrow(length: f32) -> Self {
        let shaft_radius = length * SHAFT_RADIUS_RATIO;
        let head_radius = length * HEAD_RADIUS_RATIO;
        let head_length = length * HEAD_LENGTH_RATIO;
        let shaft_length = length - head_length;

        let mut verts = Vec::new();

        // Shaft: cylinder from y=0 to y=shaft_length
        cylinder_verts(shaft_radius, 0.0, shaft_length, SEGMENTS, &mut verts);

        // Head: cone from y=shaft_length to y=length
        cone_verts(head_radius, shaft_length, length, SEGMENTS, &mut verts);

        Self::custom(verts, "arrow")
    }
}

// ──────────── internal helpers ────────────

/// Append cylinder side-wall vertices (along Y from `y_bot` to `y_top`).
fn cylinder_verts(radius: f32, y_bot: f32, y_top: f32, segments: u32, verts: &mut Vec<Vertex>) {
    for i in 0..segments {
        let a0 = 2.0 * PI * i as f32 / segments as f32;
        let a1 = 2.0 * PI * (i + 1) as f32 / segments as f32;

        let (c0, s0) = (a0.cos(), a0.sin());
        let (c1, s1) = (a1.cos(), a1.sin());

        let n0 = [c0, 0.0, s0];
        let n1 = [c1, 0.0, s1];

        let p0t = [radius * c0, y_top, radius * s0];
        let p1t = [radius * c1, y_top, radius * s1];
        let p0b = [radius * c0, y_bot, radius * s0];
        let p1b = [radius * c1, y_bot, radius * s1];

        let uv = [0.0, 0.0];

        verts.push(Vertex {
            pos: p0t,
            normal: n0,
            uv,
        });
        verts.push(Vertex {
            pos: p1t,
            normal: n1,
            uv,
        });
        verts.push(Vertex {
            pos: p1b,
            normal: n1,
            uv,
        });
        verts.push(Vertex {
            pos: p0t,
            normal: n0,
            uv,
        });
        verts.push(Vertex {
            pos: p1b,
            normal: n1,
            uv,
        });
        verts.push(Vertex {
            pos: p0b,
            normal: n0,
            uv,
        });
    }
}

/// Append cone vertices (along Y from `y_base` to `y_tip`), e.g. arrowhead.
fn cone_verts(radius: f32, y_base: f32, y_tip: f32, segments: u32, verts: &mut Vec<Vertex>) {
    let height = y_tip - y_base;
    let slope_len = (radius * radius + height * height).sqrt();
    let ny = radius / slope_len;
    let nr = height / slope_len;

    let tip = [0.0_f32, y_tip, 0.0];
    let uv = [0.0, 0.0];

    // Side wall
    for i in 0..segments {
        let a0 = 2.0 * PI * i as f32 / segments as f32;
        let a1 = 2.0 * PI * (i + 1) as f32 / segments as f32;
        let (c0, s0) = (a0.cos(), a0.sin());
        let (c1, s1) = (a1.cos(), a1.sin());

        let base0 = [radius * c0, y_base, radius * s0];
        let base1 = [radius * c1, y_base, radius * s1];

        let n0 = [nr * c0, ny, nr * s0];
        let n1 = [nr * c1, ny, nr * s1];
        let mid_a = (a0 + a1) * 0.5;
        let n_tip = [nr * mid_a.cos(), ny, nr * mid_a.sin()];

        verts.push(Vertex {
            pos: tip,
            normal: n_tip,
            uv,
        });
        verts.push(Vertex {
            pos: base1,
            normal: n1,
            uv,
        });
        verts.push(Vertex {
            pos: base0,
            normal: n0,
            uv,
        });
    }

    // Bottom cap
    let bot_n = [0.0, -1.0, 0.0];
    let bot_center = [0.0_f32, y_base, 0.0];
    for i in 0..segments {
        let a0 = 2.0 * PI * i as f32 / segments as f32;
        let a1 = 2.0 * PI * (i + 1) as f32 / segments as f32;
        let p0 = [radius * a0.cos(), y_base, radius * a0.sin()];
        let p1 = [radius * a1.cos(), y_base, radius * a1.sin()];
        verts.push(Vertex {
            pos: bot_center,
            normal: bot_n,
            uv,
        });
        verts.push(Vertex {
            pos: p0,
            normal: bot_n,
            uv,
        });
        verts.push(Vertex {
            pos: p1,
            normal: bot_n,
            uv,
        });
    }
}

#[cfg(test)]
#[path = "arrow_tests.rs"]
mod tests;
