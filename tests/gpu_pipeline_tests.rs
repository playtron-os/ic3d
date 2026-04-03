//! GPU integration tests for `RenderPipeline3D`.

#[path = "gpu_helper.rs"]
mod gpu_helper;

use iced3d::wgpu;
use iced3d::{
    compose_shader, CustomUniformBuffer, Mesh, OrthographicCamera, PipelineConfig,
    RenderPipeline3D, Scene, Transform, BLINN_PHONG_WGSL,
};

fn default_shader() -> String {
    compose_shader(BLINN_PHONG_WGSL)
}

fn output_format() -> wgpu::TextureFormat {
    wgpu::TextureFormat::Bgra8UnormSrgb
}

#[test]
fn creation_default_config() {
    let (device, _queue) = gpu_helper::gpu();
    let shader = default_shader();
    let _pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());
}

#[test]
fn creation_small_shadow_map() {
    let (device, _queue) = gpu_helper::gpu();
    let shader = default_shader();
    let config = PipelineConfig::default().shadow_map_size(1);
    let _pipeline = RenderPipeline3D::new(&device, output_format(), &shader, config);
}

#[test]
fn creation_no_msaa() {
    let (device, _queue) = gpu_helper::gpu();
    let shader = default_shader();
    let config = PipelineConfig::default().msaa_samples(1);
    let _pipeline = RenderPipeline3D::new(&device, output_format(), &shader, config);
}

#[test]
fn creation_with_custom_uniforms() {
    let (device, _queue) = gpu_helper::gpu();
    let custom = CustomUniformBuffer::new(&device, 16);
    let shader = default_shader();
    let config = PipelineConfig::default().custom_bind_group_layout(custom.layout());
    let _pipeline = RenderPipeline3D::new(&device, output_format(), &shader, config);
}

#[test]
fn set_clear_color() {
    let (device, _queue) = gpu_helper::gpu();
    let shader = default_shader();
    let mut pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());
    pipeline.set_clear_color(wgpu::Color::BLACK);
}

#[test]
fn prepare_empty_scene() {
    let (device, queue) = gpu_helper::gpu();
    let shader = default_shader();
    let mut pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());

    let scene = Scene::new(&OrthographicCamera::new()).build();
    pipeline.prepare_scene(&device, &queue, (64, 64), &scene, &[]);
}

#[test]
fn prepare_with_instances() {
    let (device, queue) = gpu_helper::gpu();
    let shader = default_shader();
    let mut pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());

    let scene = Scene::new(&OrthographicCamera::new()).build();
    let instances: Vec<_> = (0..10)
        .map(|_| Transform::default().to_instance([1.0, 1.0, 1.0, 1.0]))
        .collect();
    pipeline.prepare_scene(&device, &queue, (128, 128), &scene, &instances);
}

#[test]
fn prepare_resize_triggers_msaa_rebuild() {
    let (device, queue) = gpu_helper::gpu();
    let shader = default_shader();
    let mut pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());

    let scene = Scene::new(&OrthographicCamera::new()).build();
    pipeline.prepare_scene(&device, &queue, (64, 64), &scene, &[]);
    pipeline.prepare_scene(&device, &queue, (256, 256), &scene, &[]);
}

#[test]
fn draw_builds_draw_call() {
    let (device, _queue) = gpu_helper::gpu();
    let shader = default_shader();
    let pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());

    let cube = Mesh::cube(1.0).upload(&device);
    let call = pipeline.draw(&cube, 0..1);
    assert_eq!(call.vertex_count, 36);
}

#[test]
fn full_render_cycle() {
    let (device, queue) = gpu_helper::gpu();
    let shader = default_shader();
    let mut pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());

    let scene = Scene::new(&OrthographicCamera::new()).build();
    let instance = Transform::default().to_instance([1.0, 0.5, 0.5, 1.0]);
    pipeline.prepare_scene(&device, &queue, (64, 64), &scene, &[instance]);

    let cube = Mesh::cube(1.0).upload(&device);

    let target_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test target"),
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: output_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("test encoder"),
    });

    pipeline.render(
        &mut encoder,
        &target_view,
        (0, 0, 64, 64),
        &[pipeline.draw(&cube, 0..1)],
        None,
    );

    queue.submit(std::iter::once(encoder.finish()));
}

#[test]
fn warmup_completes() {
    let (device, queue) = gpu_helper::gpu();
    let shader = default_shader();
    let mut pipeline =
        RenderPipeline3D::new(&device, output_format(), &shader, PipelineConfig::default());

    let cube = Mesh::cube(1.0).upload(&device);
    pipeline.warmup(&device, &queue, &[cube.buffer()], None);
}

#[test]
fn warmup_with_custom_uniforms() {
    let (device, queue) = gpu_helper::gpu();
    let custom = CustomUniformBuffer::new(&device, 16);
    let shader = default_shader();
    let config = PipelineConfig::default().custom_bind_group_layout(custom.layout());
    let mut pipeline = RenderPipeline3D::new(&device, output_format(), &shader, config);

    custom.write(&queue, &[0u8; 16]);
    let cube = Mesh::cube(1.0).upload(&device);
    pipeline.warmup(&device, &queue, &[cube.buffer()], Some(custom.bind_group()));
}
