//! Gizmo example — demonstrates the built-in translation gizmo using the
//! scene graph.
//!
//! The widget handles all cursor tracking, hit testing, and gizmo rendering
//! automatically. The consumer just calls `scene.select()` and receives
//! results via `Message::Gizmo`.
//!
//! ```bash
//! cargo run --example gizmo
//! ```

use ic3d::gizmo::{GizmoMode, GizmoResult};
use ic3d::glam::{self, Vec3};
use ic3d::graph::{AmbientLight, Material, SceneGraph};
use ic3d::widget::scene_3d;
use ic3d::{DirectionalLight, Mesh, PerspectiveCamera, SceneHandle, SceneObjectId};
use iced::widget::{column, container, text};
use iced::{keyboard, Element, Length, Subscription, Theme};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application(App::new, App::update, App::view)
        .title("ic3d gizmo")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

struct App {
    handle: SceneHandle,
    graph: SceneGraph,
    cube_id: SceneObjectId,
    mode: GizmoMode,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    Gizmo(SceneObjectId, GizmoResult),
    SetMode(GizmoMode),
}

impl App {
    fn new() -> Self {
        let mut graph = SceneGraph::new();

        // ── Materials ──
        let ground_mat =
            graph.add_material(Material::new(Vec3::new(0.35, 0.35, 0.38)).with_shininess(8.0));
        let blue = graph.add_material(Material::new(Vec3::new(0.2, 0.6, 0.9)).with_shininess(64.0));

        // ── Camera ──
        graph.add_camera(
            PerspectiveCamera::new()
                .position(Vec3::new(5.0, 5.0, 8.0))
                .target(Vec3::new(0.0, 0.5, 0.0))
                .fov(std::f32::consts::FRAC_PI_4)
                .clip(0.1, 50.0),
        );

        // ── Lights ──
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

        // ── Ground ──
        let _ = graph
            .add_mesh("ground", Mesh::plane(20.0, 20.0))
            .material(ground_mat)
            .position(Vec3::new(0.0, -0.01, 0.0));

        // ── The cube being manipulated ──
        let cube_id = graph
            .add_mesh("cube", Mesh::cube(1.0))
            .material(blue)
            .position(Vec3::new(0.0, 0.5, 0.0))
            .id();

        // Select the cube — the widget shows a translation gizmo on it.
        let handle = SceneHandle::new();
        handle.select(cube_id, GizmoMode::Translate);

        Self {
            handle,
            graph,
            cube_id,
            mode: GizmoMode::Translate,
            status: "[T] Translate  [R] Rotate — drag an axis to move the cube".into(),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SetMode(mode) => {
                self.mode = mode;
                self.handle.modify_gizmo(self.cube_id, |g| g.set_mode(mode));
                let label = match mode {
                    GizmoMode::Translate => "Translate",
                    GizmoMode::Rotate => "Rotate",
                };
                self.status = format!("[T] Translate  [R] Rotate — mode: {label}");
            }
            Message::Gizmo(_id, result) => match result {
                GizmoResult::Hover(axis) => {
                    self.status = format!("Hovering: {axis:?} axis");
                }
                GizmoResult::HoverCenter => {
                    self.status = "Hovering: free rotate (center)".into();
                }
                GizmoResult::Unhover => {
                    let label = match self.mode {
                        GizmoMode::Translate => "Translate",
                        GizmoMode::Rotate => "Rotate",
                    };
                    self.status = format!("[T] Translate  [R] Rotate — mode: {label}");
                }
                GizmoResult::Translate(delta) => {
                    if let Some(node) = self.graph.node_mut(self.cube_id) {
                        let pos = node.local_transform().position + delta;
                        node.set_position(pos);
                    }
                    let pos = self.graph.world_position(self.cube_id);
                    self.status = format!("Position: ({:.2}, {:.2}, {:.2})", pos.x, pos.y, pos.z);
                }
                GizmoResult::Rotate(euler) => {
                    if let Some(node) = self.graph.node_mut(self.cube_id) {
                        let current = node.local_transform().rotation;
                        let delta =
                            glam::Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z);
                        node.set_rotation(delta * current);
                    }
                    self.status = format!(
                        "Rotation: ({:.1}°, {:.1}°, {:.1}°)",
                        euler.x.to_degrees(),
                        euler.y.to_degrees(),
                        euler.z.to_degrees(),
                    );
                }
                GizmoResult::FreeRotate(quat) => {
                    if let Some(node) = self.graph.node_mut(self.cube_id) {
                        let current = node.local_transform().rotation;
                        node.set_rotation(quat * current);
                    }
                    let (x, y, z) = quat.to_euler(glam::EulerRot::XYZ);
                    self.status = format!(
                        "Free rotate: ({:.1}°, {:.1}°, {:.1}°)",
                        x.to_degrees(),
                        y.to_degrees(),
                        z.to_degrees(),
                    );
                }
            },
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
                    .on_gizmo(Message::Gizmo)
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
        keyboard::listen().map(|event| {
            if let keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(ref c),
                ..
            } = event
            {
                match c.as_str() {
                    "t" => return Message::SetMode(GizmoMode::Translate),
                    "r" => return Message::SetMode(GizmoMode::Rotate),
                    _ => {}
                }
            }
            // Keyboard events that don't match any hotkey produce a no-op
            // gizmo message (the app ignores Unhover when nothing is hovered).
            Message::Gizmo(SceneObjectId::default(), GizmoResult::Unhover)
        })
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
