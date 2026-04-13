//! Gizmo input handling: cursor tracking, hit testing, and drag interaction.

use crate::camera::CameraInfo;
use crate::math::{screen_hit_test, HitShape};
use crate::scene::context::{SceneContext, SceneHandle};

use super::types::{GizmoAxis, GizmoMode, GizmoResult};
use super::{DragState, Gizmo, HIT_THRESHOLD_PX};
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

        self.update_with_camera(cursor_pos, mouse_pressed, &camera, viewport_size)
    }

    /// Process cursor input with explicit camera/viewport data.
    ///
    /// This is the low-level input handler used internally by the widget
    /// and by [`update`](Self::update). Call this when you have camera
    /// metadata already available (avoiding the [`SceneHandle`] lookup).
    ///
    /// Does **not** sync position from attached objects — call
    /// [`set_position`](Self::set_position) first if needed.
    pub(crate) fn update_with_camera(
        &mut self,
        cursor_pos: Vec2,
        mouse_pressed: bool,
        camera: &CameraInfo,
        viewport_size: Vec2,
    ) -> Option<GizmoResult> {
        // Auto-compute scale for hit testing from camera metadata.
        let scale = self.compute_scale(camera, viewport_size.y);
        self.scale = scale;

        match self.mode {
            GizmoMode::Translate => {
                self.update_translate(mouse_pressed, camera, viewport_size, cursor_pos)
            }
        }
    }

    /// Translation gizmo logic.
    fn update_translate(
        &mut self,
        mouse_pressed: bool,
        camera: &CameraInfo,
        viewport_size: Vec2,
        cursor_pos: Vec2,
    ) -> Option<GizmoResult> {
        // If currently dragging
        if let Some(drag) = &self.drag {
            if !mouse_pressed {
                // Drag ended
                self.drag = None;
                self.hovered = None;
                return None;
            }

            let axis = drag.axis;
            let axis_dir = axis.direction();

            // Compute world-space delta from screen-space cursor movement.
            // Project cursor delta onto the axis's screen direction, then
            // convert to world units. This is immune to camera rotation.
            let cursor_delta = cursor_pos - drag.last_cursor;
            let along_axis = cursor_delta.dot(drag.screen_axis_dir);
            let world_delta = axis_dir * along_axis * drag.world_per_px;

            self.drag = Some(DragState {
                axis,
                last_cursor: cursor_pos,
                screen_axis_dir: drag.screen_axis_dir,
                world_per_px: drag.world_per_px,
            });

            if world_delta.length() > 1e-8 {
                return Some(GizmoResult::Translate(world_delta));
            }
            return None;
        }

        // Not dragging — test hover
        let hit = self.hit_test(camera, viewport_size, cursor_pos);

        if mouse_pressed {
            if let Some((axis, _dist)) = hit {
                let axis_dir = axis.direction();
                let vp = camera.view_projection;

                // Compute the screen-space direction of this axis.
                let p0 = world_to_screen(self.position, vp, viewport_size);
                let p1 = world_to_screen(self.position + axis_dir, vp, viewport_size);

                if let (Some(s0), Some(s1)) = (p0, p1) {
                    let screen_dir = s1 - s0;
                    let screen_len = screen_dir.length();
                    if screen_len > 1e-6 {
                        let screen_axis_dir = screen_dir / screen_len;
                        // world_per_px: 1 world unit along this axis ≈ screen_len px
                        let world_per_px = 1.0 / screen_len;

                        self.drag = Some(DragState {
                            axis,
                            last_cursor: cursor_pos,
                            screen_axis_dir,
                            world_per_px,
                        });
                        self.hovered = Some(axis);
                        return Some(GizmoResult::Hover(axis));
                    }
                }
            }
        }

        let hit_axis = hit.map(|(axis, _)| axis);
        self.hovered = hit_axis;
        hit_axis.map(GizmoResult::Hover)
    }

    /// Test which axis the cursor is closest to (if within threshold).
    ///
    /// Returns the axis and the screen-space pixel distance to it.
    fn hit_test(
        &self,
        camera: &CameraInfo,
        viewport_size: Vec2,
        cursor_pos: Vec2,
    ) -> Option<(GizmoAxis, f32)> {
        let mut best: Option<(GizmoAxis, f32)> = None;

        for &axis in &GizmoAxis::ALL {
            let arrow_start = self.position;
            let arrow_end = self.position + axis.direction() * self.scale;
            let shape = HitShape::segment(arrow_start, arrow_end, HIT_THRESHOLD_PX);

            if let Some(dist) =
                screen_hit_test(&shape, cursor_pos, camera.view_projection, viewport_size)
            {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((axis, dist));
                }
            }
        }

        best
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

        // Resolve effective position (attached objects follow their target).
        let position = self
            .attached_to
            .and_then(|id| scene.object_position(id))
            .unwrap_or(self.position);

        self.probe_resolved(cursor_pos, &camera, viewport_size, position)
    }

    /// Read-only hit test with explicit context (avoids [`SceneHandle`] lock).
    ///
    /// Used by [`SceneHandle::process_gizmo`](crate::SceneHandle) to probe
    /// multiple managed gizmos without re-acquiring the lock.
    pub(crate) fn probe_at(
        &self,
        cursor_pos: Vec2,
        ctx: &SceneContext,
    ) -> Option<(GizmoAxis, f32)> {
        let position = self
            .attached_to
            .and_then(|id| ctx.object_position(id))
            .unwrap_or(self.position);
        self.probe_resolved(cursor_pos, &ctx.camera, ctx.viewport_size, position)
    }

    /// Shared hit-test logic for [`probe`](Self::probe) and
    /// [`probe_at`](Self::probe_at).
    fn probe_resolved(
        &self,
        cursor_pos: Vec2,
        camera: &CameraInfo,
        viewport_size: Vec2,
        position: glam::Vec3,
    ) -> Option<(GizmoAxis, f32)> {
        let scale = self.compute_scale_at(camera, viewport_size.y, position);

        let mut best: Option<(GizmoAxis, f32)> = None;
        for &axis in &GizmoAxis::ALL {
            let arrow_start = position;
            let arrow_end = position + axis.direction() * scale;
            let shape = HitShape::segment(arrow_start, arrow_end, HIT_THRESHOLD_PX);

            if let Some(dist) =
                screen_hit_test(&shape, cursor_pos, camera.view_projection, viewport_size)
            {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((axis, dist));
                }
            }
        }

        best
    }

    /// Clear the hover state without running input logic.
    ///
    /// Used by [`SceneHandle`] to clear hover on gizmos that lost the
    /// closest-hit contest.
    pub(crate) fn clear_hover(&mut self) {
        self.hovered = None;
    }
}

// ──────────── Helpers ────────────

/// Project a world-space point to screen-space pixels.
pub(super) fn world_to_screen(
    point: glam::Vec3,
    view_proj: glam::Mat4,
    viewport_size: Vec2,
) -> Option<Vec2> {
    crate::math::world_to_screen(point, view_proj, viewport_size)
}
