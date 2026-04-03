//! Post-processing pass trait for screen-space effects.
//!
//! Add passes to [`RenderPipeline3D`](crate::RenderPipeline3D) via
//! [`add_post_process()`](crate::RenderPipeline3D::add_post_process).
//! They execute in order after the main render, ping-ponging between
//! intermediate textures.

/// A screen-space post-processing pass (source texture → target texture).
///
/// [`prepare`](Self::prepare) is called during
/// [`RenderPipeline3D::prepare`](crate::RenderPipeline3D::prepare) with the
/// current target size. [`render`](Self::render) is called during the
/// post-processing phase after the main pass.
pub trait PostProcessPass {
    /// Prepare GPU resources for the given output size.
    ///
    /// Called each frame before render. Resize internal textures if `target_size` changed.
    fn prepare(&self, device: &wgpu::Device, queue: &wgpu::Queue, target_size: (u32, u32));

    /// Record post-processing commands. Reads `source`, writes `target`.
    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::TextureView,
        target: &wgpu::TextureView,
    );
}
