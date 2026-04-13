//! 3D manipulation gizmo for scene editing workflows.
//!
//! Provides translation handles (arrows along X/Y/Z) with mouse-based
//! interaction: hover detection, click-drag, and world-space delta output.
//!
//! Gizmo draws are rendered as **overlays** — they always appear on top
//! of scene geometry (no depth occlusion, no shadows). The gizmo implements
//! the [`Overlay`] trait so scaling is handled automatically
//! from scene context — no manual camera parameter plumbing needed.
//!
//! # Usage
//!
//! The recommended way to use gizmos is via the **widget-managed** approach.
//! The widget handles all cursor tracking, hit testing, and rendering:
//!
//! ```rust,ignore
//! use ic3d::gizmo::{GizmoMode, GizmoResult};
//! use ic3d::widget::{scene_3d, MeshDrawGroup};
//! use ic3d::{SceneHandle, SceneObjectId};
//!
//! // Create scene handle and select an object for manipulation.
//! let scene = SceneHandle::new();
//! scene.select(SceneObjectId(1), GizmoMode::Translate);
//!
//! // In view() — widget manages gizmo input and rendering:
//! scene_3d(my_scene)
//!     .scene(scene.clone())
//!     .on_gizmo(Message::Gizmo)
//!
//! // In update() — receive gizmo results as messages:
//! match message {
//!     Message::Gizmo(id, GizmoResult::Translate(delta)) => {
//!         object_position += delta;
//!     }
//!     _ => {}
//! }
//! ```

mod input;
mod types;

#[cfg(test)]
use input::world_to_screen;
pub use types::{GizmoAxis, GizmoMode, GizmoResult};

use crate::camera::CameraInfo;
use crate::mesh::Mesh;
use crate::overlay::base::Overlay;
use crate::scene::context::SceneContext;
use crate::scene::object::SceneObjectId;
use crate::scene::transform::Transform;
use crate::widget::MeshDrawGroup;
use glam::{Vec2, Vec3};

/// Screen-space hit threshold in pixels for axis hover detection.
const HIT_THRESHOLD_PX: f32 = 20.0;

/// Default on-screen gizmo size.
const DEFAULT_GIZMO_SIZE: f32 = 80.0;

/// Internal drag state tracked across frames.
#[derive(Debug, Clone, Copy)]
struct DragState {
    axis: GizmoAxis,
    /// Last cursor position in screen pixels — delta is computed from this.
    last_cursor: Vec2,
    /// Screen-space direction of the axis (unit vector), captured at drag start.
    screen_axis_dir: Vec2,
    /// World-space units per screen pixel along the axis, captured at drag start.
    world_per_px: f32,
}

/// A 3D manipulation gizmo.
///
/// Renders translation arrows along X/Y/Z and handles mouse interaction.
/// Feed it cursor position and mouse-button state each frame via [`update`](Self::update).
///
/// Implements [`Overlay`] — place it in
/// [`Scene3DSetup::overlays`](crate::widget::Scene3DSetup::overlays) as
/// `Box::new(gizmo.clone())`. Scaling is computed automatically from
/// scene context; no manual parameter plumbing needed.
///
/// Use [`attach_to`](Self::attach_to) to make the gizmo follow a scene
/// object automatically — no manual `set_position()` needed.
#[derive(Debug, Clone)]
pub struct Gizmo {
    mode: GizmoMode,
    position: Vec3,
    /// Visual scale of the gizmo handles (world units).
    scale: f32,
    /// On-screen size setting (default 80, range 16–160).
    gizmo_size: f32,
    /// Whether the gizmo is visible.
    visible: bool,
    /// Whether the gizmo participates in hit testing and input.
    interactive: bool,
    /// Scene object to follow. When set, the gizmo auto-reads the object's
    /// position from [`SceneHandle`](crate::SceneHandle) each frame.
    attached_to: Option<SceneObjectId>,
    hovered: Option<GizmoAxis>,
    drag: Option<DragState>,
}

impl Gizmo {
    /// Create a new gizmo at the origin.
    #[must_use]
    pub fn new(mode: GizmoMode) -> Self {
        Self {
            mode,
            position: Vec3::ZERO,
            scale: 1.0,
            gizmo_size: DEFAULT_GIZMO_SIZE,
            visible: true,
            interactive: true,
            attached_to: None,
            hovered: None,
            drag: None,
        }
    }

    /// Set the world-space position of the gizmo.
    #[must_use]
    pub fn position(mut self, pos: Vec3) -> Self {
        self.position = pos;
        self
    }

    /// Set the visual scale (arrow length in world units).
    ///
    /// Normally you don't need to call this — the [`Overlay`] implementation
    /// and [`update`](Self::update) compute the scale automatically from
    /// camera metadata. Use this only for manual control.
    #[must_use]
    pub fn scale(mut self, s: f32) -> Self {
        self.scale = s;
        self
    }

    /// Set the on-screen gizmo size.
    ///
    /// Default **80**, range 16–160. The resulting arrow length
    /// on screen is `1.75 × gizmo_size` pixels.
    #[must_use]
    pub fn gizmo_size(mut self, size: f32) -> Self {
        self.gizmo_size = size;
        self
    }

    /// Set visibility.
    ///
    /// When `false`, the gizmo is not drawn and does not respond to input.
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set whether the gizmo participates in hit testing.
    ///
    /// When `false`, the gizmo is still drawn but does not respond to
    /// mouse hover or drag. Default `true`.
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Attach the gizmo to a scene object.
    ///
    /// When attached, the gizmo auto-reads the object's world position from
    /// the [`SceneHandle`](crate::SceneHandle) each frame — no manual
    /// `set_position()` needed. The object must have a [`SceneObjectId`]
    /// assigned via [`MeshDrawGroup::with_id`](crate::widget::MeshDrawGroup::with_id).
    #[must_use]
    pub fn attach_to(mut self, id: SceneObjectId) -> Self {
        self.attached_to = Some(id);
        self
    }

    /// Update the gizmo position (mutable setter for use after construction).
    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    /// Update the visual scale.
    pub fn set_scale(&mut self, s: f32) {
        self.scale = s;
    }

    /// Update the on-screen gizmo size.
    pub fn set_gizmo_size(&mut self, size: f32) {
        self.gizmo_size = size;
    }

    /// Set visibility at runtime.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Whether the gizmo is visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set whether the gizmo participates in hit testing at runtime.
    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
    }

    /// Whether the gizmo participates in hit testing.
    #[must_use]
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    /// Attach to a scene object at runtime.
    pub fn set_attached_to(&mut self, id: Option<SceneObjectId>) {
        self.attached_to = id;
    }

    /// Detach from any scene object.
    pub fn detach(&mut self) {
        self.attached_to = None;
    }

    /// The scene object this gizmo is attached to, if any.
    #[must_use]
    pub fn attached_to(&self) -> Option<SceneObjectId> {
        self.attached_to
    }

    /// Current gizmo mode.
    #[must_use]
    pub fn mode(&self) -> GizmoMode {
        self.mode
    }

    /// Current world-space position.
    #[must_use]
    pub fn gizmo_position(&self) -> Vec3 {
        self.position
    }

    /// Currently hovered axis, if any.
    #[must_use]
    pub fn hovered_axis(&self) -> Option<GizmoAxis> {
        self.hovered
    }

    /// Whether the gizmo is actively being dragged.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    // ──────────── Scale computation ────────────

    /// Compute the world-space scale needed to maintain a constant screen-space
    /// size regardless of camera distance or viewport size.
    ///
    /// Uses the gizmo's `gizmo_size` setting (default 80).
    ///
    /// Normally you don't need to call this directly — the [`Overlay`]
    /// implementation handles it. Use this only if you need the scale value
    /// for custom logic (e.g. hit testing outside the standard `update()` flow).
    ///
    /// - `camera`: camera metadata (position, forward, FOV)
    /// - `viewport_height`: viewport height in logical pixels
    #[must_use]
    pub fn compute_scale(&self, camera: &CameraInfo, viewport_height: f32) -> f32 {
        self.compute_scale_at(camera, viewport_height, self.position)
    }

    /// Compute scale at an explicit position (for attached gizmos).
    pub(crate) fn compute_scale_at(
        &self,
        camera: &CameraInfo,
        viewport_height: f32,
        position: Vec3,
    ) -> f32 {
        let fov_y = match camera.fov_y {
            Some(fov) => fov,
            None => return self.scale, // orthographic: use current scale
        };

        /// Intrinsic arrow length in gizmo-local units:
        /// `GIZMO_ARROW_OFFSET(1.4) + GIZMO_ARROW_SIZE(0.35) = 1.75`.
        const ARROW_LOCAL_LEN: f32 = 1.75;
        /// Maximum fraction of viewport height the gizmo may occupy.
        const MAX_VIEWPORT_FRACTION: f32 = 0.35;

        // Use depth along the camera's forward direction, not straight-line
        // distance.  Straight-line distance inflates the scale for off-axis
        // objects (they appear larger on screen than they should).
        let depth = camera.forward.dot(position - camera.position);
        if depth < 1e-6 || viewport_height < 1.0 {
            return self.scale;
        }

        // Desired screen px = arrow_local_len × gizmo_size (e.g. 1.75 × 80 = 140 px).
        let target_px = ARROW_LOCAL_LEN * self.gizmo_size;
        // Clamp so the gizmo never exceeds MAX_VIEWPORT_FRACTION of the viewport.
        let clamped_px = target_px.min(MAX_VIEWPORT_FRACTION * viewport_height);

        // World-space size of one logical pixel at the gizmo's depth.
        let px_world = 2.0 * depth * (fov_y * 0.5).tan() / viewport_height;
        clamped_px * px_world
    }

    // ──────────── Drawing ────────────

    /// Generate [`MeshDrawGroup`]s for rendering the gizmo.
    ///
    /// Returns one draw group per axis arrow. Uses the gizmo's current
    /// [`scale`](Self::set_scale).
    ///
    /// Prefer using the [`Overlay`] trait (via `Box::new(gizmo.clone())` in
    /// [`Scene3DSetup::overlays`](crate::widget::Scene3DSetup::overlays))
    /// which handles scaling automatically from camera metadata.
    #[must_use]
    pub fn draw_groups(&self) -> Vec<MeshDrawGroup> {
        self.draw_groups_scaled(self.scale)
    }

    /// Generate [`MeshDrawGroup`]s at a specific world-space scale.
    ///
    /// The mesh is always generated at unit size so the GPU mesh buffer can
    /// be cached by the widget. The `scale` is applied via transform, which
    /// flows through the per-frame instance data (re-uploaded every frame).
    #[must_use]
    pub fn draw_groups_scaled(&self, scale: f32) -> Vec<MeshDrawGroup> {
        self.draw_groups_at(self.position, scale)
    }

    /// Generate [`MeshDrawGroup`]s at a specific position and scale.
    ///
    /// Used by the [`Overlay`] implementation for attached gizmos where the
    /// position comes from the scene context rather than `self.position`.
    #[must_use]
    fn draw_groups_at(&self, position: Vec3, scale: f32) -> Vec<MeshDrawGroup> {
        // Unit-size mesh — GPU buffer can be cached across frames.
        let arrow = Mesh::arrow(1.0);

        GizmoAxis::ALL
            .iter()
            .map(|&axis| {
                let color =
                    if self.drag.is_some_and(|d| d.axis == axis) || self.hovered == Some(axis) {
                        axis.highlight_color()
                    } else {
                        axis.color()
                    };

                // Scale applied via transform → instance data (re-uploaded every frame).
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

impl Default for Gizmo {
    fn default() -> Self {
        Self::new(GizmoMode::Translate)
    }
}

impl Overlay for Gizmo {
    fn visible(&self) -> bool {
        self.visible && self.attached_to.is_some()
    }

    fn interactive(&self) -> bool {
        self.interactive
    }

    fn draw(&self, ctx: &SceneContext) -> Vec<MeshDrawGroup> {
        // Resolve effective position: attached object or self.position.
        let position = self
            .attached_to
            .and_then(|id| ctx.object_position(id))
            .unwrap_or(self.position);
        let scale = self.compute_scale_at(&ctx.camera, ctx.viewport_size.y, position);
        self.draw_groups_at(position, scale)
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
