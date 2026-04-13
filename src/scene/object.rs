//! Lightweight scene object identifier for tracking meshes across frames.

use std::sync::atomic::{AtomicU32, Ordering};

/// Global counter for auto-generating unique object IDs.
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

/// Lightweight identifier for a scene object.
///
/// Assign to [`MeshDrawGroup`](crate::widget::MeshDrawGroup) via
/// [`with_id`](crate::widget::MeshDrawGroup::with_id) so that overlays
/// (gizmos, labels, etc.) can reference and follow the object.
///
/// # Creating IDs
///
/// Use [`new()`](Self::new) to auto-generate a unique ID — no manual
/// numbering needed:
///
/// ```rust,ignore
/// use ic3d::SceneObjectId;
///
/// // Auto-generated — each call returns a unique ID.
/// let cube_id = SceneObjectId::new();
/// let sphere_id = SceneObjectId::new();
///
/// // Use it to tag a draw group:
/// let draw = MeshDrawGroup::new(mesh, instances).with_id(cube_id);
/// ```
///
/// IDs are `Copy` and cheap to store. Create them once (e.g. in your
/// app constructor) and reuse across frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneObjectId(pub u32);

impl SceneObjectId {
    /// Create a new unique scene object ID.
    ///
    /// Each call returns a different ID, auto-incremented from a global
    /// counter. Thread-safe.
    #[must_use]
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for SceneObjectId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod tests;
