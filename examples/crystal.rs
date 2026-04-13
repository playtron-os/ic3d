//! Crystal field — iridescent hexagonal columns with energy veins.
//!
//! Demonstrates the **custom shader** path using `Scene3DProgram`:
//!
//! - Custom WGSL fragment shader (`shaders/crystal.wgsl`)
//! - Custom uniform buffer (`@group(1) @binding(0)`)
//! - Procedural hex grid generation with `MeshBuilder::extrude`
//! - Per-instance material packing for shader variety
//! - Cursor-reactive highlight via custom uniforms
//! - Orbiting camera with three-point lighting
//!
//! ```bash
//! cargo run --example crystal
//! ```

use ic3d::glam::{Quat, Vec2, Vec3};
use ic3d::math::ray::Ray;
use ic3d::math::{ease_out_elastic, hash_f32, hash_f32_range, hex_grid, smoothstep};
use ic3d::widget::{scene_3d, MeshDrawGroup, Scene3DProgram, Scene3DSetup};
use ic3d::{Camera, DirectionalLight, Mesh, PerspectiveCamera, Scene, SpotLight, Transform};
use iced::widget::container;
use iced::window;
use iced::{mouse, Element, Length, Subscription, Theme};
use std::f32::consts::FRAC_PI_2;
use std::time::Instant;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::application(App::new, App::update, App::view)
        .title("ic3d — crystal field")
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

// ─────────────── Configuration ───────────────

/// Hex cell outer radius (center to vertex).
const HEX_RADIUS: f32 = 0.55;
/// Gap between hexagons.
const HEX_GAP: f32 = 0.04;
/// Grid radius in hex rings from center.
const GRID_RINGS: i32 = 12;
/// Column height range.
const MIN_HEIGHT: f32 = 0.15;
const MAX_HEIGHT: f32 = 2.8;
/// Intro wave duration.
const WAVE_DURATION: f32 = 2.5;
/// Per-column pop animation.
const POP_DURATION: f32 = 0.8;

// ─────────────── Custom Uniforms (must match WGSL) ───────────────

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct CrystalUniforms {
    cursor_world: [f32; 2],
    cursor_active: f32,
    _pad: f32,
}

// ─────────────── Hex Grid ───────────────

struct Column {
    x: f32,
    z: f32,
    height: f32,
    color_seed: f32,
    column_id: f32,
    dist: f32,
}

/// Generate the crystal field: hex grid positions + per-column random attributes.
fn generate_columns() -> Vec<Column> {
    hex_grid(HEX_RADIUS, HEX_GAP, GRID_RINGS)
        .into_iter()
        .map(|cell| Column {
            x: cell.x,
            z: cell.z,
            height: hash_f32_range(cell.x, cell.z, 42.0, MIN_HEIGHT, MAX_HEIGHT),
            color_seed: hash_f32(cell.x, cell.z, 7.0),
            column_id: hash_f32(cell.x, cell.z, 99.0),
            dist: cell.distance,
        })
        .collect()
}

// ─────────────── App ───────────────

struct App {
    start: Instant,
    cursor_pos: Option<iced::Point>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    CursorMoved(iced::Point),
}

impl App {
    fn new() -> Self {
        Self {
            start: Instant::now(),
            cursor_pos: None,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {}
            Message::CursorMoved(pos) => self.cursor_pos = Some(pos),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn view(&self) -> Element<'_, Message> {
        let elapsed = self.start.elapsed().as_secs_f32();
        let cursor = self.cursor_pos;

        container(
            scene_3d(CrystalScene {
                time: elapsed,
                cursor,
            })
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::Tick),
            iced::event::listen_with(|event, _status, _id| {
                if let iced::Event::Mouse(mouse::Event::CursorMoved { position }) = event {
                    return Some(Message::CursorMoved(position));
                }
                None
            }),
        ])
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────── Scene3DProgram ───────────────

#[derive(Debug)]
struct CrystalScene {
    time: f32,
    cursor: Option<iced::Point>,
}

impl Scene3DProgram for CrystalScene {
    fn fragment_shader(&self) -> &str {
        include_str!("shaders/crystal.wgsl")
    }

    fn custom_uniforms_size(&self) -> usize {
        std::mem::size_of::<CrystalUniforms>()
    }

    fn warmup_meshes(&self) -> Vec<Mesh> {
        vec![Mesh::hex_column(HEX_RADIUS), Mesh::plane(1.0, 1.0)]
    }

    fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
        let t = self.time;
        let aspect = bounds.width / bounds.height.max(1.0);

        // ── Orbiting camera ──
        let orbit_speed = 0.08;
        let radius = 14.0;
        let cam_x = radius * (t * orbit_speed).cos();
        let cam_z = radius * (t * orbit_speed).sin();
        let cam_y = 8.0 + (t * 0.15).sin() * 1.5;

        let camera = PerspectiveCamera::new()
            .position(Vec3::new(cam_x, cam_y, cam_z))
            .target(Vec3::new(0.0, 0.8, 0.0))
            .aspect(aspect)
            .fov(std::f32::consts::FRAC_PI_4)
            .clip(0.1, 80.0);

        // ── Three-point lighting ──

        // Key light — warm directional from upper-right
        let sun = DirectionalLight::new(
            Vec3::new(-0.3, -0.9, -0.4).normalize(),
            Vec3::ZERO,
            25.0,
            50.0,
        )
        .with_color(Vec3::new(0.9, 0.85, 1.0))
        .with_intensity(1.0);

        // Accent spot — cool blue from below-left
        let accent = SpotLight::new(
            Vec3::new(-6.0, 1.0, 6.0),
            Vec3::new(0.4, 0.3, -0.4).normalize(),
            0.3,
            0.6,
            25.0,
        )
        .with_color(Vec3::new(0.4, 0.6, 1.0))
        .with_intensity(2.5);

        let scene = Scene::new(&camera)
            .camera_position(Vec3::new(cam_x, cam_y, cam_z).to_array())
            .light(&sun)
            .light(&accent)
            .ambient(0.06)
            .time(t)
            .screen_size(bounds.width, bounds.height)
            .build();

        // ── Generate instances ──
        let columns = generate_columns();
        let max_dist = columns
            .iter()
            .map(|c| c.dist)
            .fold(0.0_f32, f32::max)
            .max(1.0);

        // Rotation to convert extrude-Z to world-Y
        let upright = Quat::from_rotation_x(-FRAC_PI_2);

        let instances: Vec<_> = columns
            .iter()
            .map(|col| {
                // Intro animation: radial wave from center
                let norm_dist = col.dist / max_dist;
                let delay = norm_dist * WAVE_DURATION;
                let local_t = ((t - delay) / POP_DURATION).clamp(0.0, 1.0);
                let anim_t = ease_out_elastic(local_t);

                let h = col.height * anim_t;

                // Cursor bob: columns near cursor bob upward
                let cursor_bob = if let Some(_cursor) = self.cursor {
                    // Approximate world mapping — we'll pass screen cursor to shader
                    // but for geometry we apply a subtle wave
                    let wave = ((t * 3.0 + col.x * 0.5 + col.z * 0.3).sin() * 0.5 + 0.5)
                        * smoothstep(0.0, 1.0, anim_t);
                    wave * 0.05
                } else {
                    0.0
                };

                // Subtle idle sway
                let sway_x = (t * 0.4 + col.column_id * std::f32::consts::TAU).sin() * 0.01;
                let sway_z = (t * 0.35 + col.column_id * 4.17).cos() * 0.01;

                Transform::new()
                    .position(Vec3::new(col.x + sway_x, cursor_bob, col.z + sway_z))
                    .rotation(upright)
                    .scale(Vec3::new(1.0, 1.0, h.max(0.001)))
                    .to_instance([
                        col.height / MAX_HEIGHT, // height_01
                        col.color_seed,          // color_seed
                        0.0,                     // is_ground = false
                        col.column_id,           // column_id
                    ])
            })
            .collect();

        // Ground plane
        let ground = Transform::new()
            .position(Vec3::new(0.0, -0.01, 0.0))
            .to_instance([0.0, 0.0, 1.0, 0.0]); // is_ground = 1.0

        let mut draws = vec![MeshDrawGroup::new(Mesh::plane(50.0, 50.0), vec![ground])];
        draws.push(MeshDrawGroup::new(Mesh::hex_column(HEX_RADIUS), instances));

        // Custom uniforms — cast a ray from the cursor and intersect each
        // column's top face to find the exact column under the pointer.
        let inv_vp = camera.view_projection().inverse();
        let viewport = Vec2::new(bounds.width, bounds.height);
        let (cursor_world, cursor_active) = self
            .cursor
            .and_then(|p| {
                let ray = Ray::from_screen(Vec2::new(p.x, p.y), viewport, inv_vp);
                // For each column, intersect the ray with a plane at the
                // column's top (Y = height). If the XZ hit is within the hex
                // radius, it's a candidate. Pick the nearest by ray t.
                let mut best: Option<(f32, f32, f32)> = None; // (t, x, z)
                for col in &columns {
                    let center = Vec3::new(col.x, col.height, col.z);
                    if let Some(t) = ray.intersect_disk(center, Vec3::Y, HEX_RADIUS) {
                        if best.is_none_or(|(bt, _, _)| t < bt) {
                            best = Some((t, col.x, col.z));
                        }
                    }
                }
                best.map(|(_, x, z)| [x, z])
            })
            .map_or(([0.0, 0.0], 0.0), |xz| (xz, 1.0));

        Scene3DSetup {
            scene,
            draws,
            overlays: vec![],
            custom_uniforms: Some(
                bytemuck::bytes_of(&CrystalUniforms {
                    cursor_world,
                    cursor_active,
                    _pad: 0.0,
                })
                .to_vec(),
            ),
            clear_color: wgpu::Color::BLACK,
        }
    }
}
