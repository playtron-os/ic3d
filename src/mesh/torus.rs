//! Torus (donut) primitive.

use super::Mesh;
use crate::gpu_types::Vertex;
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
        let major_segments = major_segments.max(3);
        let minor_segments = minor_segments.max(3);
        let mut verts = Vec::new();

        for i in 0..major_segments {
            let u0 = i as f32 / major_segments as f32;
            let u1 = (i + 1) as f32 / major_segments as f32;
            let theta0 = u0 * 2.0 * PI;
            let theta1 = u1 * 2.0 * PI;

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
            label: "iced3d torus".into(),
        }
    }
}

#[cfg(test)]
#[path = "torus_tests.rs"]
mod tests;
