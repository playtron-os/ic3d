//! Scene graph node: named, hierarchical, typed.

use super::material::MaterialId;
use crate::mesh::Mesh;
use crate::scene::object::SceneObjectId;
use crate::scene::transform::Transform;

/// What kind of data a scene node contains.
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// Empty grouping / pivot point.
    Empty,
    /// A renderable mesh with a material.
    Mesh {
        /// The mesh geometry.
        mesh: Mesh,
        /// Material ID (must exist in the parent [`SceneGraph`](super::SceneGraph)).
        material: MaterialId,
    },
}

/// A node in the scene graph.
///
/// Each node has a unique [`SceneObjectId`], an optional name for lookup,
/// a local transform relative to its parent, and a [`NodeKind`] describing
/// what it represents.
///
/// Nodes form a tree via the parent-child relationships managed by
/// [`SceneGraph`](super::SceneGraph). The world transform is computed by
/// multiplying the chain of local transforms from root to node.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier (auto-generated via [`SceneObjectId::new`]).
    id: SceneObjectId,
    /// Optional human-readable name for lookup.
    name: Option<String>,
    /// Local transform relative to the parent node.
    local_transform: Transform,
    /// What this node represents.
    kind: NodeKind,
    /// Whether this node (and its children) should be rendered.
    visible: bool,
}

impl Node {
    /// Create a new node with the given ID and kind.
    pub(crate) fn new(id: SceneObjectId, kind: NodeKind) -> Self {
        Self {
            id,
            name: None,
            local_transform: Transform::new(),
            kind,
            visible: true,
        }
    }

    /// The node's unique ID.
    #[must_use]
    pub fn id(&self) -> SceneObjectId {
        self.id
    }

    /// The node's human-readable name, if set.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the node's name.
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Reference to the local transform.
    #[must_use]
    pub fn local_transform(&self) -> &Transform {
        &self.local_transform
    }

    /// Mutable reference to the local transform.
    pub fn local_transform_mut(&mut self) -> &mut Transform {
        &mut self.local_transform
    }

    /// Set the local position (convenience for `local_transform_mut().position`).
    pub fn set_position(&mut self, pos: glam::Vec3) -> &mut Self {
        self.local_transform.position = pos;
        self
    }

    /// Set the local rotation (convenience for `local_transform_mut().rotation`).
    pub fn set_rotation(&mut self, rot: glam::Quat) -> &mut Self {
        self.local_transform.rotation = rot;
        self
    }

    /// Set uniform scale (convenience for `local_transform_mut().scale`).
    pub fn set_scale(&mut self, scale: glam::Vec3) -> &mut Self {
        self.local_transform.scale = scale;
        self
    }

    /// Set uniform scale to a single value.
    pub fn set_uniform_scale(&mut self, s: f32) -> &mut Self {
        self.local_transform.scale = glam::Vec3::splat(s);
        self
    }

    /// Current position (shorthand for `local_transform().position`).
    #[must_use]
    pub fn position(&self) -> glam::Vec3 {
        self.local_transform.position
    }

    /// Current uniform scale (reads the X component).
    #[must_use]
    pub fn uniform_scale(&self) -> f32 {
        self.local_transform.scale.x
    }

    /// Translate by an offset (adds to current position).
    pub fn translate(&mut self, offset: glam::Vec3) -> &mut Self {
        self.local_transform.position += offset;
        self
    }

    /// Add a delta to the uniform scale (all components).
    pub fn add_uniform_scale(&mut self, delta: f32) -> &mut Self {
        self.local_transform.scale += glam::Vec3::splat(delta);
        self
    }

    /// Clamp uniform scale to `[min, max]` (all components).
    pub fn clamp_uniform_scale(&mut self, min: f32, max: f32) -> &mut Self {
        let s = self.local_transform.scale.x.clamp(min, max);
        self.local_transform.scale = glam::Vec3::splat(s);
        self
    }

    /// What kind of data this node contains.
    #[must_use]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Set the material for a mesh node.
    ///
    /// No-op if the node is not a mesh.
    pub fn set_material(&mut self, material_id: MaterialId) -> &mut Self {
        if let NodeKind::Mesh {
            ref mut material, ..
        } = self.kind
        {
            *material = material_id;
        }
        self
    }

    /// Whether this node is visible.
    #[must_use]
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// Set visibility. Hidden nodes (and their children) are skipped during rendering.
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.visible = visible;
        self
    }
}
