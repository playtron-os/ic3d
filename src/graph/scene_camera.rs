//! Object-safe camera trait for scene graph storage.

use crate::camera::Camera;
use std::any::Any;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

/// Object-safe camera trait for storage in the scene graph.
///
/// Any type implementing `Camera + Clone + Debug + Send + Sync + 'static`
/// automatically satisfies this via the blanket impl. Users can store
/// [`PerspectiveCamera`](crate::PerspectiveCamera),
/// [`OrthographicCamera`](crate::OrthographicCamera), or their own custom
/// camera types — no enum required.
///
/// For runtime mutation of concrete camera properties (e.g. `set_position`),
/// use [`SceneGraph::camera_mut`](super::SceneGraph::camera_mut) with a type
/// parameter to downcast:
///
/// ```rust,ignore
/// graph.camera_mut::<PerspectiveCamera>(cam_id)
///     .unwrap()
///     .set_position(Vec3::new(10.0, 5.0, 8.0));
/// ```
pub trait SceneCamera: Camera + fmt::Debug + Send + Sync {
    /// Clone the camera into a new boxed trait object.
    fn clone_camera(&self) -> Box<dyn SceneCamera>;

    /// Downcast to concrete type (immutable).
    fn as_any(&self) -> &dyn Any;

    /// Downcast to concrete type (mutable).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Camera + Clone + fmt::Debug + Send + Sync + 'static> SceneCamera for T {
    fn clone_camera(&self) -> Box<dyn SceneCamera> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Clone for Box<dyn SceneCamera> {
    fn clone(&self) -> Self {
        self.clone_camera()
    }
}

/// Unique identifier for a camera in the scene graph.
///
/// Created automatically by [`SceneGraph::add_camera`](super::SceneGraph::add_camera).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CameraId(u32);

impl CameraId {
    /// Create a new unique camera ID.
    #[must_use]
    pub fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for CameraId {
    fn default() -> Self {
        Self::new()
    }
}
