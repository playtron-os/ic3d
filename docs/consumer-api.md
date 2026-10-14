# Using ic3d

How a consumer app composes shaders and drives the pipeline, the scene graph widget and a custom `Scene3DProgram`.

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
