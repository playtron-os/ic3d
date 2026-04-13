//! Material definitions for scene graph nodes.

use glam::Vec3;
use std::sync::atomic::{AtomicU32, Ordering};

/// Global counter for auto-generating unique material IDs.
static NEXT_MATERIAL_ID: AtomicU32 = AtomicU32::new(1);

/// Unique identifier for a material in the scene graph.
///
/// Like [`SceneObjectId`](crate::SceneObjectId), each call to [`new()`](Self::new)
/// returns a unique ID. Materials are referenced by ID from scene nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialId(pub u32);

impl MaterialId {
    /// Create a new unique material ID.
    #[must_use]
    pub fn new() -> Self {
        Self(NEXT_MATERIAL_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for MaterialId {
    fn default() -> Self {
        Self::new()
    }
}

/// Material properties for rendering.
///
/// Maps to the `material` vec4 in [`InstanceData`](crate::InstanceData):
/// - `albedo` → `material.rgb`
/// - `shininess` → `material.a`
///
/// # Example
///
/// ```rust,ignore
/// use ic3d::graph::Material;
/// use ic3d::glam::Vec3;
///
/// let mat = Material::new(Vec3::new(0.2, 0.6, 0.9))
///     .with_shininess(64.0)
///     .with_name("blue_metal");
/// ```
#[derive(Debug, Clone)]
pub struct Material {
    /// Optional human-readable name (for lookup and debugging).
    name: Option<String>,
    /// Base color (linear RGB, 0.0–1.0).
    albedo: Vec3,
    /// Shininess exponent for Blinn-Phong (default: 32.0).
    shininess: f32,
}

impl Material {
    /// Create a material with the given albedo color.
    #[must_use]
    pub fn new(albedo: Vec3) -> Self {
        Self {
            name: None,
            albedo,
            shininess: 32.0,
        }
    }

    /// Set the optional name.
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Set the shininess exponent (default: 32.0).
    #[must_use]
    pub fn with_shininess(mut self, shininess: f32) -> Self {
        self.shininess = shininess;
        self
    }

    /// The material name, if set.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Base color.
    #[must_use]
    pub fn albedo(&self) -> Vec3 {
        self.albedo
    }

    /// Set the base color.
    pub fn set_albedo(&mut self, albedo: Vec3) {
        self.albedo = albedo;
    }

    /// Shininess exponent.
    #[must_use]
    pub fn shininess(&self) -> f32 {
        self.shininess
    }

    /// Convert to the `[f32; 4]` material format used by [`InstanceData`](crate::InstanceData).
    #[must_use]
    pub fn to_instance_material(&self) -> [f32; 4] {
        [self.albedo.x, self.albedo.y, self.albedo.z, self.shininess]
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new(Vec3::new(0.8, 0.8, 0.8))
    }
}
