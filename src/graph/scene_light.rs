//! Object-safe light trait and ambient light for scene graph storage.

use crate::light::Light;
use crate::pipeline::gpu_types::GpuLight;
use std::any::Any;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

/// Object-safe light trait for storage in the scene graph.
///
/// Any type implementing `Light + Clone + Debug + Send + Sync + 'static`
/// automatically satisfies this via the blanket impl. Users can store
/// [`DirectionalLight`](crate::DirectionalLight),
/// [`PointLight`](crate::PointLight), [`SpotLight`](crate::SpotLight),
/// [`AmbientLight`], or their own custom light types.
///
/// For runtime mutation of concrete light properties, use
/// [`SceneGraph::light_mut`](super::SceneGraph::light_mut) with a type
/// parameter to downcast:
///
/// ```rust,ignore
/// graph.light_mut::<DirectionalLight>(sun_id)
///     .unwrap()
///     .set_intensity(2.0);
/// ```
pub trait SceneLight: fmt::Debug + Send + Sync {
    /// Clone the light into a new boxed trait object.
    fn clone_light(&self) -> Box<dyn SceneLight>;

    /// Downcast to concrete type (immutable).
    fn as_any(&self) -> &dyn Any;

    /// Downcast to concrete type (mutable).
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns GPU light data for the light array.
    ///
    /// Returns `None` for lights that contribute only to the ambient
    /// uniform (e.g. [`AmbientLight`]).
    fn to_gpu_light(&self) -> Option<GpuLight>;

    /// Ambient light contribution (0.0–1.0).
    ///
    /// Returns `None` by default. Override for ambient-only lights.
    fn ambient_level(&self) -> Option<f32> {
        None
    }
}

impl<T: Light + Clone + fmt::Debug + Send + Sync + 'static> SceneLight for T {
    fn clone_light(&self) -> Box<dyn SceneLight> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_gpu_light(&self) -> Option<GpuLight> {
        Some(Light::to_gpu_light(self))
    }
}

impl Clone for Box<dyn SceneLight> {
    fn clone(&self) -> Self {
        self.clone_light()
    }
}

/// Ambient light — contributes to the scene's ambient illumination uniform.
///
/// Unlike directional/point/spot lights, ambient light doesn't go into the
/// GPU `GpuLight` array. Instead, all ambient lights' levels are summed and
/// written to the `SceneUniforms.ambient` field.
///
/// ```rust,ignore
/// graph.add_light(AmbientLight::new(0.15));
/// ```
#[derive(Debug, Clone)]
pub struct AmbientLight {
    level: f32,
}

impl AmbientLight {
    /// Create an ambient light with the given level (0.0–1.0).
    #[must_use]
    pub fn new(level: f32) -> Self {
        Self { level }
    }

    /// The ambient level.
    #[must_use]
    pub fn level(&self) -> f32 {
        self.level
    }

    /// Set the ambient level.
    pub fn set_level(&mut self, level: f32) {
        self.level = level;
    }
}

impl SceneLight for AmbientLight {
    fn clone_light(&self) -> Box<dyn SceneLight> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_gpu_light(&self) -> Option<GpuLight> {
        None
    }

    fn ambient_level(&self) -> Option<f32> {
        Some(self.level)
    }
}

/// Unique identifier for a light in the scene graph.
///
/// Created automatically by [`SceneGraph::add_light`](super::SceneGraph::add_light).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightId(u32);

impl LightId {
    /// Create a new unique light ID.
    #[must_use]
    pub fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for LightId {
    fn default() -> Self {
        Self::new()
    }
}
