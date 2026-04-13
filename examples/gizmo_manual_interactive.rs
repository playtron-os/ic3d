//! Multi-shape custom gizmo using [`InteractiveOverlay`].
//!
//! Demonstrates the **full-featured path** for building a custom overlay:
//! multiple hit shapes, per-shape hover highlighting, structured drag
//! lifecycle, and direct node mutation via [`OverlayContext`].
//!
//! Creates a **3-axis scale gizmo** with three colored segment handles
//! (X=red, Y=green, Z=blue). Drag any axis handle to scale the object
//! along that axis independently.
//!
//! For single-point drag overlays, see the simpler `gizmo_manual_draggable`
//! example instead.
//!
//! ```bash
//! cargo run --example gizmo_manual_interactive
//! ```

use ic3d::glam::{Vec2, Vec3};
use ic3d::graph::{AmbientLight, Material, SceneGraph};
use ic3d::math::HitShape;
use ic3d::widget::{scene_3d, MeshDrawGroup};
use ic3d::{
    DirectionalLight, Interactive, InteractiveContext, InteractiveOverlay, Mesh, OverlayContext,
    OverlayEvent, PerspectiveCamera, SceneHandle, SceneObjectId, ShapeHit, Transform,
};
use iced::widget::{column, container, text};
use iced::{Element, Length, Subscription, Theme};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application(App::new, App::update, App::view)
        .title("ic3d · InteractiveOverlay — 3-axis scale")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

// ─────────────── Axis constants ───────────────

const AXIS_X: usize = 0;
const AXIS_Y: usize = 1;
const AXIS_Z: usize = 2;

/// Handle length relative to gizmo scale.
const HANDLE_LENGTH: f32 = 1.0;
/// Hit threshold in screen pixels.
const HIT_THRESHOLD: f32 = 20.0;
/// Sensitivity: scale change per pixel of drag along the projected axis.
const SCALE_PER_PX: f32 = 0.005;
/// Cube cap size relative to gizmo scale.
const CAP_SIZE: f32 = 0.08;
/// Shaft thickness relative to gizmo scale.
const SHAFT_THICKNESS: f32 = 0.02;

/// Axis directions in world space.
const AXIS_DIRS: [Vec3; 3] = [Vec3::X, Vec3::Y, Vec3::Z];
/// Axis colors: X=red, Y=green, Z=blue.
const AXIS_COLORS: [[f32; 4]; 3] = [
    [0.9, 0.2, 0.2, 1.0],
    [0.2, 0.9, 0.2, 1.0],
    [0.2, 0.4, 0.9, 1.0],
];
/// Bright color when hovered or dragging.
const AXIS_ACTIVE_COLORS: [[f32; 4]; 3] = [
    [1.0, 0.5, 0.5, 1.0],
    [0.5, 1.0, 0.5, 1.0],
    [0.5, 0.7, 1.0, 1.0],
];

// ─────────────── 3-Axis Scale Gizmo ───────────────

/// A 3-axis scale gizmo with independent axis handles.
///
/// Each axis is a segment + cube cap. Hover highlights the axis, drag
/// scales the target object along that axis. Implements
/// [`InteractiveOverlay`] for multi-shape hit testing.
#[derive(Debug, Clone)]
struct AxisScaleGizmo {
    hovered_axis: Option<usize>,
    drag_axis: Option<usize>,
    drag_start_cursor: Vec2,
    drag_start_scale: Vec3,
}

impl Default for AxisScaleGizmo {
    fn default() -> Self {
        Self {
            hovered_axis: None,
            drag_axis: None,
            drag_start_cursor: Vec2::ZERO,
            drag_start_scale: Vec3::ONE,
        }
    }
}

impl InteractiveOverlay for AxisScaleGizmo {
    fn resolve_target(&self, handle: &SceneHandle) -> Option<SceneObjectId> {
        handle.selected_objects().into_iter().next()
    }

    fn screen_size(&self) -> f32 {
        100.0
    }

    fn hit_shapes(&self, ctx: &InteractiveContext) -> Vec<HitShape> {
        // One segment per axis, originating from the target position.
        AXIS_DIRS
            .iter()
            .map(|dir| {
                let end = ctx.position + *dir * ctx.scale * HANDLE_LENGTH;
                HitShape::segment(ctx.position, end, HIT_THRESHOLD)
            })
            .collect()
    }

    fn on_hover(&mut self, hit: &ShapeHit) {
        self.hovered_axis = Some(hit.shape_index);
    }

    fn on_unhover(&mut self) {
        self.hovered_axis = None;
    }

    fn on_drag_start(
        &mut self,
        hit: &ShapeHit,
        cursor: Vec2,
        _ctx: &InteractiveContext,
        nodes: &mut OverlayContext,
    ) -> bool {
        let target = nodes.handle().selected_objects().into_iter().next();
        let Some(target) = target else { return false };
        let Some(node) = nodes.node(target) else {
            return false;
        };

        self.drag_axis = Some(hit.shape_index);
        self.drag_start_cursor = cursor;
        self.drag_start_scale = node.local_transform().scale;
        true
    }

    fn on_drag_continue(
        &mut self,
        cursor: Vec2,
        ctx: &InteractiveContext,
        nodes: &mut OverlayContext,
    ) {
        let Some(axis) = self.drag_axis else { return };
        let target = nodes.handle().selected_objects().into_iter().next();
        let Some(target) = target else { return };

        // Project the axis direction to screen space to determine drag direction.
        let axis_dir = AXIS_DIRS[axis];
        let screen_axis = project_direction(ctx, axis_dir);

        // Signed drag distance along the screen-projected axis.
        let drag_delta = cursor - self.drag_start_cursor;
        let signed_dist = drag_delta.dot(screen_axis);

        // Apply scale delta to the captured start scale.
        let mut new_scale = self.drag_start_scale;
        let component = match axis {
            AXIS_X => &mut new_scale.x,
            AXIS_Y => &mut new_scale.y,
            AXIS_Z => &mut new_scale.z,
            _ => &mut new_scale.z,
        };
        *component = (self.drag_start_scale[axis] + signed_dist * SCALE_PER_PX).clamp(0.1, 5.0);

        if let Some(node) = nodes.node_mut(target) {
            node.set_scale(new_scale);
        }
    }

    fn on_drag_end(&mut self, _nodes: &mut OverlayContext) {
        self.drag_axis = None;
    }

    fn is_dragging(&self) -> bool {
        self.drag_axis.is_some()
    }

    fn draw(&self, ctx: &InteractiveContext) -> Vec<MeshDrawGroup> {
        let mut draws = Vec::new();
        let active_axis = self.drag_axis.or(self.hovered_axis);

        for (i, dir) in AXIS_DIRS.iter().enumerate() {
            let is_active = active_axis == Some(i);
            let color = if is_active {
                AXIS_ACTIVE_COLORS[i]
            } else {
                AXIS_COLORS[i]
            };

            let end = ctx.position + *dir * ctx.scale * HANDLE_LENGTH;

            // Shaft: thin elongated cube along the axis.
            let mid = (ctx.position + end) * 0.5;
            let shaft_scale = match i {
                AXIS_X => Vec3::new(
                    ctx.scale * HANDLE_LENGTH,
                    ctx.scale * SHAFT_THICKNESS,
                    ctx.scale * SHAFT_THICKNESS,
                ),
                AXIS_Y => Vec3::new(
                    ctx.scale * SHAFT_THICKNESS,
                    ctx.scale * HANDLE_LENGTH,
                    ctx.scale * SHAFT_THICKNESS,
                ),
                _ => Vec3::new(
                    ctx.scale * SHAFT_THICKNESS,
                    ctx.scale * SHAFT_THICKNESS,
                    ctx.scale * HANDLE_LENGTH,
                ),
            };
            draws.push(MeshDrawGroup::new(
                Mesh::cube(1.0),
                vec![Transform::new()
                    .position(mid)
                    .scale(shaft_scale)
                    .to_instance(color)],
            ));

            // Cap: small cube at the axis end.
            draws.push(MeshDrawGroup::new(
                Mesh::cube(1.0),
                vec![Transform::new()
                    .position(end)
                    .uniform_scale(ctx.scale * CAP_SIZE)
                    .to_instance(color)],
            ));
        }

        draws
    }
}

/// Project a world-space direction to a normalized screen-space direction.
fn project_direction(ctx: &InteractiveContext, dir: Vec3) -> Vec2 {
    let vp = ctx.camera.view_projection;
    let w = ctx.viewport.x;
    let h = ctx.viewport.y;

    let project = |p: Vec3| -> Vec2 {
        let clip = vp * p.extend(1.0);
        let ndc = clip.truncate() / clip.w;
        Vec2::new((ndc.x * 0.5 + 0.5) * w, (1.0 - (ndc.y * 0.5 + 0.5)) * h)
    };

    let a = project(ctx.position);
    let b = project(ctx.position + dir);
    let d = b - a;
    if d.length() < 1e-6 {
        Vec2::X
    } else {
        d.normalize()
    }
}

// ─────────────── App ───────────────

struct App {
    handle: SceneHandle,
    graph: SceneGraph,
    cube_id: SceneObjectId,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl App {
    fn new() -> Self {
        let mut graph = SceneGraph::new();

        let ground_mat =
            graph.add_material(Material::new(Vec3::new(0.35, 0.35, 0.38)).with_shininess(8.0));
        let blue = graph.add_material(Material::new(Vec3::new(0.2, 0.6, 0.9)).with_shininess(64.0));

        graph.add_camera(
            PerspectiveCamera::new()
                .position(Vec3::new(6.0, 5.0, 8.0))
                .target(Vec3::new(0.0, 1.5, 0.0))
                .clip(0.1, 50.0),
        );

        graph.add_light(
            DirectionalLight::new(
                Vec3::new(-0.4, -0.8, -0.3).normalize(),
                Vec3::ZERO,
                20.0,
                40.0,
            )
            .with_color(Vec3::new(1.0, 0.95, 0.85))
            .with_intensity(1.2),
        );
        graph.add_light(AmbientLight::new(0.2));

        let _ = graph
            .add_mesh("ground", Mesh::plane(20.0, 20.0))
            .material(ground_mat)
            .position(Vec3::new(0.0, -0.01, 0.0));

        let cube_id = graph
            .add_mesh("cube", Mesh::cube(1.0))
            .material(blue)
            .position(Vec3::new(0.0, 1.0, 0.0))
            .id();

        // Register the interactive 3-axis scale gizmo.
        graph.add_overlay(Interactive::new(AxisScaleGizmo::default()));

        let handle = SceneHandle::new();
        handle.select_object(cube_id);

        Self {
            handle,
            graph,
            cube_id,
            status: "Drag an axis handle (X/Y/Z) to scale the cube".into(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                for (_id, event) in self.graph.process_input(&self.handle) {
                    match event {
                        OverlayEvent::HoverStart => {
                            self.status = "Hovering — drag to scale along this axis".into();
                        }
                        OverlayEvent::HoverEnd => {
                            self.status = "Drag an axis handle (X/Y/Z) to scale the cube".into();
                        }
                        OverlayEvent::DragMove(_) => {
                            if let Some(node) = self.graph.node(self.cube_id) {
                                let s = node.local_transform().scale;
                                self.status =
                                    format!("Scale: X={:.2}  Y={:.2}  Z={:.2}", s.x, s.y, s.z);
                            }
                        }
                        OverlayEvent::DragEnd => {
                            self.status = "Drag an axis handle (X/Y/Z) to scale the cube".into();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            container(
                scene_3d(self.graph.clone())
                    .scene(self.handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill),
            container(text(&self.status).size(14))
                .width(Length::Fill)
                .center_x(Length::Fill),
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        ic3d::frames().map(|_| Message::Tick)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
