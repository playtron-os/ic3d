//! Translate gizmo: drag objects along axes with arrow handles.

use crate::camera::CameraInfo;
use crate::math::{world_to_screen, HitShape};
use crate::mesh::Mesh;
use crate::scene::transform::Transform;
use crate::widget::MeshDrawGroup;
use glam::Vec2;

use super::handler::MeshGizmo;
use super::types::{GizmoAxis, GizmoHit, GizmoMode, GizmoResult};
use super::HIT_THRESHOLD_PX;

/// Internal drag state for translation.
#[derive(Debug, Clone, Copy)]
struct TranslateDrag {
    axis: GizmoAxis,
    last_cursor: Vec2,
    screen_axis_dir: Vec2,
    world_per_px: f32,
}

/// Translate gizmo handler.
#[derive(Debug, Clone)]
pub(crate) struct TranslateGizmo {
    drag: Option<TranslateDrag>,
}

impl TranslateGizmo {
    pub fn new() -> Self {
        Self { drag: None }
    }

    fn draw_arrows(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
    ) -> Vec<MeshDrawGroup> {
        let arrow = Mesh::arrow(1.0);
        let drag_axis = self.drag_axis();

        GizmoAxis::ALL
            .iter()
            .map(|&axis| {
                let color = if drag_axis == Some(axis) || hovered == Some(axis) {
                    axis.highlight_color()
                } else {
                    axis.color()
                };

                let instance = Transform::new()
                    .position(position)
                    .rotation(axis.rotation())
                    .uniform_scale(scale)
                    .to_instance(color);

                MeshDrawGroup::new(arrow.clone(), vec![instance])
            })
            .collect()
    }
}

impl MeshGizmo for TranslateGizmo {
    fn mode(&self) -> GizmoMode {
        GizmoMode::Translate
    }

    fn build_hit_shapes(
        &self,
        position: glam::Vec3,
        scale: f32,
        _camera: &CameraInfo,
        _viewport: Vec2,
    ) -> Vec<HitShape> {
        GizmoAxis::ALL
            .iter()
            .map(|axis| {
                HitShape::segment(
                    position,
                    position + axis.direction() * scale,
                    HIT_THRESHOLD_PX,
                )
            })
            .collect()
    }

    fn interpret_hit(&self, index: usize, dist: f32) -> GizmoHit {
        GizmoHit::Axis(GizmoAxis::ALL[index], dist)
    }

    fn start_drag(
        &mut self,
        axis: GizmoAxis,
        cursor: Vec2,
        position: glam::Vec3,
        _scale: f32,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Option<GizmoResult> {
        let vp = camera.view_projection;
        let p0 = world_to_screen(position, vp, viewport);
        let p1 = world_to_screen(position + axis.direction(), vp, viewport);

        if let (Some(s0), Some(s1)) = (p0, p1) {
            let screen_dir = s1 - s0;
            let screen_len = screen_dir.length();
            if screen_len > 1e-6 {
                self.drag = Some(TranslateDrag {
                    axis,
                    last_cursor: cursor,
                    screen_axis_dir: screen_dir / screen_len,
                    world_per_px: 1.0 / screen_len,
                });
                return Some(GizmoResult::Hover(axis));
            }
        }
        None
    }

    fn continue_drag(
        &mut self,
        mouse_pressed: bool,
        _camera: &CameraInfo,
        cursor: Vec2,
    ) -> Option<GizmoResult> {
        let drag = self.drag.as_mut()?;

        if !mouse_pressed {
            self.drag = None;
            return Some(GizmoResult::Unhover);
        }

        let cursor_delta = cursor - drag.last_cursor;
        let along_axis = cursor_delta.dot(drag.screen_axis_dir);
        let world_delta = drag.axis.direction() * along_axis * drag.world_per_px;
        drag.last_cursor = cursor;

        if world_delta.length() > 1e-8 {
            Some(GizmoResult::Translate(world_delta))
        } else {
            None
        }
    }

    fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    fn drag_axis(&self) -> Option<GizmoAxis> {
        self.drag.map(|d| d.axis)
    }

    fn draw_at(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
        _center_hovered: bool,
        _camera: &CameraInfo,
        _viewport: Vec2,
    ) -> Vec<MeshDrawGroup> {
        self.draw_arrows(position, scale, hovered)
    }

    fn draw_simple(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
    ) -> Vec<MeshDrawGroup> {
        self.draw_arrows(position, scale, hovered)
    }
}
