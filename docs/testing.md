# Testing

Where unit and GPU tests live, what each new type must be tested for, and how GPU tests get a device.

## Test Organization
- **Unit tests** (`src/**/*_tests.rs`): Pure CPU logic — math, transforms, scene building, struct sizes, shader strings. Each source file has its own `_tests.rs` file (e.g., `math.rs` → `math_tests.rs`, `camera/orthographic.rs` → `camera/orthographic_tests.rs`). Do NOT combine tests for multiple source files into one test file.
- **GPU integration tests** (`tests/gpu_*.rs`): Require a Vulkan device (real GPU locally, lavapipe in CI). Test pipeline creation, buffer operations, shadow pass, custom uniforms, full render cycles. Use `tests/gpu_helper.rs` for device creation.

## Coverage Requirements
- Target **80%+ test coverage** overall. Run `go-task coverage` to check.
- When adding a new feature or type, **always add corresponding tests**:
  - Pure logic (builders, math, accessors, conversions) → unit test in `_tests.rs` file
  - GPU resource creation or buffer operations → GPU integration test in `tests/gpu_*.rs`
- New camera types → add `camera/<name>_tests.rs` with builder, matrix, and accessor tests
- New light types → add `light/<name>_tests.rs` with builder, `to_gpu_light()`, and field tests
- New mesh primitives → add `mesh/<name>_tests.rs` with vertex count and triangle divisibility tests
- New shader preludes → add assertions in `shaders_tests.rs` (non-empty, contains expected keywords)
- New GPU types → add size/alignment assertions in `gpu_types_tests.rs`

## Running Tests
```bash
go-task test              # All tests (unit + GPU, requires Vulkan)
go-task test:unit         # Unit tests only (no GPU needed)
go-task test:gpu          # GPU integration tests only
go-task coverage          # Coverage report (HTML in coverage/)
```

## GPU Test Helper
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
