# ic3d

Lightweight 3D instanced rendering for [iced](https://iced.rs) applications. Shadow mapping, configurable MSAA, camera/light/mesh abstractions, and reusable WGSL shader preludes. Consumers write only a fragment shader.

![ic3d showcase](docs/example.png)

## Quick Start (Widget API)

The simplest path — implement `Scene3DProgram` and call `scene_3d()`. The built-in Blinn-Phong shader handles lighting automatically:

```rust
use ic3d::widget::{scene_3d, Scene3DProgram, Scene3DSetup, MeshDrawGroup};
use ic3d::{Scene, PerspectiveCamera, DirectionalLight, Mesh, Transform};
use ic3d::glam::Vec3;

#[derive(Debug)]
struct MyScene { time: f32 }

impl Scene3DProgram for MyScene {
    fn setup(&self, bounds: iced::Rectangle) -> Scene3DSetup {
        let camera = PerspectiveCamera::new()
            .position(Vec3::new(0.0, 2.0, 5.0))
            .aspect(bounds.width / bounds.height.max(1.0));
        let sun = DirectionalLight::new(
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Vec3::ZERO, 15.0, 30.0,
        );
        let scene = Scene::new(&camera).light(&sun).ambient(0.15).time(self.time).build();
        let cube = MeshDrawGroup::new(
            Mesh::cube(1.0),
            vec![Transform::new().to_instance([0.8, 0.2, 0.2, 64.0])],
        );
        Scene3DSetup { scene, draws: vec![cube], custom_uniforms: None }
    }
}

// In view():
scene_3d(MyScene { time: elapsed }).width(Length::Fill).height(Length::Fill)
```

Override `fragment_shader()` to replace the default Blinn-Phong, `custom_uniforms_size()` for `@group(1)` data, and `post_process_factory()` for screen-space effects.

## Low-Level Pipeline API

For full control, use `compose_shader()` + `RenderPipeline3D` directly:

```rust
let shader = ic3d::compose_shader(include_str!("my_fragment.wgsl"));
let pipeline = RenderPipeline3D::new(device, format, &shader, PipelineConfig::default());

// Per frame:
let scene_data = Scene::new(&camera)
    .light(&sun)
    .light(&point_light)
    .ambient(0.15)
    .time(elapsed)
    .screen_size(width, height)
    .build();

pipeline.prepare_scene(device, queue, bounds, &scene_data, &instances);

let cube = Mesh::cube(1.0).upload(device);
pipeline.render(encoder, target, bounds, &[
    pipeline.draw(&cube, 0..instance_count),
], None);
```

## GPU Bind Groups

| Group | Binding | Owner    | Contents                                          |
|-------|---------|----------|---------------------------------------------------|
| 0     | 0       | Engine   | `SceneUniforms` (camera, time, screen, ambient)   |
| 0     | 1       | Engine   | `array<GpuLight>` storage buffer (up to 16)       |
| 0     | 2       | Engine   | `shadow_map` depth texture                        |
| 0     | 3       | Engine   | `shadow_sampler` comparison sampler               |
| 1     | 0+      | Consumer | Optional custom uniforms (e.g., debug modes)      |

## Lights

- **`DirectionalLight`** — Direction + color + intensity + orthographic shadow projection
- **`PointLight`** — Position + range + color + intensity (omnidirectional)
- **`SpotLight`** — Position + direction + inner/outer cone angles + range + color + intensity

## Mesh Primitives

`Mesh::cube`, `Mesh::sphere`, `Mesh::cylinder`, `Mesh::cone`, `Mesh::torus`, `Mesh::plane`, `Mesh::custom`

## Example

```bash
cargo run --example showcase
```

Renders all built-in primitives on a ground plane with directional, point, and spot lights. Camera orbits automatically. Press **1-6** to cycle debug views:

| Key | Mode | Shows |
|-----|------|-------|
| 1 | Lit | Full Blinn-Phong + shadows |
| 2 | Normals | Surface normals as RGB |
| 3 | NdotL | Primary light coverage |
| 4 | Shadow | Shadow factor (green=lit, red=shadow) |
| 5 | No-Shadow | Lighting without shadows |
| 6 | Flat | Raw material colors |

## Dependencies

- **iced** — Playtron fork (GUI framework, re-exported as `ic3d::iced`)
- **wgpu** — Playtron fork (GPU access, re-exported as `ic3d::wgpu`)
- **glam** 0.29 — Math (re-exported as `ic3d::glam`)
- **bytemuck** 1.14 — Zero-copy GPU uploads

## Build

```bash
cargo build
cargo test
cargo doc --no-deps
```

## WGSL Alignment

WGSL has strict alignment rules that differ from Rust's `#[repr(C)]`:

- `vec2<f32>` has **8-byte** alignment (not 4)
- `vec3<f32>` has **16-byte** alignment
- `mat3x3<f32>` columns are padded to 16 bytes each

The WGSL preludes and Rust types are kept in sync. When adding new uniform structs, verify byte sizes match on both sides.
