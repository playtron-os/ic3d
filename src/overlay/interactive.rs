//! Engine-managed interactive overlay with multi-shape hit testing.
//!
//! [`InteractiveOverlay`] sits between the simple [`DraggableOverlay`](super::draggable::DraggableOverlay)
//! (single point, pixel deltas) and raw [`Overlay`](super::base::Overlay)
//! (everything manual). The engine handles:
//!
//! - Multi-shape hit testing via [`HitShape`](crate::math::HitShape)
//! - Shape index identification (which handle was hit)
//! - Hover state tracking
//! - Drag lifecycle (start / continue / end)
//! - Scale computation for constant screen-space size
//!
//! Consumers define: what to target, what shapes to test, what to do on
//! hover/drag, and how to draw. Wrap with [`Interactive::new`] when adding
//! as an overlay.
//!
//! # Example
//!
//! ```rust,ignore
//! use ic3d::{InteractiveOverlay, Interactive, InteractiveContext, ShapeHit};
//! use ic3d::math::HitShape;
//! use ic3d::{SceneHandle, SceneObjectId};
//! use ic3d::widget::MeshDrawGroup;
//! use ic3d::glam::Vec2;
//!
//! #[derive(Debug, Clone)]
//! struct ScaleHandle {
//!     hovered: Option<usize>,
//!     dragging: bool,
//! }
//!
//! impl InteractiveOverlay for ScaleHandle {
//!     fn resolve_target(&self, handle: &SceneHandle) -> Option<SceneObjectId> {
//!         handle.selected_objects().into_iter().next()
//!     }
//!
//!     fn hit_shapes(&self, ctx: &InteractiveContext) -> Vec<HitShape> {
//!         // 3 axis-aligned segments
//!         vec![
//!             HitShape::segment(ctx.position, ctx.position + ctx.camera.forward, 20.0),
//!         ]
//!     }
//!
//!     fn on_hover(&mut self, hit: &ShapeHit) { self.hovered = Some(hit.shape_index); }
//!     fn on_unhover(&mut self) { self.hovered = None; }
//!
//!     fn on_drag_start(&mut self, _hit: &ShapeHit, _cursor: Vec2, _ctx: &InteractiveContext,
//!                      _nodes: &mut OverlayContext) -> bool {
//!         self.dragging = true;
//!         true
//!     }
//!     fn on_drag_continue(&mut self, _cursor: Vec2, _ctx: &InteractiveContext,
//!                         _nodes: &mut OverlayContext) {}
//!     fn on_drag_end(&mut self, _nodes: &mut OverlayContext) { self.dragging = false; }
//!     fn is_dragging(&self) -> bool { self.dragging }
//!
//!     fn draw(&self, _ctx: &InteractiveContext) -> Vec<MeshDrawGroup> {
//!         Vec::new()
//!     }
//! }
//!
//! // Register:
//! // graph.add_overlay(Interactive::new(ScaleHandle { hovered: None, dragging: false }));
//! ```

use crate::camera::CameraInfo;
use crate::math::{screen_constant_scale, screen_hit_test_closest, HitShape};
use crate::overlay::base::{Overlay, OverlayContext, OverlayEvent, OverlayInput};
use crate::scene::context::{SceneContext, SceneHandle};
use crate::scene::object::SceneObjectId;
use crate::widget::MeshDrawGroup;
use glam::Vec2;
use std::fmt;

/// Context passed to [`InteractiveOverlay`] methods.
///
/// Provides the resolved target position, computed scale, camera metadata,
/// and viewport dimensions. Created by the engine wrapper — consumers
/// never construct this directly.
#[derive(Debug, Clone)]
pub struct InteractiveContext {
    /// World-space position of the target object.
    pub position: glam::Vec3,
    /// Visual scale for constant screen-space size.
    pub scale: f32,
    /// Camera metadata for this frame.
    pub camera: CameraInfo,
    /// Viewport dimensions in logical pixels.
    pub viewport: Vec2,
}

/// Result of shape-level hit testing.
///
/// Identifies which shape from [`InteractiveOverlay::hit_shapes`] was hit
/// and how far the cursor is from it in screen pixels.
#[derive(Debug, Clone, Copy)]
pub struct ShapeHit {
    /// Index into the array returned by [`InteractiveOverlay::hit_shapes`].
    pub shape_index: usize,
    /// Screen-space pixel distance to the shape.
    pub distance: f32,
}

/// A multi-shape interactive overlay with engine-managed hit testing and
/// drag tracking.
///
/// Implement this for overlays that need complex hit shapes (segments, arcs,
/// points) and drag interaction. The engine handles hover/drag state and
/// hit testing — you define how to respond.
///
/// Wrap with [`Interactive::new`] when registering as an overlay:
///
/// ```rust,ignore
/// graph.add_overlay(Interactive::new(my_handle));
/// ```
pub trait InteractiveOverlay: fmt::Debug + Send + Sync {
    /// Resolve the target scene object for this frame.
    ///
    /// Return `None` to hide and disable the overlay.
    fn resolve_target(&self, handle: &SceneHandle) -> Option<SceneObjectId>;

    /// Desired on-screen size in pixels (default: 80).
    ///
    /// The engine computes a world-space scale factor from this value,
    /// camera distance, and FOV so the overlay maintains constant
    /// screen-space size regardless of zoom.
    fn screen_size(&self) -> f32 {
        80.0
    }

    /// Build hittable shapes for engine-side hit testing.
    ///
    /// Return an ordered list of [`HitShape`]s. The engine tests all shapes
    /// against the cursor and passes the closest hit's index to
    /// [`on_hover`](Self::on_hover) or [`on_drag_start`](Self::on_drag_start).
    fn hit_shapes(&self, ctx: &InteractiveContext) -> Vec<HitShape>;

    /// Called when a shape is hovered (cursor over, not pressed).
    fn on_hover(&mut self, hit: &ShapeHit);

    /// Called when the cursor leaves all hit shapes.
    fn on_unhover(&mut self);

    /// Called when the user clicks on a shape, starting a drag.
    ///
    /// Return `true` to accept the drag, `false` to reject it.
    /// Use `nodes` to read or mutate scene node transforms.
    fn on_drag_start(
        &mut self,
        hit: &ShapeHit,
        cursor: Vec2,
        ctx: &InteractiveContext,
        nodes: &mut OverlayContext,
    ) -> bool;

    /// Called each frame during an active drag.
    ///
    /// Use `nodes` to read or mutate scene node transforms.
    fn on_drag_continue(
        &mut self,
        cursor: Vec2,
        ctx: &InteractiveContext,
        nodes: &mut OverlayContext,
    );

    /// Called when a drag ends (mouse released).
    ///
    /// Use `nodes` for final mutations (e.g. commit undo snapshots).
    fn on_drag_end(&mut self, nodes: &mut OverlayContext);

    /// Whether a drag is currently active.
    fn is_dragging(&self) -> bool;

    /// Generate [`MeshDrawGroup`]s for rendering.
    fn draw(&self, ctx: &InteractiveContext) -> Vec<MeshDrawGroup>;
}

/// Engine wrapper that adds hit testing and drag state management to an
/// [`InteractiveOverlay`].
///
/// Created via [`Interactive::new`]. Register with
/// [`SceneGraph::add_overlay`](crate::graph::SceneGraph::add_overlay).
///
/// ```rust,ignore
/// graph.add_overlay(Interactive::new(my_handle));
/// ```
#[derive(Debug, Clone)]
pub struct Interactive<T: InteractiveOverlay> {
    inner: T,
    target: Option<SceneObjectId>,
    hovered: bool,
}

impl<T: InteractiveOverlay> Interactive<T> {
    /// Wrap an [`InteractiveOverlay`] with engine-managed state.
    #[must_use]
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            target: None,
            hovered: false,
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

    /// The currently cached target.
    #[must_use]
    pub fn target(&self) -> Option<SceneObjectId> {
        self.target
    }

    /// Whether any shape is hovered.
    #[must_use]
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Build an [`InteractiveContext`] from a [`SceneContext`] and target.
    fn build_context(&self, ctx: &SceneContext) -> Option<InteractiveContext> {
        let target = self.target?;
        let position = ctx.object_position(target)?;
        let scale = screen_constant_scale(
            position,
            &ctx.camera,
            ctx.viewport_size.y,
            self.inner.screen_size(),
        );
        Some(InteractiveContext {
            position,
            scale,
            camera: ctx.camera,
            viewport: ctx.viewport_size,
        })
    }

    /// Build an [`InteractiveContext`] from a [`SceneHandle`] and target.
    fn build_context_from_handle(&self, handle: &SceneHandle) -> Option<InteractiveContext> {
        let target = self.target?;
        let camera = handle.camera()?;
        let viewport = handle.viewport_size();
        let position = handle.object_position(target)?;
        let scale = screen_constant_scale(position, &camera, viewport.y, self.inner.screen_size());
        Some(InteractiveContext {
            position,
            scale,
            camera,
            viewport,
        })
    }
}

impl<T: InteractiveOverlay + Clone + 'static> Overlay for Interactive<T> {
    fn visible(&self) -> bool {
        // Always visible so on_input runs and can resolve the target.
        // When target is None, draw() and hit_shapes() return empty.
        true
    }

    fn interactive(&self) -> bool {
        true
    }

    fn hit_shapes(&self, ctx: &SceneContext) -> Vec<HitShape> {
        match self.build_context(ctx) {
            Some(ictx) => self.inner.hit_shapes(&ictx),
            None => Vec::new(),
        }
    }

    fn draw(&self, ctx: &SceneContext) -> Vec<MeshDrawGroup> {
        match self.build_context(ctx) {
            Some(ictx) => self.inner.draw(&ictx),
            None => Vec::new(),
        }
    }

    fn on_input(&mut self, input: &OverlayInput, ctx: &mut OverlayContext) -> Vec<OverlayEvent> {
        let mut events = Vec::new();

        // Resolve target each frame.
        self.target = self.inner.resolve_target(ctx.handle());
        let Some(ictx) = self.build_context_from_handle(ctx.handle()) else {
            // No target or no camera — clear state if needed.
            if self.hovered {
                self.inner.on_unhover();
                self.hovered = false;
                events.push(OverlayEvent::HoverEnd);
            }
            return events;
        };

        // ── Active drag ──
        if self.inner.is_dragging() {
            if !input.mouse_pressed {
                self.inner.on_drag_end(ctx);
                events.push(OverlayEvent::DragEnd);

                // Re-check hover after drag end.
                if let Some(hit) = self.find_closest_hit(input.cursor, &ictx) {
                    self.inner.on_hover(&hit);
                    self.hovered = true;
                } else {
                    self.inner.on_unhover();
                    self.hovered = false;
                    events.push(OverlayEvent::HoverEnd);
                }
            } else {
                self.inner.on_drag_continue(input.cursor, &ictx, ctx);
                events.push(OverlayEvent::DragMove(Vec2::ZERO));
            }
            return events;
        }

        // ── Hit test ──
        let hit = self.find_closest_hit(input.cursor, &ictx);
        let was_hovered = self.hovered;

        if let Some(shape_hit) = hit {
            if !was_hovered {
                events.push(OverlayEvent::HoverStart);
            }
            self.hovered = true;

            if input.mouse_pressed
                && self
                    .inner
                    .on_drag_start(&shape_hit, input.cursor, &ictx, ctx)
            {
                events.push(OverlayEvent::DragStart);
            } else {
                self.inner.on_hover(&shape_hit);
            }
        } else {
            self.hovered = false;
            if was_hovered {
                self.inner.on_unhover();
                events.push(OverlayEvent::HoverEnd);
            }
        }

        events
    }
}

impl<T: InteractiveOverlay> Interactive<T> {
    /// Find the closest hit shape at the given cursor position.
    fn find_closest_hit(&self, cursor: Vec2, ictx: &InteractiveContext) -> Option<ShapeHit> {
        let shapes = self.inner.hit_shapes(ictx);
        screen_hit_test_closest(
            shapes.into_iter().enumerate(),
            cursor,
            ictx.camera.view_projection,
            ictx.viewport,
        )
        .map(|(shape_index, distance)| ShapeHit {
            shape_index,
            distance,
        })
    }
}

#[cfg(test)]
#[path = "interactive_tests.rs"]
mod tests;
