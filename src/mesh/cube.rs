//! Axis-aligned cube primitive.

use super::Mesh;
use crate::pipeline::gpu_types::Vertex;

impl Mesh {
    /// Axis-aligned cube centered at the origin.
    ///
    /// Each face has outward normals and unique vertices (no sharing).
    /// 6 faces × 2 triangles × 3 vertices = 36 vertices.
    #[must_use]
    pub fn cube(size: f32) -> Self {
        let s = size * 0.5;
        let mut verts = Vec::with_capacity(36);

        // [normal, [v0, v1, v2, v3]] for each face
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            // +X
            (
                [1.0, 0.0, 0.0],
                [[s, -s, -s], [s, s, -s], [s, s, s], [s, -s, s]],
            ),
            // -X
            (
                [-1.0, 0.0, 0.0],
                [[-s, -s, s], [-s, s, s], [-s, s, -s], [-s, -s, -s]],
            ),
            // +Y
            (
                [0.0, 1.0, 0.0],
                [[-s, s, s], [s, s, s], [s, s, -s], [-s, s, -s]],
            ),
            // -Y
            (
                [0.0, -1.0, 0.0],
                [[-s, -s, -s], [s, -s, -s], [s, -s, s], [-s, -s, s]],
            ),
            // +Z
            (
                [0.0, 0.0, 1.0],
                [[-s, -s, s], [s, -s, s], [s, s, s], [-s, s, s]],
            ),
            // -Z
            (
                [0.0, 0.0, -1.0],
                [[s, -s, -s], [-s, -s, -s], [-s, s, -s], [s, s, -s]],
            ),
        ];

        for (normal, [v0, v1, v2, v3]) in &faces {
            let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            // Triangle 1: v0, v1, v2
            verts.push(Vertex {
                pos: *v0,
                normal: *normal,
                uv: uvs[0],
            });
            verts.push(Vertex {
                pos: *v1,
                normal: *normal,
                uv: uvs[1],
            });
            verts.push(Vertex {
                pos: *v2,
                normal: *normal,
                uv: uvs[2],
            });
            // Triangle 2: v0, v2, v3
            verts.push(Vertex {
                pos: *v0,
                normal: *normal,
                uv: uvs[0],
            });
            verts.push(Vertex {
                pos: *v2,
                normal: *normal,
                uv: uvs[2],
            });
            verts.push(Vertex {
                pos: *v3,
                normal: *normal,
                uv: uvs[3],
            });
        }

        Self {
            vertices: verts,
            label: "ic3d cube".into(),
        }
    }
}

#[cfg(test)]
#[path = "cube_tests.rs"]
mod tests;
