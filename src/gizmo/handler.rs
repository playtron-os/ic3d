//! `MeshGizmo` trait and handler dispatch enum.
//!
//! Each gizmo mode (translate, rotate, scale…) implements [`MeshGizmo`].
//! The [`Gizmo`](super::Gizmo) struct delegates mode-specific work through
//! a [`GizmoHandler`] enum that dispatches to the concrete implementation.
//!
//! To add a new gizmo mode:
//! 1. Create a new file (e.g. `scale.rs`) with a struct implementing `MeshGizmo`
//! 2. Add a variant to [`GizmoHandler`] and wire up the dispatch macro
//! 3. Add the mode to [`GizmoMode`](super::types::GizmoMode)

use crate::camera::CameraInfo;
use crate::math::HitShape;
use crate::widget::MeshDrawGroup;
use glam::Vec2;
use std::fmt;

use super::rotate::RotateGizmo;
use super::translate::TranslateGizmo;
use super::types::{GizmoAxis, GizmoHit, GizmoMode, GizmoResult};

/// Mode-specific gizmo behavior for hit testing, drag interaction, and drawing.
pub(crate) trait MeshGizmo: fmt::Debug + Send + Sync {
    /// The gizmo mode this handler represents.
    fn mode(&self) -> GizmoMode;

    /// Build hit shapes for engine-side hit testing.
    fn build_hit_shapes(
        &self,
        position: glam::Vec3,
        scale: f32,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Vec<HitShape>;

    /// Map a shape index from [`build_hit_shapes`] to a [`GizmoHit`].
    fn interpret_hit(&self, index: usize, dist: f32) -> GizmoHit;

    /// Start a drag interaction on the given axis.
    fn start_drag(
        &mut self,
        axis: GizmoAxis,
        cursor: Vec2,
        position: glam::Vec3,
        scale: f32,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Option<GizmoResult>;

    /// Continue or end an active drag.
    fn continue_drag(
        &mut self,
        mouse_pressed: bool,
        camera: &CameraInfo,
        cursor: Vec2,
    ) -> Option<GizmoResult>;

    /// Whether a drag is currently active.
    fn is_dragging(&self) -> bool;

    /// The axis being dragged, if any.
    fn drag_axis(&self) -> Option<GizmoAxis>;

    /// Whether this mode supports center-area interaction (e.g. free rotation).
    fn supports_center(&self) -> bool {
        false
    }

    /// Start a center-area drag (e.g. free rotation).
    fn start_center_drag(&mut self, _cursor: Vec2) {}

    /// Draw the gizmo with full camera info for advanced rendering.
    fn draw_at(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
        center_hovered: bool,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Vec<MeshDrawGroup>;

    /// Draw the gizmo without camera info (simple fallback).
    fn draw_simple(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
    ) -> Vec<MeshDrawGroup>;
}

/// Dispatch enum wrapping concrete gizmo mode handlers.
#[derive(Debug, Clone)]
pub(crate) enum GizmoHandler {
    /// Translation (arrow handles along axes).
    Translate(TranslateGizmo),
    /// Rotation (ring handles around axes).
    Rotate(RotateGizmo),
}

impl GizmoHandler {
    /// Create a handler for the given gizmo mode.
    pub fn from_mode(mode: GizmoMode) -> Self {
        match mode {
            GizmoMode::Translate => Self::Translate(TranslateGizmo::new()),
            GizmoMode::Rotate => Self::Rotate(RotateGizmo::new()),
        }
    }
}

/// Forward a method call to the inner handler variant.
macro_rules! dispatch {
    ($self:expr, $method:ident ( $($arg:expr),* $(,)? )) => {
        match $self {
            Self::Translate(h) => h.$method($($arg),*),
            Self::Rotate(h) => h.$method($($arg),*),
        }
    };
}

impl MeshGizmo for GizmoHandler {
    fn mode(&self) -> GizmoMode {
        dispatch!(self, mode())
    }

    fn build_hit_shapes(
        &self,
        position: glam::Vec3,
        scale: f32,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Vec<HitShape> {
        dispatch!(self, build_hit_shapes(position, scale, camera, viewport))
    }

    fn interpret_hit(&self, index: usize, dist: f32) -> GizmoHit {
        dispatch!(self, interpret_hit(index, dist))
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
        dispatch!(
            self,
            start_drag(axis, cursor, position, scale, camera, viewport)
        )
    }

    fn continue_drag(
        &mut self,
        mouse_pressed: bool,
        camera: &CameraInfo,
        cursor: Vec2,
    ) -> Option<GizmoResult> {
        dispatch!(self, continue_drag(mouse_pressed, camera, cursor))
    }

    fn is_dragging(&self) -> bool {
        dispatch!(self, is_dragging())
    }

    fn drag_axis(&self) -> Option<GizmoAxis> {
        dispatch!(self, drag_axis())
    }

    fn supports_center(&self) -> bool {
        dispatch!(self, supports_center())
    }

    fn start_center_drag(&mut self, cursor: Vec2) {
        dispatch!(self, start_center_drag(cursor))
    }

    fn draw_at(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
        center_hovered: bool,
        camera: &CameraInfo,
        viewport: Vec2,
    ) -> Vec<MeshDrawGroup> {
        dispatch!(
            self,
            draw_at(position, scale, hovered, center_hovered, camera, viewport)
        )
    }

    fn draw_simple(
        &self,
        position: glam::Vec3,
        scale: f32,
        hovered: Option<GizmoAxis>,
    ) -> Vec<MeshDrawGroup> {
        dispatch!(self, draw_simple(position, scale, hovered))
    }
}
