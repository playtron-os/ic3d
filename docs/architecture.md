# Render pipeline

How the render pipeline is put together: its passes, bind groups, public types and WGSL preludes.

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
