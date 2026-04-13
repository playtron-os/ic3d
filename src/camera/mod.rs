//! Camera types: orthographic and perspective.

mod orthographic;
mod perspective;

pub use orthographic::OrthographicCamera;
pub use perspective::PerspectiveCamera;

use glam::{Mat4, Vec3};

/// A camera that produces view and projection matrices.
///
/// The base methods (`view_matrix`, `projection_matrix`) are required.
/// Metadata methods (`camera_position`, `camera_forward`, `fov_y`) have
/// defaults that extract values from the view matrix, but concrete cameras
/// should override them for efficiency.
pub trait Camera {
    /// The view matrix (world → camera space).
    fn view_matrix(&self) -> Mat4;

    /// The projection matrix (camera → clip space).
    fn projection_matrix(&self) -> Mat4;

    /// Combined view-projection matrix.
    fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// World-space camera position.
    ///
    /// Default: extracted from the inverse view matrix (4th column).
    /// Concrete cameras should override for efficiency.
    fn camera_position(&self) -> Vec3 {
        let inv = self.view_matrix().inverse();
        Vec3::new(inv.w_axis.x, inv.w_axis.y, inv.w_axis.z)
    }

    /// World-space forward direction (unit vector, the direction the camera looks).
    ///
    /// Default: extracted from the inverse view matrix (-Z axis in RH coords).
    /// Concrete cameras should override for efficiency.
    fn camera_forward(&self) -> Vec3 {
        let inv = self.view_matrix().inverse();
        -Vec3::new(inv.z_axis.x, inv.z_axis.y, inv.z_axis.z).normalize()
    }

    /// Vertical field of view in radians, if applicable.
    ///
    /// Returns `None` for orthographic cameras (no perspective foreshortening).
    /// Perspective cameras should return `Some(fov_y)`.
    fn fov_y(&self) -> Option<f32> {
        None
    }

    /// World-space look-at target.
    ///
    /// Default: `position + forward`. Concrete cameras with stored targets
    /// should override for accuracy.
    fn camera_target(&self) -> Vec3 {
        self.camera_position() + self.camera_forward()
    }

    /// Update the projection aspect ratio for viewport changes.
    ///
    /// Called by the scene graph when viewport dimensions change.
    /// Default is a no-op; perspective cameras should override.
    fn set_aspect(&mut self, _aspect: f32) {}
}

/// Camera metadata snapshot for overlay scaling and hit testing.
///
/// Created via [`CameraInfo::from_camera`] during scene setup.
/// Carried through [`SceneData`](crate::SceneData) so overlays and
/// gizmos can auto-scale without manual camera parameter plumbing.
///
/// Designed for multi-camera support: each camera produces its own
/// `CameraInfo`, and overlays scale themselves per-camera.
#[derive(Debug, Clone, Copy)]
pub struct CameraInfo {
    /// World-space camera position.
    pub position: Vec3,
    /// Unit-length forward direction.
    pub forward: Vec3,
    /// Vertical FOV in radians (`None` for orthographic).
    pub fov_y: Option<f32>,
    /// Combined view-projection matrix.
    pub view_projection: Mat4,
}

impl CameraInfo {
    /// Snapshot camera metadata from any [`Camera`] implementor.
    #[must_use]
    pub fn from_camera(camera: &dyn Camera) -> Self {
        Self {
            position: camera.camera_position(),
            forward: camera.camera_forward(),
            fov_y: camera.fov_y(),
            view_projection: camera.view_projection(),
        }
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod camera_tests;
