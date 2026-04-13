//! Scene context: per-frame state shared between the widget and overlays.
//!
//! [`SceneContext`] carries camera metadata, viewport dimensions, and named
//! object transforms — everything an [`Overlay`](crate::Overlay) needs to
//! position and scale itself automatically.
//!
//! [`SceneHandle`] is the cross-frame shared wrapper that the widget
//! populates each frame. It manages built-in gizmos for selected objects —
//! the widget handles all input (closest-hit picking, drag tracking) and
//! rendering automatically.

use crate::camera::CameraInfo;
use crate::gizmo::{Gizmo, GizmoMode, GizmoResult};
use crate::math::screen_hit_test_closest;
use crate::overlay::base::{Overlay, OverlayInput};
use crate::scene::object::SceneObjectId;
use crate::widget::MeshDrawGroup;
use glam::{Mat4, Vec2, Vec3};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Per-frame scene context for overlay rendering and input handling.
///
/// Built by the widget during `draw()` from [`SceneData`](crate::SceneData)
/// and the current draw groups. Passed to [`Overlay::draw`](crate::Overlay::draw)
/// so overlays can auto-scale and attach to scene objects.
///
/// Also stored in [`SceneHandle`] for cross-frame access during `update()`.
#[derive(Debug, Clone)]
pub struct SceneContext {
    /// Camera metadata for this frame.
    pub camera: CameraInfo,
    /// Viewport dimensions in logical pixels.
    pub viewport_size: Vec2,
    /// Model matrices of named scene objects, keyed by [`SceneObjectId`].
    ///
    /// Populated from [`MeshDrawGroup`](crate::widget::MeshDrawGroup)s that
    /// have an [`id`](crate::widget::MeshDrawGroup::id) set. Uses the first
    /// instance's model matrix.
    pub objects: HashMap<SceneObjectId, Mat4>,
}

impl SceneContext {
    /// Get the world-space position of a scene object (translation column of its model matrix).
    #[must_use]
    pub fn object_position(&self, id: SceneObjectId) -> Option<Vec3> {
        self.objects
            .get(&id)
            .map(|m| Vec3::new(m.col(3).x, m.col(3).y, m.col(3).z))
    }
}

/// Internal state behind the `Arc<RwLock<>>`.
#[derive(Debug)]
struct SceneHandleInner {
    /// Latest scene context from the last draw frame.
    context: Option<SceneContext>,
    /// Managed gizmos, one per selected object.
    gizmos: HashMap<SceneObjectId, Gizmo>,
    /// Lightweight selection set (no managed gizmo). Used by custom overlays
    /// via [`SceneHandle::select_object`] / [`SceneHandle::selected_objects`].
    selection: HashSet<SceneObjectId>,
    /// Latest input state, written by the widget each event.
    input: OverlayInput,
}

/// Shared scene state that persists across frames.
///
/// Created once by the consumer and passed to the widget via
/// [`scene_3d().scene(handle)`](crate::widget::scene_3d). The widget
/// populates it each frame with camera, viewport, and object data.
///
/// # Managed Gizmos
///
/// Call [`select`](Self::select) to attach a translation gizmo to a scene
/// object. Multiple objects can be selected simultaneously — each gets its
/// own gizmo. The widget handles cursor tracking, closest-hit picking, and
/// rendering automatically. When gizmos overlap, the closest one wins.
///
/// Results are delivered via the
/// [`on_gizmo`](crate::widget::Scene3DBuilder::on_gizmo) callback.
///
/// # Example
///
/// ```rust,ignore
/// use ic3d::{SceneHandle, SceneObjectId};
/// use ic3d::gizmo::GizmoMode;
///
/// let scene = SceneHandle::new();
/// let cube_a = SceneObjectId::new();
/// let cube_b = SceneObjectId::new();
///
/// // Select multiple objects — each gets its own managed gizmo.
/// scene.select(cube_a, GizmoMode::Translate);
/// scene.select(cube_b, GizmoMode::Translate);
///
/// // In view():
/// scene_3d(MyScene { .. })
///     .scene(scene.clone())
///     .on_gizmo(Message::Gizmo)
///     .into()
/// ```
#[derive(Debug, Clone)]
pub struct SceneHandle(Arc<RwLock<SceneHandleInner>>);

impl SceneHandle {
    /// Create a new empty scene handle.
    ///
    /// No camera or object data is available until the first frame is drawn.
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(SceneHandleInner {
            context: None,
            gizmos: HashMap::new(),
            selection: HashSet::new(),
            input: OverlayInput::default(),
        })))
    }

    /// Select a scene object and show a managed gizmo on it.
    ///
    /// Multiple objects can be selected at once — each gets its own gizmo.
    /// The widget handles closest-hit picking automatically when gizmos
    /// overlap.
    ///
    /// The object must have a [`SceneObjectId`] assigned via
    /// [`MeshDrawGroup::with_id`](crate::widget::MeshDrawGroup::with_id).
    /// The gizmo automatically follows the object's position.
    pub fn select(&self, id: SceneObjectId, mode: GizmoMode) {
        let mut inner = self.0.write();
        inner.gizmos.insert(id, Gizmo::new(mode).attach_to(id));
    }

    /// Access a managed gizmo for configuration.
    ///
    /// Use this to modify gizmo properties (visibility, interactivity, size)
    /// after selection. The closure receives a mutable reference to the
    /// [`Gizmo`] if the object is currently selected.
    ///
    /// ```rust,ignore
    /// scene.select(cube_id, GizmoMode::Translate);
    /// scene.modify_gizmo(cube_id, |g| g.set_interactive(false));
    /// ```
    pub fn modify_gizmo(&self, id: SceneObjectId, f: impl FnOnce(&mut Gizmo)) {
        let mut inner = self.0.write();
        if let Some(gizmo) = inner.gizmos.get_mut(&id) {
            f(gizmo);
        }
    }

    /// Remove the selection and hide the managed gizmo for one object.
    pub fn deselect(&self, id: SceneObjectId) {
        let mut inner = self.0.write();
        inner.gizmos.remove(&id);
    }

    /// Remove all selections and hide all managed gizmos.
    pub fn deselect_all(&self) {
        let mut inner = self.0.write();
        inner.gizmos.clear();
    }

    /// The currently selected scene objects (those with managed gizmos).
    #[must_use]
    pub fn selected(&self) -> Vec<SceneObjectId> {
        self.0.read().gizmos.keys().copied().collect()
    }

    /// Whether a specific object is selected.
    #[must_use]
    pub fn is_selected(&self, id: SceneObjectId) -> bool {
        self.0.read().gizmos.contains_key(&id)
    }

    /// Whether any managed gizmo is actively being dragged.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.0.read().gizmos.values().any(Gizmo::is_dragging)
    }

    /// Whether any managed gizmo's axis handle is being hovered.
    #[must_use]
    pub(crate) fn gizmo_hovered(&self) -> bool {
        self.0.read().gizmos.values().any(|g| g.is_hovered())
    }

    // ──────────── Lightweight selection ────────────

    /// Mark an object as selected (without creating a managed gizmo).
    ///
    /// Custom overlays can read this via
    /// [`selected_objects`](Self::selected_objects) or
    /// [`DraggableOverlay::resolve_target`](crate::DraggableOverlay::resolve_target).
    pub fn select_object(&self, id: SceneObjectId) {
        self.0.write().selection.insert(id);
    }

    /// Remove an object from the lightweight selection.
    pub fn deselect_object(&self, id: SceneObjectId) {
        self.0.write().selection.remove(&id);
    }

    /// Clear the lightweight selection.
    pub fn deselect_all_objects(&self) {
        self.0.write().selection.clear();
    }

    /// The currently selected objects (lightweight, no managed gizmo).
    #[must_use]
    pub fn selected_objects(&self) -> Vec<SceneObjectId> {
        self.0.read().selection.iter().copied().collect()
    }

    /// Whether a specific object is in the lightweight selection.
    #[must_use]
    pub fn is_object_selected(&self, id: SceneObjectId) -> bool {
        self.0.read().selection.contains(&id)
    }

    /// Get the current camera metadata.
    ///
    /// Returns `None` before the first frame is drawn.
    #[must_use]
    pub fn camera(&self) -> Option<CameraInfo> {
        self.0.read().context.as_ref().map(|ctx| ctx.camera)
    }

    /// Get the current viewport dimensions in logical pixels.
    #[must_use]
    pub fn viewport_size(&self) -> Vec2 {
        self.0
            .read()
            .context
            .as_ref()
            .map_or(Vec2::ZERO, |ctx| ctx.viewport_size)
    }

    /// Get the model matrix of a named scene object.
    #[must_use]
    pub fn object_transform(&self, id: SceneObjectId) -> Option<Mat4> {
        self.0
            .read()
            .context
            .as_ref()
            .and_then(|ctx| ctx.objects.get(&id).copied())
    }

    /// Get the world-space position of a named scene object.
    #[must_use]
    pub fn object_position(&self, id: SceneObjectId) -> Option<Vec3> {
        self.0
            .read()
            .context
            .as_ref()
            .and_then(|ctx| ctx.object_position(id))
    }

    /// Update the scene context. Called by the widget at the end of `draw()`.
    pub fn update_context(&self, ctx: SceneContext) {
        self.0.write().context = Some(ctx);
    }

    /// Update the input state. Called by the widget on each mouse event.
    ///
    /// Usually you don't need to call this — the widget populates it
    /// automatically. Useful for testing or headless scenarios.
    pub fn update_input(&self, input: OverlayInput) {
        self.0.write().input = input;
    }

    /// Get the current input state (cursor + mouse pressed).
    ///
    /// Populated automatically by the widget — no manual tracking needed.
    #[must_use]
    pub fn input(&self) -> OverlayInput {
        self.0.read().input
    }

    /// Process one frame of managed gizmo input.
    ///
    /// Called by the widget's `Program::update()` with cursor state.
    /// Uses engine-side hit testing: collects [`hit_shapes`](Overlay::hit_shapes)
    /// from all managed gizmos, runs [`screen_hit_test_closest`] once to
    /// find the winning gizmo and shape, then passes the result via
    /// [`update_with_hit`](Gizmo::update_with_hit). This eliminates
    /// redundant hit testing inside each gizmo.
    ///
    /// During an active drag, only the dragging gizmo is updated (no
    /// hit testing needed).
    ///
    /// Returns the interacted object ID and result if interaction occurred.
    pub(crate) fn process_gizmo(
        &self,
        cursor: Vec2,
        mouse_pressed: bool,
    ) -> Option<(SceneObjectId, GizmoResult)> {
        let mut guard = self.0.write();
        let inner = &mut *guard;

        if inner.gizmos.is_empty() {
            return None;
        }

        let ctx = inner.context.as_ref()?;
        let camera = ctx.camera;
        let viewport = ctx.viewport_size;

        // If any gizmo is actively dragging, only update that one.
        let dragging_id =
            inner
                .gizmos
                .iter()
                .find_map(|(&id, g)| if g.is_dragging() { Some(id) } else { None });

        if let Some(id) = dragging_id {
            if let Some(pos) = ctx.object_position(id) {
                if let Some(gizmo) = inner.gizmos.get_mut(&id) {
                    gizmo.set_position(pos);
                }
            }
            let gizmo = inner.gizmos.get_mut(&id)?;
            return gizmo
                .update_with_hit(cursor, mouse_pressed, &camera, viewport, None)
                .map(|r| (id, r));
        }

        // No active drag — engine-side hit testing across all gizmos.
        // Collect hit shapes from all interactive gizmos, tagged with
        // (gizmo_id, shape_index).
        let shapes = inner
            .gizmos
            .iter()
            .filter(|(_, g)| g.is_interactive())
            .flat_map(|(&id, gizmo)| {
                let pos = ctx.object_position(id).unwrap_or(gizmo.gizmo_position());
                let scale = gizmo.compute_scale_at(&camera, viewport.y, pos);
                gizmo
                    .build_hit_shapes(pos, scale, &camera, viewport)
                    .into_iter()
                    .enumerate()
                    .map(move |(i, shape)| ((id, i), shape))
            });

        let closest = screen_hit_test_closest(shapes, cursor, camera.view_projection, viewport);

        let winner = closest.map(|((id, shape_idx), dist)| {
            let gizmo = &inner.gizmos[&id];
            let hit = gizmo.interpret_hit(shape_idx, dist);
            (id, hit)
        });

        let winner_id = winner.map(|(id, _)| id);

        // Update winner gizmo with the hit, clear hover on all others.
        let mut result = None;
        let ids: Vec<SceneObjectId> = inner.gizmos.keys().copied().collect();
        for id in ids {
            if let Some(gizmo) = inner.gizmos.get_mut(&id) {
                if Some(id) == winner_id {
                    if let Some(pos) = ctx.object_position(id) {
                        gizmo.set_position(pos);
                    }
                    let hit = winner.map(|(_, h)| h);
                    result = gizmo
                        .update_with_hit(cursor, mouse_pressed, &camera, viewport, hit)
                        .map(|r| (id, r));
                } else {
                    gizmo.clear_hover();
                }
            }
        }

        result
    }

    /// Get all managed gizmos' overlay draw groups for rendering.
    ///
    /// Called by the widget's `draw()` to add all managed gizmos to the
    /// overlay pass. Returns empty if no selections are active.
    pub(crate) fn gizmo_overlays(&self) -> Vec<MeshDrawGroup> {
        let inner = self.0.read();
        if inner.gizmos.is_empty() {
            return Vec::new();
        }
        match inner.context.as_ref() {
            Some(ctx) => inner
                .gizmos
                .values()
                .filter(|g| g.visible())
                .flat_map(|g| g.draw(ctx))
                .collect(),
            None => Vec::new(),
        }
    }
}

impl Default for SceneHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
