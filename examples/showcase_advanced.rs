//! Advanced showcase — demonstrates the low-level `Scene3DProgram` API.
//!
//! This example shows the **power-user path** for consumers who need full
//! control over the render pipeline:
//!
//! - Custom fragment shader (debug visualization modes)
//! - Custom uniform buffer (`@group(1) @binding(0)`)
//! - Manual `Scene3DSetup` construction (no scene graph)
//! - Direct `Scene` builder + `Transform` → `InstanceData` workflow
//!
//! For most use cases, prefer the **scene graph** approach shown in
//! `showcase.rs` and `gizmo.rs`.
//!
//! ```bash
//! cargo run --example showcase_advanced --features debug
//! ```

use ic3d::debug;
use ic3d::glam::{Quat, Vec3};
use ic3d::widget::{scene_3d, MeshDrawGroup, Scene3DProgram, Scene3DSetup};
use ic3d::{DirectionalLight, Mesh, PerspectiveCamera, PointLight, Scene, SpotLight, Transform};
use iced::keyboard;
use iced::widget::{column, container, text};
use iced::window;
use iced::{Element, Length, Subscription, Theme};
use std::time::Instant;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application(App::new, App::update, App::view)
        .title("ic3d advanced showcase")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

struct App {
    start: Instant,
    debug_mode: u32,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    DebugMode(u32),
}

impl App {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            debug_mode: 0,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {}
            Message::DebugMode(m) => self.debug_mode = m,
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<'_, Message> {
        let elapsed = self.start.elapsed().as_secs_f32();

        let mode_names = ["Lit", "Normals", "NdotL", "Shadow", "No-Shadow", "Flat"];
        let label: String = mode_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if i as u32 == self.debug_mode {
                    format!("[{}:{}]", i + 1, name)
                } else {
                    format!(" {}:{} ", i + 1, name)
                }
            })
            .collect();

        column![
            container(
                scene_3d(AdvancedScene {
                    time: elapsed,
                    debug_mode: self.debug_mode,
                })
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill),
            container(text(label).size(14))
                .width(Length::Fill)
                .center_x(Length::Fill),
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::Tick),
            keyboard::listen().map(|event| {
                if let keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    ..
                } = event
                {
                    match c.as_str() {
                        "1" => return Message::DebugMode(0),
                        "2" => return Message::DebugMode(1),
                        "3" => return Message::DebugMode(2),
                        "4" => return Message::DebugMode(3),
                        "5" => return Message::DebugMode(4),
                        "6" => return Message::DebugMode(5),
                        _ => {}
                    }
                }
                Message::Tick
            }),
        ])
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────── Scene3DProgram (advanced path) ───────────────

/// Implements `Scene3DProgram` directly — no scene graph.
///
/// Uses a custom debug fragment shader and custom uniform buffer to
/// switch between rendering modes at runtime: lit, normals, NdotL,
/// shadow map, unlit, and flat shading.
#[derive(Debug)]
struct AdvancedScene {
    time: f32,
    debug_mode: u32,
}

impl Scene3DProgram for AdvancedScene {
    fn fragment_shader(&self) -> &str {
        debug::FRAGMENT_WGSL
    }

    fn custom_uniforms_size(&self) -> usize {
        debug::UNIFORM_SIZE
    }

    fn warmup_meshes(&self) -> Vec<Mesh> {
        vec![
            Mesh::cube(1.0),
            Mesh::sphere(0.5, 32, 24),
            Mesh::plane(1.0, 1.0),
        ]
    }

    fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
        let t = self.time;
        let aspect = bounds.width / bounds.height.max(1.0);

        // Orbit camera
        let orbit_speed = 0.15;
        let radius = 12.0;
        let cam_x = radius * (t * orbit_speed).cos();
        let cam_z = radius * (t * orbit_speed).sin();
        let cam_pos = Vec3::new(cam_x, 6.0, cam_z);

        let camera = PerspectiveCamera::new()
            .position(cam_pos)
            .target(Vec3::new(0.0, 0.8, 0.0))
            .aspect(aspect)
            .fov(std::f32::consts::FRAC_PI_4)
            .clip(0.1, 50.0);

        // Three-point lighting
        let sun = DirectionalLight::new(
            Vec3::new(-0.4, -0.8, -0.3).normalize(),
            Vec3::ZERO,
            20.0,
            40.0,
        )
        .with_color(Vec3::new(1.0, 0.95, 0.85))
        .with_intensity(1.2);

        let fill = PointLight::new(Vec3::new(4.0, 3.0, 3.0), 15.0)
            .with_color(Vec3::new(1.0, 0.85, 0.6))
            .with_intensity(0.8);

        let accent = SpotLight::new(
            Vec3::new(-3.0, 5.0, -4.0),
            Vec3::new(0.5, -0.7, 0.5).normalize(),
            0.25,
            0.45,
            18.0,
        )
        .with_color(Vec3::new(0.6, 0.8, 1.0))
        .with_intensity(2.0);

        let scene = Scene::new(&camera)
            .camera_position(cam_pos.to_array())
            .light(&sun)
            .light(&fill)
            .light(&accent)
            .ambient(0.15)
            .time(t)
            .screen_size(bounds.width, bounds.height)
            .build();

        // ── Ground plane ──
        let ground = MeshDrawGroup::new(
            Mesh::plane(20.0, 20.0),
            vec![Transform::new()
                .position(Vec3::new(0.0, -0.01, 0.0))
                .to_instance([0.35, 0.35, 0.38, 8.0])],
        );

        // ── Objects arranged in a gentle arc ──
        let objects: Vec<(Mesh, [f32; 4], Vec3)> = vec![
            (
                Mesh::cube(1.0),
                [0.85, 0.20, 0.18, 48.0],
                Vec3::new(-4.0, 0.5, 0.0),
            ),
            (
                Mesh::sphere(0.65, 48, 32),
                [0.20, 0.75, 0.30, 128.0],
                Vec3::new(-1.6, 0.65, 1.2),
            ),
            (
                Mesh::cylinder(0.45, 1.3, 32),
                [0.20, 0.40, 0.85, 64.0],
                Vec3::new(0.8, 0.65, 0.0),
            ),
            (
                Mesh::cone(0.55, 1.2, 32),
                [0.90, 0.70, 0.10, 48.0],
                Vec3::new(3.0, 0.6, 1.0),
            ),
            (
                Mesh::torus(0.55, 0.22, 32, 16),
                [0.75, 0.25, 0.75, 80.0],
                Vec3::new(5.0, 0.55, -0.3),
            ),
        ];

        let mut draws = vec![ground];

        for (i, (mesh, color, base_pos)) in objects.into_iter().enumerate() {
            let bob = (t * 1.2 + i as f32 * 1.3).sin() * 0.15;
            let spin = t * 0.4 + i as f32 * 0.9;
            let pos = base_pos + Vec3::new(0.0, bob, 0.0);

            let instance = Transform::new()
                .position(pos)
                .rotation(Quat::from_rotation_y(spin))
                .to_instance(color);

            draws.push(MeshDrawGroup::new(mesh, vec![instance]));
        }

        Scene3DSetup {
            scene,
            draws,
            overlays: vec![],
            custom_uniforms: Some(debug::uniforms(self.debug_mode)),
            clear_color: wgpu::Color::BLACK,
        }
    }
}
