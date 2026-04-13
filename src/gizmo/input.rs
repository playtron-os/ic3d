//! Gizmo input handling: cursor tracking, hit testing, and drag interaction.
//!
//! Contains the high-level input methods on [`Gizmo`] that delegate
//! mode-specific work to the [`MeshGizmo`](super::handler::MeshGizmo) handler.

use crate::camera::CameraInfo;
use crate::math::{screen_hit_test_closest, HitShape};
use crate::scene::context::SceneHandle;

use super::handler::MeshGizmo;
use super::types::{GizmoAxis, GizmoHit, GizmoResult};
use super::Gizmo;
use glam::Vec2;

impl Gizmo {
    /// Process cursor input for one frame.
    ///
    /// Call this every frame with the current cursor state. The gizmo
    /// reads camera and viewport info from the [`SceneHandle`] — no
    /// camera construction or parameter plumbing needed.
    ///
    /// For attached gizmos ([`attach_to`](Self::attach_to)), the position
    /// is automatically synced from the scene object each frame.
    ///
    /// - `cursor_pos`: screen-space cursor position in pixels (top-left origin)
    /// - `mouse_pressed`: whether the primary mouse button is currently held
    /// - `scene`: shared scene state (camera, viewport, object transforms)
    ///
    /// Returns `None` if no camera data is available (first frame) or no
    /// interaction occurred. Returns `Some(GizmoResult)` on hover or drag.
    pub fn update(
        &mut self,
        cursor_pos: Vec2,
        mouse_pressed: bool,
        scene: &SceneHandle,
    ) -> Option<GizmoResult> {
        let camera = scene.camera()?;
        let viewport_size = scene.viewport_size();

        // If attached, sync position from scene object.
        if let Some(target_id) = self.attached_to {
            if let Some(pos) = scene.object_position(target_id) {
                self.position = pos;
            }
        }

        let scale = self.compute_scale(&camera, viewport_size.y);
        self.scale = scale;

        // Engine-side hit testing: build shapes, test closest, pass to update.
        let hit = if !self.handler.is_dragging() {
            let shapes = self.build_hit_shapes(self.position, scale, &camera, viewport_size);
            screen_hit_test_closest(
                shapes.into_iter().enumerate(),
                cursor_pos,
                camera.view_projection,
                viewport_size,
            )
            .map(|(idx, dist)| self.interpret_hit(idx, dist))
        } else {
            None
        };

        self.update_with_hit(cursor_pos, mouse_pressed, &camera, viewport_size, hit)
    }

    /// Build hit shapes for this gizmo at the given position and scale.
    ///
    /// Delegates to the mode-specific handler.
    pub(crate) fn build_hit_shapes(
        &self,
        position: glam::Vec3,
        scale: f32,
        camera: &CameraInfo,
        viewport_size: Vec2,
    ) -> Vec<HitShape> {
        self.handler
            .build_hit_shapes(position, scale, camera, viewport_size)
    }

    /// Map a shape index from [`build_hit_shapes`] to a [`GizmoHit`].
    pub(crate) fn interpret_hit(&self, index: usize, dist: f32) -> GizmoHit {
        self.handler.interpret_hit(index, dist)
    }

    /// Process cursor input with a pre-computed hit from engine-side testing.
    ///
    /// Called by [`SceneHandle::process_gizmo`](crate::SceneHandle) after
    /// testing [`hit_shapes`](crate::Overlay::hit_shapes) via
    /// [`screen_hit_test_closest`](crate::math::screen_hit_test_closest).
    /// The engine passes the winning shape as a [`GizmoHit`], eliminating
    /// the need for the gizmo to re-test internally.
    ///
    /// During an active drag, `hit` is ignored — the gizmo continues
    /// tracking the cursor without hit testing.
    pub(crate) fn update_with_hit(
        &mut self,
        cursor_pos: Vec2,
        mouse_pressed: bool,
        camera: &CameraInfo,
        viewport_size: Vec2,
        hit: Option<GizmoHit>,
    ) -> Option<GizmoResult> {
        let scale = self.compute_scale(camera, viewport_size.y);
        self.scale = scale;

        // ── Active drag: continue regardless of hit ──
        if self.handler.is_dragging() {
            let result = self
                .handler
                .continue_drag(mouse_pressed, camera, cursor_pos);
            if !self.handler.is_dragging() {
                // Drag just ended — clear common hover state.
                self.hovered = None;
                self.center_hovered = false;
            }
            return result;
        }

        // ── Not dragging: use engine-provided hit ──
        let axis_hit = match hit {
            Some(GizmoHit::Axis(axis, _)) => Some(axis),
            _ => None,
        };
        let center_hit = matches!(hit, Some(GizmoHit::Center));

        if mouse_pressed {
            // Axis takes priority over center.
            if let Some(axis) = axis_hit {
                if let Some(result) = self.handler.start_drag(
                    axis,
                    cursor_pos,
                    self.position,
                    scale,
                    camera,
                    viewport_size,
                ) {
                    self.hovered = Some(axis);
                    self.center_hovered = false;
                    return Some(result);
                }
                return None;
            }
            if center_hit && self.handler.supports_center() {
                self.handler.start_center_drag(cursor_pos);
                self.hovered = None;
                self.center_hovered = true;
                return Some(GizmoResult::HoverCenter);
            }
        }

        // Update hover state.
        if let Some(axis) = axis_hit {
            self.hovered = Some(axis);
            self.center_hovered = false;
            return Some(GizmoResult::Hover(axis));
        }

        if center_hit {
            self.hovered = None;
            let was_center = self.center_hovered;
            self.center_hovered = true;
            return if was_center {
                None
            } else {
                Some(GizmoResult::HoverCenter)
            };
        }

        // Not hovering anything.
        let had_hover = self.hovered.is_some() || self.center_hovered;
        self.hovered = None;
        self.center_hovered = false;
        if had_hover {
            return Some(GizmoResult::Unhover);
        }
        None
    }

    /// Test if the cursor hits this gizmo without modifying internal state.
    ///
    /// Returns the closest axis and the screen-space pixel distance to it.
    /// Use this to compare multiple gizmos and pick the best hit before
    /// calling [`update`](Self::update) on only the winner.
    ///
    /// ```rust,ignore
    /// // Probe all gizmos, pick the one with smallest screen distance:
    /// let hits: Vec<_> = gizmos.iter()
    ///     .filter_map(|(g, id)| g.probe(cursor, &scene).map(|(a, d)| (id, a, d)))
    ///     .collect();
    /// if let Some((best_id, ..)) = hits.iter().min_by(|a, b| a.2.partial_cmp(&b.2).unwrap()) {
    ///     // Only update the closest gizmo
    /// }
    /// ```
    #[must_use]
    pub fn probe(&self, cursor_pos: Vec2, scene: &SceneHandle) -> Option<(GizmoAxis, f32)> {
        let camera = scene.camera()?;
        let viewport_size = scene.viewport_size();

        let position = self
            .attached_to
            .and_then(|id| scene.object_position(id))
            .unwrap_or(self.position);
        let scale = self.compute_scale_at(&camera, viewport_size.y, position);

        let shapes = self.build_hit_shapes(position, scale, &camera, viewport_size);
        let closest = screen_hit_test_closest(
            shapes.into_iter().enumerate(),
            cursor_pos,
            camera.view_projection,
            viewport_size,
        )?;
        let (index, dist) = closest;
        match self.interpret_hit(index, dist) {
            GizmoHit::Axis(axis, d) => Some((axis, d)),
            // Center hit uses large distance so axis hits on other gizmos
            // take priority in multi-gizmo contests.
            GizmoHit::Center => Some((GizmoAxis::Y, f32::MAX / 2.0)),
        }
    }

    /// Clear the hover state without running input logic.
    ///
    /// Used by [`SceneHandle`] to clear hover on gizmos that lost the
    /// closest-hit contest.
    pub(crate) fn clear_hover(&mut self) {
        self.hovered = None;
        self.center_hovered = false;
    }
}
