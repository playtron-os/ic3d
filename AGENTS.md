# ic3d

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
- `Gizmo` — 3D manipulation gizmo for CAD workflows. Translation mode with X/Y/Z axis handles. Produces `MeshDrawGroup`s for rendering and `GizmoResult` events for interaction.
- `GizmoMode` — Enum: `Translate` (future: `Rotate`, `Scale`).
- `GizmoAxis` — Enum: `X`, `Y`, `Z`.
- `GizmoResult` — Enum: `Hover(GizmoAxis)`, `Translate(Vec3)`.
- `Ray` — Screen-to-world ray casting with plane intersection and line closest-approach.
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

## Build & Validate

```bash
# Using go-task (recommended)
go-task build            # Build the library
go-task lint             # fmt check + clippy
go-task fmt              # Format all code
go-task test             # Run all tests
go-task ci               # Full CI pipeline (fmt + clippy + test + build)
go-task coverage         # Test coverage report (HTML in coverage/)

# Docker builds (for CI)
go-task docker:build     # Build the CI container image
go-task docker:run TARGET=ci  # Run full CI checks in Docker

# Using cargo directly
cargo build
cargo test
cargo clippy --all-targets --all-features
cargo doc --no-deps
```

## Dependencies

- **iced** — Playtron fork (`github.com/playtron-os/iced.git`), re-exported as `ic3d::iced`
- **wgpu** — Playtron fork (`github.com/playtron-os/wgpu.git`), re-exported as `ic3d::wgpu`
- **glam** 0.29 — Math (re-exported as `ic3d::glam`)
- **bytemuck** 1.14 — Zero-copy GPU uploads

## Conventions

- `#[forbid(unsafe_code)]`
- All public types have doc comments
- `#[must_use]` on builder methods
- All struct fields private with accessors — do not expose internal GPU buffers or state
- WGSL lives in `shaders/*.wgsl`, embedded via `include_str!`
- Rust types and WGSL structs must stay in sync (byte-for-byte)
- `glam` is re-exported — consumers use `ic3d::glam` instead of a direct dependency
- Keep files small and modular: one type/concept per file, grouped by folder (camera/, light/)
- New camera types → new file in `camera/`, new light types → new file in `light/`
- Tests live in separate `_tests.rs` files, included via `#[cfg(test)] #[path = "foo_tests.rs"] mod tests;`

## Testing

### Test Organization
- **Unit tests** (`src/**/*_tests.rs`): Pure CPU logic — math, transforms, scene building, struct sizes, shader strings. Each source file has its own `_tests.rs` file (e.g., `math.rs` → `math_tests.rs`, `camera/orthographic.rs` → `camera/orthographic_tests.rs`). Do NOT combine tests for multiple source files into one test file.
- **GPU integration tests** (`tests/gpu_*.rs`): Require a Vulkan device (real GPU locally, lavapipe in CI). Test pipeline creation, buffer operations, shadow pass, custom uniforms, full render cycles. Use `tests/gpu_helper.rs` for device creation.

### Coverage Requirements
- Target **80%+ test coverage** overall. Run `go-task coverage` to check.
- When adding a new feature or type, **always add corresponding tests**:
  - Pure logic (builders, math, accessors, conversions) → unit test in `_tests.rs` file
  - GPU resource creation or buffer operations → GPU integration test in `tests/gpu_*.rs`
- New camera types → add `camera/<name>_tests.rs` with builder, matrix, and accessor tests
- New light types → add `light/<name>_tests.rs` with builder, `to_gpu_light()`, and field tests
- New mesh primitives → add `mesh/<name>_tests.rs` with vertex count and triangle divisibility tests
- New shader preludes → add assertions in `shaders_tests.rs` (non-empty, contains expected keywords)
- New GPU types → add size/alignment assertions in `gpu_types_tests.rs`

### Running Tests
```bash
go-task test              # All tests (unit + GPU, requires Vulkan)
go-task test:unit         # Unit tests only (no GPU needed)
go-task test:gpu          # GPU integration tests only
go-task coverage          # Coverage report (HTML in coverage/)
```

### GPU Test Helper
GPU integration tests use `tests/gpu_helper.rs` which creates a headless Vulkan device:
```rust
#[path = "gpu_helper.rs"]
mod gpu_helper;

#[test]
fn my_gpu_test() {
    let (device, queue) = gpu_helper::gpu();
    // ... test with real GPU device
}
```
In CI, this runs against lavapipe (Mesa software Vulkan renderer) installed in the Docker image.

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
let shader = ic3d::compose_shader(include_str!("my_fragment.wgsl"));
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

For most consumers, the **scene graph** is the simplest path — build a `SceneGraph` with camera, lights, materials, and meshes. It implements `Scene3DProgram` directly, so it plugs straight into `scene_3d()`:

```rust
use ic3d::graph::{SceneGraph, Material, AmbientLight};
use ic3d::widget::scene_3d;
use ic3d::{Mesh, PerspectiveCamera, DirectionalLight, SceneHandle};
use ic3d::glam::Vec3;

let mut graph = SceneGraph::new();

// Materials, camera, lights
let blue = graph.add_material(Material::new(Vec3::new(0.2, 0.6, 0.9)).with_shininess(64.0));
let cam_id = graph.add_camera(PerspectiveCamera::new()
    .position(Vec3::new(5.0, 5.0, 8.0))
    .target(Vec3::ZERO)
    .clip(0.1, 50.0));
graph.add_light(DirectionalLight::new(
    Vec3::new(-0.5, -1.0, -0.3), Vec3::ZERO, 20.0, 40.0));
graph.add_light(AmbientLight::new(0.15));

// Meshes with hierarchy
let body = graph.add_mesh("body", Mesh::cube(1.0)).material(blue)
    .position(Vec3::new(0.0, 1.0, 0.0)).id();
let _arm = graph.add_mesh("arm", Mesh::cube(1.0)).material(blue)
    .parent(body).position(Vec3::new(0.9, 0.3, 0.0)).id();

// In view() — graph implements Scene3DProgram
let handle = SceneHandle::new();
scene_3d(graph.clone()).scene(handle.clone()).width(Length::Fill).height(Length::Fill)
```

Mutate the scene at runtime via `graph.node_mut(id)`, `graph.camera_mut::<PerspectiveCamera>(cam_id)`, and `graph.light_mut::<DirectionalLight>(sun_id)`.

### Widget API (Advanced — Custom Scene3DProgram)

For full control (custom fragment shaders, custom uniforms, manual instance transforms), implement `Scene3DProgram` directly:

```rust
use ic3d::widget::{scene_3d, Scene3DProgram, Scene3DSetup, MeshDrawGroup};

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
            overlays: Vec::new(),
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

## Examples

```bash
# Scene graph — all primitives with orbiting camera and 3-point lighting
cargo run --example showcase

# Translation gizmo — drag axes to move a cube (scene graph)
cargo run --example gizmo

# Custom overlay — scale gizmo built with DraggableOverlay (scene graph)
cargo run --example gizmo_manual

# Advanced — manual Scene3DProgram with debug shader modes (1-6 to switch)
cargo run --example showcase_advanced --features debug
```
