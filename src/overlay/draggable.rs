//! Engine-managed draggable overlay trait.
//!
//! [`DraggableOverlay`] handles all the common drag interaction mechanics:
//! screen-space hit testing, hover detection, drag start/end, and cursor
//! delta computation. Consumers only define what object to target, what
//! mutation to apply on drag, and how to draw the handle.
//!
//! The engine stores drag state and target cache inside [`Draggable<T>`] —
//! consumers never touch [`DragState`] directly.
//!
//! # Example
//!
//! ```rust,ignore
//! use ic3d::{DraggableOverlay, OverlayContext, SceneHandle, SceneObjectId};
//! use ic3d::widget::MeshDrawGroup;
//!
//! #[derive(Debug, Clone, Default)]
//! struct ScaleGizmo;
//!
//! impl DraggableOverlay for ScaleGizmo {
//!     fn resolve_target(&self, handle: &SceneHandle) -> Option<SceneObjectId> {
//!         handle.selected_objects().into_iter().next()
//!     }
//!
//!     fn on_drag(&mut self, delta: Vec2, ctx: &mut OverlayContext) {
//!         // apply mutations
//!     }
//!
//!     fn draw_overlay(&self, target: SceneObjectId, state: &DragState,
//!                     ctx: &SceneContext) -> Vec<MeshDrawGroup> {
//!         Vec::new()
//!     }
//! }
//! ```

use crate::math::{screen_hit_test, HitShape};
use crate::overlay::base::{Overlay, OverlayContext, OverlayEvent, OverlayInput};
use crate::scene::context::{SceneContext, SceneHandle};
use crate::scene::object::SceneObjectId;
use crate::widget::MeshDrawGroup;
use glam::Vec2;
use std::fmt;

/// Engine-managed drag interaction state.
///
/// Stored inside [`Draggable<T>`] — consumers read it via the `state`
/// parameter in [`DraggableOverlay::draw_overlay`] but never create or
/// mutate it directly.
#[derive(Debug, Clone, Default)]
pub struct DragState {
    pub(crate) hovered: bool,
    pub(crate) last_cursor: Option<Vec2>,
}

impl DragState {
    /// Whether the handle is being hovered.
    #[must_use]
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Whether an active drag is in progress.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.last_cursor.is_some()
    }

    /// Whether the handle is hovered or being dragged.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.hovered || self.last_cursor.is_some()
    }

    /// Reset to idle state (not hovered, not dragging).
    pub fn reset(&mut self) {
        self.hovered = false;
        self.last_cursor = None;
    }
}

/// A draggable overlay with engine-managed hit testing and drag tracking.
///
/// Implement this instead of [`Overlay`] when your overlay needs drag
/// interaction. The engine handles:
/// - Screen-space hit testing against the target object
/// - Hover state management
/// - Drag start/end detection
/// - Cursor delta computation
/// - Drag state and target caching
///
/// You only define: what to target, what to do on drag, and how to draw.
///
/// Wrap your type with [`Draggable::new`] when adding it as an overlay:
///
/// ```rust,ignore
/// graph.add_overlay(Draggable::new(ScaleGizmo::default()));
/// ```
pub trait DraggableOverlay: fmt::Debug + Send + Sync {
    /// Resolve the target scene object for this frame.
    ///
    /// Return `None` to disable the overlay. The engine caches the result
    /// and passes it to [`draw_overlay`](Self::draw_overlay).
    ///
    /// # Example — follow the first selected object
    ///
    /// ```rust,ignore
    /// fn resolve_target(&self, handle: &SceneHandle) -> Option<SceneObjectId> {
    ///     handle.selected_objects().into_iter().next()
    /// }
    /// ```
    fn resolve_target(&self, handle: &SceneHandle) -> Option<SceneObjectId>;

    /// Hit radius in screen pixels (default: 24.0).
    fn hit_radius(&self) -> f32 {
        24.0
    }

    /// Called each frame during an active drag with the screen-space delta.
    ///
    /// `delta` is in screen pixels: positive X = right, positive Y = down.
    /// Use [`OverlayContext::node_mut`] to apply mutations to scene nodes.
    fn on_drag(&mut self, delta: Vec2, ctx: &mut OverlayContext);

    /// Generate draw groups for rendering this overlay.
    ///
    /// `target` is the resolved scene object ID (from `resolve_target`).
    /// `state` provides hover/drag status for visual feedback.
    /// `ctx` has camera, viewport, and object transforms.
    fn draw_overlay(
        &self,
        target: SceneObjectId,
        state: &DragState,
        ctx: &SceneContext,
    ) -> Vec<MeshDrawGroup>;
}

/// Engine wrapper that adds drag state and target caching to a
/// [`DraggableOverlay`].
///
/// Created via [`Draggable::new`]. Register with
/// [`SceneGraph::add_overlay`](crate::graph::SceneGraph::add_overlay).
///
/// ```rust,ignore
/// graph.add_overlay(Draggable::new(ScaleGizmo::default()));
/// ```
#[derive(Debug, Clone)]
pub struct Draggable<T: DraggableOverlay> {
    inner: T,
    state: DragState,
    target: Option<SceneObjectId>,
}

impl<T: DraggableOverlay> Draggable<T> {
    /// Wrap a [`DraggableOverlay`] with engine-managed drag state.
    #[must_use]
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            state: DragState::default(),
            target: None,
        }
    }

    /// Access the inner overlay.
    #[must_use]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Access the inner overlay mutably.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// The current drag state.
    #[must_use]
    pub fn drag_state(&self) -> &DragState {
        &self.state
    }

    /// The currently cached target.
    #[must_use]
    pub fn target(&self) -> Option<SceneObjectId> {
        self.target
    }
}

impl<T: DraggableOverlay + Clone + 'static> Overlay for Draggable<T> {
    fn draw(&self, ctx: &SceneContext) -> Vec<MeshDrawGroup> {
        match self.target {
            Some(target) => self.inner.draw_overlay(target, &self.state, ctx),
            None => Vec::new(),
        }
    }

    fn on_input(&mut self, input: &OverlayInput, ctx: &mut OverlayContext) -> Vec<OverlayEvent> {
        let mut events = Vec::new();
        let resolved = self.inner.resolve_target(ctx.handle());
        self.target = resolved;
        let Some(target) = resolved else {
            return events;
        };
        let Some(camera) = ctx.handle().camera() else {
            return events;
        };
        let viewport = ctx.handle().viewport_size();
        let Some(obj_pos) = ctx.handle().object_position(target) else {
            return events;
        };

        // Hit test: project object center to screen, check pixel distance.
        let shape = HitShape::point(obj_pos, self.inner.hit_radius());
        let hit = screen_hit_test(&shape, input.cursor, camera.view_projection, viewport).is_some();

        // Handle active drag.
        if self.state.is_dragging() {
            if !input.mouse_pressed {
                self.state.last_cursor = None;
                events.push(OverlayEvent::DragEnd);
                // Re-check hover after drag end.
                if !hit {
                    self.state.hovered = false;
                    events.push(OverlayEvent::HoverEnd);
                }
                return events;
            }
            // Safe: is_dragging() guarantees last_cursor is Some.
            let last = self.state.last_cursor.unwrap();
            let delta = input.cursor - last;
            self.state.last_cursor = Some(input.cursor);
            if delta.length() > 1e-6 {
                self.inner.on_drag(delta, ctx);
                events.push(OverlayEvent::DragMove(delta));
            }
            return events;
        }

        // Hover / click detection.
        let was_hovered = self.state.is_hovered();
        if hit {
            if !was_hovered {
                events.push(OverlayEvent::HoverStart);
            }
            self.state.hovered = true;
            if input.mouse_pressed {
                self.state.last_cursor = Some(input.cursor);
                events.push(OverlayEvent::DragStart);
            }
        } else {
            self.state.hovered = false;
            if was_hovered {
                events.push(OverlayEvent::HoverEnd);
            }
        }

        events
    }
}

#[cfg(test)]
#[path = "draggable_tests.rs"]
mod tests;
