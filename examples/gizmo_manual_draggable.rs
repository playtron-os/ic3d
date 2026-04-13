//! Simple custom gizmo using [`DraggableOverlay`].
//!
//! Demonstrates the **simplest path** for building a custom overlay:
//! single hit point, pixel deltas, 3 methods. The engine handles hit
//! testing, hover detection, drag start/end, and delta computation.
//!
//! Creates a **uniform-scale gizmo**: a small center cube that the user
//! can drag vertically to scale an object up/down.
//!
//! For multi-shape hit testing (segments, arcs, per-axis handles), see
//! the `gizmo_manual_interactive` example instead.
//!
//! ```bash
//! cargo run --example gizmo_manual_draggable
//! ```

use ic3d::glam::{Vec2, Vec3};
use ic3d::graph::{AmbientLight, Material, SceneGraph};
use ic3d::math::screen_constant_scale;
use ic3d::widget::{scene_3d, MeshDrawGroup};
use ic3d::{
    DirectionalLight, DragState, Draggable, DraggableOverlay, Mesh, OverlayContext, OverlayEvent,
    PerspectiveCamera, SceneContext, SceneHandle, SceneObjectId, Transform,
};
use iced::widget::{column, container, text};
use iced::{Element, Length, Subscription, Theme};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application(App::new, App::update, App::view)
        .title("ic3d · DraggableOverlay — uniform scale")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

// ─────────────── Custom Scale Gizmo ───────────────

/// On-screen gizmo size in pixels.
const GIZMO_SIZE: f32 = 80.0;
/// Sensitivity: scale change per pixel of vertical drag.
const SCALE_PER_PX: f32 = 0.005;
/// Center cube size relative to gizmo scale.
const CENTER_SIZE: f32 = 0.12;

/// A uniform-scale gizmo rendered as a single center cube.
///
/// Drag vertically on the cube to scale the attached object uniformly.
/// Implements [`DraggableOverlay`] — the engine handles hit testing,
/// drag tracking, and input routing automatically.
#[derive(Debug, Clone, Default)]
struct ScaleGizmo;

impl DraggableOverlay for ScaleGizmo {
    fn resolve_target(&self, handle: &SceneHandle) -> Option<SceneObjectId> {
        handle.selected_objects().into_iter().next()
    }

    fn on_drag(&mut self, delta: Vec2, ctx: &mut OverlayContext) {
        let Some(target) = ctx.handle().selected_objects().into_iter().next() else {
            return;
        };
        if let Some(node) = ctx.node_mut(target) {
            // up = negative Y = grow
            node.add_uniform_scale(-delta.y * SCALE_PER_PX)
                .clamp_uniform_scale(0.1, 5.0);
        }
    }

    fn draw_overlay(
        &self,
        target: SceneObjectId,
        state: &DragState,
        ctx: &SceneContext,
    ) -> Vec<MeshDrawGroup> {
        let Some(obj_pos) = ctx.object_position(target) else {
            return Vec::new();
        };
        let scale = screen_constant_scale(obj_pos, &ctx.camera, ctx.viewport_size.y, GIZMO_SIZE);

        let color = if state.is_active() {
            [1.0, 1.0, 0.6, 1.0] // bright yellow when active
        } else {
            [0.85, 0.85, 0.85, 1.0] // white
        };

        vec![MeshDrawGroup::new(
            Mesh::cube(1.0),
            vec![Transform::new()
                .position(obj_pos)
                .uniform_scale(scale * CENTER_SIZE)
                .to_instance(color)],
        )]
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

        graph.add_overlay(Draggable::new(ScaleGizmo));

        let handle = SceneHandle::new();
        handle.select_object(cube_id);

        Self {
            handle,
            graph,
            cube_id,
            status: "Drag the yellow handle to scale the cube".into(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                for (_id, event) in self.graph.process_input(&self.handle) {
                    match event {
                        OverlayEvent::HoverStart => {
                            self.status = "Hovering scale handle — drag vertically".into();
                        }
                        OverlayEvent::HoverEnd => {
                            self.status = "Drag the yellow handle to scale the cube".into();
                        }
                        OverlayEvent::DragMove(_) => {
                            if let Some(node) = self.graph.node(self.cube_id) {
                                self.status = format!("Scale: {:.2}", node.uniform_scale());
                            }
                        }
                        OverlayEvent::DragEnd => {
                            self.status = "Drag the yellow handle to scale the cube".into();
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
