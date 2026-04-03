//! Light types: directional, point, and spot.

mod directional;
mod point;
mod spot;

pub use directional::DirectionalLight;
pub use point::PointLight;
pub use spot::SpotLight;

use crate::gpu_types::GpuLight;

/// Trait for light types that can produce GPU-ready data.
pub trait Light {
    /// Convert to GPU-ready [`GpuLight`].
    fn to_gpu_light(&self) -> GpuLight;
}
