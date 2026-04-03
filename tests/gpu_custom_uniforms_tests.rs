//! GPU integration tests for `CustomUniformBuffer`.

#[path = "gpu_helper.rs"]
mod gpu_helper;

use iced3d::CustomUniformBuffer;

#[test]
fn creation() {
    let (device, _queue) = gpu_helper::gpu();
    let custom = CustomUniformBuffer::new(&device, 64);
    let _layout = custom.layout();
    let _bind_group = custom.bind_group();
    let _buffer = custom.buffer();
}

#[test]
fn write_data() {
    let (device, queue) = gpu_helper::gpu();
    let custom = CustomUniformBuffer::new(&device, 16);
    let data = [1.0_f32, 2.0, 3.0, 4.0];
    custom.write(&queue, bytemuck::bytes_of(&data));
}

#[test]
fn different_sizes() {
    let (device, _queue) = gpu_helper::gpu();
    for size in [16, 64, 128, 256] {
        let custom = CustomUniformBuffer::new(&device, size);
        let _layout = custom.layout();
    }
}
