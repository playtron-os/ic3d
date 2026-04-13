//! Overlay trait for elements rendered on top of the 3D scene.
//!
//! Overlays (gizmos, guides, grid lines) render with no depth testing
//! and no shadows, so they are always visible. The library handles
//! scaling automatically using [`SceneContext`] — consumers never need
//! to manually extract camera parameters.
//!
//! # Multi-camera support
//!
//! Each camera produces its own [`CameraInfo`](crate::CameraInfo). When
//! rendering multiple viewports, each overlay receives the correct
//! [`SceneContext`] and scales itself accordingly.
//!
//! # Object attachment
//!
//! Overlays can read object transforms from [`SceneContext::objects`] to
//! follow scene objects. See [`Gizmo::attach_to`](crate::gizmo::Gizmo::attach_to).
//!
//! # Input handling
//!
//! Interactive overlays (gizmos, handles) can implement [`on_input`](Overlay::on_input)
//! to process cursor and mouse state each frame. The overlay receives an
//! [`OverlayContext`] with mutable node access and scene handle, so it can
//! apply its own mutations (scaling, translating, etc.) directly. Call
//! [`SceneGraph::process_input`](crate::graph::SceneGraph::process_input)
//! once per frame to drive all overlays.

use crate::graph::Node;
use crate::scene::context::{SceneContext, SceneHandle};
use crate::scene::object::SceneObjectId;
use crate::widget::MeshDrawGroup;
use glam::Vec2;
use std::collections::HashMap;
use std::fmt;

/// Input state for overlay interaction processing.
///
/// Passed to [`Overlay::on_input`] by
/// [`SceneGraph::process_input`](crate::graph::SceneGraph::process_input).
#[derive(Debug, Clone, Copy, Default)]
pub struct OverlayInput {
    /// Current cursor position in screen coordinates (logical pixels).
    pub cursor: Vec2,
    /// Whether the primary mouse button is pressed.
    pub mouse_pressed: bool,
}

/// Events emitted by overlays during input processing.
///
/// Returned by [`Overlay::on_input`] and collected by
/// [`SceneGraph::process_input`](crate::graph::SceneGraph::process_input).
/// Consumers match on these to react to overlay interactions.
///
/// For [`DraggableOverlay`](crate::DraggableOverlay) types, events are
/// produced automatically by the engine.
#[derive(Debug, Clone)]
pub enum OverlayEvent {
    /// The cursor entered the overlay's hit region.
    HoverStart,
    /// The cursor left the overlay's hit region.
    HoverEnd,
    /// A drag interaction started.
    DragStart,
    /// The cursor moved during an active drag (screen-space delta in pixels).
    DragMove(Vec2),
    /// A drag interaction ended.
    DragEnd,
    /// Custom event for raw [`Overlay`] implementations.
    ///
    /// Use this when the built-in variants don't cover your interaction.
    Custom(String),
}

/// Mutable access to scene nodes during overlay input processing.
///
/// Provided to [`Overlay::on_input`] so overlays can read camera/viewport
/// state via the [`SceneHandle`] and mutate node transforms (position,
/// scale, rotation) in response to user input.
pub struct OverlayContext<'a> {
    nodes: &'a mut HashMap<SceneObjectId, Node>,
    handle: &'a SceneHandle,
}

impl<'a> OverlayContext<'a> {
    /// Create a new overlay context.
    pub(crate) fn new(
        nodes: &'a mut HashMap<SceneObjectId, Node>,
        handle: &'a SceneHandle,
    ) -> Self {
        Self { nodes, handle }
    }

    /// Get a node by ID for reading its transform.
    #[must_use]
    pub fn node(&self, id: SceneObjectId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable reference to a node for updating its transform.
    pub fn node_mut(&mut self, id: SceneObjectId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Access the scene handle for camera, viewport, and object position queries.
    #[must_use]
    pub fn handle(&self) -> &SceneHandle {
        self.handle
    }
}

/// An overlay rendered on top of the 3D scene (no depth testing, no shadows).
///
/// Implement this trait for elements that should always be visible regardless
/// of scene geometry. The library calls [`draw`](Self::draw) with scene
/// context (camera, viewport, object transforms), so the overlay can handle
/// its own scaling and positioning internally.
///
/// Interactive overlays can implement [`on_input`](Self::on_input) to process
/// cursor/mouse state each frame and mutate scene nodes directly.
///
/// # Example
///
/// ```rust,ignore
/// use ic3d::gizmo::Gizmo;
///
/// // In Scene3DProgram::setup():
/// Scene3DSetup {
///     scene,
///     draws: vec![/* ... */],
///     overlays: vec![Box::new(gizmo.clone())],
///     custom_uniforms: None,
/// }
/// ```
pub trait Overlay: fmt::Debug + Send + Sync {
    /// Whether the overlay should be drawn and receive input.
    ///
    /// Returns `true` by default. Override for custom visibility logic
    /// (e.g. hide when not attached to an object, hide during transitions).
    /// When `false`, both [`draw`](Self::draw) and [`on_input`](Self::on_input)
    /// are skipped by the engine.
    fn visible(&self) -> bool {
        true
    }

    /// Whether the overlay participates in hit testing and input handling.
    ///
    /// Returns `true` by default. When `false`, the overlay is still drawn
    /// (if [`visible`](Self::visible)) but does not receive
    /// [`on_input`](Self::on_input) calls and is excluded from hit-test
    /// probing. Use this for decorative overlays (guides, grids, labels)
    /// that should never intercept mouse events.
    fn interactive(&self) -> bool {
        true
    }

    /// Generate [`MeshDrawGroup`]s for rendering this overlay.
    ///
    /// Called each frame with the current scene context. The implementation
    /// should compute its own scale from the camera metadata and can read
    /// object transforms for attachment.
    ///
    /// - `ctx`: scene context (camera, viewport size, named object transforms)
    fn draw(&self, ctx: &SceneContext) -> Vec<MeshDrawGroup>;

    /// Process input and mutate scene nodes.
    ///
    /// Called by [`SceneGraph::process_input`](crate::graph::SceneGraph::process_input)
    /// once per frame. The overlay can read camera/viewport data and modify
    /// node transforms (position, scale, rotation) via the [`OverlayContext`].
    ///
    /// Returns a list of [`OverlayEvent`]s describing what happened this
    /// frame (hover start/end, drag start/move/end, etc.). These are
    /// collected and returned by `process_input` so the consumer can react.
    ///
    /// Default implementation is a no-op returning no events.
    fn on_input(&mut self, _input: &OverlayInput, _ctx: &mut OverlayContext) -> Vec<OverlayEvent> {
        Vec::new()
    }
}

#[cfg(test)]
#[path = "base_tests.rs"]
mod tests;
