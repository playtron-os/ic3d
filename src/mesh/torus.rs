//! Torus (donut) and partial arc primitives.

use super::Mesh;
use crate::pipeline::gpu_types::Vertex;
use std::f32::consts::PI;

impl Mesh {
    /// Torus (donut) centered at the origin in the XZ plane.
    ///
    /// - `major_radius`: distance from center to tube center
    /// - `minor_radius`: tube radius
    /// - `major_segments`: divisions around the ring (min 3)
    /// - `minor_segments`: divisions around the tube cross-section (min 3)
    #[must_use]
    pub fn torus(
        major_radius: f32,
        minor_radius: f32,
        major_segments: u32,
        minor_segments: u32,
    ) -> Self {
        Self::torus_arc(
            major_radius,
            minor_radius,
            0.0,
            2.0 * PI,
            major_segments,
            minor_segments,
        )
    }

    /// Partial torus arc centered at the origin in the XZ plane.
    ///
    /// - `major_radius`: distance from center to tube center
    /// - `minor_radius`: tube radius
    /// - `start_angle`: start angle in radians (0 = +X direction in XZ plane)
    /// - `sweep`: sweep angle in radians (positive = counter-clockwise)
    /// - `major_segments`: divisions along the arc (min 3)
    /// - `minor_segments`: divisions around the tube cross-section (min 3)
    #[must_use]
    pub fn torus_arc(
        major_radius: f32,
        minor_radius: f32,
        start_angle: f32,
        sweep: f32,
        major_segments: u32,
        minor_segments: u32,
    ) -> Self {
        let major_segments = major_segments.max(3);
        let minor_segments = minor_segments.max(3);
        let mut verts = Vec::new();

        for i in 0..major_segments {
            let u0 = i as f32 / major_segments as f32;
            let u1 = (i + 1) as f32 / major_segments as f32;
            let theta0 = start_angle + u0 * sweep;
            let theta1 = start_angle + u1 * sweep;

            for j in 0..minor_segments {
                let v0 = j as f32 / minor_segments as f32;
                let v1 = (j + 1) as f32 / minor_segments as f32;
                let phi0 = v0 * 2.0 * PI;
                let phi1 = v1 * 2.0 * PI;

                let point = |theta: f32, phi: f32| -> ([f32; 3], [f32; 3]) {
                    let ct = theta.cos();
                    let st = theta.sin();
                    let cp = phi.cos();
                    let sp = phi.sin();
                    let r = major_radius + minor_radius * cp;
                    let pos = [r * ct, minor_radius * sp, r * st];
                    let nx = cp * ct;
                    let ny = sp;
                    let nz = cp * st;
                    (pos, [nx, ny, nz])
                };

                let (p00, n00) = point(theta0, phi0);
                let (p10, n10) = point(theta1, phi0);
                let (p01, n01) = point(theta0, phi1);
                let (p11, n11) = point(theta1, phi1);

                verts.push(Vertex {
                    pos: p00,
                    normal: n00,
                    uv: [u0, v0],
                });
                verts.push(Vertex {
                    pos: p11,
                    normal: n11,
                    uv: [u1, v1],
                });
                verts.push(Vertex {
                    pos: p10,
                    normal: n10,
                    uv: [u1, v0],
                });

                verts.push(Vertex {
                    pos: p00,
                    normal: n00,
                    uv: [u0, v0],
                });
                verts.push(Vertex {
                    pos: p01,
                    normal: n01,
                    uv: [u0, v1],
                });
                verts.push(Vertex {
                    pos: p11,
                    normal: n11,
                    uv: [u1, v1],
                });
            }
        }

        Self {
            vertices: verts,
            label: "ic3d torus arc".into(),
        }
    }

    /// Flat disc (filled circle) in the XZ plane at y=0.
    ///
    /// Uses a triangle fan from center to `segments` rim vertices.
    /// Suitable for angle indicator wedges when combined with a partial sweep.
    ///
    /// - `radius`: disc radius
    /// - `start_angle`: start angle in radians
    /// - `sweep`: sweep angle in radians
    /// - `segments`: number of triangle-fan slices (min 3)
    #[must_use]
    pub fn disc_arc(radius: f32, start_angle: f32, sweep: f32, segments: u32) -> Self {
        let segments = segments.max(3);
        let mut verts = Vec::new();
        let normal = [0.0, 1.0, 0.0];
        let center = [0.0_f32, 0.0, 0.0];

        for i in 0..segments {
            let a0 = start_angle + sweep * (i as f32 / segments as f32);
            let a1 = start_angle + sweep * ((i + 1) as f32 / segments as f32);

            let p0 = [radius * a0.cos(), 0.0, radius * a0.sin()];
            let p1 = [radius * a1.cos(), 0.0, radius * a1.sin()];

            verts.push(Vertex {
                pos: center,
                normal,
                uv: [0.5, 0.5],
            });
            verts.push(Vertex {
                pos: p0,
                normal,
                uv: [0.5 + 0.5 * a0.cos(), 0.5 + 0.5 * a0.sin()],
            });
            verts.push(Vertex {
                pos: p1,
                normal,
                uv: [0.5 + 0.5 * a1.cos(), 0.5 + 0.5 * a1.sin()],
            });
        }

        Self {
            vertices: verts,
            label: "ic3d disc arc".into(),
        }
    }
}

#[cfg(test)]
#[path = "torus_tests.rs"]
mod tests;
