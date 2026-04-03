//! GPU buffer management with pool recycling and frame-delayed deletion.
//!
//! [`DynBuffer`] auto-grows with 2× amortization. When it outgrows its
//! capacity, the old buffer is retired to a [`BufferPool`] rather than dropped
//! immediately. After a configurable latency (default: 3 frames), retired
//! buffers become available for reuse — ensuring the GPU has finished any
//! in-flight reads before they are repurposed.
//!
//! This avoids two costs that cause frame stalls:
//! - **Allocation**: driver-level GPU memory allocation (page table updates)
//! - **Deallocation**: cleanup of the old buffer's native resources
//!
//! By reusing pooled buffers, both are eliminated once the pool is warm.

/// Frames to wait before a retired buffer can be reused (triple-buffering depth).
const DEFAULT_POOL_LATENCY: u64 = 3;

/// Maximum buffers kept in the available pool. Beyond this, retired buffers
/// are dropped to free VRAM rather than hoarded indefinitely.
const MAX_POOL_SIZE: usize = 16;

struct PendingBuffer {
    buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
    retire_frame: u64,
}

struct AvailableBuffer {
    buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
}

/// GPU buffer pool with frame-delayed recycling.
///
/// When a [`DynBuffer`] grows, the old buffer is retired here rather than
/// dropped immediately. After `latency` frames (default: 3), retired buffers
/// become available for reuse — preventing GPU stalls from reading freed memory.
///
/// Uses best-fit allocation: the smallest sufficient buffer with matching
/// usage flags is chosen, minimizing VRAM waste.
///
/// [`RenderPipeline3D`](crate::RenderPipeline3D) manages its own internal pool
/// automatically. Create a standalone pool only if using [`DynBuffer`] outside
/// the pipeline.
pub struct BufferPool {
    pending: Vec<PendingBuffer>,
    available: Vec<AvailableBuffer>,
    frame: u64,
    latency: u64,
}

impl BufferPool {
    /// Create a pool with default latency (3 frames).
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            available: Vec::new(),
            frame: 0,
            latency: DEFAULT_POOL_LATENCY,
        }
    }

    /// Create a pool with custom frame latency.
    #[must_use]
    pub fn with_latency(latency: u64) -> Self {
        Self {
            latency,
            ..Self::new()
        }
    }

    /// Advance the frame counter and recycle pending buffers.
    ///
    /// Call once per frame, before any buffer operations. Buffers that have
    /// waited at least `latency` frames move to the available pool.
    pub fn advance_frame(&mut self) {
        self.frame += 1;
        let mut i = 0;
        while i < self.pending.len() {
            if self.frame - self.pending[i].retire_frame >= self.latency {
                let p = self.pending.swap_remove(i);
                if self.available.len() < MAX_POOL_SIZE {
                    self.available.push(AvailableBuffer {
                        buffer: p.buffer,
                        capacity: p.capacity,
                        usage: p.usage,
                    });
                }
                // else buffer is dropped — pool full, free VRAM
            } else {
                i += 1;
            }
        }
    }

    /// Retire a buffer for later reuse.
    ///
    /// The buffer will not be available until `latency` frames have passed,
    /// ensuring the GPU has finished any in-flight reads.
    pub fn retire(&mut self, buffer: wgpu::Buffer, capacity: u64, usage: wgpu::BufferUsages) {
        self.pending.push(PendingBuffer {
            buffer,
            capacity,
            usage,
            retire_frame: self.frame,
        });
    }

    /// Acquire a buffer with at least `min_capacity` bytes and matching `usage`.
    ///
    /// Checks the available pool first (best-fit: smallest sufficient buffer).
    /// Creates a new buffer if no suitable one is pooled.
    pub fn acquire(
        &mut self,
        device: &wgpu::Device,
        label: &'static str,
        min_capacity: u64,
        usage: wgpu::BufferUsages,
    ) -> (wgpu::Buffer, u64) {
        let best = self
            .available
            .iter()
            .enumerate()
            .filter(|(_, a)| a.usage == usage && a.capacity >= min_capacity)
            .min_by_key(|(_, a)| a.capacity);

        if let Some((idx, _)) = best {
            let a = self.available.swap_remove(idx);
            return (a.buffer, a.capacity);
        }

        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: min_capacity,
            usage,
            mapped_at_creation: false,
        });
        (buf, min_capacity)
    }

    /// Current frame number.
    #[must_use]
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Buffers waiting to be recycled (still in GPU pipeline).
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Buffers available for immediate reuse.
    #[must_use]
    pub fn available_count(&self) -> usize {
        self.available.len()
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A GPU buffer that reallocates when more capacity is needed.
///
/// Grows by 2× when the requested size exceeds capacity, amortizing
/// reallocation cost. Old buffers are retired to a [`BufferPool`] for
/// frame-delayed recycling rather than dropped immediately.
pub struct DynBuffer {
    raw: wgpu::Buffer,
    label: &'static str,
    capacity: u64,
    usage: wgpu::BufferUsages,
}

impl DynBuffer {
    /// Create a new dynamic buffer with the given initial capacity.
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        capacity: u64,
        usage: wgpu::BufferUsages,
    ) -> Self {
        Self {
            raw: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: capacity,
                usage,
                mapped_at_creation: false,
            }),
            label,
            capacity,
            usage,
        }
    }

    /// The underlying wgpu buffer.
    #[must_use]
    pub fn raw(&self) -> &wgpu::Buffer {
        &self.raw
    }

    /// Ensure the buffer is at least `needed` bytes.
    ///
    /// If growth is required, the old buffer is retired to the pool and a new
    /// one is acquired (from pool or freshly allocated). The 2× growth strategy
    /// amortizes reallocation cost.
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, pool: &mut BufferPool, needed: u64) {
        if needed <= self.capacity {
            return;
        }
        let new_capacity = (needed * 2).max(self.capacity * 2);
        let (new_buf, actual_cap) = pool.acquire(device, self.label, new_capacity, self.usage);
        let old = std::mem::replace(&mut self.raw, new_buf);
        pool.retire(old, self.capacity, self.usage);
        self.capacity = actual_cap;
    }

    /// Current buffer capacity in bytes.
    #[must_use]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Write data to the buffer (convenience for `queue.write_buffer`).
    pub fn write(&self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(&self.raw, 0, data);
    }
}

#[cfg(test)]
#[path = "buffer_tests.rs"]
mod tests;
