//! iced `shader::Program` implementation for the 3D scene widget.
//!
//! Wires up event handling (mouse tracking, managed gizmo input) and
//! per-frame drawing into [`Scene3DPrimitive`](super::render::Scene3DPrimitive).

use super::render::Scene3DPrimitive;
use super::types::{MeshDrawGroup, Scene3DProgram};
use crate::gizmo::{GizmoAxis, GizmoResult};
use crate::scene::context::{SceneContext, SceneHandle};
use crate::scene::object::SceneObjectId;
use iced::widget::shader;
use iced::{mouse, Rectangle};

/// Widget-internal state for cursor and mouse tracking.
///
/// Created automatically by iced via `Default`. Tracks the mouse button
/// state needed for managed gizmo interaction.
#[derive(Debug, Default)]
pub(crate) struct Scene3DState {
    mouse_pressed: bool,
    /// Last hover state to avoid re-publishing identical hover messages.
    last_hover: Option<(SceneObjectId, GizmoAxis)>,
}

/// The iced `Program` wrapper. Not constructed directly — use [`scene_3d()`](super::scene_3d).
pub(crate) struct Scene3DWidget<Message> {
    pub(super) program: Box<dyn Scene3DProgram>,
    pub(super) scene_handle: Option<SceneHandle>,
    pub(super) on_gizmo: Option<Box<dyn Fn(SceneObjectId, GizmoResult) -> Message>>,
}

impl<Message: 'static> shader::Program<Message> for Scene3DWidget<Message> {
    type State = Scene3DState;
    type Primitive = Scene3DPrimitive;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        // Track mouse button state.
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.mouse_pressed = true;
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.mouse_pressed = false;
            }
            _ => {}
        }

        // Write input state to scene handle so SceneGraph::process_input
        // can read it without the consumer tracking cursor/mouse manually.
        if let Some(handle) = &self.scene_handle {
            // Resolve cursor in widget-local coordinates.
            let cursor_pos = if handle.is_dragging() {
                cursor
                    .position()
                    .map(|p| glam::Vec2::new(p.x - bounds.x, p.y - bounds.y))
            } else {
                cursor
                    .position_in(bounds)
                    .map(|p| glam::Vec2::new(p.x, p.y))
            }
            .unwrap_or(glam::Vec2::new(-1.0, -1.0));

            handle.update_input(crate::overlay::base::OverlayInput {
                cursor: cursor_pos,
                mouse_pressed: state.mouse_pressed,
            });
        }

        // Process managed gizmo if scene handle and callback are both set.
        let handle = self.scene_handle.as_ref()?;
        let on_gizmo = self.on_gizmo.as_ref()?;

        // Get cursor position in widget-local coordinates.
        // During drag: track cursor even outside widget bounds.
        // Not dragging: only process within bounds.
        let cursor_pos = if handle.is_dragging() {
            cursor
                .position()
                .map(|p| glam::Vec2::new(p.x - bounds.x, p.y - bounds.y))
        } else {
            cursor
                .position_in(bounds)
                .map(|p| glam::Vec2::new(p.x, p.y))
        };

        let cursor_pos = match cursor_pos {
            Some(pos) => pos,
            None => {
                return if let Some((prev_id, _)) = state.last_hover.take() {
                    let msg = on_gizmo(prev_id, GizmoResult::Unhover);
                    Some(shader::Action::publish(msg))
                } else {
                    None
                };
            }
        };

        let gizmo_result = handle.process_gizmo(cursor_pos, state.mouse_pressed);

        let (id, result) = match gizmo_result {
            Some(pair) => pair,
            None => {
                return if let Some((prev_id, _)) = state.last_hover.take() {
                    let msg = on_gizmo(prev_id, GizmoResult::Unhover);
                    Some(shader::Action::publish(msg))
                } else {
                    None
                };
            }
        };

        // Capture events during drag to prevent other widgets from interfering.
        match result {
            GizmoResult::Translate(_) => {
                state.last_hover = None;
                let msg = on_gizmo(id, result);
                Some(shader::Action::publish(msg).and_capture())
            }
            GizmoResult::Hover(axis) => {
                // Only publish if the hovered axis actually changed.
                let current = (id, axis);
                if state.last_hover == Some(current) {
                    return None;
                }
                state.last_hover = Some(current);
                let msg = on_gizmo(id, result);
                Some(shader::Action::publish(msg))
            }
            GizmoResult::Unhover => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        let setup = self.program.setup(bounds);

        // Build object transform map from draws with IDs.
        let mut objects = std::collections::HashMap::new();
        for draw in &setup.draws {
            if let Some(id) = draw.id {
                if let Some(first) = draw.instances.first() {
                    let m = &first.model;
                    let mat = glam::Mat4::from_cols_array_2d(m);
                    objects.insert(id, mat);
                }
            }
        }

        // Build scene context for overlays and scene handle.
        let ctx = SceneContext {
            camera: setup.scene.camera,
            viewport_size: glam::Vec2::new(bounds.width, bounds.height),
            objects,
        };

        // Resolve consumer overlay trait objects into concrete MeshDrawGroups.
        let mut overlay_groups: Vec<MeshDrawGroup> = setup
            .overlays
            .iter()
            .filter(|o| o.visible())
            .flat_map(|o| o.draw(&ctx))
            .collect();

        // Update scene handle and add managed gizmo overlays.
        if let Some(handle) = &self.scene_handle {
            handle.update_context(ctx);
            overlay_groups.extend(handle.gizmo_overlays());
        }

        Scene3DPrimitive {
            scene: setup.scene,
            draws: setup.draws,
            overlay_groups,
            custom_uniforms: setup.custom_uniforms,
            clear_color: setup.clear_color,
            program_name: std::any::type_name_of_val(&*self.program),
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(handle) = &self.scene_handle {
            if handle.is_dragging() {
                return mouse::Interaction::Grabbing;
            }
            if handle.gizmo_hovered() {
                return mouse::Interaction::Grab;
            }
        }
        mouse::Interaction::default()
    }
}
