//! ic3d showcase — demonstrates all built-in mesh primitives using the scene graph.
//!
//! Renders cubes, spheres, cylinders, cones, and a torus arranged on a
//! ground plane with directional, point, and spot lights. Objects cast
//! soft shadows onto the ground. The camera orbits automatically.
//!
//! ```bash
//! cargo run --example showcase
//! ```

use ic3d::glam::{Quat, Vec3};
use ic3d::graph::{AmbientLight, Material, SceneGraph};
use ic3d::widget::scene_3d;
use ic3d::{DirectionalLight, Mesh, PerspectiveCamera, PointLight, SceneHandle, SpotLight};
use iced::widget::{column, container, text};
use iced::{Element, Length, Subscription, Theme};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application(App::new, App::update, App::view)
        .title("ic3d showcase")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

/// Stores the scene object IDs for the animated primitives.
struct AnimatedObject {
    id: ic3d::SceneObjectId,
    base_pos: Vec3,
}

struct App {
    handle: SceneHandle,
    graph: SceneGraph,
    cam_id: ic3d::graph::CameraId,
    objects: Vec<AnimatedObject>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl App {
    fn new() -> Self {
        let mut graph = SceneGraph::new();

        // ── Materials ──
        let ground_mat =
            graph.add_material(Material::new(Vec3::new(0.35, 0.35, 0.38)).with_shininess(8.0));
        let red =
            graph.add_material(Material::new(Vec3::new(0.85, 0.20, 0.18)).with_shininess(48.0));
        let green =
            graph.add_material(Material::new(Vec3::new(0.20, 0.75, 0.30)).with_shininess(128.0));
        let blue =
            graph.add_material(Material::new(Vec3::new(0.20, 0.40, 0.85)).with_shininess(64.0));
        let yellow =
            graph.add_material(Material::new(Vec3::new(0.90, 0.70, 0.10)).with_shininess(48.0));
        let purple =
            graph.add_material(Material::new(Vec3::new(0.75, 0.25, 0.75)).with_shininess(80.0));

        // ── Camera (orbiting, updated each tick) ──
        let cam_id = graph.add_camera(
            PerspectiveCamera::new()
                .position(Vec3::new(12.0, 6.0, 0.0))
                .target(Vec3::new(0.0, 0.8, 0.0))
                .fov(std::f32::consts::FRAC_PI_4)
                .clip(0.1, 50.0),
        );

        // ── Lights ──
        // Key light — strong directional from above-left, casts shadows
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
        // Fill light — warm point light, adds depth on the opposite side
        graph.add_light(
            PointLight::new(Vec3::new(4.0, 3.0, 3.0), 15.0)
                .with_color(Vec3::new(1.0, 0.85, 0.6))
                .with_intensity(0.8),
        );
        // Accent spot — cool blue spot from behind, rim highlights
        graph.add_light(
            SpotLight::new(
                Vec3::new(-3.0, 5.0, -4.0),
                Vec3::new(0.5, -0.7, 0.5).normalize(),
                0.25,
                0.45,
                18.0,
            )
            .with_color(Vec3::new(0.6, 0.8, 1.0))
            .with_intensity(2.0),
        );
        graph.add_light(AmbientLight::new(0.15));

        // ── Ground ──
        let _ = graph
            .add_mesh("ground", Mesh::plane(20.0, 20.0))
            .material(ground_mat)
            .position(Vec3::new(0.0, -0.01, 0.0));

        // ── Primitives arranged in a gentle arc ──
        let primitives: Vec<(&str, Mesh, ic3d::graph::MaterialId, Vec3)> = vec![
            ("cube", Mesh::cube(1.0), red, Vec3::new(-4.0, 0.5, 0.0)),
            (
                "sphere",
                Mesh::sphere(0.65, 48, 32),
                green,
                Vec3::new(-1.6, 0.65, 1.2),
            ),
            (
                "cylinder",
                Mesh::cylinder(0.45, 1.3, 32),
                blue,
                Vec3::new(0.8, 0.65, 0.0),
            ),
            (
                "cone",
                Mesh::cone(0.55, 1.2, 32),
                yellow,
                Vec3::new(3.0, 0.6, 1.0),
            ),
            (
                "torus",
                Mesh::torus(0.55, 0.22, 32, 16),
                purple,
                Vec3::new(5.0, 0.55, -0.3),
            ),
        ];

        let mut objects = Vec::new();
        for (name, mesh, mat, pos) in primitives {
            let id = graph.add_mesh(name, mesh).material(mat).position(pos).id();
            objects.push(AnimatedObject { id, base_pos: pos });
        }

        Self {
            handle: SceneHandle::new(),
            graph,
            cam_id,
            objects,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                let _dt = self.graph.tick();
                let t = self.graph.elapsed();

                // Orbit camera
                let orbit_speed = 0.15;
                let radius = 12.0;
                let cam_x = radius * (t * orbit_speed).cos();
                let cam_z = radius * (t * orbit_speed).sin();
                if let Some(cam) = self.graph.camera_mut::<PerspectiveCamera>(self.cam_id) {
                    cam.set_position(Vec3::new(cam_x, 6.0, cam_z));
                }

                // Animate objects — gentle float and spin
                for (i, obj) in self.objects.iter().enumerate() {
                    let bob = (t * 1.2 + i as f32 * 1.3).sin() * 0.15;
                    let spin = Quat::from_rotation_y(t * 0.4 + i as f32 * 0.9);
                    let pos = obj.base_pos + Vec3::new(0.0, bob, 0.0);

                    if let Some(node) = self.graph.node_mut(obj.id) {
                        node.set_position(pos);
                        node.set_rotation(spin);
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
            container(
                text("All built-in primitives: cube, sphere, cylinder, cone, torus").size(14),
            )
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
