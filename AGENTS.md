# iced3d

## Project Overview

Lightweight 3D instanced rendering library for iced applications. Provides a shadow-mapped render pipeline with optional post-processing, WGSL shader preludes, camera/light/mesh abstractions, and custom bind group support. Consumers write only a fragment shader.

## Architecture

```
shaders/*.wgsl  →  include_str!()  →  shaders.rs constants
                                         ↓
Consumer: format!("{preludes}\n{my_fragment.wgsl}")
                                         ↓
RenderPipeline3D::new(device, format, &shader, config)
  Pass 1: Shadow depth (light POV → depth texture)
  Pass 2: Main render (camera POV, samples shadow map, MSAA)
  Pass 3+: Post-process chain (ping-pong between cached textures)
```

### Code Organization

One concept per file, grouped by folder (e.g., `camera/`, `light/`). When adding new camera or light types, add a new file in the appropriate folder and re-export from `mod.rs`. Do not merge unrelated types into a single file.

### GPU Bind Groups

- **Group 0 (engine)**:
  - `binding(0)`: `SceneUniforms` uniform (camera, time, screen, ambient, light count)
  - `binding(1)`: `array<GpuLight>` storage buffer (up to 16 lights)
  - `binding(2)`: `shadow_map` depth texture
  - `binding(3)`: `shadow_sampler` comparison sampler
- **Group 1 (consumer)**: Optional custom uniforms (e.g., reveal effect)

### Key Types

- `RenderPipeline3D` — Full pipeline: shadow + MSAA + post-process. All fields private; use accessors and `prepare()`/`prepare_scene()`.
- `PipelineConfig` — Shadow map size, MSAA samples, custom bind group layout.
- `SceneUniforms` — Camera + screen + time + ambient + light count + shadow map size (matches WGSL exactly).
- `GpuLight` — Per-light GPU data (128 bytes): shadow projection, direction, color, intensity, position, range, cone angles.
- `MAX_LIGHTS` — Maximum lights per frame (16).
- `LIGHT_TYPE_DIRECTIONAL` / `LIGHT_TYPE_POINT` / `LIGHT_TYPE_SPOT` — Light type discriminants.
- `Vertex` — Per-vertex: position + normal + UV (slot 0).
- `InstanceData` — Per-instance: model mat4 + normal mat3x3 + material vec4 (128 bytes, slot 1).
- `DrawCall` — Vertex buffer + instance buffer + ranges for a single draw. Prefer `pipeline.draw()` over manual construction.
- `Scene` — Builder: camera + lights → `SceneData`. Uses fixed-size `[GpuLight; MAX_LIGHTS]` array internally (no heap allocation per frame).
- `Camera` trait — `view_matrix()`, `projection_matrix()`, `view_projection()`. Implemented by `OrthographicCamera` and `PerspectiveCamera`.
- `Light` trait — `fn to_gpu_light(&self) -> GpuLight`. Implemented by `DirectionalLight`, `PointLight`, `SpotLight`.
- `DirectionalLight` — Direction + color + intensity + shadow projection.
- `PointLight` — Position + range + color + intensity (omnidirectional).
- `SpotLight` — Position + direction + inner/outer cone angles + range + color + intensity.
- `Transform` — TRS → model + normal matrices → `InstanceData`.
- `Mesh` — CPU-side vertex data + primitive builders (cube, sphere, cylinder, cone, torus, plane). Use `mesh.upload(device)` to get a `MeshBuffer`.
- `MeshBuffer` — Uploaded mesh: GPU vertex buffer + vertex count. Created via `Mesh::upload()`. Use with `pipeline.draw()` for the simplest render path.
- `DynBuffer` — Auto-growing GPU buffer (2× growth strategy). Private `raw` field; use `.raw()` accessor or `.write()` convenience. Old buffers are retired to a `BufferPool` rather than dropped.
- `BufferPool` — Frame-delayed buffer recycling. Retired buffers wait 3 frames (GPU pipeline depth) before becoming available for best-fit reuse. `RenderPipeline3D` manages its own pool internally; create a standalone pool only if using `DynBuffer` outside the pipeline.
- `PostProcessPass` trait — Screen-space effects: `prepare()` + `render()`. Wired into pipeline via `add_post_process()` or widget's `post_process_factory()`.
- `PostProcessFactory` — Type alias for the factory closure that creates post-process passes at pipeline init time.

### WGSL Preludes (`shaders/`)

| File | Constant | Contents |
|------|----------|----------|
| `scene_uniforms.wgsl` | `SCENE_UNIFORMS_WGSL` | `SceneUniforms` + `GpuLight` structs + group 0 bindings (4 entries) |
| `vertex_io.wgsl` | `VERTEX_IO_WGSL` | `VertexIn` / `VertexOut` structs |
| `standard_vs.wgsl` | `STANDARD_VS_WGSL` | Standard `vs_main` vertex shader |
| `shadow_pcf.wgsl` | `SHADOW_PCF_WGSL` | 16-tap rotated Poisson disk PCF shadow sampling |
| `blinn_phong.wgsl` | `BLINN_PHONG_WGSL` | Default fragment shader: Blinn-Phong + Fresnel + tone mapping |
| `shadow_pass.wgsl` | `SHADOW_WGSL` (internal) | Shadow depth pass shader |

## Build

```bash
cargo build
cargo doc --no-deps
```

## Dependencies

- **iced** — Playtron fork (`github.com/playtron-os/iced.git`), re-exported as `iced3d::iced`
- **wgpu** — Playtron fork (`github.com/playtron-os/wgpu.git`), re-exported as `iced3d::wgpu`
- **glam** 0.29 — Math (re-exported as `iced3d::glam`)
- **bytemuck** 1.14 — Zero-copy GPU uploads

## Conventions

- `#[forbid(unsafe_code)]`
- All public types have doc comments
- `#[must_use]` on builder methods
- All struct fields private with accessors — do not expose internal GPU buffers or state
- WGSL lives in `shaders/*.wgsl`, embedded via `include_str!`
- Rust types and WGSL structs must stay in sync (byte-for-byte)
- `glam` is re-exported — consumers use `iced3d::glam` instead of a direct dependency
- Keep files small and modular: one type/concept per file, grouped by folder (camera/, light/)
- New camera types → new file in `camera/`, new light types → new file in `light/`
- Tests live in separate `_tests.rs` files, included via `#[cfg(test)] #[path = "foo_tests.rs"] mod tests;`

## WGSL Alignment (Critical)

WGSL alignment differs from Rust `#[repr(C)]`:

- `vec2<f32>` → 8-byte alignment (not 4)
- `vec3<f32>` → 16-byte alignment
- `mat3x3<f32>` columns → 16 bytes each

When adding uniform structs, verify Rust `size_of` matches WGSL. Mismatches cause silent data corruption or runtime validation errors.

- `compose_shader(fragment_wgsl)` — Convenience: prepends all engine WGSL preludes to a consumer fragment shader.

## Consumer Integration

```rust
// Compose shader — preludes are auto-prepended
let shader = iced3d::compose_shader(include_str!("my_fragment.wgsl"));
let pipeline = RenderPipeline3D::new(device, format, &shader, config);
```

The consumer's `.wgsl` file only needs:
1. Custom uniform struct + `@group(1)` binding (if needed)
2. Helper functions (noise, effects, etc.)
3. `@fragment fn fs_main(in: VertexOut) -> @location(0) vec4<f32>`
4. Access light data via `lights[0].direction`, `lights[0].color`, etc.

### Pipeline API

```rust
// Build scene (no heap allocation — fixed-size light array internally)
let scene_data = Scene::new(&camera)
    .light(&directional_light)   // accepts any &dyn Light
    .light(&point_light)
    .ambient(0.15)
    .time(elapsed)
    .screen(width, height)
    .build();

// Option A: prepare_scene() convenience
pipeline.prepare_scene(device, queue, bounds, &scene_data, &instances);

// Option B: prepare() with explicit params
pipeline.prepare(device, queue, bounds, &scene_data.uniforms, &scene_data.lights, &instances);

// Post-processing (register once after creation)
pipeline.add_post_process(Box::new(MyBloomPass::new()));

// Render — draw() builds DrawCall automatically from MeshBuffer
let cube = Mesh::cube(1.0).upload(device);
pipeline.render(encoder, target, bounds, &[
    pipeline.draw(&cube, 0..instance_count),
], None);
```

Call `pipeline.warmup()` after creation to avoid NVIDIA deferred shader compilation stalls.

### iced Constraints

- `shader::Primitive::render()` does NOT provide `device` or `queue` — create/resize textures in `prepare()`, not `render()`
- `Pipeline` trait requires `Send + Sync` — `Box<dyn PostProcessPass>` must have `Send + Sync` bounds

### Widget API (High-Level)

For most consumers, the `widget` module provides the simplest path — implement `Scene3DProgram` and call `scene_3d()`. Only `setup()` is required; the built-in Blinn-Phong shader handles lighting automatically:

```rust
use iced3d::widget::{scene_3d, Scene3DProgram, Scene3DSetup, MeshDrawGroup};

#[derive(Debug)]
struct MyScene { time: f32 }

impl Scene3DProgram for MyScene {
    fn setup(&self, bounds: Rectangle) -> Scene3DSetup {
        let camera = PerspectiveCamera::new().position(Vec3::new(0.0, 2.0, 5.0));
        let sun = DirectionalLight::new(Vec3::new(-0.5, -1.0, -0.3), Vec3::ZERO, 15.0, 30.0);
        let scene = Scene::new(&camera).light(&sun).ambient(0.1).time(self.time).build();
        Scene3DSetup {
            scene,
            draws: vec![MeshDrawGroup::new(Mesh::cube(1.0), vec![instance])],
            custom_uniforms: None,
        }
    }
}

// In view():
scene_3d(MyScene { time: elapsed }).width(Length::Fill).height(Length::Fill)
```

**Custom fragment shader**: override `fragment_shader()` to replace the default Blinn-Phong:
```rust
fn fragment_shader(&self) -> &str { include_str!("my_effect.wgsl") }
```

**Custom uniforms** (`@group(1) @binding(0)`): implement `custom_uniforms_size()` and return raw bytes in `setup()`:
```rust
impl Scene3DProgram for MyScene {
    fn custom_uniforms_size(&self) -> usize { std::mem::size_of::<MyUniforms>() }
    fn setup(&self, bounds: Rectangle) -> Scene3DSetup {
        Scene3DSetup {
            // ...
            custom_uniforms: Some(bytemuck::bytes_of(&my_uniforms).to_vec()),
        }
    }
}
```

**Post-processing**: return a factory closure from `post_process_factory()`:
```rust
fn post_process_factory(&self) -> Option<PostProcessFactory> {
    Some(Box::new(|device, _queue| {
        vec![Box::new(MyBloomPass::new(device))]
    }))
}
```

The widget handles buffer creation, bind group layout, uploading, and passing to the render pipeline automatically.

## Example

```bash
cargo run --example showcase
```

Renders all built-in primitives (cube, sphere, cylinder, cone, torus, plane) with directional, point, and spot lights. Camera orbits automatically.
