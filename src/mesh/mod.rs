//! Mesh data and primitive geometry builders.

mod cone;
mod cube;
mod cylinder;
mod plane;
mod sphere;
mod torus;

use crate::gpu_types::Vertex;

/// A CPU-side mesh: a named list of vertices (triangle list topology).
pub struct Mesh {
    vertices: Vec<Vertex>,
    label: String,
}

impl Mesh {
    /// Create a mesh from pre-built vertices.
    #[must_use]
    pub fn custom(vertices: Vec<Vertex>, label: impl Into<String>) -> Self {
        Self {
            vertices,
            label: label.into(),
        }
    }

    /// Number of vertices (= triangle count × 3 for triangle lists).
    #[must_use]
    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    /// Vertex data.
    #[must_use]
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Debug label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Upload vertices to a GPU buffer.
    #[must_use]
    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&self.label),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    /// Upload this mesh to the GPU, producing a [`MeshBuffer`] that tracks
    /// the vertex count automatically.
    ///
    /// Use with [`RenderPipeline3D::draw()`](crate::RenderPipeline3D::draw)
    /// for the simplest rendering path.
    #[must_use]
    pub fn upload(&self, device: &wgpu::Device) -> MeshBuffer {
        MeshBuffer {
            buffer: self.to_buffer(device),
            vertex_count: self.vertex_count(),
        }
    }

    /// Create a mirrored copy of this mesh along the Y axis.
    ///
    /// Negates Y components of positions and normals, then reverses triangle
    /// winding order to maintain correct face orientation. Useful for creating
    /// "inverted" variants of symmetric geometry.
    #[must_use]
    pub fn mirror_y(&self) -> Self {
        let mut verts = self.vertices.clone();
        for v in &mut verts {
            v.pos[1] = -v.pos[1];
            v.normal[1] = -v.normal[1];
        }
        // Reverse winding order for each triangle
        for tri in verts.chunks_exact_mut(3) {
            tri.swap(1, 2);
        }
        Self {
            vertices: verts,
            label: format!("{} (mirror_y)", self.label),
        }
    }
}

/// Incremental mesh construction with convenience methods for triangles and quads.
///
/// ```rust,ignore
/// let mesh = MeshBuilder::new("my mesh")
///     .triangle([0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0])
///     .quad([0.0, 0.0, 1.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0])
///     .build();
/// ```
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    label: String,
}

impl MeshBuilder {
    /// Create a new empty builder with the given debug label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            vertices: Vec::new(),
            label: label.into(),
        }
    }

    /// Push a single triangle. Normal is computed automatically from the
    /// cross product of edges `ab` and `ac`.
    #[must_use]
    pub fn triangle(self, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Self {
        let n = face_normal(a, b, c);
        self.triangle_with_normal(a, b, c, n)
    }

    /// Push a single triangle with an explicit normal override.
    #[must_use]
    pub fn triangle_with_normal(
        mut self,
        a: [f32; 3],
        b: [f32; 3],
        c: [f32; 3],
        normal: [f32; 3],
    ) -> Self {
        self.vertices.push(Vertex {
            pos: a,
            normal,
            uv: [0.5, 0.0],
        });
        self.vertices.push(Vertex {
            pos: b,
            normal,
            uv: [0.0, 1.0],
        });
        self.vertices.push(Vertex {
            pos: c,
            normal,
            uv: [1.0, 1.0],
        });
        self
    }

    /// Push a quad (two triangles). Normal is computed automatically from
    /// the cross product of edges `ab` and `ac`.
    ///
    /// Vertices: `a → b → c → d` (CCW winding).
    /// Produces triangles `(a, b, c)` and `(a, c, d)`.
    #[must_use]
    pub fn quad(self, a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> Self {
        let n = face_normal(a, b, c);
        self.quad_with_normal(n, a, b, c, d)
    }

    /// Push a quad (two triangles) with an explicit normal override.
    ///
    /// Vertices: `a → b → c → d` (CCW winding).
    /// UVs: a=(0,0), b=(1,0), c=(1,1), d=(0,1).
    #[must_use]
    pub fn quad_with_normal(
        mut self,
        normal: [f32; 3],
        a: [f32; 3],
        b: [f32; 3],
        c: [f32; 3],
        d: [f32; 3],
    ) -> Self {
        self.vertices.push(Vertex {
            pos: a,
            normal,
            uv: [0.0, 0.0],
        });
        self.vertices.push(Vertex {
            pos: b,
            normal,
            uv: [1.0, 0.0],
        });
        self.vertices.push(Vertex {
            pos: c,
            normal,
            uv: [1.0, 1.0],
        });
        self.vertices.push(Vertex {
            pos: a,
            normal,
            uv: [0.0, 0.0],
        });
        self.vertices.push(Vertex {
            pos: c,
            normal,
            uv: [1.0, 1.0],
        });
        self.vertices.push(Vertex {
            pos: d,
            normal,
            uv: [0.0, 1.0],
        });
        self
    }

    /// Extrude a convex polygon along the Z axis.
    ///
    /// `points` are 2D `[x, y]` vertices of the polygon outline (CCW winding
    /// when viewed from +Z). The polygon is placed at `z = depth` (top face)
    /// and extruded down to `z = 0` (side walls only — no bottom cap).
    ///
    /// The top face is fan-triangulated from the first vertex. Each edge
    /// produces a side-wall quad connecting top to bottom.
    ///
    /// ```rust,ignore
    /// let prism = MeshBuilder::new("prism")
    ///     .extrude(&[[0.0, 0.5], [-0.5, -0.25], [0.5, -0.25]], 1.0)
    ///     .build();
    /// ```
    #[must_use]
    pub fn extrude(mut self, points: &[[f32; 2]], depth: f32) -> Self {
        assert!(points.len() >= 3, "extrude requires at least 3 points");

        // Build top and bottom vertex rings
        let top: Vec<[f32; 3]> = points.iter().map(|p| [p[0], p[1], depth]).collect();
        let bot: Vec<[f32; 3]> = points.iter().map(|p| [p[0], p[1], 0.0]).collect();

        // Top face: fan triangulation from vertex 0
        for i in 1..top.len() - 1 {
            self = self.triangle(top[0], top[i], top[i + 1]);
        }

        // Side walls: one quad per edge
        let n = top.len();
        for i in 0..n {
            let j = (i + 1) % n;
            self = self.quad(top[i], top[j], bot[j], bot[i]);
        }

        self
    }

    /// Consume the builder and produce the final [`Mesh`].
    #[must_use]
    pub fn build(self) -> Mesh {
        Mesh {
            vertices: self.vertices,
            label: self.label,
        }
    }
}

/// Compute a unit face normal from three vertices via cross product.
#[must_use]
fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = ab[1] * ac[2] - ab[2] * ac[1];
    let ny = ab[2] * ac[0] - ab[0] * ac[2];
    let nz = ab[0] * ac[1] - ab[1] * ac[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [nx / len, ny / len, nz / len]
    }
}

/// An uploaded mesh ready for GPU rendering.
///
/// Created via [`Mesh::upload()`]. Tracks the vertex buffer and count together
/// so consumers don't need to manage them separately.
///
/// ```rust,ignore
/// let cube = Mesh::cube(1.0).upload(device);
/// // later, in render():
/// pipeline.draw(&cube, 0..instance_count)
/// ```
pub struct MeshBuffer {
    buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl MeshBuffer {
    /// The underlying GPU vertex buffer.
    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Number of vertices in this mesh.
    #[must_use]
    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}

#[cfg(test)]
#[path = "mesh_tests.rs"]
mod tests;
