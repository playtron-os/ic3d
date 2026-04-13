//! Rotate gizmo: rotate objects around axes with ring handles.

use crate::camera::CameraInfo;
use crate::math::{
    front_arc_params, rotation_sign, screen_angle, view_facing_rotation, world_radius_to_screen,
    world_to_screen, wrap_angle_delta, HitShape,
};
use crate::mesh::Mesh;
use crate::scene::transform::Transform;
use crate::widget::MeshDrawGroup;
use glam::Vec2;

use super::handler::MeshGizmo;
use super::types::{GizmoAxis, GizmoHit, GizmoMode, GizmoResult};
use super::HIT_THRESHOLD_PX;

// ── Rotate-specific constants ──

/// Ring mesh radius in local units (ring diameter = 2 × this).
pub(super) const RING_MESH_RADIUS: f32 = 0.55;

/// Ring tube thickness in local units.
const RING_TUBE_RADIUS: f32 = 0.008;

/// Number of segments for ring mesh and hit testing.
const RING_SEGMENTS: u32 = 48;

/// Minor segments for ring tube cross-section.
const RING_TUBE_SEGMENTS: u32 = 12;

/// Tube thickness when hovered (1.5× normal).
const RING_TUBE_RADIUS_HOVER: f32 = 0.012;

/// Radius of the view-facing (trackball) circle, slightly larger than axis rings.
const VIEW_CIRCLE_RADIUS: f32 = 0.6;

/// Tube thickness of the view-facing circle.
const VIEW_CIRCLE_TUBE_RADIUS: f32 = 0.005;

/// Color of the view-facing circle (very light white, semi-transparent feel).
const VIEW_CIRCLE_COLOR: [f32; 4] = [0.85, 0.85, 0.85, 1.0];

/// Highlight color during active axis drag (Unity yellow).
const DRAG_ACTIVE_COLOR: [f32; 4] = [1.0, 0.92, 0.016, 1.0];

/// Semi-transparent fill color for the angle indicator wedge.
const ANGLE_INDICATOR_COLOR: [f32; 4] = [1.0, 0.92, 0.016, 0.35];

/// Semi-transparent dark overlay shown when hovering the center area.
const CENTER_HOVER_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.25];

/// Sensitivity for free rotation: radians per screen pixel of mouse movement.
const FREE_ROTATE_SENSITIVITY: f32 = 0.007;

/// Internal drag state for rotation.
#[derive(Debug, Clone, Copy)]
enum RotateDrag {
    /// Constrained rotation around a single axis.
    Axis {
        axis: GizmoAxis,
        screen_center: Vec2,
        last_angle: f32,
        sign: f32,
        start_ring_angle: f32,
        total_rotation: f32,
    },
    /// Free rotation (trackball) — rotates around camera right/up.
    Free { last_cursor: Vec2 },
}

/// Rotate gizmo handler.
#[derive(Debug, Clone)]
pub(crate) struct RotateGizmo {
    drag: Option<RotateDrag>,
}

impl RotateGizmo {
    pub fn new() -> Self {
        Self { drag: None }
    }
}

impl MeshGizmo for RotateGizmo {
    fn mode(&self) -> GizmoMode {
        GizmoMode::Rotate
    }

    fn build_hit_shapes(
        &self,
        position: glam::Vec3,
        scale: f32,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Vec<HitShape> {
        let cam_forward = camera.forward.normalize_or_zero();
        let ring_radius = RING_MESH_RADIUS * scale;
        let mut shapes: Vec<HitShape> = GizmoAxis::ALL
            .iter()
            .map(|axis| {
                let rot = axis.rotation();
                let (start, sweep) = front_arc_params(rot, axis.direction(), cam_forward);
                let segments = ((sweep / std::f32::consts::TAU) * RING_SEGMENTS as f32) as u32;
                HitShape::arc(
                    position,
                    ring_radius,
                    rot,
                    start,
                    sweep,
                    segments.max(6),
                    HIT_THRESHOLD_PX,
                )
            })
            .collect();

        // Center disc as a screen-space point with pre-computed radius.
        if let Some(screen_r) = world_radius_to_screen(position, ring_radius, camera, viewport) {
            let r = screen_r - HIT_THRESHOLD_PX;
            if r > 0.0 {
                shapes.push(HitShape::point(position, r));
            }
        }
        shapes
    }

    fn interpret_hit(&self, index: usize, dist: f32) -> GizmoHit {
        if index < 3 {
            GizmoHit::Axis(GizmoAxis::ALL[index], dist)
        } else {
            GizmoHit::Center
        }
    }

    fn start_drag(
        &mut self,
        axis: GizmoAxis,
        cursor: Vec2,
        position: glam::Vec3,
        scale: f32,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Option<GizmoResult> {
        let vp = camera.view_projection;
        let sc = world_to_screen(position, vp, viewport)?;
        let screen_ang = screen_angle(sc, cursor);
        let sign = rotation_sign(camera.forward, axis.direction());

        let (tangent, bitangent) = axis.ring_plane();
        let cursor_dir = (cursor - sc).normalize_or_zero();
        let t_screen = world_to_screen(position + tangent * scale * RING_MESH_RADIUS, vp, viewport);
        let b_screen = world_to_screen(
            position + bitangent * scale * RING_MESH_RADIUS,
            vp,
            viewport,
        );
        let ring_angle = if let (Some(ts), Some(bs)) = (t_screen, b_screen) {
            let td = (ts - sc).normalize_or_zero();
            let bd = (bs - sc).normalize_or_zero();
            let proj_t = cursor_dir.dot(td);
            let proj_b = cursor_dir.dot(bd);
            proj_b.atan2(proj_t)
        } else {
            screen_ang
        };

        self.drag = Some(RotateDrag::Axis {
            axis,
            screen_center: sc,
            last_angle: screen_ang,
            sign,
            start_ring_angle: ring_angle,
            total_rotation: 0.0,
        });
        Some(GizmoResult::Hover(axis))
    }

    fn continue_drag(
        &mut self,
        mouse_pressed: bool,
        camera: &CameraInfo,
        cursor: Vec2,
    ) -> Option<GizmoResult> {
        match self.drag {
            Some(RotateDrag::Free { last_cursor }) => {
                if !mouse_pressed {
                    self.drag = None;
                    return Some(GizmoResult::Unhover);
                }
                let delta_px = cursor - last_cursor;
                self.drag = Some(RotateDrag::Free {
                    last_cursor: cursor,
                });
                if delta_px.length_squared() > 1e-6 {
                    let right = camera.forward.cross(glam::Vec3::Y).normalize_or_zero();
                    let up = right.cross(camera.forward).normalize_or_zero();
                    let angle_x = -delta_px.y * FREE_ROTATE_SENSITIVITY;
                    let angle_y = delta_px.x * FREE_ROTATE_SENSITIVITY;
                    let q = glam::Quat::from_axis_angle(up, angle_y)
                        * glam::Quat::from_axis_angle(right, angle_x);
                    Some(GizmoResult::FreeRotate(q))
                } else {
                    None
                }
            }
            Some(RotateDrag::Axis {
                axis,
                screen_center,
                last_angle,
                sign,
                start_ring_angle,
                total_rotation,
            }) => {
                if !mouse_pressed {
                    self.drag = None;
                    return Some(GizmoResult::Unhover);
                }
                let new_angle = screen_angle(screen_center, cursor);
                let delta = wrap_angle_delta(new_angle - last_angle);
                self.drag = Some(RotateDrag::Axis {
                    axis,
                    screen_center,
                    last_angle: new_angle,
                    sign,
                    start_ring_angle,
                    total_rotation: total_rotation + sign * delta,
                });
                let rotation = sign * delta;
                if rotation.abs() > 1e-6 {
                    Some(GizmoResult::Rotate(axis.direction() * rotation))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    fn drag_axis(&self) -> Option<GizmoAxis> {
        match self.drag {
            Some(RotateDrag::Axis { axis, .. }) => Some(axis),
            _ => None,
        }
    }

    fn supports_center(&self) -> bool {
        true
    }

    fn start_center_drag(&mut self, cursor: Vec2) {
        self.drag = Some(RotateDrag::Free {
            last_cursor: cursor,
        });
    }

    fn draw_at(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
        center_hovered: bool,
        camera: &CameraInfo,
        _viewport: Vec2,
    ) -> Vec<MeshDrawGroup> {
        let cam_forward = camera.forward.normalize_or_zero();
        let mut groups = Vec::new();
        let drag_axis = self.drag_axis();
        let is_free_rotating = matches!(self.drag, Some(RotateDrag::Free { .. }));

        // ── Center hover overlay (semi-transparent dark disc) ──
        if center_hovered || is_free_rotating {
            let disc = Mesh::disc_arc(
                VIEW_CIRCLE_RADIUS,
                0.0,
                std::f32::consts::TAU,
                RING_SEGMENTS,
            );
            let instance = Transform::new()
                .position(position)
                .rotation(view_facing_rotation(cam_forward))
                .uniform_scale(scale)
                .to_instance(CENTER_HOVER_COLOR);
            groups.push(MeshDrawGroup::new(disc, vec![instance]));
        }

        // ── Axis arcs (front-facing portion) ──
        for &axis in &GizmoAxis::ALL {
            let (start, sweep) = front_arc_params(axis.rotation(), axis.direction(), cam_forward);
            let arc_segments = ((sweep / std::f32::consts::TAU) * RING_SEGMENTS as f32) as u32;
            let arc_segments = arc_segments.max(6);

            let is_dragging_this = drag_axis == Some(axis);
            let is_hovered = hovered == Some(axis);

            let tube_radius = if is_hovered || is_dragging_this {
                RING_TUBE_RADIUS_HOVER
            } else {
                RING_TUBE_RADIUS
            };

            let color = if is_dragging_this {
                DRAG_ACTIVE_COLOR
            } else if is_hovered {
                axis.highlight_color()
            } else {
                axis.color()
            };

            let arc = Mesh::torus_arc(
                RING_MESH_RADIUS,
                tube_radius,
                start,
                sweep,
                arc_segments,
                RING_TUBE_SEGMENTS,
            );
            let instance = Transform::new()
                .position(position)
                .rotation(axis.rotation())
                .uniform_scale(scale)
                .to_instance(color);
            groups.push(MeshDrawGroup::new(arc, vec![instance]));
        }

        // ── Angle indicator wedge (during drag) ──
        if let Some(RotateDrag::Axis {
            axis,
            start_ring_angle,
            total_rotation,
            ..
        }) = self.drag
        {
            let abs_rot = total_rotation.abs();
            if abs_rot > 0.01 {
                let (wedge_start, wedge_sweep) = if total_rotation > 0.0 {
                    (start_ring_angle, total_rotation)
                } else {
                    (start_ring_angle + total_rotation, -total_rotation)
                };

                let wedge = Mesh::disc_arc(
                    RING_MESH_RADIUS,
                    wedge_start,
                    wedge_sweep,
                    (abs_rot / std::f32::consts::PI * 24.0).max(3.0) as u32,
                );
                let instance = Transform::new()
                    .position(position)
                    .rotation(axis.rotation())
                    .uniform_scale(scale)
                    .to_instance(ANGLE_INDICATOR_COLOR);
                groups.push(MeshDrawGroup::new(wedge, vec![instance]));

                // Radial lines from center to start/end of wedge.
                let line_start =
                    Mesh::torus_arc(RING_MESH_RADIUS * 0.5, 0.004, start_ring_angle, 0.01, 2, 4);
                let line_end_angle = start_ring_angle + total_rotation;
                let line_end =
                    Mesh::torus_arc(RING_MESH_RADIUS * 0.5, 0.004, line_end_angle, 0.01, 2, 4);
                let line_instance = Transform::new()
                    .position(position)
                    .rotation(axis.rotation())
                    .uniform_scale(scale)
                    .to_instance(DRAG_ACTIVE_COLOR);
                groups.push(MeshDrawGroup::new(line_start, vec![line_instance]));
                groups.push(MeshDrawGroup::new(line_end, vec![line_instance]));
            }
        }

        // ── View-facing circle ──
        let view_circle = Mesh::torus(
            VIEW_CIRCLE_RADIUS,
            VIEW_CIRCLE_TUBE_RADIUS,
            RING_SEGMENTS,
            RING_TUBE_SEGMENTS,
        );
        let instance = Transform::new()
            .position(position)
            .rotation(view_facing_rotation(cam_forward))
            .uniform_scale(scale)
            .to_instance(VIEW_CIRCLE_COLOR);
        groups.push(MeshDrawGroup::new(view_circle, vec![instance]));

        groups
    }

    fn draw_simple(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
    ) -> Vec<MeshDrawGroup> {
        let ring = Mesh::torus(
            RING_MESH_RADIUS,
            RING_TUBE_RADIUS,
            RING_SEGMENTS,
            RING_TUBE_SEGMENTS,
        );
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

                MeshDrawGroup::new(ring.clone(), vec![instance])
            })
            .collect()
    }
}
