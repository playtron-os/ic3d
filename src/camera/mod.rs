//! Camera types: orthographic and perspective.

mod orthographic;
mod perspective;

pub use orthographic::OrthographicCamera;
pub use perspective::PerspectiveCamera;

use glam::Mat4;

/// A camera that produces view and projection matrices.
pub trait Camera {
    /// The view matrix (world → camera space).
    fn view_matrix(&self) -> Mat4;

    /// The projection matrix (camera → clip space).
    fn projection_matrix(&self) -> Mat4;

    /// Combined view-projection matrix.
    fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}
