//! Flat quad primitive.

use super::Mesh;
use crate::gpu_types::Vertex;

impl Mesh {
    /// Flat quad in the XZ plane, centered at the origin.
    ///
    /// Normal points up (+Y). Two triangles, 6 vertices.
    #[must_use]
    pub fn plane(width: f32, depth: f32) -> Self {
        let hw = width * 0.5;
        let hd = depth * 0.5;
        let n = [0.0, 1.0, 0.0];
        let vertices = vec![
            Vertex {
                pos: [-hw, 0.0, -hd],
                normal: n,
                uv: [0.0, 0.0],
            },
            Vertex {
                pos: [hw, 0.0, hd],
                normal: n,
                uv: [1.0, 1.0],
            },
            Vertex {
                pos: [hw, 0.0, -hd],
                normal: n,
                uv: [1.0, 0.0],
            },
            Vertex {
                pos: [-hw, 0.0, -hd],
                normal: n,
                uv: [0.0, 0.0],
            },
            Vertex {
                pos: [-hw, 0.0, hd],
                normal: n,
                uv: [0.0, 1.0],
            },
            Vertex {
                pos: [hw, 0.0, hd],
                normal: n,
                uv: [1.0, 1.0],
            },
        ];
        Self {
            vertices,
            label: "iced3d plane".into(),
        }
    }
}
