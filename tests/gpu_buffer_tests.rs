//! GPU integration tests for buffer management.

#[path = "gpu_helper.rs"]
mod gpu_helper;

use iced3d::wgpu;
use iced3d::{BufferPool, DynBuffer};

#[test]
fn dyn_buffer_creation() {
    let (device, _queue) = gpu_helper::gpu();
    let buf = DynBuffer::new(
        &device,
        "test",
        1024,
        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    );
    assert_eq!(buf.capacity(), 1024);
}

#[test]
fn dyn_buffer_write() {
    let (device, queue) = gpu_helper::gpu();
    let buf = DynBuffer::new(
        &device,
        "test",
        256,
        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    );
    let data = [0u8; 128];
    buf.write(&queue, &data);
}

#[test]
fn dyn_buffer_ensure_capacity_no_growth() {
    let (device, _queue) = gpu_helper::gpu();
    let mut pool = BufferPool::new();
    let mut buf = DynBuffer::new(
        &device,
        "test",
        1024,
        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    );
    buf.ensure_capacity(&device, &mut pool, 512);
    assert_eq!(buf.capacity(), 1024);
    assert_eq!(pool.pending_count(), 0);
}

#[test]
fn dyn_buffer_ensure_capacity_grows() {
    let (device, _queue) = gpu_helper::gpu();
    let mut pool = BufferPool::new();
    let mut buf = DynBuffer::new(
        &device,
        "test",
        256,
        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    );
    buf.ensure_capacity(&device, &mut pool, 1024);
    assert!(buf.capacity() >= 1024);
    assert_eq!(pool.pending_count(), 1);
}

#[test]
fn pool_acquire_creates_new() {
    let (device, _queue) = gpu_helper::gpu();
    let mut pool = BufferPool::new();
    let (_buf, cap) = pool.acquire(
        &device,
        "test",
        512,
        wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    );
    assert!(cap >= 512);
}

#[test]
fn pool_retire_and_reuse() {
    let (device, _queue) = gpu_helper::gpu();
    let mut pool = BufferPool::new();
    let usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;

    let (buf, cap) = pool.acquire(&device, "test", 1024, usage);
    pool.retire(buf, cap, usage);
    assert_eq!(pool.pending_count(), 1);
    assert_eq!(pool.available_count(), 0);

    for _ in 0..4 {
        pool.advance_frame();
    }
    assert_eq!(pool.pending_count(), 0);
    assert_eq!(pool.available_count(), 1);

    let (_buf2, cap2) = pool.acquire(&device, "test", 512, usage);
    assert!(cap2 >= 1024);
    assert_eq!(pool.available_count(), 0);
}
