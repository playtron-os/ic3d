//! Transparent window — floating 3D meshes on the desktop.
//!
//! Demonstrates transparent window rendering:
//!
//! - `transparent(true)` on the iced application
//! - `Color::TRANSPARENT` background (iced clears with zero alpha)
//! - `wgpu::Color::TRANSPARENT` clear color (ic3d clears with zero alpha)
//!
//! Only geometry pixels are visible — uncovered pixels are fully transparent,
//! letting the desktop show through (requires a Wayland/X11 compositor that
//! supports window transparency).
//!
//! ```bash
//! cargo run --example transparent
//! ```

use ic3d::glam::Vec3;
use ic3d::widget::{scene_3d, MeshDrawGroup, Scene3DProgram, Scene3DSetup};
use ic3d::{DirectionalLight, Mesh, PerspectiveCamera, Scene, Transform};
use iced::widget::container;
use iced::window;
use iced::{Color, Element, Length, Subscription, Theme};
use std::time::Instant;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application(App::new, App::update, App::view)
        .title("ic3d — transparent")
        .subscription(App::subscription)
        .theme(App::theme)
        .transparent(true)
        .style(|_state, _theme| iced::theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
        })
        .run()
}

struct App {
    start: Instant,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl App {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    fn update(&mut self, _message: Message) {}

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<'_, Message> {
        let elapsed = self.start.elapsed().as_secs_f32();

        container(
            scene_3d(FloatingScene { time: elapsed })
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(|_| Message::Tick)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────── Scene ───────────────

#[derive(Debug)]
struct FloatingScene {
    time: f32,
}

impl Scene3DProgram for FloatingScene {
    fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
        let t = self.time;
        let aspect = bounds.width / bounds.height.max(1.0);

        let camera = PerspectiveCamera::new()
            .position(Vec3::new(0.0, 3.0, 8.0))
            .target(Vec3::ZERO)
            .aspect(aspect)
            .fov(std::f32::consts::FRAC_PI_4)
            .clip(0.1, 50.0);

        let sun = DirectionalLight::new(
            Vec3::new(-0.4, -1.0, -0.3).normalize(),
            Vec3::ZERO,
            20.0,
            40.0,
        )
        .with_color(Vec3::new(1.0, 0.95, 0.9))
        .with_intensity(1.2);

        let scene = Scene::new(&camera)
            .camera_position(Vec3::new(0.0, 3.0, 8.0).to_array())
            .light(&sun)
            .ambient(0.15)
            .time(t)
            .screen_size(bounds.width, bounds.height)
            .build();

        // Orbiting primitives — no ground plane.
        let cube = Transform::new()
            .position(Vec3::new(
                2.0 * (t * 0.5).cos(),
                (t * 0.7).sin() * 0.5,
                2.0 * (t * 0.5).sin(),
            ))
            .scale(Vec3::splat(0.8))
            .to_instance([0.3, 0.6, 0.9, 1.0]);

        let sphere = Transform::new()
            .position(Vec3::new(
                -2.0 * (t * 0.4).cos(),
                1.0 + (t * 0.6).sin() * 0.3,
                -2.0 * (t * 0.4).sin(),
            ))
            .scale(Vec3::splat(0.6))
            .to_instance([0.9, 0.3, 0.4, 1.0]);

        let torus = Transform::new()
            .position(Vec3::new(0.0, -0.5 + (t * 0.3).sin() * 0.4, 0.0))
            .scale(Vec3::splat(1.0))
            .to_instance([0.4, 0.9, 0.5, 1.0]);

        let draws = vec![
            MeshDrawGroup::new(Mesh::cube(1.0), vec![cube]),
            MeshDrawGroup::new(Mesh::sphere(0.5, 24, 16), vec![sphere]),
            MeshDrawGroup::new(Mesh::torus(0.6, 0.2, 24, 12), vec![torus]),
        ];

        Scene3DSetup {
            scene,
            draws,
            overlays: vec![],
            custom_uniforms: None,
            clear_color: wgpu::Color::TRANSPARENT,
        }
    }
}
